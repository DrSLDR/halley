mod cli;

use halley::*;

use anyhow::anyhow;
use tracing::error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::parse();
    println!("{:?}", args);

    handle_logging(&args);

    let config_file = args.config.unwrap();
    if !config_file.is_file() {
        error!("The config file at {:?} does not exist", config_file);
        return Err(anyhow!("Failed to open config file!"));
    }

    match &args.command {
        cli::Commands::Validate => {
            validate_config(config_file)?;
        }
    }

    Ok(())
}

fn handle_logging(args: &cli::Args) {
    // Godforsaken matcher for the log flag combinations
    match (args.quiet, args.verbose, args.debug) {
        (false, false, 0) => {
            #[cfg(debug_assertions)]
            log::init_trace_logging();
            #[cfg(not(debug_assertions))]
            log::init_warn_logging();
        }
        (true, _, _) => log::init_error_logging(),
        (false, _, 1) => log::init_debug_logging(),
        (false, _, 2) => log::init_trace_logging(),
        (false, true, 0) => log::init_info_logging(),
        _ => panic!("Somehow got an illegal log flag combination!"),
    }
}
