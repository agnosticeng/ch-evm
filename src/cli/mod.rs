mod function;
mod utils;

use clap::{Subcommand,Parser};
use anyhow::Result;
use tokio::runtime::Builder;
use crate::cli::function::Function;

#[derive(Debug, Subcommand)]
pub enum Command {
    Function(Function)
}

#[derive(Parser)]
pub struct CLI {
    #[command(subcommand)]
    pub cmd: Command,
}

impl CLI {
    pub fn run(&self) -> Result<()> {
        Builder::new_multi_thread()
            .build()?
            .block_on(async {
                match &self.cmd {
                    Command::Function(cmd) => cmd.run().await,
                }
            })
    }
}
