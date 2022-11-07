use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Config file, ~/.halley/config.toml by default
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Verbose (sets log level to INFO)
    #[arg(short, long)]
    verbose: bool,

    /// Debug (sets log level to DEBUG, or TRACE if given twice)
    ///
    /// Be aware that these log levels can leak credentials
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

pub fn parse() -> Args {
    let mut args = Args::parse();

    args.config = match args.config {
        None => Some(PathBuf::from("~/.halley/config.toml")),
        Some(p) => Some(p),
    };

    args
}
