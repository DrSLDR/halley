//! Common types for the entire Halley library

use crate::trace_call;

pub use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{trace, trace_span};

// First off, the entire restic group of Repo types.

/// Container for an AWS key pair
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

/// Struct for feeding in the information needed to invoke Halley
#[derive(Debug)]
pub struct RunSpec {
    pub dry: bool,
    pub specific_repo: Option<String>,
    pub config: PathBuf,
    pub state_dir: PathBuf,
}
