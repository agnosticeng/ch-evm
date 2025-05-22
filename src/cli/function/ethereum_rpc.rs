use std::str;
use std::clone::Clone;
use std::fmt::Debug;
use anyhow::Result;
use clap::Args;
use serde::{Deserialize,Serialize};
use tokio::runtime::Builder;

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
    pub fn run(&self) -> Result<()> {
        let runtime = Builder::new_multi_thread().build()?;
        runtime.block_on(self.run_async())
    }

    pub async fn run_async(&self) -> Result<()> {
        Ok(())
    }
}
