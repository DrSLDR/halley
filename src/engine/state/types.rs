//! Types belonging to the statefile processor

use std::path::PathBuf;

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
pub(crate) struct CheckArgs {
    statefile: PathBuf,
    config: Config,
    dry: bool,
    specific_repo: Option<String>,
}

/// Return Enum from the `check` function
pub(crate) enum StateStatus {
    NothingToDo,
    NextRepo(String),
}
