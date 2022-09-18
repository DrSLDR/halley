use super::*;
use crate::types::*;
use crate::util::test_utils::*;

use tracing::{error, trace, Level};

fn log_init() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_test_writer()
        .try_init();
}
