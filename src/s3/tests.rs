use super::*;
use crate::types::*;
use crate::util::test_utils::*;

use rusoto_mock;

macro_rules! get_s3_repo {
    () => {
        match s3_repo_def() {
            Repo::S3 { data } => data,
            _ => unimplemented!(),
        }
    };
}

#[test]
fn spawn_handler() {
    let repo = get_s3_repo!();
    let _h: S3Handler = S3Handler::new(repo);
}

#[tokio::test]
async fn bucket_exists() {
    let cp = rusoto_mock::MockCredentialsProvider;
    let rd = rusoto_mock::MockRequestDispatcher::default();
    let repo = get_s3_repo!();
    let client = S3Client::new_with(rd, cp, repo.region.clone());
    let h = S3Handler::new_with_client(repo, client);

    assert!(h.bucket_exists().await);
}
