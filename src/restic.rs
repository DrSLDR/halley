use anyhow;
use std::process::{Command, Output};
use tracing::{debug, debug_span, info, info_span, trace};

#[cfg(test)]
use mockall::{automock, predicate};
#[cfg(test)]
use simulacrum as sim;

#[cfg_attr(test, automock)]
trait WrappedCall {
    fn invoke(&mut self) -> Result<Output, std::io::Error>;
    fn arg(&mut self, arg: String) -> &mut Self;
    fn env(&mut self, key: String, value: String) -> &mut Self;
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
    fn arg(&mut self, arg: String) -> &mut Self {
        self.cmd.arg(arg);
        self
    }
    fn env(&mut self, key: String, value: String) -> &mut Self {
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
    wc.arg("version".to_string())
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
    let span = info_span!("repo base config");
    let _enter = span.enter();
    debug!("Setting repo base config as {:?}", data);
    wc.env("RESTIC_PASSWORD".to_string(), data.passwd);
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
            info!("Initializing local repo at {}", data.path);
            let wc = prepare_init_base(wc, data.base).arg("init".to_string());
            // .arg("--repo");
            // .arg(data.path.as_str());
            return wc;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulacrum::*;
    use tracing::Level;

    create_mock_struct! {
        struct WrappedCallMock: {
            expect_arg("arg") String => Self;
            expect_env("env") (String, String) => Self;
        }
    }

    impl WrappedCall for WrappedCallMock {
        fn invoke(&mut self) -> Result<Output, std::io::Error> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "not allowed to do that",
            ))
        }
        fn arg(&mut self, arg: String) -> &mut Self {
            self.e.was_called_returning("arg", arg)
        }
        fn env(&mut self, key: String, value: String) -> &mut Self {
            self.e.was_called_returning("env", (key, value))
        }
    }

    // impl WrappedCallMock {
    //     pub fn new() -> Self {
    //         Self {
    //             e: sim::Expectations::new()
    //         }
    //     }
    // }

    fn log_init() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_test_writer()
            .try_init();
    }

    // macro_rules! earg {
    //     ($mock:tt, $arg:tt, $count:literal) => {
    //         $mock
    //             .expect_arg()
    //             .with(predicate::eq($arg))
    //             .times($count)
    //             .return_var(MockWrappedCall::new());
    //     };
    // }

    // macro_rules! eenv {
    //     ($mock:tt, $key:literal, $val:literal, $count:literal) => {
    //         $mock
    //             .expect_env()
    //             .with(predicate::eq($key), predicate::eq($val))
    //             .times($count)
    //             .return_var(MockWrappedCall::new())
    //     };
    // }

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
                base,
            },
        };
        let mut mock = MockWrappedCall::new();
        // eenv!(mock, ("RESTIC_PASSWORD".to_string()), ("test".to_string()), 1);
        // earg!(mock, "init".to_string(), 1);
        // earg!(mock, "--repo", 1);
        // earg!(mock, "/tmp/restic/foo", 1);
        prepare_init(&mut mock, repo);
    }
}
