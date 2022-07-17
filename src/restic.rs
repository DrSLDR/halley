use anyhow;
use std::process::{Command, Output};
use tracing::{debug, debug_span, error, info, info_span, trace};

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
        trace!("Adding argument {:?}", arg);
        self.cmd.arg(arg);
        self
    }
    fn env(&mut self, key: String, val: String) -> &mut Self {
        trace!("Adding envvar {:?} = {:?}", key, val);
        self.cmd.env(key, val);
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

fn presence() -> bool {
    let mut rc = ResticCall::new();
    let rc = prepare_present(&mut rc);
    rc.invoke()
        .expect("restic is not installed (or not on path)");
    true
}

#[derive(Debug)]
pub(crate) struct RepoCommon {
    passwd: String,
}

#[derive(Debug)]
pub(crate) struct AWSKey {
    id: String,
    secret: String,
}

#[derive(Debug)]
pub(crate) struct LocalRepo {
    path: String,
    common: RepoCommon,
}

#[derive(Debug)]
pub(crate) struct S3Repo {
    url: String,
    bucket: String,
    region: String,
    path: Option<String>,
    key: AWSKey,
    common: RepoCommon,
}

#[derive(Debug)]
pub(crate) enum Repo {
    Local { data: LocalRepo },
    S3 { data: S3Repo },
}

fn prepare_init_common<C: WrappedCall>(wc: &mut C, data: RepoCommon) -> &mut C {
    let span = info_span!("repo common config");
    let _enter = span.enter();
    debug!("Setting repo common config as {:?}", data);
    wc.env("RESTIC_PASSWORD".to_string(), data.passwd);
    wc
}

fn prepare_init<C: WrappedCall>(wc: &mut C, repo: Repo) -> &mut C {
    let span = info_span!("repo init");
    let _enter = span.enter();

    #[cfg(not(test))]
    assert!(presence());

    debug!("Initializing repo with {:?}", repo);

    match repo {
        Repo::Local { data } => {
            info!("Initializing local repo at {}", data.path);
            prepare_init_common(wc, data.common)
                .arg("init".to_string())
                .arg("--repo".to_string())
                .arg(data.path);
        }
        Repo::S3 { data } => {
            let url = match data.path {
                Some(path) => format!(
                    "{url}/{bucket}/{path}",
                    url = data.url,
                    bucket = data.bucket,
                    path = path
                ),
                None => format!("{url}/{bucket}", url = data.url, bucket = data.bucket),
            };
            info!("Initializing S3 repo at {}", url);
            prepare_init_common(wc, data.common)
                .env("AWS_ACCESS_KEY_ID".to_string(), data.key.id)
                .env("AWS_SECRET_ACCESS_KEY".to_string(), data.key.secret)
                .arg("init".to_string())
                .arg("--repo".to_string())
                .arg(format!("s3:{url}"));
        }
    }
    wc
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use mockall::predicate::*;
    use mockall::*;
    use tracing::Level;

    struct WCall {}

    mock! {
        WCall {}
        impl WrappedCall for WCall {
            fn invoke(&mut self) -> Result<Output, std::io::Error>;
            fn arg(&mut self, arg: String) -> &mut Self;
            fn env(&mut self, key: String, value: String) -> &mut Self;
        }
    }

    #[derive(Debug)]
    struct ArgCall {
        arg: String,
    }

    #[derive(Debug)]
    struct EnvCall {
        key: String,
        value: String,
    }

    #[derive(Debug)]
    enum CallChainLink {
        Arg { data: ArgCall },
        Env { data: EnvCall },
    }

    macro_rules! chain_arg {
        ($arg:expr) => {
            CallChainLink::Arg {
                data: ArgCall {
                    arg: $arg.to_string(),
                },
            }
        };
    }

    macro_rules! chain_env {
        ($key:expr, $value:expr) => {
            CallChainLink::Env {
                data: EnvCall {
                    key: $key.to_string(),
                    value: $value.to_string(),
                }
            }
        }
    }

    fn construct_call_chain(ops: Vec<CallChainLink>) -> MockWCall {
        let mut chain_link = MockWCall::new();
        for op in ops.into_iter().rev() {
            chain_link = {
                let mut mock = MockWCall::new();
                match op {
                    CallChainLink::Arg { data } => {
                        mock.expect_arg()
                            .once()
                            .with(predicate::eq(data.arg))
                            .return_var(chain_link);
                    }
                    CallChainLink::Env { data } => {
                        mock.expect_env()
                            .once()
                            .with(predicate::eq(data.key), predicate::eq(data.value))
                            .return_var(chain_link);
                    }
                };
                mock
            }
        }
        chain_link
    }

    fn log_init() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn presence() {
        log_init();
        let mut ops: Vec<CallChainLink> = Vec::new();
        ops.push(chain_arg!("version"));
        let mut cc = construct_call_chain(ops);
        prepare_present(&mut cc);
    }

    macro_rules! common_repo_def {
        () => {
            RepoCommon {
                passwd: "test".to_string(),
            }
        };
    }

    macro_rules! local_repo_def {
        ($name:expr) => {
            Repo::Local {
                data: LocalRepo {
                    path: $name.to_string(),
                    common: common_repo_def!(),
                },
            }
        };
    }

    // #[test]
    // fn init_local() {
    //     log_init();
    //     let repo = local_repo_def!("/tmp/restic/foo");
    //     let mut mock_init = MockWCall::new();
    //     mock_init.expect_arg().once().with(predicate::eq("init".to_string())).return_var(MockWCall::new());
    //     // eenv!(mock, "RESTIC_PASSWORD".to_string(), "test".to_string());
    //     // mallarg!(mock, "init".to_string());
    //     // mallarg!(mock, "--repo".to_string());
    //     // mallarg!(mock, "/tmp/restic/foo".to_string());
    //     prepare_init(&mut mock_init, repo);
    // }

    // #[test]
    // fn init_s3() {
    //     log_init();
    //     let repo = Repo::S3 {
    //         data: S3Repo {
    //             url: "example.org".to_string(),
    //             bucket: "foo".to_string(),
    //             region: "eu-west-1".to_string(),
    //             path: Some("bar".to_string()),
    //             key: AWSKey {
    //                 id: "the_id".to_string(),
    //                 secret: "the_secret".to_string(),
    //             },
    //             common: common_repo_def!(),
    //         },
    //     };
    //     let mut mock = WrappedCallMock::new();
    //     eenv!(mock, "RESTIC_PASSWORD".to_string(), "test".to_string());
    //     eenv!(mock, "AWS_ACCESS_KEY_ID".to_string(), "the_id".to_string());
    //     eenv!(
    //         mock,
    //         "AWS_SECRET_ACCESS_KEY".to_string(),
    //         "the_secret".to_string()
    //     );
    //     earg!(mock, "--repo".to_string());
    //     earg!(
    //         mock,
    //         format!(
    //             "s3:{url}/{bucket}/{path}",
    //             url = "example.org",
    //             bucket = "foo",
    //             path = "bar"
    //         )
    //     );
    //     earg!(mock, "init".to_string());
    //     prepare_init(&mut mock, repo);
    // }

    // #[test]
    // #[ignore]
    // fn integration_init() {
    //     log_init();
    //     let temp = assert_fs::TempDir::new().unwrap();
    //     let repo = local_repo_def!(temp.path().to_string_lossy());
    //     let mut com = ResticCall::new();
    //     prepare_init(&mut com, repo);
    //     debug!("Call: {:?}", com);
    //     com.invoke()
    //         .expect("Failed to invoke restic to init a repo");
    //     temp.child("config").assert(predicate::path::is_file());
    // }
}
