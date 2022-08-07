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

macro_rules! make_s3_client {
    ($rd:expr, $reg:expr) => {
        S3Client::new_with($rd, rusoto_mock::MockCredentialsProvider, $reg.clone())
    };
    ($reg:expr) => {
        make_s3_client!(rusoto_mock::MockRequestDispatcher::default(), $reg)
    };
}

#[tokio::test]
async fn bucket_exists() {
    let repo = get_s3_repo!();
    let client = make_s3_client!(repo.region);
    let h = S3Handler::new_with_client(repo, client);

    assert!(h.bucket_exists().await);
}
