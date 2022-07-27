//! S3 interface
//!
//! This module handles talking to an S3 endpoint, in order to check whether a
//! repository exists, as well as ferrying objects in and out of the Glacier storage
//! class.

#[cfg(test)]
mod tests;

use crate::trace_call;
use crate::types::S3Repo;

use rusoto_core::{credential, Client};
use rusoto_s3::{HeadBucketRequest, S3Client, S3};
use tracing::{error, trace, trace_span};

pub(crate) struct S3Handler {
    _url: String,
    _bucket: String,
    client: S3Client,
}

impl std::fmt::Debug for S3Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("S3Handler")
            .field("url", &self._url)
            .finish()
    }
}

impl S3Handler {
    /// Creates a new [`S3Handler`] which can be used to communicate with a given repository.
    pub fn new(repo: S3Repo) -> S3Handler {
        trace_call!("new", "called with {:?}", repo);
        S3Handler {
            _url: repo.render_full_url(),
            _bucket: repo.bucket,
            client: S3Client::new_with_client(
                Client::new_with(
                    credential::StaticProvider::new_minimal(repo.key.id, repo.key.secret),
                    rusoto_core::HttpClient::new().unwrap(),
                ),
                repo.region,
            ),
        }
    }

    pub async fn bucket_exists(&self) -> bool {
        let response = self
            .client
            .head_bucket(HeadBucketRequest {
                bucket: self._bucket.to_owned(),
                expected_bucket_owner: None,
            })
            .await;
        match response {
            Ok(()) => panic!("BUCKET"),
            Err(e) => panic!("{:?}", e),
        }
    }
}
