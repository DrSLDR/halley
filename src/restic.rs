use anyhow;
use std::process::{Command, Output};
use tracing::{debug, debug_span, info_span, trace};

#[cfg(test)]
use mockall::{automock, mock, predicate::*};

#[cfg_attr(test, automock)]
trait MockableCall {
    fn invoke(&self) -> Result<Output, std::io::Error>;
}

#[derive(Debug)]
struct ResticCall {
    cmd: Command,
}

impl ResticCall {
    fn new() -> ResticCall {
        ResticCall {
            cmd: Command::new("restic")
        }
    }
}

impl Default for ResticCall {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) struct AWSKey {
    id: String,
    secret: String,
}

#[derive(Debug)]
pub(crate) enum Repo {
    Local {
        path: String,
    },
    SFTP {
        user: String,
        host: String,
    },
    REST,
    S3 {
        key: AWSKey,
        region: String,
        url: String,
        port: Option<u32>,
        bucket: String,
    },
}

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

pub(crate) fn init(repo: Repo) -> anyhow::Result<()> {
    let span = info_span!("repo init");
    let _enter = span.enter();
    assert!(present().is_ok());

    debug!("Initializing repo with {:?}", repo);

    todo!("Init function not yet written");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    fn log_init() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn presence() {
        log_init();
        assert!(present().is_ok());
    }

    #[test]
    fn test_init() {
        log_init();
        let repo = Repo::Local { path: "/tmp/restic/foo".to_string() };
        assert!(init(repo).is_ok());
    }
}
