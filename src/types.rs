//! Common types for the entire Halley library

use crate::trace_call;

use figment::{providers::Toml, Figment};
pub use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use tracing::{trace, trace_span};

// First off, the entire restic group of Repo types.

/// Container for an AWS key pair
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AWSKey {
    /// The `AWS_ACCESS_KEY_ID` portion
    pub(crate) id: String,
    /// The `AWS_SECRET_ACCESS_KEY` portion
    pub(crate) secret: String,
}

impl Default for AWSKey {
    fn default() -> Self {
        Self {
            id: "id".to_string(),
            secret: "secret".to_string(),
        }
    }
}

/// Container for data common to any type of repository
#[derive(Debug)]
pub(crate) struct RepoCommon {
    /// Encryption password for the repository
    pub(crate) passwd: String,
}

/// Definition for a local `restic` repository
#[derive(Debug)]
pub struct LocalRepo {
    /// Path to the repository on disk
    pub path: String,
    pub(crate) common: RepoCommon,
}

/// Definition for a `restic` repository stored in S3
#[derive(Debug)]
pub struct S3Repo {
    /// URL to the endpoint, not including protocol specifier
    pub url: String,
    /// Bucket name
    pub bucket: String,
    /// Region definition, using `rusoto_core::Region` enum
    pub region: Region,
    /// An optional path. Without a path, the repository resides in the root of the
    /// bucket.
    pub path: Option<String>,
    pub(crate) key: AWSKey,
    pub(crate) common: RepoCommon,
}

impl S3Repo {
    /// Returns the full URL string of the repository
    pub fn render_full_url(&self) -> String {
        trace_call!("render_full_url");
        match &self.path {
            Some(path) => format!(
                "{url}/{bucket}/{path}",
                url = self.url,
                bucket = self.bucket,
                path = path
            ),
            None => format!("{url}/{bucket}", url = self.url, bucket = self.bucket),
        }
    }
}

/// Enum for the two (supported) repository types
#[derive(Debug)]
pub enum Repo {
    Local { data: LocalRepo },
    S3 { data: S3Repo },
}

// Then, the configuration file types

/// Top-level configuration struct
///
/// Contains all the configuration for Halley
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    statefile_name: String,
    s3_buckets: Option<Vec<BucketConfig>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            statefile_name: "halley".to_string(),
            s3_buckets: Some(vec![BucketConfig::default()]),
        }
    }
}

/// Configuration for an S3 bucket
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BucketConfig {
    id: String,
    endpoint: Option<String>,
    region: String,
    bucket_name: String,
    credentials: AWSKey,
}

impl Default for BucketConfig {
    fn default() -> Self {
        Self {
            id: "a_bucket".to_string(),
            endpoint: Some("s3.example.org".to_string()),
            region: "eu-west-1".to_string(),
            bucket_name: "foo".to_string(),
            credentials: AWSKey::default(),
        }
    }
}
