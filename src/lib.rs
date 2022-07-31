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
        bucket: "testbucket-2".to_owned(),
        path: Some("bar".to_owned()),
        region: Region::Custom {
            name: "fr-par".to_owned(),
            endpoint: "s3.fr-par.scw.cloud".to_owned(),
        },
        key: AWSKey {
            id: "[redacted]".to_owned(),
            secret: "[redacted]".to_owned(),
        },
        common: RepoCommon {
            passwd: "test".to_owned(),
        },
    });

    h.list_all_items().await?;
    match h.get_storage_class("foo/config".to_owned()).await? {
        s3::StorageClass::STANDARD => h.archive_object("foo/config".to_owned()).await?,
        s3::StorageClass::GLACIER => h.restore_object("foo/config".to_owned()).await?,
    }

    Ok(())
}
