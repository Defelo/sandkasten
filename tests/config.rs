use std::env;

use sandkasten::config;

#[test]
fn test_config() {
    env::set_var(
        "CONFIG_PATH",
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml"),
    );
    config::load().unwrap();

    env::set_var(
        "CONFIG_PATH",
        concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
    );
    assert!(config::load().is_err());
}
