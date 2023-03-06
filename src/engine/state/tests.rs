use std::str::FromStr;

use super::{types::HexDigest, *};
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

#[test]
fn hexdigest_from_str_null() {
    log_init();
    let h = HexDigest::from_str("").unwrap();
    let v: Vec<u8> = vec![];
    assert_eq!(h.get(), &v);
}

#[test]
fn hexdigest_from_str_simple() {
    log_init();
    let h = HexDigest::from_str("0");
    assert!(h.is_err());
    let h = HexDigest::from_str("0v");
    assert!(h.is_err());
    let h = HexDigest::from_str("00").unwrap();
    let v: Vec<u8> = vec![0];
    assert_eq!(h.get(), &v);
    let v: Vec<u8> = vec![0, 1];
    assert_ne!(h.get(), &v);
    let h = HexDigest::from_str("0001").unwrap();
    assert_eq!(h.get(), &v);
}

#[test]
fn hexdigest_from_str_complex() {
    log_init();
    let h = HexDigest::from_str("a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a")
        .unwrap();
    let v: Vec<u8> = vec![
        0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61, 0xd6,
        0x62, 0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b, 0x80, 0xf8,
        0x43, 0x4a,
    ];
    assert_eq!(h.get(), &v);
}

#[test]
fn hexdigest_to_from_string() {
    log_init();
    let s = String::from("");
    assert_eq!(s, HexDigest::from_str(&s).unwrap().to_string());
    let s = String::from("c0ffee");
    assert_eq!(s, HexDigest::from_str(&s).unwrap().to_string());
}

#[test]
fn toml_parse_to_hexdigest() {
    log_init();
    let s = String::from(
        "
    version = 1
    [states.foo]
    time = 1
    digest = 'c0ffee'
    ",
    );
    let state: State = toml::from_str(&s).unwrap();
    assert_eq!(
        HexDigest::from_str("c0ffee").unwrap(),
        state.states.get("foo").unwrap().digest
    );
}
