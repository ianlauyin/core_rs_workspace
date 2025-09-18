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
        let current_dir = std::env::current_dir()
            .map_err(|err| exception!(message = "failed to get current directory", source = err))?;
        Ok(current_dir.join(self))
    }
}
