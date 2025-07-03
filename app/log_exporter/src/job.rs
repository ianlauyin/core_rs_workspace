use std::sync::Arc;

use anyhow::Result;
use core_ng::schedule::JobContext;

use crate::AppState;

pub async fn process_log_job(_state: Arc<AppState>, context: JobContext) -> Result<()> {
    println!("daily executed: {}", context.name);
    Ok(())
}
