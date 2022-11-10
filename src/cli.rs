use std::path::PathBuf;

use clap::{Parser, Subcommand};
use shellexpand::tilde;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Config file, ~/.halley/config.toml by default
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Quiet (sets log level to ERROR)
    ///
    /// Takes precedence over -v or -d
    #[arg(short, long)]
    pub quiet: bool,

    /// Verbose (sets log level to INFO)
    #[arg(short, long)]
    pub verbose: bool,

    /// Debug (sets log level to DEBUG, or TRACE if given twice)
    ///
    /// Takes precedence over -v
    ///
    /// Be aware that these log levels WILL leak credentials
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Validate a configuration file
    Validate,
}

pub fn parse() -> Args {
    let mut args = Args::parse();

    // Enforce default on config file path
    // Also make sure we do any tilde expansion we may need to do
    args.config = Some({
        let pb = match args.config {
            None => PathBuf::from("~/.halley/config.toml"),
            Some(p) => p,
        };
        PathBuf::from(tilde(&pb.to_string_lossy().into_owned()).into_owned())
    });

    // Clamp number of debug flags
    args.debug = if args.debug > 2 { 2 } else { args.debug };

    args
}
