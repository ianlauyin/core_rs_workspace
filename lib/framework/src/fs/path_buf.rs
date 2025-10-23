use std::env;
use std::fs;
use std::path::PathBuf;

use crate::exception::Exception;

pub trait PathBufExt {
    fn into_absolute_path(self) -> Result<PathBuf, Exception>;
}

impl PathBufExt for PathBuf {
    fn into_absolute_path(self) -> Result<PathBuf, Exception> {
        if self.is_absolute() {
            return Ok(self);
        }
        let current_dir =
            env::current_dir().map_err(|err| exception!(message = "failed to get current directory", source = err))?;

        let absolute_path = current_dir.join(self);
        let cannonical_path = fs::canonicalize(absolute_path)?;

        Ok(cannonical_path)
    }
}
