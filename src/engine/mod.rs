//! Engine module
//!
//! The engine contains all business logic for the Halley `run` command, from managing
//! the statefile to S3 pre- and post-work and eventually invoking `restic`.

use crate::trace_call;
use crate::types::RunSpec;

use anyhow::anyhow;

use tracing::debug;

pub(crate) fn run(spec: RunSpec) -> anyhow::Result<()> {
    trace_call!("run", "called with spec {:?}", spec);

    Ok(())
}
