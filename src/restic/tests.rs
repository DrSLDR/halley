use super::*;
use types::*;

use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Output;
use tracing::{error, trace, Level};

fn log_init() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_test_writer()
        .try_init();
}

/*
    TEST BLOCK FOR THE HANDCRAFTED MOCK

    Writing tests for the tests. What a joke.
*/

macro_rules! mc {
    () => {{
        log_init();
        MockCall::new()
    }};
}

#[test]
fn mock_empty() {
    let mut m = mc!();
    m.assert_empty();
}

#[test]
#[should_panic]
fn mock_failing_empty() {
    let mut m = mc!();
    m.arg("foo".to_string());
    m.assert_empty();
}

#[test]
fn mock_arg() {
    let mut m = mc!();
    m.arg("foo".to_string());
    m.assert_arg_s("foo");
    m.assert_empty();
}

#[test]
#[should_panic]
fn mock_arg_assertion() {
    let mut m = mc!();
    m.assert_arg_s("foo");
}

#[test]
fn mock_multiarg() {
    let mut m = mc!();
    m.arg("foo".to_string());
    m.arg("bar".to_string());
    m.arg("baz".to_string());
    m.assert_arg_s("foo");
    m.assert_arg_s("baz");
    m.assert_arg_s("bar");
    m.assert_empty();
}

#[test]
fn mock_env() {
    let mut m = mc!();
    m.env("foo".to_string(), "bar".to_string());
    m.assert_env_s("foo", "bar");
    m.assert_empty();
}

#[test]
#[should_panic]
fn mock_env_assertion() {
    let mut m = mc!();
    m.assert_env_s("key", "value");
}

#[test]
#[should_panic]
fn mock_env_disorder() {
    let mut m = mc!();
    m.env("foo".to_string(), "bar".to_string());
    m.assert_env_s("bar", "foo");
    m.assert_empty();
}

/*
    TEST BLOCK FOR THE ACTUAL CODE
*/

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

#[test]
fn presence() {
    let mut m = mc!();
    prepare_present(&mut m);
    m.assert_arg_s("version");
    m.assert_empty();
}

#[test]
fn init_local() {
    let mut m = mc!();
    let repo = local_repo_def!("/tmp/restic/foo");
    prepare_init(&mut m, repo);

    m.assert_env_s("RESTIC_PASSWORD", "test");
    m.assert_arg_s("init");
    m.assert_arg_s("--repo");
    m.assert_arg_s("/tmp/restic/foo");
    m.assert_empty();
}

#[test]
fn init_s3() {
    let mut m = mc!();
    let repo = Repo::S3 {
        data: S3Repo {
            url: "example.org".to_string(),
            bucket: "foo".to_string(),
            region: "eu-west-1".to_string(),
            path: Some("bar".to_string()),
            key: AWSKey {
                id: "the_id".to_string(),
                secret: "the_secret".to_string(),
            },
            common: common_repo_def!(),
        },
    };
    prepare_init(&mut m, repo);

    m.assert_env_s("RESTIC_PASSWORD", "test");
    m.assert_env_s("AWS_ACCESS_KEY_ID", "the_id");
    m.assert_env_s("AWS_SECRET_ACCESS_KEY", "the_secret");
    m.assert_arg_s("init");
    m.assert_arg_s("--repo");
    m.assert_arg(format!(
        "s3:{url}/{bucket}/{path}",
        url = "example.org",
        bucket = "foo",
        path = "bar"
    ));
    m.assert_empty();
}

#[test]
#[ignore]
fn integration_init() {
    log_init();
    let temp = assert_fs::TempDir::new().unwrap();
    let repo = local_repo_def!(temp.path().to_string_lossy());
    let mut com = ResticCall::new();
    prepare_init(&mut com, repo);
    debug!("Call: {:?}", com);
    com.invoke()
        .expect("Failed to invoke restic to init a repo");
    temp.child("config").assert(predicate::path::is_file());
}

mod types {
    use super::*;

    pub struct MockCall {
        args: Vec<String>,
        envs: Vec<(String, String)>,
    }

    impl WrappedCall for MockCall {
        fn invoke(&mut self) -> Result<Output, std::io::Error> {
            error!("Someone tried to do the naughty!");
            panic!("The mocked call should not be invoked!");
        }
        fn arg(&mut self, arg: String) -> &mut Self {
            trace!("Mocked arg: {:?}", arg);
            self.args.push(arg);
            self
        }
        fn env(&mut self, key: String, value: String) -> &mut Self {
            trace!("Mocked env: {:?} = {:?}", key, value);
            self.envs.push((key, value));
            self
        }
    }

    impl MockCall {
        pub fn new() -> MockCall {
            MockCall {
                args: Vec::new(),
                envs: Vec::new(),
            }
        }

        /// Asserts that there are no (more) arguments or environment variables to evaluate
        ///
        /// Returns `true` if both vectors are empty, `panic!()` otherwise.
        pub fn assert_empty(&mut self) -> bool {
            trace!("Asserting MockCall is empty");
            if self.args.is_empty() && self.envs.is_empty() {
                true
            } else {
                panic!(
                    "Args or envvars remain!\nArgs: {:?}\nEnvs: {:?}\n",
                    self.args, self.envs
                );
            }
        }

        /// Asserts that `arg` has been called with the given argument
        pub fn assert_arg(&mut self, arg: String) -> bool {
            trace!("Asserting MockCall arg {:?}", arg);
            match self.args.iter().position(|s| s.eq(&arg)) {
                None => panic!("Arg {:?} not called (argv: {:?})", arg, self.args),
                Some(p) => {
                    self.args.remove(p);
                    true
                }
            }
        }

        /// Helper function for calling with string literals
        pub fn assert_arg_s(&mut self, arg: &str) -> bool {
            self.assert_arg(arg.to_owned())
        }

        /// Asserts that `env` has been called with the given environment variable
        pub fn assert_env(&mut self, key: String, value: String) -> bool {
            trace!("Asserting MockCall env {:?} = {:?}", key, value);
            match self
                .envs
                .iter()
                .position(|(k, v)| k.eq(&key) && v.eq(&value))
            {
                None => panic!(
                    "Env {:?}:{:?} not called (envv: {:?})",
                    key, value, self.envs
                ),
                Some(p) => {
                    self.envs.remove(p);
                    true
                }
            }
        }

        /// Helper function for calling with string literals
        pub fn assert_env_s(&mut self, key: &str, value: &str) -> bool {
            self.assert_env(key.to_owned(), value.to_owned())
        }
    }
}
