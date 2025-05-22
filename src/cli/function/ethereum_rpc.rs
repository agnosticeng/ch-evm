use std::str;
use std::clone::Clone;
use std::fmt::Debug;
use anyhow::Result;
use clap::Args;
use serde::{Deserialize,Serialize};

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
        Ok(())
    }
}
