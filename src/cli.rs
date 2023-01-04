use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use shellexpand::tilde;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Arguments {
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
    Validate(ValidateArgs),
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Config file, ~/.halley/config.toml by default
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

/// Enforce the defaults in the arguments
fn enforce_defaults(args: &mut Arguments) {
    // Handle the config file and do any needed tilde expansion
    enforce_default_config(args);

    // Clamp the number of debug flags
    args.debug = if args.debug > 2 { 2 } else { args.debug };
}

/// Enforce default specifically when handling the configuration file
///
/// The config flag, when given to subcommands that support it, take priority over the
/// "global" flag
fn enforce_default_config(args: &mut Arguments) {
    let default = PathBuf::from("~/.halley/config.toml");
    let subc_config = match &args.command {
        Commands::Validate(ValidateArgs { config }) => config,
    };
    let config = match &subc_config {
        None => &args.config,
        Some(_) => subc_config,
    };

    args.config = Some({
        let pb = match config {
            None => default,
            Some(p) => p.to_owned(),
        };
        PathBuf::from(tilde(&pb.to_string_lossy().into_owned()).into_owned())
    });
}

pub fn parse() -> Arguments {
    let mut args = Arguments::parse();

    enforce_defaults(&mut args);

    args
}
