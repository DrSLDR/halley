//! Statefile manager
//!
//! The Statefile manager contains all the logic involved in creating, maintaining, and
//! reading the Halley state files. It also includes scheduling logic, as it determines
//! which repo will be updated next, if any.

use std::path::PathBuf;
use std::{collections::HashMap, fs};

use crate::config::types::Repo;
use crate::trace_call;

pub(crate) use self::types::{CheckArgs, ErrorKind, StateError, StateStatus};

use tracing::{error, info, warn};

mod types;

/// Entrypoint to state handler
///
/// Opens and reads the statefile (if one exists), checks which repo has had the longest
/// wait since its last update, checks if it needs an update (repeating over the other
/// repos as necessary), and returning the Next repo, if any.
pub(crate) fn check(args: CheckArgs) -> anyhow::Result<StateStatus> {
    trace_call!("check", "called with {:?}", args);

    let statefile = match usable_state_file(&args.statefile) {
        Ok(p) => Ok(p),
        Err(StateError::Internal(ErrorKind::StateFileDoesNotExist)) => {
            if args.dry {
                warn!("DRY RUN: No statefile exists, will not create one, so cannot continue");
                Err(StateError::Internal(ErrorKind::StateFileDoesNotExist))
            } else {
                create_statefile(&args.statefile, &args.config.repositories)
            }
        }
        Err(e) => Err(e),
    }?;

    Ok(StateStatus::NothingToDo)
}

/// Ensures a statefile exists and is readable
fn usable_state_file(path: &PathBuf) -> Result<&PathBuf, StateError> {
    trace_call!("usable_state_file", "called on path {:?}", path);
    match fs::File::open(path) {
        Ok(_) => Ok(path),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                info!("Statefile at {:?} not found", path);
                Err(StateError::Internal(ErrorKind::StateFileDoesNotExist))
            }
            _ => {
                error!("An unhandled IO error occured in trying to open the statefile!");
                Err(StateError::Io(e))
            }
        },
    }
}

/// Create and populate a statefile with some default data
fn create_statefile<'a>(
    path: &'a PathBuf,
    repos: &HashMap<String, Repo>,
) -> Result<&'a PathBuf, StateError> {
    trace_call!(
        "create_statefile",
        "called with path {:?}, repos {:?}",
        path,
        repos
    );
    Ok(path)
}
