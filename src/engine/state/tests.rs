use super::*;
use crate::util::test_utils::*;

#[test]
fn usable_state_file_null_path() {
    log_init();
    let p = PathBuf::from("");
    let r = usable_state_file(&p);
    assert!(r.is_err());
}

#[test]
fn usable_state_file_dir_path() {
    log_init();
    let p = PathBuf::from("/tmp");
    let r = usable_state_file(&p);
    assert!(r.is_err());
}
