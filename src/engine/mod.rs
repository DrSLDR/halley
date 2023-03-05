//! Engine module
//!
//! The engine contains all business logic for the Halley `run` command, from managing
//! the statefile to S3 pre- and post-work and eventually invoking `restic`.

mod state;

use state::StateStatus;

use crate::config::{self, Config};
use crate::types::*;
use crate::{trace_call, RunSpec};

use anyhow::anyhow;

use tracing::{debug, info};

/// Entrypoint to the Engine
///
/// The first step of Engine operations is to call the `state` handler to check if any
/// of the repositories actually need to be backed up. If nothing has changed, then
/// there's nothing to do, and the engine returns. If there have been changes, however,
/// then the next step will depend on the repository backend.
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
            backup_cycle(id, conf)
        }
        Err(e) => Err(e),
    }
}

/// Run a backup cycle on the repository specified by `id`
fn backup_cycle(id: String, conf: Config) -> anyhow::Result<()> {
    trace_call!("backup_cycle", "called with id {:?}, conf {:?}", id, conf);
    let repo = conf
        .repositories
        .get(&id)
        .expect("Failed to get repository from configuration");
    match &repo.restic {
        Repo::Local { data } => todo!(),
        Repo::S3 { data } => todo!(),
    }
}
