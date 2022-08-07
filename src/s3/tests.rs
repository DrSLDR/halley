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
