use std::fmt::Debug;
use std::fs::read_to_string;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use serde::Serialize;
use serde::de::Deserialize;
use serde::de::DeserializeOwned;

pub fn load_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let json = read_to_string(path).with_context(|| format!("failed to read file, path={}", path.to_string_lossy()))?;
    serde_json::from_str(&json).with_context(|| format!("failed to deserialize, json={json}"))
}

pub fn from_json<'a, T>(json: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    serde_json::from_str(json).with_context(|| format!("failed to deserialize, json={json}"))
}

pub fn to_json<T>(object: &T) -> Result<String>
where
    T: Serialize + Debug,
{
    serde_json::to_string(object).with_context(|| format!("failed to serialize, object={object:?}"))
}

pub fn to_json_value<T>(enum_value: &T) -> String
where
    T: Serialize + Debug,
{
    if let Ok(value) = serde_json::to_string(enum_value) {
        value[1..value.len() - 1].to_string()
    } else {
        Default::default()
    }
}
