//! Halley, an offsite backup manager
//!
//! Halley wraps around restic in order to manage when it is called, on what data, and
//! also manages moving the relevant repository in and out of cold storage, for cost
//! saving.

mod restic;
mod s3;
mod types;
mod util;

use crate::types::*;

pub async fn test_real() -> anyhow::Result<()> {
    let h = s3::S3Handler::new(S3Repo {
        url: "s3.fr-par.scw.cloud".to_owned(),
        bucket: std::env::var("HALLEY_TEST_BUCKET").unwrap_or("testbucket-2".to_owned()),
        path: match std::env::var("HALLEY_TEST_PATH") {
            Ok(path) => Some(path),
            Err(_) => None,
        },
        region: Region::Custom {
            name: "fr-par".to_owned(),
            endpoint: "s3.fr-par.scw.cloud".to_owned(),
        },
        key: AWSKey {
            id: std::env::var("HALLEY_TEST_KEY").unwrap_or("[redacted]".to_owned()),
            secret: std::env::var("HALLEY_TEST_SECRET").unwrap_or("[redacted]".to_owned()),
        },
        common: RepoCommon {
            passwd: "test".to_owned(),
        },
    });

    h.list_all_objects().await?;
    h.archive_all_objects().await?;
    std::thread::sleep(std::time::Duration::from_secs(15));
    h.restore_all_objects_blocking().await?;

    Ok(())
}
