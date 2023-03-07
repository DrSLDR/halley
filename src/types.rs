//! Common types for the entire Halley library

use crate::trace_call;

pub use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};
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

/// Wrapper around a `PathBuf` that is known to exist
///
/// Exists so that we don't have to represent paths as `PathBuf` or, worse, `String`
/// without knowing that they actually exist.
///
/// The guarantee is not absolute --- the path may cease to exist while Halley is
/// running --- but `VerifiedPath` tries its best to ensure the path is still good when
/// returning it.
#[derive(Debug, PartialEq)]
pub struct VerifiedPath {
    path: PathBuf,
}

impl VerifiedPath {
    /// Consumes the `VerifiedPath` to return the `PathBuf`
    ///
    /// This is the only way to mutably access the contained path, as there is no way to
    /// guarantee "verified" status once the path has been mutated.
    pub fn get_inner(self) -> Result<PathBuf, VerifiedPathError> {
        Self::verify_pathbuf(self.path)
    }

    /// Takes a PathBuf and makes damn sure it actually exists
    pub fn from_pathbuf(p: PathBuf) -> Result<Self, VerifiedPathError> {
        Ok(Self {
            path: Self::verify_pathbuf(p)?,
        })
    }

    fn verify_pathbuf(p: PathBuf) -> Result<PathBuf, VerifiedPathError> {
        if !p.is_absolute() {
            Err(VerifiedPathError::NotAbsolute)
        } else if !p.exists() {
            Err(VerifiedPathError::DoesNotExist)
        } else {
            Ok(p)
        }
    }
}

/// Error types for the Verified Path
#[derive(Debug, PartialEq)]
pub enum VerifiedPathError {
    NotAbsolute,
    DoesNotExist,
}

impl Display for VerifiedPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifiedPathError::NotAbsolute => write!(f, "The path was not absolute!"),
            VerifiedPathError::DoesNotExist => {
                write!(f, "The path points to a location that does not exist!")
            }
        }
    }
}
