mod restic;

use tracing::Level;
use tracing_subscriber;

fn main() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::TRACE)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().compact().init();

    println!("Hello, world!");
}
