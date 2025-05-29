use std::str;
use std::sync::Arc;
use std::clone::Clone;
use std::fmt::{Debug};
use anyhow::{anyhow, Result};
use arrow::array::{BinaryArray, GenericByteBuilder, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, BinaryType};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use clap::Args;
use crate::cli::utils::*;

#[derive(Debug, Clone, Args)]
pub struct ConvertFormatCommand {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long, default_value = "")]
    output_file: String
}

impl ConvertFormatCommand {
    pub async fn run(&self) -> Result<()> {
        let mut input_file = open_file_or_stdin(&self.input_file)?;
        let mut output_file = create_file_or_stdout(&self.output_file)?;
        let output_schema = Arc::new(Schema::new(vec![Field::new("result", DataType::Binary, false)]));

        loop {
            let reader = StreamReader::try_new_buffered(&mut input_file, None)?;
            let mut writer = StreamWriter::try_new_buffered(&mut output_file, &output_schema)?;

            for input_batch in reader {
                let input_batch = input_batch?;

                let mut result_col_builder = GenericByteBuilder::<BinaryType>::with_capacity(
                    input_batch.num_rows(),
                    input_batch.num_rows() * 1024
                );

                let from_format_col: &BinaryArray = input_batch.get_column("from_format")?; 
                let to_format_col: &BinaryArray = input_batch.get_column("to_format")?; 
                let str_col: &BinaryArray = input_batch.get_column("str")?; 

                for i in 0..input_batch.num_rows() {
                    let from = str::from_utf8(from_format_col.value(i))?;
                    let to = str::from_utf8(to_format_col.value(i))?;
                    let str = str::from_utf8(str_col.value(i))?;

                    let res = match (from.to_uppercase().as_str(), to.to_uppercase().as_str()) {
                        ("JSON", "YAML") => Ok(serde_yaml::to_string(&serde_json::from_str::<serde_yaml::Value>(str)?)?), 
                        ("JSON", "TOML") => Ok(serde_json::from_str::<toml::Value>(str)?.to_string()),
                        ("YAML", "JSON") => Ok(serde_yaml::from_str::<serde_json::Value>(str)?.to_string()),
                        ("YAML", "TOML") => Ok(serde_yaml::from_str::<toml::Value>(str)?.to_string()),
                        ("TOML", "JSON") => Ok(toml::from_str::<serde_json::Value>(str)?.to_string()),
                        ("TOML", "YAML") => Ok(serde_yaml::to_string(&toml::from_str::<serde_yaml::Value>(str)?)?),
                        _ => Err(anyhow!("invalid conversion"))
                    };

                    result_col_builder.append_value(res?);
                }

                let result_col = result_col_builder.finish();
                let output_batch = RecordBatch::try_new(output_schema.clone(), vec![Arc::new(result_col)])?;

                writer.write(&output_batch)?;
                writer.flush()?;
            }
        }
    }
}