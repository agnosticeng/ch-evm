use core::str;
use std::sync::Arc;
use std::clone::Clone;
use std::fmt::{Debug};
use anyhow::{Context, Result};
use arrow::array::{BinaryArray, GenericByteBuilder, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, BinaryType};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use alloy::json_abi::AbiItem;
use clap::Args;
use serde_json::json;
use crate::cli::utils::*;

#[derive(Debug, Clone, Args)]
pub struct EVMDescriptorFromFullsigCommand {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long, default_value = "")]
    output_file: String
}

impl EVMDescriptorFromFullsigCommand {
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

                let fullsig_col: &BinaryArray = input_batch.get_column("fullsig")?; 

                for i in 0..input_batch.num_rows() {
                    let f= str::from_utf8(fullsig_col.value(i))
                        .context("invalid UTF-8")
                        .and_then(|s| Ok(AbiItem::parse(s)?))
                        .map(|i| json!({"value": &i}).to_string());

                    match f {
                        Err(err) => result_col_builder.append_value(json!({"error": err.to_string()}).to_string()),
                        Ok(s) => result_col_builder.append_value(s),
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