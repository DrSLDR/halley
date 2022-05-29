use anyhow;
use std::process::{Command, Output};
use tracing::{trace, debug_span};

fn invoke(mut cmd: Command) -> Result<Output, std::io::Error> {
    trace!("Invoking {:?}", &cmd);
    cmd.output()
}

pub(crate) fn present() -> anyhow::Result<()> {
    let span = debug_span!("restic presence");
    let _enter = span.enter();
    let mut cmd = Command::new("restic");
    cmd.arg("version");
    invoke(cmd).expect("Restic is not on path");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    fn init() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn presence() {
        init();
        assert!(present().is_ok());
    }
}
