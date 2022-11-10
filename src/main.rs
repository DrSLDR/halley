use tracing::Level;
use tracing_subscriber;

mod cli;

use halley::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let c = cli::parse();
    println!("{:?}", c);

    handle_logging(c);

    Ok(())
}

fn handle_logging(args: cli::Args) {
    // Default logging level check
    if !args.quiet && !args.verbose && args.debug == 0 {
        #[cfg(debug_assertions)]
        log::init_trace_logging();
        #[cfg(not(debug_assertions))]
        log::init_info_logging();
    }
}
