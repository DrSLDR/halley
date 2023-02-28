//! Statefile manager
//!
//! The Statefile manager contains all the logic involved in creating, maintaining, and
//! reading the Halley state files. It also includes scheduling logic, as it determines
//! which repo will be updated next, if any.

use std::fs;
use std::path::PathBuf;

use crate::trace_call;

pub(crate) use self::types::{CheckArgs, ErrorKind, StateError, StateStatus};

use tracing::{error, info};

mod types;

/// Entrypoint to state handler
///
/// Opens and reads the statefile (if one exists), checks which repo has had the longest
/// wait since its last update, checks if it needs an update (repeating over the other
/// repos as necessary), and returning the Next repo, if any.
pub(crate) fn check(args: CheckArgs) -> anyhow::Result<StateStatus> {
    trace_call!("check", "called with {:?}", args);

    let statefile = match usable_state_file(args.statefile) {
        Ok(p) => Ok(p),
        Err(StateError::Internal(ErrorKind::StateFileDoesNotExist)) => {
            error!("The statefile does not exist; will need to create and populate");
            unimplemented!()
        }
        Err(e) => Err(e),
    }?;

    Ok(StateStatus::NothingToDo)
}

/// Ensures a statefile exists and is readable
fn usable_state_file(path: PathBuf) -> Result<PathBuf, StateError> {
    trace_call!("usable_state_file", "called on path {:?}", path);
    match fs::File::open(&path) {
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
