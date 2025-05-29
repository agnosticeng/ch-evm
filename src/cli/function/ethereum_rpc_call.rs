use core::str;
use std::clone::Clone;
use std::fmt::Debug;
use std::sync::Arc;
use anyhow::{bail, Result, Ok};
use clap::Args;
use serde_json::{json, Value};
use alloy::rpc::types::BlockNumberOrTag;
use alloy::primitives::hex::decode;
use arrow::datatypes::{Schema,DataType,Field,BinaryType};
use arrow::array::{Array, BinaryArray, GenericByteBuilder, Int64Array, RecordBatch};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use quick_cache::sync::Cache;
use crate::evm::abi::json_encoding::*;
use crate::evm::rpc::{RpcCall,RpcClient,RpcResult};
use crate::cli::utils::*;

#[derive(Debug, Clone, Args)]
pub struct EthereumRPCCallCommand {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long, default_value = "")]
    output_file: String,

    #[arg(short, long, default_value_t = 100)]
    max_batch_size: usize
}

impl EthereumRPCCallCommand {
    pub async fn run(&self) -> Result<()> {
        let cache = Arc::new(Cache::new(100));
        let mut input_file = open_file_or_stdin(&self.input_file)?;
        let mut output_file = create_file_or_stdout(&self.output_file)?;
        let output_schema = Arc::new(Schema::new(vec![
            Field::new("result", DataType::Binary, false),
        ]));

        loop {
            let reader = StreamReader::try_new_buffered(&mut input_file, None)?;
            let mut writer = StreamWriter::try_new_buffered(&mut output_file, &output_schema)?;

            for input_batch in reader {
                let input_batch = input_batch?;

                let mut result_col_builder = GenericByteBuilder::<BinaryType>::with_capacity(
                    input_batch.num_rows(),
                    input_batch.num_rows() * 1024
                );

                let to_col: &BinaryArray = input_batch.get_column("to")?;
                let fullsig_col: &BinaryArray = input_batch.get_column("fullsig")?;
                let data_col: &BinaryArray = input_batch.get_column("data")?;
                let block_number_col: &Int64Array = input_batch.get_column("block_number")?;
                let endpoint_col: &BinaryArray = input_batch.get_column("endpoint")?;

                if !endpoint_col.iter().all(|x| x.is_some() && x.unwrap() == endpoint_col.value(0)) {
                    bail!("endpoint must be constant for an input block");
                }

                let client = RpcClient::new(str::from_utf8(endpoint_col.value(0))?)?;

                let call_futs= (0..to_col.len())
                    .map(|i| {
                            let cache = cache.clone();
                            let func = get_cached_func_sync(cache, fullsig_col.value(i))?;
                            let data: Value = serde_json::from_str(str::from_utf8(data_col.value(i))?)?;

                            if !data.is_array() {
                                bail!("data must be an array");
                            }

                            let data = data.as_array().expect("must be an array");
                            let block_number = to_block_number_or_tag(block_number_col.value(i))?;

                            Ok(RpcCall{
                                method: "eth_call".to_string(),
                                params: vec![
                                    json!({
                                        "to": str::from_utf8(to_col.value(i))?,
                                        "data": func.abi_encode_input(&func.coerce_inputs(data)?)?
                                    }),
                                    serde_json::Value::String(block_number.to_string())
                                ]
                            })
                    })
                    .collect::<Result<Vec<RpcCall>>>()?;

                client.calls(call_futs).await?
                    .into_iter()
                    .enumerate()
                    .try_for_each(|(i, res)| {
                        let cache = cache.clone();
                        let func = get_cached_func_sync(cache, fullsig_col.value(i))?;

                        match res {
                            RpcResult::Error(s) => result_col_builder.append_value(json!({"error": s}).to_string()),
                            RpcResult::Value(Value::String(s)) => {
                                let d = decode(s)?;
                                let dec = func.abi_decode_output(&d)?;
                                let it = dec
                                    .iter()
                                    .enumerate()
                                    .map(|(i, param)| (format!("arg{}", i), param));
                                result_col_builder.append_value(encode_values(it)?);
                            }
                            RpcResult::Value(_) => result_col_builder.append_value(json!({"error": "failed to decode result data"}).to_string())
                        }

                        Ok(())
                    })?;

                let result_col = result_col_builder.finish();
                let output_batch = RecordBatch::try_new(output_schema.clone(), vec![Arc::new(result_col)])?;
                writer.write(&output_batch)?;
                writer.flush()?;
            }
        }
    }
}

fn to_block_number_or_tag(i: i64) -> Result<BlockNumberOrTag> {
    match i {
        -4 => Ok(BlockNumberOrTag::Safe),
        -3 => Ok(BlockNumberOrTag::Finalized),
        -2 => Ok(BlockNumberOrTag::Latest),
        -1 => Ok(BlockNumberOrTag::Pending),
        0 => Ok(BlockNumberOrTag::Earliest),
        _ => Ok(BlockNumberOrTag::from(u64::try_from(i)?))
    }
}