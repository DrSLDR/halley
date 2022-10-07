//! Configuration handler
//!
//! This module reads, writes, and generally handles the configuration file.

#[cfg(test)]
mod tests;

pub mod types;
use types::StorageBackend::*;
use types::*;

use crate::trace_call;
use crate::types as general;

use std::collections::HashMap;
use std::str::FromStr;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use tracing::{debug, error, info, warn};

/// Processes a `ReadConfig` into a valid `Config`
fn validate_config(rc: ReadConfig) -> anyhow::Result<Config> {
    trace_call!("validate_config", "called with rc {:?}", rc);

    let mut buckets: HashMap<String, PartialBucket> = HashMap::new();

    if rc.s3_buckets.is_some() {
        for bucket in rc.s3_buckets.as_ref().unwrap() {
            debug!("Processing bucket {:?}", bucket);
            let key = bucket.id.clone();
            buckets.insert(key, process_bucket(&bucket));
        }
    }
    debug!("Mapped S3 buckets: {:?}", buckets);

    let mut repos: HashMap<String, Repo> = HashMap::new();

    for repo in &rc.repositories {
        debug!("Processing repo {:?}", repo);
        let key = repo.id.clone();
        repos.insert(key, process_repo(repo, &mut buckets)?);
    }
    debug!("Mapped repositories: {:?}", repos);

    for (k, v) in buckets.iter() {
        if !v.used {
            warn!("Bucket {} is defined but never used!", k);
        };
    }

    let c = Config {
        origin: rc,
        repositories: repos,
    };

    info!(
        "Validated configuration with {} repositories",
        c.repositories.len()
    );

    anyhow::Ok(c)
}

/// Processes a single bucket configuration
fn process_bucket(b: &BucketConfig) -> PartialBucket {
    let region = match general::Region::from_str(&b.region) {
        Ok(region) => region,
        Err(_) => general::Region::Custom {
            name: b.region.clone(),
            endpoint: b.endpoint.clone(),
        },
    };
    PartialBucket {
        bucket: b.bucket_name.clone(),
        endpoint: b.endpoint.clone(),
        region,
        key: b.credentials.clone(),
        used: false,
    }
}

/// Processes a single repo configuration
fn process_repo(
    r: &RepoConfig,
    buckets: &mut HashMap<String, PartialBucket>,
) -> anyhow::Result<Repo> {
    trace_call!("process_repo", "called with {:?}", r);

    let common = general::RepoCommon {
        passwd: r.password.clone(),
    };

    match &r.backend {
        dummy => {
            error!("Repository {} is using the dummy backend!", &r.id);
            Err(anyhow::Error::msg("Dummy backends not validatable!"))
        }
        local(data) => Ok(Repo {
            restic: general::Repo::Local {
                data: general::LocalRepo {
                    path: data.path.clone(),
                    common,
                },
            },
        }),
        s3(data) => match buckets.get_mut(&data.bucket) {
            Some(bucket) => {
                bucket.used = true;
                Ok(Repo {
                    restic: general::Repo::S3 {
                        data: general::S3Repo {
                            bucket: bucket.bucket.clone(),
                            url: bucket.endpoint.clone(),
                            region: bucket.region.clone(),
                            path: data.prefix.clone(),
                            key: bucket.key.clone(),
                            common,
                        },
                    },
                })
            }
            None => {
                error!(
                    "Repository {} references unknown bucket {}",
                    &r.id, data.bucket
                );
                Err(anyhow::Error::msg("Repository reference to unknown bucket"))
            }
        },
    }
}

/// Collects a Config from the available sources
///
/// The sources in use are the provided toml configuration file as well as the
/// environment.
///
/// This Config could be internally inconsistent, so validation is needed before it is
/// used.
pub(crate) fn make_config(toml_path: String) -> anyhow::Result<ReadConfig> {
    trace_call!("make_config", "called with conf. file {}", toml_path);
    let figment = Figment::new()
        .merge(Toml::file(&toml_path))
        .merge(Env::prefixed("HALLEY_"));
    debug!("Raw configuration figment: {:?}", figment);
    let config: ReadConfig = figment.extract()?;
    debug!("Pre-validation configuration: {:#?}", config);

    Ok(config)
}
