//! Statefile manager
//!
//! The Statefile manager contains all the logic involved in creating, maintaining, and
//! reading the Halley state files. It also includes scheduling logic, as it determines
//! which repo will be updated next, if any.

use self::types::CheckArgs;
pub(crate) use self::types::StateStatus;

mod types;

/// Entrypoint to state handler
///
/// Opens and reads the statefile (if one exists), checks which repo has had the longest
/// wait since its last update, checks if it needs an update (repeating over the other
/// repos as necessary), and returning the Next repo, if any.
pub(crate) fn check(args: CheckArgs) -> anyhow::Result<StateStatus> {
    Ok(StateStatus::NothingToDo)
}
