mod evm_decode_event;
mod evm_decode_call;
mod evm_decode_calldata;
mod ethereum_decode_tx;
mod ethereum_rpc;
mod ethereum_rpc_call;
mod evm_descriptor_from_fullsig;
mod evm_signature_from_descriptor;
mod keccak256;
mod convert_format;

use clap::{Args, Subcommand};
use anyhow::Result;
use evm_decode_event::EVMDecodeEventCommand;
use evm_decode_call::EVMDecodeCallCommand;
use evm_decode_calldata::EVMDecodeCalldataCommand;
use ethereum_decode_tx::EthereumDecodeTxCommand;
use ethereum_rpc::EthereumRPCCommand;
use ethereum_rpc_call::EthereumRPCCallCommand;
use evm_descriptor_from_fullsig::EVMDescriptorFromFullsigCommand;
use evm_signature_from_descriptor::EVMSignatureFromDescriptorCommand;
use keccak256::Keccak256Command;
use convert_format::ConvertFormatCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum FunctionCommand {
    EVMDecodeEvent(EVMDecodeEventCommand),
    EVMDecodeCall(EVMDecodeCallCommand),
    EVMDecodeCalldata(EVMDecodeCalldataCommand),
    EthereumDecodeTx(EthereumDecodeTxCommand),
    EthereumRPC(EthereumRPCCommand),
    EthereumRPCCall(EthereumRPCCallCommand),
    EVMDescriptorFromFullsig(EVMDescriptorFromFullsigCommand),
    EVMSignatureFromDescriptor(EVMSignatureFromDescriptorCommand),
    Keccak256(Keccak256Command),
    ConvertFormat(ConvertFormatCommand)
}

#[derive(Clone, Debug, Args)]
pub struct Function {
    #[command(subcommand)]
    pub cmd: FunctionCommand
}

impl Function {
    pub async fn run(&self) -> Result<()> {
        match &self.cmd {
            FunctionCommand::EVMDecodeEvent(cmd) => cmd.run().await,
            FunctionCommand::EVMDecodeCall(cmd) => cmd.run().await,
            FunctionCommand::EVMDecodeCalldata(cmd) => cmd.run().await,
            FunctionCommand::EthereumDecodeTx(cmd) => cmd.run().await,
            FunctionCommand::EthereumRPC(cmd) => cmd.run().await,
            FunctionCommand::EthereumRPCCall(cmd) => cmd.run().await,
            FunctionCommand::EVMDescriptorFromFullsig(cmd) => cmd.run().await,
            FunctionCommand::EVMSignatureFromDescriptor(cmd) => cmd.run().await,
            FunctionCommand::Keccak256(cmd) => cmd.run().await,
            FunctionCommand::ConvertFormat(cmd) => cmd.run().await
        }
    }
}

