use anyhow;
use log::{debug};
use std::process::Command;

pub(crate) fn present() -> anyhow::Result<()> {
    let output = Command::new("restic")
        .arg("version")
        .output()
        .expect("Restic is not installed");
    debug!("status {:?}", output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
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
