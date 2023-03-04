//! Statefile manager
//!
//! The Statefile manager contains all the logic involved in creating, maintaining, and
//! reading the Halley state files. It also includes scheduling logic, as it determines
//! which repo will be updated next, if any.

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::{collections::HashMap, fs, time};

use crate::config::types::Repo;
use crate::config::Config;
use crate::trace_call;

pub(crate) use self::types::{CheckArgs, ErrorKind, RepoState, State, StateError, StateStatus};

use dasher;
use glob;
use shellexpand;
use toml;
use tracing::{debug, error, info, warn};

mod types;

/// Entrypoint to state handler
///
/// Opens and reads the statefile (if one exists), checks which repo has had the longest
/// wait since its last update, checks if it needs an update (repeating over the other
/// repos as necessary), and returning the Next repo, if any.
pub(crate) fn check(args: CheckArgs) -> anyhow::Result<StateStatus> {
    trace_call!("check", "called with {:?}", args);

    // Local bindings
    let statefile = args.statefile;
    let config = args.config;
    let specific_repo = args.specific_repo;
    let dry = args.dry;

    let mut state = match usable_state_file(&statefile) {
        Ok(p) => open_statefile(p, &config.repositories),
        Err(StateError::Internal(ErrorKind::StateFileDoesNotExist)) => {
            if args.dry {
                warn!("DRY RUN: No statefile exists, will not create one, so cannot continue");
                Err(StateError::Internal(ErrorKind::StateFileDoesNotExist))
            } else {
                create_statefile(&statefile, &config.repositories)
            }
        }
        Err(e) => Err(e),
    }?;

    let status = next_up(&mut state, config, specific_repo)?;

    if dry {
        warn!("DRY RUN: Will not update state file on disk!")
    } else {
        write_statefile(&statefile, &state)?;
    }

    Ok(status)
}

/// Ensures a statefile exists and is readable
fn usable_state_file(path: &PathBuf) -> Result<&PathBuf, StateError> {
    trace_call!("usable_state_file", "called on path {:?}", path);
    if !path.is_file() {
        return Err(StateError::Internal(ErrorKind::BadPath));
    }
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

    write_statefile(path, &state)?;

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
                "Repository {:?} was not present in statefile, adding it...",
                id
            );
            state.states.insert(id.clone(), RepoState::default());
        }
    }

    // Check if any unconfigured states are present
    for (id, _) in state.states.iter() {
        if !repos.contains_key(id) {
            warn!(
                "Repository {:?} has a state, but is not in configuration!",
                id
            );
        }
    }

    Ok(state)
}

/// Write out the statefile
fn write_statefile<'a>(path: &'a PathBuf, state: &'a State) -> Result<(), StateError> {
    trace_call!(
        "write_statefile",
        "called with path {:?}, state {:?}",
        path,
        state
    );

    fs::write(path, toml::to_string_pretty(state)?)?;
    Ok(())
}

/// Given a state, returns the repo that's waited the longest for an update
///
/// If given a specific_repo, short circuits comparison and just checks the specified
/// repo
fn next_up(
    state: &mut State,
    config: &Config,
    specific: &Option<String>,
) -> Result<StateStatus, StateError> {
    trace_call!(
        "next_up",
        "called with state {:?}, config {:?}, specific repo {:?}",
        state,
        config,
        specific
    );
    let state_repos = &state.states;
    let config_repos = &config.repositories;
    match specific {
        Some(id) => {
            if config_repos.contains_key(id) {
                info!("Short-circuiting on repository {}", id);
                if needs_update(id.to_string(), &mut state.states, &config_repos)? {
                    Ok(StateStatus::NextRepo(id.to_string()))
                } else {
                    Ok(StateStatus::NothingToDo)
                }
            } else {
                warn!(
                    "Repository with id {:?} is not defined in configuration",
                    id
                );
                Ok(StateStatus::NothingToDo)
            }
        }
        None => {
            debug!("No specific repository defined, so checking them in order");
            let mut ids = config_repos.keys().collect::<Vec<&String>>();
            ids.sort_unstable_by(|a, b| {
                state_repos
                    .get(*a)
                    .unwrap()
                    .time
                    .cmp(&state_repos.get(*b).unwrap().time)
            });
            debug!("Sorted repository IDs: {:#?}", ids);
            for id in ids {
                if needs_update(id.to_string(), &mut state.states, &config_repos)? {
                    return Ok(StateStatus::NextRepo(id.to_string()));
                }
            }
            Ok(StateStatus::NothingToDo)
        }
    }
}

/// Checks the given repo to see if it needs updating
///
/// Calculates the recursive directory hash on the paths, then checks that against the
/// digest kept in the state. Returns `true` if the repo needs to be updated, `false`
/// otherwise.
fn needs_update(
    id: String,
    state: &mut HashMap<String, RepoState>,
    config: &HashMap<String, Repo>,
) -> Result<bool, StateError> {
    trace_call!(
        "needs_update",
        "called with id {:?}, repo-state {:?}, config {:?}",
        id,
        state,
        config
    );

    let repo_state = state.get_mut(&id).unwrap();
    let repo_paths = &config.get(&id).unwrap().paths;
    debug!(
        "Got the relevant repo-state {:?} and paths {:?}",
        repo_state, repo_paths
    );

    let mut paths: Vec<PathBuf> = Vec::new();
    for path in repo_paths {
        let expanded = shellexpand::tilde(path).into_owned();
        for globbed in glob::glob(&expanded).expect("Failed to read glob pattern") {
            match globbed {
                Ok(p) => {
                    paths.push(p);
                    Ok(())
                }
                Err(e) => Err(e),
            }?;
        }
    }
    debug!("Glob-expanded to the paths: {:#?}", paths);

    debug!("Checking digest of repo {:?}, calling dasher...", id);
    let need = dash_check_and_update(repo_state, paths)?;
    if need {
        info!(
            "Digest mismatch - Repository {:?} is in need of an update!",
            id
        );
    } else {
        info!(
            "Digest match - Repository {:?} is NOT in need of an update",
            id
        );
    }
    Ok(need)
}

/// Calls to `dasher` and updates the passed-in `RepoState` accordingly
fn dash_check_and_update(state: &mut RepoState, paths: Vec<PathBuf>) -> Result<bool, StateError> {
    trace_call!(
        "dash_check_and_update",
        "called with state {:?}, paths {:?}",
        state,
        paths
    );

    let current_digest = state.digest.clone();
    let new_digest = dasher::hash_directories(paths)?
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let timestamp = time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .expect("EPOCH ERROR")
        .as_secs();
    debug!(
        "Got digests:\nFrom state:  {}\nFrom dasher: {}",
        current_digest, new_digest
    );
    state.digest = new_digest.clone();
    state.time = timestamp;
    debug!("Updated repository state to {:?}", state);
    Ok(current_digest != new_digest)
}
