use std::path::PathBuf;
use tracing::info;

pub struct WebDirectory {
    pub root: Option<PathBuf>,
}

impl WebDirectory {
    pub fn new() -> Self {
        Self {
            root: locate_root_directory(),
        }
    }
}

fn locate_root_directory() -> Option<PathBuf> {
    let Ok(path_str) = std::env::var("WEB_PATH") else {
        return find_local_root_directory();
    };

    let path = PathBuf::from(&path_str);
    if path.is_dir() {
        info!("found WEB_PATH as web directory, path={}", path.to_string_lossy());
        return Some(path);
    }

    info!("can not locate web directory");
    return None;
}

fn find_local_root_directory() -> Option<PathBuf> {
    let path = PathBuf::from("./src/main/dist/web");

    if path.is_dir() {
        info!("found local web directory, path={}", path.to_string_lossy());
        return Some(path);
    }

    return None;
}
