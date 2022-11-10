use tracing::Level;
use tracing_subscriber;

mod cli;

use halley::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    log::init_trace_logging();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().compact().init();

    let c = cli::parse();
    println!("{:?}", c);

    Ok(())
}
