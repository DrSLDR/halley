use crate::types::{VerifiedPath, VerifiedPathError};

use super::*;

use assert_fs;

#[test]
fn verified_path_dir() {
    let td = assert_fs::TempDir::new().unwrap();
    let p = PathBuf::from(td.as_os_str());
    let v = VerifiedPath::from_pathbuf(p.clone());
    assert!(v.is_ok());
    drop(td);
    let v = VerifiedPath::from_pathbuf(p);
    assert!(v.is_err());
    assert_eq!(v.err().unwrap(), VerifiedPathError::DoesNotExist);
}
