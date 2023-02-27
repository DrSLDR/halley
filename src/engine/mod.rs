//! Engine module
//!
//! The engine contains all business logic for the Halley `run` command, from managing
//! the statefile to S3 pre- and post-work and eventually invoking `restic`.

mod state;

use crate::config;
use crate::{trace_call, RunSpec};

use anyhow::anyhow;

use tracing::debug;

/// Entrypoint to the Engine
pub(crate) fn run(spec: RunSpec) -> anyhow::Result<()> {
    trace_call!("run", "called with conf {:?}", spec);

    let dry = spec.dry;
    let specific_repo = spec.specific_repo;
    let mut state_dir = spec.state_dir;
    let conf = config::make_and_validate_config(spec.config)?;

    state_dir.push(conf.statefile_name);

    Ok(())
}
