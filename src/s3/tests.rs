use super::*;
use crate::types::*;
use crate::util::test_utils::*;

use rusoto_mock;

#[test]
fn spawn_handler() {
    let _h: S3Handler = match s3_repo_def() {
        Repo::S3 { data } => S3Handler::new(data),
        _ => unimplemented!(),
    };
}
