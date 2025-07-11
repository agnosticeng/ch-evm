use anyhow::Result;
use clap::Parser;
use mimalloc::MiMalloc;

pub mod evm;
pub mod cli;
pub mod json;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<()> {
    cli::CLI::parse().run()
}