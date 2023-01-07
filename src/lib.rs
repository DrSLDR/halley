//! Halley, an offsite backup manager
//!
//! Halley wraps around restic in order to manage when it is called, on what data, and
//! also manages moving the relevant repository in and out of cold storage, for cost
//! saving.

mod config;
mod restic;
mod s3;
mod types;
mod util;

use crate::types::*;

use anyhow::anyhow;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use std::path::PathBuf;
use tracing::{debug, error};

pub async fn test_real() -> anyhow::Result<()> {
    let h = s3::S3Handler::new(S3Repo {
        url: "s3.fr-par.scw.cloud".to_owned(),
        bucket: std::env::var("HALLEY_TEST_BUCKET").unwrap_or("testbucket-2".to_owned()),
        path: match std::env::var("HALLEY_TEST_PATH") {
            Ok(path) => Some(path),
            Err(_) => None,
        },
        region: Region::Custom {
            name: "fr-par".to_owned(),
            endpoint: "s3.fr-par.scw.cloud".to_owned(),
        },
        key: AWSKey {
            id: std::env::var("HALLEY_TEST_KEY").unwrap_or("[redacted]".to_owned()),
            secret: std::env::var("HALLEY_TEST_SECRET").unwrap_or("[redacted]".to_owned()),
        },
        common: RepoCommon {
            passwd: "test".to_owned(),
        },
    });

    h.list_all_objects().await?;
    h.archive_all_objects().await?;
    std::thread::sleep(std::time::Duration::from_secs(15));
    h.restore_all_objects_blocking().await?;

    Ok(())
}

/// Ensures a configuration file exists and is readable
fn usable_config_file(path: PathBuf) -> anyhow::Result<PathBuf> {
    match std::fs::File::open(&path) {
        Ok(_) => Ok(path),
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    error!("The config file at {:?} does not exist", path);
                }
                std::io::ErrorKind::PermissionDenied => {
                    error!("The config file at {:?} could not be opened", path);
                }
                _ => {
                    error!("Unhandled IO error when opening config file!");
                }
            }
            Err(anyhow!("Failed to open config file!"))
        }
    }
}

/// Reads, and validates the configuration at the provided path
pub fn validate_config(path: PathBuf) -> anyhow::Result<config::Config> {
    let c = config::make_and_validate_config(usable_config_file(path)?)?;
    Ok(c)
}

/// Returns the most basic, minimal configuration file
pub fn minimal_config() -> String {
    config::minimal_config()
}

/// Returns the example configuration file, if you don't have it to hand
pub fn example_config() -> String {
    config::example_config()
}

pub mod log {
    use super::*;

    use tracing::Level;
    use tracing_subscriber;

    fn init_logging(level: Level) {
        let sb = tracing_subscriber::fmt().with_max_level(level);
        if level < Level::DEBUG {
            sb.compact().init();
        } else {
            sb.init();
            debug!("Logging started");
        }
    }

    pub fn init_error_logging() {
        init_logging(Level::ERROR);
    }

    pub fn init_warn_logging() {
        init_logging(Level::WARN);
    }

    pub fn init_info_logging() {
        init_logging(Level::INFO);
    }

    pub fn init_debug_logging() {
        init_logging(Level::DEBUG);
    }

    pub fn init_trace_logging() {
        init_logging(Level::TRACE);
    }
}
