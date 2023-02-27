//! Types belonging to the statefile processor

use serde::{Deserialize, Serialize};

/// Top-level statefile representation
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct State {
    pub(crate) version: u32,
    pub(crate) states: Vec<RepoState>,
}

/// Representation of an individual repository state
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RepoState {
    pub(crate) id: String,
    pub(crate) time: u64,
    pub(crate) digest: String,
}
