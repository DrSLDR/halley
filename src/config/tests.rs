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
    let c_parsed = make_config(cf.to_path_buf()).unwrap();
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
    statefile_name = 'anchor'";
    let _: ReadConfig = figment_read!(toml_string);
}

#[test]
#[should_panic]
fn no_backend_validation() {
    log_init();
    let toml_string = "version = 1
    statefile_name = 'anchor'
    [[repositories]]
    id = 'scrapbook'
    paths = ['/home']
    password = 'unopened'";
    let _: ReadConfig = figment_read!(toml_string);
}

#[test]
fn no_path_validation() {
    log_init();
    let toml_string = "version = 1
    statefile_name = 'anchor'
    [[repositories]]
    id = 'scrapbook'
    paths = []
    password = 'unopened'
    [repositories.backend.local]
    path = '/tmp'";
    let rc: ReadConfig = figment_read!(toml_string);
    let c = validate_config(rc);
    assert!(c.is_err());
}

#[test]
fn single_validation() {
    log_init();
    let toml_string = "version = 1
    statefile_name = 'anchor'
    [[repositories]]
    id = 'scrapbook'
    paths = ['/home']
    password = 'unopened'
    [repositories.backend.local]
    path = '/tmp'";
    let rc: ReadConfig = figment_read!(toml_string);
    let c = validate_config(rc);
    assert!(c.is_ok());
    let c = c.unwrap();
    assert!(c.repositories.contains_key("scrapbook"));
}

#[test]
fn single_validation_s3_no_bucket() {
    log_init();
    let toml_string = "version = 1
    statefile_name = 'anchor'
    [[repositories]]
    id = 'scrapbook'
    paths = ['/home']
    password = 'unopened'
    [repositories.backend.s3]
    bucket = 'human'";
    let rc: ReadConfig = figment_read!(toml_string);
    let c = validate_config(rc);
    assert!(c.is_err());
}

#[test]
fn single_validation_s3() {
    log_init();
    let toml_string = "version = 1
    statefile_name = 'anchor'
    [[repositories]]
    id = 'scrapbook'
    paths = ['/home']
    password = 'unopened'
    [repositories.backend.s3]
    bucket = 'human'
    [[s3_buckets]]
    id = 'human'
    endpoint = 's3.eu-west-1.amazonaws.com'
    region = 'eu-west-1'
    bucket_name = 'feasibly'
    [s3_buckets.credentials]
    id = 'upkeep'
    secret = 'lightbulb'";
    let rc: ReadConfig = figment_read!(toml_string);
    let c = validate_config(rc);
    assert!(c.is_ok());
    let c = c.unwrap();
    assert!(c.repositories.contains_key("scrapbook"))
}
