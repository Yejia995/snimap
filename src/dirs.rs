use std::env;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    env::current_dir()
        .expect("Failed to determine the current directory")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn hosts_path() -> Option<PathBuf> {
    let path = if cfg!(windows) {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    } else {
        PathBuf::from("/etc/hosts")
    };
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
