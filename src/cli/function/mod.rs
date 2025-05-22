mod evm_decode_event;
mod ethereum_rpc;

use clap::{Args, Subcommand};
use anyhow::Result;
use evm_decode_event::EVMDecodeEventCommand;
use ethereum_rpc::EthereumRPCCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum FunctionCommand {
    EVMDecodeEvent(EVMDecodeEventCommand),
    EthereumRPC(EthereumRPCCommand)
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
            FunctionCommand::EthereumRPC(cmd) => cmd.run()
        }
    }
}