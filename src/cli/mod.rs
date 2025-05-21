mod function;
mod utils;

use clap::{Subcommand,Parser};
use anyhow::Result;
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
        match &self.cmd {
            Command::Function(cmd) => cmd.run(),
        }
    }
}
