use anyhow;
use std::process::{Command, Output};
use tracing::{trace, span, Level};

fn invoke(mut cmd: Command) -> Result<Output, std::io::Error> {
    trace!("Invoking {:?}", &cmd);
    cmd.output()
}

pub(crate) fn present() -> anyhow::Result<()> {
    let span = span!(Level::DEBUG, "restic presence");
    let _enter = span.enter();
    let mut cmd = Command::new("restic");
    cmd.arg("version");
    invoke(cmd).expect("Restic is not on path");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .try_init();
    }

    #[test]
    fn presence() {
        init();
        assert!(present().is_ok());
    }
}
