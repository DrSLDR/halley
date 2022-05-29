use anyhow;
use std::process::Command;

pub(crate) fn restic_present() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn dense() {
        init();
        assert_eq!(1, 1);
    }
}
