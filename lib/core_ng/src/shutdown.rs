use tokio::signal;
use tokio::signal::unix::SignalKind;
use tokio::sync::broadcast::Sender;
use tokio::sync::broadcast::{self};
use tracing::info;

pub struct Shutdown {
    signal: Sender<()>,
}

impl Default for Shutdown {
    fn default() -> Self {
        Shutdown::new()
    }
}

impl Shutdown {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self { signal: tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.signal.subscribe()
    }

    pub fn listen(self) {
        tokio::spawn(async move {
            let ctrl_c = async {
                signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                signal::unix::signal(SignalKind::terminate())
                    .expect("failed to install signal handler")
                    .recv()
                    .await;
            };

            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
            }

            info!("recieved shutdown signal");
            self.signal.send(()).unwrap();
        });
    }
}
