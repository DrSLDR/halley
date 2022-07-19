#[cfg(test)]
mod tests;

mod types;
use crate::restic::types::{AWSKey, RepoCommon, ResticCall, WrappedCall};
pub use crate::restic::types::{LocalRepo, Repo, S3Repo};

use tracing::{debug, debug_span, info, info_span};

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

pub fn init(repo: Repo) -> anyhow::Result<()> {
    let mut rc = ResticCall::new();
    let rc = prepare_init(&mut rc, repo);
    match rc.invoke() {
        Ok(_) => Ok(()),
        Err(_e) => todo!("Error handling for init missing!"),
    }
}
