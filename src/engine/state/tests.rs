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
    let fpath = PathBuf::from(_tdir.as_os_str()).join("sample.state");
    let repos: HashMap<String, Repo> = HashMap::new();
    let r = create_statefile(&fpath, &repos);
    assert!(r.is_ok());
    let state = r.unwrap();
    let read_state: State = toml::from_str(&fs::read_to_string(&fpath).unwrap()).unwrap();
    assert_eq!(state, read_state);
}

#[test]
#[ignore]
fn integration_create_statefile_one_repo_three_ways() {
    log_init();
    let _tdir = assert_fs::TempDir::new().unwrap();
    let fpath = PathBuf::from(_tdir.as_os_str()).join("sample.state");
    let mut repos: HashMap<String, Repo> = HashMap::new();
    repos.insert(
        "beef".to_string(),
        Repo {
            paths: vec![fpath.to_string_lossy().to_string()],
            restic: local_repo_def("beef"),
        },
    );
    let mut state: State = State::default();
    let repo = RepoState::default();
    state.states.insert("beef".to_string(), repo);

    let create_state = create_statefile(&fpath, &repos).unwrap();
    assert_eq!(state, create_state);

    let read_state: State = toml::from_str(&fs::read_to_string(&fpath).unwrap()).unwrap();
    assert_eq!(state, read_state);
    assert_eq!(read_state, create_state);

    write_statefile(&fpath, &state).unwrap();
    let read_state: State = toml::from_str(&fs::read_to_string(&fpath).unwrap()).unwrap();
    assert_eq!(state, read_state);
    assert_eq!(read_state, create_state);
}
