mod cli;
mod core;
mod hardware;
mod npu;
pub mod presets;
mod utils;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::args::{Cli, Commands};
use crate::cli::commands::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        "c3avcompressor=debug,info"
    } else {
        "c3avcompressor=info,warn,error"
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(tracing_subscriber::fmt::layer().with_target(false).without_time())
        .init();

    match cli.command {
        Commands::Compress(args) => handle_compress(args)?,
        Commands::Presets { query } => handle_presets(query)?,
        Commands::Split(args) => handle_split(args)?,
        Commands::Extract(args) => handle_extract(args)?,
        Commands::Probe(args) => handle_probe(args)?,
        Commands::NpuStatus(args) => handle_npu_status(args)?,
        Commands::Benchmark(args) => handle_benchmark(args)?,
        Commands::Completions { shell } => handle_completions(shell)?,
    }

    Ok(())
}
