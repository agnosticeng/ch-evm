use anyhow::Result;
use clap::Parser;

pub mod evm;
pub mod cli;
pub mod json;

extern crate jemallocator;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> Result<()> {
    cli::CLI::parse().run()
}