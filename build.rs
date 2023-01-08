use std::io::Write;
use std::{fs, io};

fn main() {
    println!("cargo:rerun-if-changed=example.toml");

    let examplefile = fs::read_to_string("./example.toml").expect("Unable to read 'example.toml'");
    let mut f = io::BufWriter::new(
        fs::File::create("./src/config/example.toml")
            .expect("Unable to create config-local example file"),
    );
    write!(f, "{}", examplefile).expect("Unable to write config-local example file");
}
