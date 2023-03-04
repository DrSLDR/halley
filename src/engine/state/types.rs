//! Types belonging to the statefile processor

use std::{collections::HashMap, error::Error, fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Top-level statefile representation
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct State {
    pub(crate) version: u32,
    pub(crate) states: HashMap<String, RepoState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: 1,
            states: HashMap::new(),
        }
    }
}

/// Representation of an individual repository state
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RepoState {
    pub(crate) time: u64,
    pub(crate) digest: String,
}

impl Default for RepoState {
    fn default() -> Self {
        Self {
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
    Glob(glob::GlobError),
    IO(std::io::Error),
    TOMLD(toml::de::Error),
    TOMLS(toml::ser::Error),
    Dasher(dasher::HashError),
    Internal(ErrorKind),
}

impl Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            StateError::Internal(ref err) => {
                write!(f, "An internal error occurred: {:?}", err)
            }
            StateError::Glob(ref err) => err.fmt(f),
            StateError::IO(ref err) => err.fmt(f),
            StateError::TOMLD(ref err) => err.fmt(f),
            StateError::TOMLS(ref err) => err.fmt(f),
            StateError::Dasher(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for StateError {
    fn from(err: std::io::Error) -> StateError {
        StateError::IO(err)
    }
}

impl From<toml::de::Error> for StateError {
    fn from(err: toml::de::Error) -> StateError {
        StateError::TOMLD(err)
    }
}

impl From<toml::ser::Error> for StateError {
    fn from(err: toml::ser::Error) -> StateError {
        StateError::TOMLS(err)
    }
}

impl From<glob::GlobError> for StateError {
    fn from(err: glob::GlobError) -> StateError {
        StateError::Glob(err)
    }
}

impl From<dasher::HashError> for StateError {
    fn from(err: dasher::HashError) -> StateError {
        StateError::Dasher(err)
    }
}

impl Error for StateError {}

/// Error flavors
#[derive(Clone, Debug)]
pub(crate) enum ErrorKind {
    StateFileDoesNotExist,
}
