use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Config file, ~/.halley/config.toml by default
    #[arg(short, long)]
    config: Option<PathBuf>,
}

pub fn parse() -> Args {
    let mut args = Args::parse();

    args.config = match args.config {
        None => Some(PathBuf::from("~/.halley/config.toml")),
        Some(p) => Some(p),
    };

    args
}
