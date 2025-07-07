use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use chrono::Datelike;
use chrono::NaiveDate;
use core_ng::shell;
use tracing::info;

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

pub fn cleanup_archive(date: NaiveDate, state: &Arc<AppState>) -> Result<()> {
    info!("cleaning up archives, date={date}");

    let action_log_path = local_file_path("action", date, state)?;
    if action_log_path.exists() {
        fs::remove_file(&action_log_path)?;
    }

    let event_path = local_file_path("event", date, state)?;
    if event_path.exists() {
        fs::remove_file(&event_path)?;
    }

    Ok(())
}

pub async fn upload_archive(date: NaiveDate, state: &Arc<AppState>) -> Result<()> {
    info!("uploading archives, date={date}");

    let action_log_path = local_file_path("action", date, state)?;
    if action_log_path.exists() {
        let local_path = action_log_path.to_string_lossy();
        let parquet_path_buf = action_log_path.with_extension("parquet");
        let parquet_path = parquet_path_buf.to_string_lossy();

        let command = &format!(
            r#"SET memory_limit = '256MB';SET temp_directory = '/tmp/duckdb';COPY (SELECT * FROM read_ndjson( ['{local_path}'],
            columns = {{'date': 'TIMESTAMPTZ', id: 'STRING', app: 'STRING', host: 'STRING', result: 'STRING', action: 'STRING', ref_ids: 'STRING[]', correlation_ids: 'STRING[]', clients: 'STRING[]', error_code: 'STRING', error_message: 'STRING', elapsed: 'LONG', context: 'MAP(STRING, STRING[])', stats: 'MAP(STRING, DOUBLE)', perf_stats: 'MAP(STRING, MAP(STRING, DOUBLE))'}}
        )) TO '{parquet_path}' (FORMAT 'parquet');"#
        );
        shell::run(&format!("duckdb -c \"{command}\"")).await?;

        let remote_path = remote_path("action", date, state);

        // requires '-q', otherwise standard output may block if buffer is full, Shell reads std after process ends
        // and '-m' may stress network bandwidth, currently not really necessary
        let bucket = &state.bucket;
        let command = format!("gcloud storage cp --quiet cp {parquet_path} gs://{bucket}{remote_path}",);
        shell::run(&command).await?;

        fs::remove_file(parquet_path_buf)?;
    }

    let event_path = local_file_path("event", date, state)?;
    if event_path.exists() {
        let local_path = event_path.to_string_lossy();
        let parquet_path_buf = event_path.with_extension("parquet");
        let parquet_path = parquet_path_buf.to_string_lossy();

        let command = &format!(
            r#"SET memory_limit = '256MB';SET temp_directory = '/tmp/duckdb';COPY (SELECT * FROM read_ndjson( ['{local_path}'],
            columns = {{'date': 'TIMESTAMPTZ', id: 'STRING', app: 'STRING', received_time: 'TIMESTAMPTZ', result: 'STRING', action: 'STRING', error_code: 'STRING', error_message: 'STRING', elapsed: 'LONG', context: 'MAP(STRING, STRING)', stats: 'MAP(STRING, DOUBLE)', info: 'MAP(STRING, STRING)'}}
        )) TO '{parquet_path}' (FORMAT 'parquet');"#
        );
        shell::run(&format!("duckdb -c \"{command}\"")).await?;

        let remote_path = remote_path("event", date, state);

        // requires '-q', otherwise standard output may block if buffer is full, Shell reads std after process ends
        // and '-m' may stress network bandwidth, currently not really necessary
        let bucket = &state.bucket;
        let command = format!("gcloud storage cp --quiet cp {parquet_path} gs://{bucket}{remote_path}",);
        shell::run(&command).await?;

        fs::remove_file(parquet_path_buf)?;
    }

    Ok(())
}

fn remote_path(name: &str, date: NaiveDate, state: &Arc<AppState>) -> String {
    let year = date.year();
    let hash = &state.hash;
    format!("/{name}/{year}/{name}-{date}-{hash}.parquet")
}
