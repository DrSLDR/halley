//! S3 interface
//!
//! This module handles talking to an S3 endpoint, in order to check whether a
//! repository exists, as well as ferrying objects in and out of the Glacier storage
//! class.

#[cfg(test)]
mod tests;

use crate::trace_call;
use crate::types::{Region, S3Repo};

use rusoto_core::{credential, Client};
use rusoto_s3::S3Client;
use tracing::{trace, trace_span};

/// Creates a [`S3Client`] which can be used to communicate with a given repository.
///
/// [`S3Client`]: rusoto_s3::S3Client
fn init(repo: S3Repo) -> S3Client {
    trace_call!("init", "called with {:?}", repo);
    S3Client::new_with_client(
        Client::new_with(
            credential::StaticProvider::new_minimal(repo.key.id, repo.key.secret),
            rusoto_core::HttpClient::new().unwrap(),
        ),
        repo.region,
    )
}

pub(crate) struct S3Handler {
    client: S3Client,
}

impl S3Handler {
    pub fn new(repo: S3Repo) -> S3Handler {
        unimplemented!()
    }
}
