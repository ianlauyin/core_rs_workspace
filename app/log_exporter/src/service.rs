use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use chrono::Datelike;
use chrono::NaiveDate;

use crate::AppState;

pub fn local_file_path(name: &str, date: NaiveDate, state: &Arc<AppState>) -> Result<PathBuf> {
    let dir = &state.log_dir;
    let year = date.year();
    let hash = &state.hash;
    let path = PathBuf::from(format!("{dir}/{name}/{year}/{name}-{date}-{hash}.ndjson"));
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(path)
}

// impl ArchiveService {
//     fn remote_action_log_path(&self, date: NaiveDate) -> String {
//         format!("/action/{}/action-{}-{}.parquet", date.year(), date, self.hash)
//     }

//     fn remote_event_path(&self, date: NaiveDate) -> String {
//         format!("/event/{}/event-{}-{}.parquet", date.year(), date, self.hash)
//     }

//     pub fn local_event_file_path(&self, date: &NaiveDate) -> PathBuf {
//         let path = format!("/event/{}/event-{}-{}.avro", date.year(), date, self.hash);
//         self.log_dir.join(path.trim_start_matches('/'))
//     }

//     pub fn create_parent_dir(&self, path: &Path) -> Result<()> {
//         if let Some(parent) = path.parent() {
//             if !parent.exists() {
//                 fs::create_dir_all(parent)?;
//             }
//         }
//         Ok(())
//     }
// }
