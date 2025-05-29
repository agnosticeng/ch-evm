use core::str;
use std::clone::Clone;
use std::fmt::Debug;
use std::sync::Arc;
use clap::Args;
use anyhow::{bail, anyhow, Context, Result,Ok};
use serde_json::Value;
use arrow::datatypes::{Schema,DataType,Field,BinaryType};
use arrow::array::{Array, BinaryArray, GenericByteBuilder, ListArray, RecordBatch};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use crate::evm::rpc::{RpcCall,RpcClient};
use crate::cli::utils::*;

#[derive(Debug, Clone, Args)]
pub struct EthereumRPCCommand {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long, default_value = "")]
    output_file: String,

    #[arg(short, long, default_value_t = 100)]
    max_batch_size: usize
}

impl EthereumRPCCommand {
    pub async fn run(&self) -> Result<()> {
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

                let method_col: &BinaryArray = input_batch.get_column("method")?;
                let endpoint_col: &BinaryArray = input_batch.get_column("endpoint")?;
                let params_col: &ListArray = input_batch.get_column("params")?; 

                if !endpoint_col.iter().all(|x| x.is_some() && x.unwrap() == endpoint_col.value(0)) {
                    bail!("endpoint must be constant for an input block");
                }

                let client = RpcClient::new(str::from_utf8(endpoint_col.value(0))?)?;
                let call_futs: Vec<RpcCall> = (0..method_col.len())
                    .map(|i| {
                        let params = params_col
                            .value(i)
                            .as_array::<BinaryArray>()?
                            .iter()
                            .map(|p| {
                                let p = match str::from_utf8(p.ok_or(anyhow!("param is not valid UTF-8"))?)? {
                                    s if s.starts_with("0x") => format!("\"{}\"", s),
                                    s => s.to_string()
                                };
                                serde_json::from_str::<Value>(&p).context("failed to get JSON from param")
                            })
                            .collect::<Result<Vec<Value>>>()?;

                        Ok(RpcCall{
                                method: str::from_utf8(method_col.value(i))?.to_string(),
                                params
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                client.calls(call_futs).await?
                    .into_iter()
                    .try_for_each(|res| {
                        result_col_builder.append_value(serde_json::to_string(&res)?.as_bytes());
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
