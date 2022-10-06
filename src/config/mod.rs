//! Configuration handler
//!
//! This module reads, writes, and generally handles the configuration file.

#[cfg(test)]
mod tests;

pub mod types;
use types::*;

use crate::trace_call;
use crate::types::*;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use tracing::debug;

fn validate_config(cfg: ReadConfig) -> anyhow::Result<Config> {
    unimplemented!();
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
