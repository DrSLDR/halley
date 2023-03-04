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

#[test]
#[ignore]
fn integration_create_statefile_no_repos() {
    log_init();
    let _tdir = assert_fs::TempDir::new().unwrap();
    let dir = PathBuf::from(_tdir.as_os_str());
    let fpath = dir.join("sample.state");
    let repos: HashMap<String, Repo> = HashMap::new();
    let r = create_statefile(&fpath, &repos);
    assert!(r.is_ok());
    let state = r.unwrap();
    let read_state: State = toml::from_slice(fs::read(fpath).unwrap().as_slice()).unwrap();
    assert_eq!(state, read_state);
}
