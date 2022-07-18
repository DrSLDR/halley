use super::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use simulacrum::*;
use tracing::Level;

struct WrappedCallMock {
    e: Expectations,
}

impl WrappedCallMock {
    fn new() -> Self {
        Self {
            e: Expectations::new(),
        }
    }

    fn then(&mut self) -> &mut Self {
        self.e.then();
        self
    }

    fn expect_arg(&mut self) -> Method<String, Self> {
        self.e.expect::<String, Self>("arg")
    }

    fn expect_env(&mut self) -> Method<(String, String), Self> {
        self.e.expect::<(String, String), Self>("env")
    }
}

impl WrappedCall for WrappedCallMock {
    fn invoke(&mut self) -> Result<Output, std::io::Error> {
        error!("Something tried to invoke the mock!");
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "not allowed to do that",
        ))
    }
    fn arg(&mut self, arg: String) -> &mut Self {
        trace!("Adding argument {:?}", arg);
        self.e.was_called::<String, Self>("arg", arg);
        self
    }
    fn env(&mut self, key: String, value: String) -> &mut Self {
        trace!("Adding envvar {:?} = {:?}", key, value);
        self.e
            .was_called::<(String, String), Self>("env", (key, value));
        self
    }
}

fn log_init() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_test_writer()
        .try_init();
}

macro_rules! earg {
    ($mock:tt, $arg:expr) => {
        $mock.then().expect_arg().called_once().with($arg)
    };
}

macro_rules! eenv {
    ($mock:tt, $key:expr, $val:expr) => {
        $mock
            .then()
            .expect_env()
            .called_once()
            .with(params!($key, $val))
    };
}

#[test]
fn presence() {
    log_init();
    let mut mock = WrappedCallMock::new();
    earg!(mock, "version".to_string());
    prepare_present(&mut mock);
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

#[test]
fn init_local() {
    log_init();
    let repo = local_repo_def!("/tmp/restic/foo");
    let mut mock = WrappedCallMock::new();
    eenv!(mock, "RESTIC_PASSWORD".to_string(), "test".to_string());
    earg!(mock, "init".to_string());
    earg!(mock, "--repo".to_string());
    earg!(mock, "/tmp/restic/foo".to_string());
    prepare_init(&mut mock, repo);
}

#[test]
fn init_s3() {
    log_init();
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
    let mut mock = WrappedCallMock::new();
    eenv!(mock, "RESTIC_PASSWORD".to_string(), "test".to_string());
    eenv!(mock, "AWS_ACCESS_KEY_ID".to_string(), "the_id".to_string());
    eenv!(
        mock,
        "AWS_SECRET_ACCESS_KEY".to_string(),
        "the_secret".to_string()
    );
    earg!(mock, "--repo".to_string());
    earg!(
        mock,
        format!(
            "s3:{url}/{bucket}/{path}",
            url = "example.org",
            bucket = "foo",
            path = "bar"
        )
    );
    earg!(mock, "init".to_string());
    prepare_init(&mut mock, repo);
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
