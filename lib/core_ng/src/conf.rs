use std::path::Path;

use anyhow::Result;
use serde::de::DeserializeOwned;
use tracing::info;

use crate::json::load_file;

pub fn load_conf<T>() -> Result<T>
where
    T: DeserializeOwned + Default,
{
    let mut args = std::env::args();
    if let Some(arg) = args.nth(1) {
        info!(path = arg, "load conf");
        let config: T = load_file(Path::new(&arg))?;
        Ok(config)
    } else {
        info!("no conf path, using default value");
        Ok(T::default())
    }
}
