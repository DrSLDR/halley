#[cfg(test)]
mod tests;

mod types;
use crate::restic::types::{AWSKey, RepoCommon, ResticCall, WrappedCall};
pub use crate::restic::types::{LocalRepo, Repo, S3Repo};

use tracing::{debug, debug_span, info, info_span, trace, trace_span};

macro_rules! trace_call {
    ($fn:literal) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!("called");
    };
    ($fn:literal, $estr:literal) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!($estr);
    };
    ($fn:literal, $estr:literal, $($arg:ident),+) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!($estr, $($arg),+);
    };
}

fn prepare_present<C: WrappedCall>(wc: &mut C) -> &mut C {
    trace_call!("prepare_present");
    wc.arg("version".to_string())
}

fn presence() -> bool {
    trace_call!("presence");
    let mut rc = ResticCall::new();
    let rc = prepare_present(&mut rc);
    rc.invoke()
        .expect("restic is not installed (or not on path)");
    true
}

fn prepare_init_common<C: WrappedCall>(wc: &mut C, data: RepoCommon) -> &mut C {
    trace_call!("prepare_init_common", "called with {:?}", data);
    wc.env("RESTIC_PASSWORD".to_string(), data.passwd);
    wc
}

fn prepare_init<C: WrappedCall>(wc: &mut C, repo: Repo) -> &mut C {
    trace_call!("prepare_init", "called with {:?}", repo);

    #[cfg(not(test))]
    assert!(presence());

    match repo {
        Repo::Local { data } => {
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

/// Initializes the repository defined in `repo`
pub fn init(repo: Repo) -> anyhow::Result<()> {
    trace_call!("init", "called with {:?}", repo);
    let mut rc = ResticCall::new();
    let rc = prepare_init(&mut rc, repo);
    match rc.invoke() {
        Ok(_) => Ok(()),
        Err(_e) => todo!("Error handling for init missing!"),
    }
}

fn prepare_backup<C: WrappedCall>(wc: &mut C, repo: Repo) -> &mut C {
    unimplemented!();
}

pub fn backup(repo: Repo) -> anyhow::Result<()> {
    unimplemented!();
}

pub fn forget(repo: Repo) -> anyhow::Result<()> {
    unimplemented!();
}

pub fn prune(repo: Repo) -> anyhow::Result<()> {
    unimplemented!();
}

pub fn stats(repo: Repo) -> anyhow::Result<()> {
    unimplemented!();
}
