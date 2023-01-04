mod cli;

use halley::*;

use tracing::debug;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::parse();

    handle_logging(&args);
    debug!("Got args\n{:#?}", args);

    match &args.command {
        cli::Commands::Validate(_) => {
            validate_config(args.config.unwrap())?;
            println!("Ok!");
        }
        cli::Commands::InitConfig => unimplemented!(),
    }

    Ok(())
}

fn handle_logging(args: &cli::Arguments) {
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
