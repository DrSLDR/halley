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

#[test]
fn make_and_parse_default() {
    log_init();
    let c = Config::default();
    let c_s = toml::to_string(&c).unwrap();
    let c_p = parse_config(&c_s);
    assert_eq!(c, c_p);
}
