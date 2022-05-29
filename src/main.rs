mod restic;

fn main() {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    #[cfg(not(debug_assertions))]
    env_logger::init();

    println!("Hello, world!");
}
