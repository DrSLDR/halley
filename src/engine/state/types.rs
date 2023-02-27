//! Types belonging to the statefile processor

use serde::{Deserialize, Serialize};

/// Top-level statefile representation
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct State {
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
pub(crate) struct RepoState {
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
