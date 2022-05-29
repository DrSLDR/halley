use anyhow;
use log::{debug, trace};
use std::process::{Command, Output};

fn invoke(mut cmd: Command) -> Result<Output, std::io::Error> {
    trace!("Invoking {:?}", &cmd);
    cmd.output()
}

pub(crate) fn present() -> anyhow::Result<()> {
    let mut cmd = Command::new("restic");
    cmd.arg("version");
    invoke(cmd).expect("Restic is not on path");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
                .is_test(true)
                .try_init();
    }

    #[test]
    fn presence() {
        init();
        assert!(match present() {
            Ok(_) => true,
            Err(_) => false,
        });
    }
}
