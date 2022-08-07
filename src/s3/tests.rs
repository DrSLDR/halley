use super::*;
use crate::types::*;
use crate::util::test_utils::*;

use rusoto_mock::*;

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
        S3Client::new_with($rd, MockCredentialsProvider, $reg.clone())
    };
    ($reg:expr) => {
        make_s3_client!(MockRequestDispatcher::default(), $reg)
    };
}

macro_rules! s3h {
    ($rd:expr, $repo:expr) => {
        S3Handler::new_with_client($repo, make_s3_client!($rd, $repo.region))
    };
    ($rd:expr) => {
        s3h!($rd, get_s3_repo!())
    };
    () => {
        s3h!(MockRequestDispatcher::default(), get_s3_repo!())
    };
}

#[tokio::test]
async fn bucket_exists() {
    let h = s3h!();
    assert!(h.bucket_exists().await);

    let rd = MockRequestDispatcher::with_status(403);
    let h = s3h!(rd);
    assert!(!h.bucket_exists().await);

    let rd = MockRequestDispatcher::with_status(404);
    let h = s3h!(rd);
    assert!(!h.bucket_exists().await);
}
