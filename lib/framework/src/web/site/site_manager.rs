use super::web_directory::WebDirectory;

pub struct SiteManager {
    pub web_directory: WebDirectory,
}

impl SiteManager {
    pub fn new() -> Self {
        Self {
            web_directory: WebDirectory::new(),
        }
    }
}
