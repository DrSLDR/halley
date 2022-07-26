use super::*;
use crate::types::*;
use crate::util::test_utils::*;

#[test]
fn spawn_client() {
    match s3_repo_def() {
        Repo::S3 { data } => init(data),
        Repo::Local { data: _ } => unimplemented!(),
    };
}
