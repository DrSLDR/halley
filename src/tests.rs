use crate::{
    types::{VerifiedPath, VerifiedPathError},
    util::test_utils::log_init,
};

use super::*;

use std::fs;

use assert_fs;

#[test]
fn verified_path_dir() {
    log_init();
    let td = assert_fs::TempDir::new().unwrap();
    let p = PathBuf::from(td.as_os_str());
    let v = VerifiedPath::from_pathbuf(p.clone());
    assert!(v.is_ok());
    drop(td);
    let v = VerifiedPath::from_pathbuf(p);
    assert!(v.is_err());
    assert_eq!(v.err().unwrap(), VerifiedPathError::DoesNotExist);
}

#[test]
fn verified_path_file() {
    log_init();
    let td = assert_fs::TempDir::new().unwrap();
    let p = PathBuf::from(td.as_os_str());
    let fp = p.join("afile");
    let v = VerifiedPath::from_pathbuf(fp.clone());
    assert_eq!(v.err().unwrap(), VerifiedPathError::DoesNotExist);
    let _f = fs::File::create(&fp).unwrap();
    let v = VerifiedPath::from_pathbuf(fp.clone());
    assert!(v.is_ok());
    drop(_f);
    drop(td);
    let v = VerifiedPath::from_pathbuf(fp);
    assert_eq!(v.err().unwrap(), VerifiedPathError::DoesNotExist);
}
