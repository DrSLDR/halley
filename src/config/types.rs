//! Types belonging to the configuration processor

use crate::types as general;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Top-level configuration struct as read from a configuration file
///
/// Contains all the configuration for Halley, prior to it being processed and
/// validated. Defines the structure of the `toml` configuration file.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadConfig {
    version: u32,
    statefile_name: String,
    s3_buckets: Option<Vec<BucketConfig>>,
    repositories: Vec<RepoConfig>,
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self {
            version: 1,
            statefile_name: "halley".to_string(),
            s3_buckets: Some(vec![BucketConfig::default()]),
            repositories: vec![RepoConfig::default()],
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
    credentials: general::AWSKey,
}

impl Default for BucketConfig {
    fn default() -> Self {
        Self {
            id: "a_bucket".to_string(),
            endpoint: Some("s3.example.org".to_string()),
            region: "eu-west-1".to_string(),
            bucket_name: "foo".to_string(),
            credentials: general::AWSKey::default(),
        }
    }
}

/// Configuration for a repository
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoConfig {
    id: String,
    paths: Vec<String>,
    password: String,
    backend: StorageBackend,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            id: "a_repo".to_string(),
            paths: vec!["/home".to_string()],
            password: "foo".to_string(),
            backend: StorageBackend::dummy,
        }
    }
}

/// Allowed storage backends
#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageBackend {
    /// Used for readback tests only, since enums can't be serialized
    dummy,
    local(LocalStorageBackend),
    s3(S3StorageBackend),
}

/// Local storage backend
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalStorageBackend {
    path: String,
}

/// S3 Backend configuration
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct S3StorageBackend {
    bucket: String,
    prefix: Option<String>,
}

/// Top-level validated configuration struct
///
/// This is Halley's internal configuration object, which will be used to inform how it
/// runs.
#[derive(Debug)]
pub struct Config {
    pub(crate) origin: ReadConfig,
    pub(crate) repositories: HashMap<String, Repo>,
}

/// Validated configuration for a repo
///
/// Contains both local and remote (restic) components
#[derive(Debug)]
pub struct Repo {}
