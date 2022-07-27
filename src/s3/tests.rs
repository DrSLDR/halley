use super::*;
use crate::types::*;
use crate::util::test_utils::*;

#[test]
fn spawn_client() {
    let _h: S3Handler = match s3_repo_def() {
        Repo::S3 { data } => S3Handler::new(data),
        _ => unimplemented!(),
    };
}
