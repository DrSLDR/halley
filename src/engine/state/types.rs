//! Types belonging to the statefile processor

use std::{error::Error, fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Top-level statefile representation
#[derive(Debug, Serialize, Deserialize)]
struct State {
    pub(crate) version: u32,
    pub(crate) states: Vec<RepoState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: 1,
            states: Vec::new(),
        }
    }
}

/// Representation of an individual repository state
#[derive(Debug, Serialize, Deserialize)]
struct RepoState {
    pub(crate) id: String,
    pub(crate) time: u64,
    pub(crate) digest: String,
}

impl Default for RepoState {
    fn default() -> Self {
        Self {
            id: "a_repo".to_string(),
            time: 0,
            digest: "xxx".to_string(),
        }
    }
}

/// Struct used to call the `check` function
///
/// Looks a lot like `RunSpec`, but isn't really
#[derive(Debug)]
pub(crate) struct CheckArgs<'a> {
    pub statefile: PathBuf,
    pub config: &'a Config,
    pub dry: bool,
    pub specific_repo: &'a Option<String>,
}

/// Return Enum from the `check` function
#[derive(Debug)]
pub(crate) enum StateStatus {
    NothingToDo,
    NextRepo(String),
}

/// Special error types related to state management
#[derive(Debug)]
pub(crate) enum StateError {
    Io(std::io::Error),
    Internal(ErrorKind),
}

impl Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            StateError::Internal(ref err) => {
                write!(f, "An internal error occurred: {:?}", err)
            }
            StateError::Io(ref err) => err.fmt(f),
        }
    }
}

impl Error for StateError {}

/// Error flavors
#[derive(Clone, Debug)]
pub(crate) enum ErrorKind {
    StateFileDoesNotExist,
}
