use std::str;
use std::rc::Rc;
use std::sync::Arc;
use std::clone::Clone;
use std::fmt::Debug;
use anyhow::{bail, Result};
use alloy_json_abi::AbiItem;
use clap::Args;
use alloy_primitives::FixedBytes;
use arrow::array::{BinaryArray, FixedSizeBinaryArray, ListArray, RecordBatch, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use serde_json::json;
use mini_moka::unsync::Cache;
use crate::evm::{EventJSONExt,AbiItemProviderFactory};
use crate::cache::CacheExt;
use crate::cli::utils::*;


#[derive(Debug, Clone, Args)]
pub struct EVMDecodeEventCommand {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long, default_value = "")]
    output_file: String,

    #[arg(short, long, default_value_t = 2000)]
    abi_provider_cache_size: u64
}

impl EVMDecodeEventCommand {
    pub async fn run(&self) -> Result<()> {
        let mut cache = Cache::new(self.abi_provider_cache_size);
        let mut input_file = open_file_or_stdin(&self.input_file)?;
        let mut output_file = create_file_or_stdout(&self.output_file)?;
        let output_schema = Arc::new(Schema::new(vec![
            Field::new("result", DataType::Utf8, false),
        ]));

        loop {
            let reader = StreamReader::try_new_buffered(&mut input_file, None)?;
            let mut writer = StreamWriter::try_new_buffered(&mut output_file, &output_schema)?;

            for input_batch in reader {
                let input_batch = input_batch?;

                let mut result_col_builder = StringBuilder::with_capacity(
                    input_batch.num_rows(),
                    input_batch.num_rows() * 4 * 1024
                );

                let topics_col: &ListArray = input_batch.get_column("topics")?;
                let data_col: &BinaryArray = input_batch.get_column("data")?; 
                let abis_col: &ListArray = input_batch.get_column("abis")?;

                for i in 0..input_batch.num_rows() {
                    let topics = topics_col.value(i);
                    let topics: &FixedSizeBinaryArray = topics.as_array()?;

                    let data = data_col.value(i);

                    let abis = abis_col.value(i);
                    let abis: &BinaryArray  = abis.as_array()?;

                    let js = abis
                        .into_iter()
                        .flatten()
                        .map(|key| {
                            let key = str::from_utf8(key)?;

                            let abi_item_provider = cache.get_or_create(&key.to_string(), || {
                                AbiItemProviderFactory::create(key).map(Rc::new)
                            })?;
                           
                            let abi_field = abi_item_provider.get_abi_item(topics.value(0))?;

                            match abi_field {
                                AbiItem::Event(evt) => { 
                                    evt.decode_log_parts_to_json_value(
                                        topics
                                            .iter()
                                            .flatten()
                                            .map(|x| FixedBytes::from_slice(x)), 
                                        data
                                    )
                                },
                                _ => bail!("abi item is not an event")
                            }
                        })
                        .find(|res| res.is_ok())
                        .unwrap_or(Ok(json!({"error": "cannot decode event"})))?;

                    result_col_builder.append_value(js.to_string());
                }

                let result_col = result_col_builder.finish();
                let output_batch = RecordBatch::try_new( output_schema.clone(), vec![Arc::new(result_col)])?;

                writer.write(&output_batch)?;
                writer.flush()?;
            }
        }
    }
}