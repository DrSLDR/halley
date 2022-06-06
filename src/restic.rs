use anyhow;
use std::process::{Command, Output};
use tracing::{debug, debug_span, info_span, trace};

#[cfg(test)]
use mockall::{automock, predicate};

#[cfg_attr(test, automock)]
trait WrappedCall {
    fn invoke(&mut self) -> Result<Output, std::io::Error>;
    fn arg(&mut self, arg: &str) -> &mut Self;
}

#[derive(Debug)]
struct ResticCall {
    cmd: Command,
}

impl ResticCall {
    fn new() -> ResticCall {
        ResticCall {
            cmd: Command::new("restic"),
        }
    }
}

impl WrappedCall for ResticCall {
    fn invoke(&mut self) -> Result<Output, std::io::Error> {
        trace!("Invoking {:?}", self.cmd);
        self.cmd.output()
    }
    fn arg(&mut self, arg: &str) -> &mut Self {
        self.cmd.arg(arg);
        self
    }
}

impl Default for ResticCall {
    fn default() -> Self {
        Self::new()
    }
}

fn prepare_present<C: WrappedCall>(wc: &mut C) -> &mut C {
    let span = debug_span!("restic presence");
    let _enter = span.enter();
    wc.arg("version")
}

#[derive(Debug)]
pub(crate) struct RepoBase {
    passwd: String,
}

#[derive(Debug)]
pub(crate) struct LocalRepo {
    path: String,
    base: RepoBase,
}

// #[derive(Debug)]
// pub(crate) struct AWSKey {
//     id: String,
//     secret: String,
// }

#[derive(Debug)]
pub(crate) enum Repo {
    Local { data: LocalRepo },
}

fn prepare_init<C: WrappedCall>(wc: &mut C, repo: Repo) -> &mut C {
    let span = info_span!("repo init");
    let _enter = span.enter();
    // assert!(present().is_ok());

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
        let mut mock = MockWrappedCall::new();
        mock.expect_arg()
            .with(predicate::eq("version"))
            .times(1)
            .returning(|_| MockWrappedCall::new());
        prepare_present(&mut mock);
    }

    #[test]
    fn test_init_local() {
        log_init();
        let base = RepoBase {
            passwd: "test".to_string(),
        };
        let repo = Repo::Local {
            data: LocalRepo {
                path: "/tmp/restic/foo".to_string(),
                base: base,
            },
        };
        let mut mock = MockWrappedCall::new();
        prepare_init(&mut mock, repo);
        panic!();
    }
}
