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
    let c = ReadConfig::default();
    let c_string = toml::to_string_pretty(&c).unwrap();
    let cf = assert_fs::NamedTempFile::new("test.toml").unwrap();
    let mut cf_handle = File::create(cf.path()).unwrap();
    cf_handle.write_all(c_string.as_bytes()).unwrap();
    cf_handle.flush().unwrap();
    drop(cf_handle);
    let c_parsed = make_config(cf.path().to_string_lossy().to_string()).unwrap();
    assert_eq!(c, c_parsed);
}

macro_rules! figment_read {
    ($s:ident) => {
        Figment::new().join(Toml::string($s)).extract().unwrap()
    };
}

#[test]
#[should_panic]
fn no_repo_validation() {
    log_init();
    let toml_string = "version = 1
    statefile_name = 'foo'";
    let _: ReadConfig = figment_read!(toml_string);
}

#[test]
#[should_panic]
fn no_backend_validation() {
    let toml_string = "version = 1
    statefile_name = 'foo'
    [[repositories]]
    id = 'a'
    paths = ['/home']
    password = 'b'";
    let _: ReadConfig = toml::from_str(&toml_string).unwrap();
}
