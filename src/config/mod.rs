//! Configuration handler
//!
//! This module reads, writes, and generally handles the configuration file.

#[cfg(test)]
mod tests;

pub mod types;
use types::*;

use crate::trace_call;
use crate::types as general;

use std::collections::HashMap;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use tracing::debug;

/// Processes a `ReadConfig` into a valid `Config`
fn validate_config(rc: ReadConfig) -> anyhow::Result<Config> {
    trace_call!("validate_config", "called with rc {:?}", rc);

    let mut repos: HashMap<String, Repo> = HashMap::new();

    let c = Config {
        origin: rc,
        repositories: repos,
    };

    anyhow::Ok(c)
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

    anyhow::Ok(config)
}
