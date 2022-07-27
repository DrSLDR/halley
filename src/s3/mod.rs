//! S3 interface
//!
//! This module handles talking to an S3 endpoint, in order to check whether a
//! repository exists, as well as ferrying objects in and out of the Glacier storage
//! class.

#[cfg(test)]
mod tests;

use crate::trace_call;
use crate::types::{S3Repo};

use rusoto_core::{credential, Client};
use rusoto_s3::S3Client;
use tracing::{trace, trace_span};

pub(crate) struct S3Handler {
    client: S3Client,
}

impl S3Handler {
    /// Creates a new [`S3Handler`] which can be used to communicate with a given repository.
    pub fn new(repo: S3Repo) -> S3Handler {
        trace_call!("new", "called with {:?}", repo);
        S3Handler {
            client: S3Client::new_with_client(
                Client::new_with(
                    credential::StaticProvider::new_minimal(repo.key.id, repo.key.secret),
                    rusoto_core::HttpClient::new().unwrap(),
                ),
                repo.region,
            ),
        }
    }
}
