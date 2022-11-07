use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    task: Option<String>,
}

pub fn parse() -> Args {
    Args::parse()
}
