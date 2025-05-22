use std::str;
use std::rc::Rc;
use std::clone::Clone;
use std::fmt::Debug;
use anyhow::{Result,bail};
use alloy_json_abi::AbiItem;
use clap::Args;
use alloy_primitives::B256;
use serde::{Deserialize,Serialize};
use arrow::datatypes::{FieldRef,Schema};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use serde_arrow::schema::{SchemaLike,TracingOptions};
use serde_json::json;
use mini_moka::unsync::Cache;
use crate::evm::{EventJSONExt,AbiItemProviderFactory};
use crate::cache::CacheExt;
use crate::cli::utils::*;

#[derive(Serialize, Deserialize)]
struct InputRow {
    topics: Vec<B256>,
    data: Vec<u8>,
    abis: Vec<Vec<u8>>
}

#[derive(Serialize, Deserialize)]
struct OutputRow {
    result: String
}

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
        let fields: Vec<std::sync::Arc<arrow::datatypes::Field>> = Vec::<FieldRef>::from_type::<OutputRow>(TracingOptions::default())?;
        let schema = Schema::new(fields.clone());
        let mut cache = Cache::new(self.abi_provider_cache_size);
        let mut input_file = open_file_or_stdin(&self.input_file)?;
        let mut output_file = create_file_or_stdout(&self.output_file)?;

        loop {
            let reader = StreamReader::try_new(&mut input_file, None)?;
            let mut writer = StreamWriter::try_new(&mut output_file, &schema)?;

            for batch in reader {
                let batch = batch?;
                let input_rows: Vec<InputRow> =  serde_arrow::from_record_batch(&batch)?;
                let mut output_rows: Vec<OutputRow> = Vec::new();

                for input_row in input_rows {
                    let js = input_row.abis
                        .into_iter()
                        .map(|key| {
                            let key = String::from_utf8(key)?;
                            let abi_item_provider = cache.get_or_create(&key, || {
                                AbiItemProviderFactory::create(&key).map(Rc::new)
                            })?;
                            let abi_field = abi_item_provider.get_abi_item(input_row.topics[0].as_slice())?;

                            match abi_field {
                                AbiItem::Event(evt) => evt.decode_log_parts_to_json_value(input_row.topics.clone(),  &input_row.data),
                                _ => bail!("abi item is not an event")
                            }
                        })
                        .find(|res| res.is_ok())
                        .unwrap_or(Ok(json!({"error": "cannot decode event"})))?;

                    
                    output_rows.push(OutputRow{result: js.to_string()});
                }

                let batch = serde_arrow::to_record_batch(&fields, &output_rows)?;
                writer.write(&batch)?;
                writer.flush()?;
            }
        }
    }
}
