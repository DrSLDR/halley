//! Engine module
//!
//! The engine contains all business logic for the Halley `run` command, from managing
//! the statefile to S3 pre- and post-work and eventually invoking `restic`.

mod state;

use state::StateStatus;

use crate::config;
use crate::{trace_call, RunSpec};

use anyhow::anyhow;

use tracing::{debug, info};

/// Entrypoint to the Engine
pub(crate) fn run(spec: RunSpec) -> anyhow::Result<()> {
    trace_call!("run", "called with conf {:?}", spec);

    let dry = spec.dry;
    let specific_repo = spec.specific_repo;

    let conf = config::make_and_validate_config(spec.config)?;

    let statefile = {
        let mut state_dir = spec.state_dir;
        state_dir.push(conf.statefile_name.clone());
        state_dir
    };
    debug!("Will check statefile at {:?}", statefile);

    match state::check(state::CheckArgs {
        statefile,
        config: &conf,
        dry,
        specific_repo: &specific_repo,
    }) {
        Ok(StateStatus::NothingToDo) => {
            info!("State manager reported there is nothing to do!");
            Ok(())
        }
        Ok(StateStatus::NextRepo(id)) => {
            info!("State manager reports repository '{}' is up next", id);
            unimplemented!()
        }
        Err(e) => Err(e),
    }
}
