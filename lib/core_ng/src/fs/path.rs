use std::path::Path;

use crate::error::Exception;

pub trait PathExt {
    fn file_extension(&self) -> Result<&str, Exception>;
}

impl PathExt for Path {
    fn file_extension(&self) -> Result<&str, Exception> {
        self.extension()
            .ok_or_else(|| exception!(message = format!("file must have extension, path={}", self.to_string_lossy())))?
            .to_str()
            .ok_or_else(|| exception!(message = format!("path is invalid, path={}", self.to_string_lossy())))
    }
}
