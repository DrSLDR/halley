use tracing::Level;
use tracing_subscriber;

use halley::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().compact().init();

    println!("Hello, world!");

    test_config()?;

    Ok(())
}
