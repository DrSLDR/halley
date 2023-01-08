// #[cfg(test)]
// mod tests;

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
    /// Output an example configuration file
    InitConfig(InitArgs),

    /// Validate a configuration file
    Validate(ValidateArgs),

    /// Run Halley
    ///
    /// By default, this will look at the configuration file, check the states of each
    /// of the defined repositories, and run against the one that's been without an
    /// update the longest. This can be modified using the options, like doing a dry
    /// run, forcibly running a specific repository, and so on.
    Run(RunArgs),
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Config file, ~/.halley/config.toml by default
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Print the minimal configuration instead of the more verbose example
    #[arg(short, long)]
    pub minimal: bool,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Dry run
    ///
    /// When running dry, Halley will attempt to do as much as it possibly can without
    /// ever writing anything to disk. Since some operations may be impossible against
    /// certain backends (e.g. S3 glacier storage), it can't guarantee that it will do
    /// everything.
    #[arg(short, long)]
    pub dry: bool,

    /// Config file, ~/.halley/config.toml by default
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Statefile directory, ~/.halley by default
    ///
    /// This does not really make sense unless you also specify a config file, since the
    /// name of the statefile is given in it. But you do you.
    #[arg(long)]
    pub state_dir: Option<PathBuf>,

    /// Force a specific repository to update
    ///
    /// This overrides the last-to-update check and forces a specific repository to
    /// update, even if the update schedule says otherwise.
    ///
    /// This can be useful to combine with dry run.
    #[arg(long)]
    pub force_repo: Option<String>,
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
        Commands::Validate(c_args) => &c_args.config,
        Commands::Run(c_args) => &c_args.config,
        _ => &None,
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

fn enforce_default_statepath(args: &mut Arguments) {
    match &args.command {
        Commands::Run(c_args) => unimplemented!(),
        _ => (),
    }
    let default = PathBuf::from("~/.halley");
}

pub fn parse() -> Arguments {
    let mut args = Arguments::parse();

    enforce_defaults(&mut args);

    args
}
