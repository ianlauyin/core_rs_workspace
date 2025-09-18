use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

use crate::fs::path_buf::PathBufExt;

pub struct SiteDirectory {
    pub root: Option<PathBuf>,
}

impl SiteDirectory {
    pub fn new(path_str: &str) -> Self {
        Self {
            root: resolve_valid_directory(path_str),
        }
    }

    pub fn into_service(&self) -> Option<ServeDir<ServeFile>> {
        let Some(root) = &self.root else {
            return None;
        };

        let serve_file = ServeFile::new(root.join("index.html"));
        Some(ServeDir::new(root).fallback(serve_file))
    }
}

impl Default for SiteDirectory {
    fn default() -> Self {
        Self {
            root: resolve_valid_directory("./src/main/dist/web"),
        }
    }
}

fn resolve_valid_directory(path: &str) -> Option<PathBuf> {
    let path = PathBuf::from(path).into_absolute_path().unwrap();

    if path.is_dir() {
        info!("found web directory, path={}", path.to_string_lossy());
        return Some(path);
    }

    info!("can not locate web directory");
    return None;
}
