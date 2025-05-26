use std::str;
use std::clone::Clone;
use std::fmt::Debug;
use alloy::transports::http::reqwest::Url;
use clap::Args;
use anyhow::{Context,Result};
use alloy_rpc_client::{ClientBuilder,Waiter};
use serde::{Deserialize,Serialize};
use arrow::datatypes::{FieldRef,Schema};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use serde_arrow::schema::{SchemaLike,TracingOptions};
use serde_json::{Value,json};
use futures::future::join_all;
use crate::cli::utils::*;

#[derive(Serialize, Deserialize)]
struct InputRow {
    method: String,
    params: Vec<String>,
    endpoint: String
}

#[derive(Serialize, Deserialize)]
struct OutputRow {
    result: String
}

#[derive(Debug, Clone, Args)]
pub struct EthereumRPCCommand {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(short, long, default_value = "")]
    output_file: String,
}

impl EthereumRPCCommand {
    pub async fn run(&self) -> Result<()> {
        let fields: Vec<std::sync::Arc<arrow::datatypes::Field>> = Vec::<FieldRef>::from_type::<OutputRow>(TracingOptions::default())?;
        let schema = Schema::new(fields.clone());
        let mut input_file = open_file_or_stdin(&self.input_file)?;
        let mut output_file = create_file_or_stdout(&self.output_file)?;

        loop {
            let reader = StreamReader::try_new_buffered(&mut input_file, None)?;
            let mut writer = StreamWriter::try_new_buffered(&mut output_file, &schema)?;

            for batch in reader {
                let batch = batch?;
                let input_rows: Vec<InputRow> =  serde_arrow::from_record_batch(&batch)?;
                let endpoint = Url::parse(&(input_rows[0].endpoint))?;
                let client = ClientBuilder::default().http(endpoint);
                let mut rpc_batch = client.new_batch();

                let rpc_futs: Result<Vec<Waiter<Value>>> = input_rows
                    .iter()
                    .map(|row| {
                        let params = row.params.iter()
                            .map(|p| {
                                let p: String = match p {
                                    s if s.starts_with("0x") => format!("\"{}\"", s),
                                    _ => p.clone()
                                };
                                serde_json::from_str::<Value>(&p).context("failed to get JSON from param")
                            })
                            .collect::<Result<Vec<Value>>>()?;
                        rpc_batch.add_call::<Vec<Value>,Value>(row.method.clone(), &params).context("failed to add call to batch")
                    })
                    .collect();

                let rpc_futs = rpc_futs?;
                rpc_batch.send().await?;

                let output_rows: Vec<OutputRow> = join_all(rpc_futs).await
                    .into_iter()
                    .map(|resp| {
                        match resp {
                            Err(e) => OutputRow{result: json!({"error": e.to_string()}).to_string()},
                            Ok(v) => OutputRow { result: v.to_string() }
                        }
                    })
                    .collect();

                let batch = serde_arrow::to_record_batch(&fields, &output_rows)?;
                writer.write(&batch)?;
                writer.flush()?;
            }
        }
    }
}
