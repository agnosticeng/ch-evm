use std::str;
use std::sync::Arc;
use std::clone::Clone;
use std::fmt::{Debug};
use futures::stream::{iter,StreamExt};
use anyhow::Result;
use arrow::array::{BinaryArray, FixedSizeBinaryArray, GenericByteBuilder, ListArray, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, BinaryType};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use alloy::primitives::FixedBytes;
use clap::Args;
use quick_cache::sync::Cache;
use crate::evm::abi::json_encoding::*;
use crate::cli::utils::*;

#[derive(Debug, Clone, Args)]
pub struct EVMDecodeEventCommand {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long, default_value = "")]
    output_file: String,

    #[arg(short, long, default_value_t = 2000)]
    abi_provider_cache_size: usize
}

impl EVMDecodeEventCommand {
    pub async fn run(&self) -> Result<()> {
        let cache = Arc::new(Cache::new(self.abi_provider_cache_size));
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

                let topics_col: &ListArray = input_batch.get_column("topics")?;
                let data_col: &BinaryArray = input_batch.get_column("data")?; 
                let abis_col: &ListArray = input_batch.get_column("abis")?;

                for i in 0..input_batch.num_rows() {
                    let topics = topics_col.value(i);
                    let topics: &FixedSizeBinaryArray = topics.as_array()?;

                    let data = data_col.value(i);

                    let abis = abis_col.value(i);
                    let abis: &BinaryArray  = abis.as_array()?;

                    let res= iter(abis)
                        .map(|key| {
                            let cache = cache.clone();

                            async move {
                                let p = get_cached_abi_item_provider(cache, key.unwrap()).await?;
                                let evt = p.get_event(topics.value(0))?;
                                let topics = topics
                                    .iter()
                                    .flatten()
                                    .map(|x| FixedBytes::from_slice(x));
                                let decoded_evt = evt.decode_log_parts(topics, data)?;
                                encode_event(evt, &decoded_evt)
                            }
                        })
                        .filter_map(|f| Box::pin(async { f.into_future().await.ok() }))
                        .next()
                        .await;

                        match res {
                            Some(js) => result_col_builder.append_value(js),
                            None => result_col_builder.append_value(b"{\"error\": \"cannot decode event\"}"),
                        }
                    }

                let result_col = result_col_builder.finish();
                let output_batch = RecordBatch::try_new(output_schema.clone(), vec![Arc::new(result_col)])?;

                writer.write(&output_batch)?;
                writer.flush()?;
            }
        }
    }
}