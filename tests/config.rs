use std::{env, path::PathBuf, sync::Mutex};

use sandkasten::config;

static LOCK: Mutex<()> = Mutex::new(());

#[test]
fn valid() {
    let _guard = LOCK.lock().unwrap();
    env::set_var("NSJAIL_PATH", "/");
    env::set_var("TIME_PATH", "/");
    env::set_var(
        "CONFIG_PATH",
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml"),
    );
    let conf = config::load().unwrap();
    assert_eq!(conf.nsjail_path, PathBuf::from("/"));
    assert_eq!(conf.time_path, PathBuf::from("/"));
}

#[test]
fn invalid() {
    let _guard = LOCK.lock().unwrap();
    env::set_var(
        "CONFIG_PATH",
        concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
    );
    assert!(config::load().is_err());
}

#[test]
fn not_found() {
    let _guard = LOCK.lock().unwrap();
    env::set_var("CONFIG_PATH", "/does/not/exist.toml");
    assert!(config::load().is_err());
}

#[test]
fn cannot_canonicalize() {
    let _guard = LOCK.lock().unwrap();
    env::set_var("NSJAIL_PATH", "./does/not/exist");
    env::set_var("TIME_PATH", "/");
    env::set_var(
        "CONFIG_PATH",
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml"),
    );
    let err = config::load().unwrap_err();
    assert!(err
        .to_string()
        .starts_with("Failed to resolve `nsjail_path`"));

    env::set_var("NSJAIL_PATH", "/");
    env::set_var("TIME_PATH", "./does/not/exist");
    env::set_var(
        "CONFIG_PATH",
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml"),
    );
    let err = config::load().unwrap_err();
    assert!(err.to_string().starts_with("Failed to resolve `time_path`"));
}

#[test]
fn environments_path_from_env() {
    let _guard = LOCK.lock().unwrap();
    env::set_var("NSJAIL_PATH", "/");
    env::set_var("TIME_PATH", "/");
    env::set_var("ENVIRONMENTS_PATH", "/foo:/bar:/baz");
    env::set_var(
        "CONFIG_PATH",
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml"),
    );
    let conf = config::load().unwrap();
    assert_eq!(conf.nsjail_path, PathBuf::from("/"));
    assert_eq!(conf.time_path, PathBuf::from("/"));
    assert_eq!(
        conf.environments_path,
        [
            PathBuf::from("/foo"),
            PathBuf::from("/bar"),
            PathBuf::from("/baz"),
        ]
    );
}
