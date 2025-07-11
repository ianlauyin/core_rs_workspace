use std::path::Path;

use serde::de::DeserializeOwned;
use tracing::info;

use crate::exception::Exception;
use crate::json::load_file;

pub fn load_conf<T>() -> Result<T, Exception>
where
    T: DeserializeOwned + Default,
{
    let mut args = std::env::args();
    if let Some(arg) = args.nth(1)
        && !arg.is_empty()
    {
        info!(path = arg, "load conf");
        let config: T = load_file(Path::new(&arg))?;
        Ok(config)
    } else {
        info!("no conf path, using default value");
        Ok(T::default())
    }
}
