//! Common types for the entire Halley library

pub use rusoto_core::Region;

// First off, the entire restic group of Repo types.

/// Container for an AWS key pair
#[derive(Debug)]
pub(crate) struct AWSKey {
    /// The `AWS_ACCESS_KEY_ID` portion
    pub(crate) id: String,
    /// The `AWS_SECRET_ACCESS_KEY` portion
    pub(crate) secret: String,
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


/// Enum for the two (supported) repository types
#[derive(Debug)]
pub enum Repo {
    Local { data: LocalRepo },
    S3 { data: S3Repo },
}
