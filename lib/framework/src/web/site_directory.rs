use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

use crate::exception::Exception;
use crate::fs::path_buf::PathBufExt;

pub struct SiteDirectory {
    pub root: PathBuf,
}

impl SiteDirectory {
    pub fn new(path_str: &str) -> Result<Self, Exception> {
        Ok(Self {
            root: resolve_valid_directory(path_str)?,
        })
    }

    pub fn service(&self) -> ServeDir<ServeFile> {
        let root = self.root.clone();
        let serve_file = ServeFile::new(root.join("index.html"));
        ServeDir::new(root).fallback(serve_file)
    }
}

impl Default for SiteDirectory {
    fn default() -> Self {
        Self {
            root: resolve_valid_directory("./src/main/dist/web").unwrap(),
        }
    }
}

fn resolve_valid_directory(path: &str) -> Result<PathBuf, Exception> {
    let path = PathBuf::from(path).into_absolute_path().unwrap();

    if path.is_dir() {
        info!("found web directory, path={}", path.to_string_lossy());
        return Ok(path);
    }

    Err(exception!(
        message = format!("web directory not found, path={}", path.to_string_lossy())
    ))
}
