//! Statefile manager
//!
//! The Statefile manager contains all the logic involved in creating, maintaining, and
//! reading the Halley state files. It also includes scheduling logic, as it determines
//! which repo will be updated next, if any.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::{collections::HashMap, fs};

use crate::config::types::Repo;
use crate::trace_call;

pub(crate) use self::types::{CheckArgs, ErrorKind, RepoState, State, StateError, StateStatus};

use toml;
use tracing::{error, info, warn};

mod types;

/// Entrypoint to state handler
///
/// Opens and reads the statefile (if one exists), checks which repo has had the longest
/// wait since its last update, checks if it needs an update (repeating over the other
/// repos as necessary), and returning the Next repo, if any.
pub(crate) fn check(args: CheckArgs) -> anyhow::Result<StateStatus> {
    trace_call!("check", "called with {:?}", args);

    let state = match usable_state_file(&args.statefile) {
        Ok(p) => open_statefile(p, &args.config.repositories),
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
                Err(StateError::from(e))
            }
        },
    }
}

/// Create and populate a statefile with some default data
fn create_statefile<'a>(
    path: &'a PathBuf,
    repos: &HashMap<String, Repo>,
) -> Result<State, StateError> {
    trace_call!(
        "create_statefile",
        "called with path {:?}, repos {:?}",
        path,
        repos
    );

    let mut state = State::default();

    for (id, _) in repos.iter() {
        state.states.insert(id.clone(), RepoState::default());
    }

    let mut file = fs::File::create(path)?;
    file.write_all(toml::to_string_pretty(&state).unwrap().as_bytes())?;

    info!(
        "Initialized statefile with {} repositories",
        state.states.len()
    );

    Ok(state)
}

/// Opens and reads a statefile
///
/// Also verifies that all the configured repositories are present
fn open_statefile<'a>(
    path: &'a PathBuf,
    repos: &HashMap<String, Repo>,
) -> Result<State, StateError> {
    trace_call!(
        "open_statefile",
        "called with path {:?}, repos {:?}",
        path,
        repos
    );

    let data = fs::read(path)?;
    let mut state: State = toml::from_slice(&data)?;
    info!("Read statefile with {} repositories", state.states.len());

    // Check if all configured states are present
    for (id, _) in repos.iter() {
        if !state.states.contains_key(id) {
            warn!(
                "Repository {} was not present in statefile, adding it...",
                id
            );
            state.states.insert(id.clone(), RepoState::default());
        }
    }

    // Check if any unconfigured states are present
    for (id, _) in state.states.iter() {
        if !repos.contains_key(id) {
            warn!(
                "Repository {} has a state, but is not in configuration!",
                id
            );
        }
    }

    Ok(state)
}
