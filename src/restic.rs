use anyhow;
use std::process::{Command, Output};
use tracing::{debug, debug_span, info_span, trace};

#[cfg(test)]
use mockall::{automock, predicate};

#[cfg_attr(test, automock)]
trait WrappedCall {
    fn invoke(&mut self) -> Result<Output, std::io::Error>;
    fn arg(&mut self, arg: &str) -> &mut Self;
    fn env(&mut self, key: &str, value: &str) -> &mut Self;
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
    fn env(&mut self, key: &str, value: &str) -> &mut Self {
        self.cmd.env(key, value);
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

fn assert_present() -> bool {
    let mut rc = ResticCall::new();
    let rc = prepare_present(&mut rc);
    rc.invoke()
        .expect("restic is not installed (or not on path)");
    true
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

fn prepare_init_base<C: WrappedCall>(wc: &mut C, data: RepoBase) -> &mut C {
    wc.env("RESTIC_PASSWD", data.passwd.as_str());
    wc
}

fn prepare_init<C: WrappedCall>(wc: &mut C, repo: Repo) -> &mut C {
    let span = info_span!("repo init");
    let _enter = span.enter();

    #[cfg(not(test))]
    assert!(assert_present());

    debug!("Initializing repo with {:?}", repo);

    match repo {
        Repo::Local { data } => {
            debug!("Initializing local repo");
            let wc = prepare_init_base(wc, data.base);
            wc.arg("init").arg("--repo").arg(data.path.as_str());
            return wc
        }
    }
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

    macro_rules! mwc {
        (1) => {
            |_| MockWrappedCall::new()
        };
        (2) => {
            |_, _| MockWrappedCall::new()
        };
    }

    macro_rules! earg {
        ($mock:tt, $arg:literal, $count:literal) => {
            $mock
                .expect_arg()
                .with(predicate::eq($arg))
                .times($count)
                .returning(mwc!(1))
        };
    }

    macro_rules! eenv {
        ($mock:tt, $key:literal, $val:literal, $count:literal) => {
            $mock
                .expect_env()
                .with(predicate::eq($key), predicate::eq($val))
                .times($count)
                .returning(mwc!(2))
        };
    }

    #[test]
    fn presence() {
        log_init();
        let mut mock = MockWrappedCall::new();
        earg!(mock, "version", 1);
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
        eenv!(mock, "RESTIC_PASSWORD", "test", 1);
        earg!(mock, "init", 1);
        earg!(mock, "--repo", 1);
        earg!(mock, "/tmp/restic/foo", 1);
        prepare_init(&mut mock, repo);
        panic!();
    }
}
