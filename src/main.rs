use tracing::Level;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::TRACE)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().compact().init();

    println!("Hello, world!");

    lib::s3::S3Handler::new();
}
