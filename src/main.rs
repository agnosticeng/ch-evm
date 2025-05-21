use anyhow::Result;
use clap::Parser;

pub mod evm;
pub mod cli;
pub mod cache;


fn main() -> Result<()> {
    cli::CLI::parse().run()
}