use anyhow::Result;
use anyhow::anyhow;
use tokio::process::Command;
use tracing::Instrument;
use tracing::debug;
use tracing::debug_span;

pub async fn run(command: &str) -> Result<String> {
    let span = debug_span!("shell", command);

    async {
        let output = Command::new("sh").arg("-c").arg(command).output().await?;
        debug!(status = output.status.code());
        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!(stdout = %stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!(stderr = %stderr);
        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Err(anyhow!("command failed, status={}", output.status.code().unwrap_or(-1)))
        }
    }
    .instrument(span)
    .await
}
