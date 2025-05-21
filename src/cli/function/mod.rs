mod evm_decode_event;

use clap::{Args, Subcommand};
use anyhow::Result;
use evm_decode_event::EVMDecodeEventCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum FunctionCommand {
    EVMDecodeEvent(EVMDecodeEventCommand),
}

#[derive(Clone, Debug, Args)]
pub struct Function {
    #[command(subcommand)]
    pub cmd: FunctionCommand
}

impl Function {
    pub fn run(&self) -> Result<()> {
        match &self.cmd {
            FunctionCommand::EVMDecodeEvent(cmd) => cmd.run(),
        }
    }
}