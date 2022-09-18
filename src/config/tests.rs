use super::*;
use crate::types::*;
use crate::util::test_utils::*;

use assert_fs::prelude::*;
use std::{fs::File, io::Write};
use tracing::{error, trace};

#[test]
#[ignore]
fn integration_default_readback() {
    log_init();
    let c = Config::default();
    let c_string = toml::to_string_pretty(&c).unwrap();
    let cf = assert_fs::NamedTempFile::new("test.toml").unwrap();
    let mut cf_handle = File::create(cf.path()).unwrap();
    cf_handle.write_all(c_string.as_bytes()).unwrap();
    cf_handle.flush().unwrap();
    drop(cf_handle);
    let c_parsed = make_config(cf.path().to_string_lossy().to_string()).unwrap();
    assert_eq!(c, c_parsed);
}
