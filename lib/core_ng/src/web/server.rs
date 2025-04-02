use std::time::SystemTime;

use anyhow::Result;
use axum::Router;
use axum::extract::MatchedPath;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use chrono::DateTime;
use chrono::Utc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::Instrument;
use tracing::info;
use tracing::info_span;
use tracing::trace;
use uuid::Uuid;

pub async fn start_http_server(router: Router, mut shutdown_signal: broadcast::Receiver<()>) -> Result<()> {
    let app = Router::new();
    let app = app.route("/health-check", get(health_check));
    let app = app.merge(router);
    let app = app.layer(middleware::from_fn(trace_layer));

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    info!("http server stated");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal.recv().await.unwrap();
        })
        .await?;
    info!("http server stopped");

    Ok(())
}

async fn health_check() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn trace_layer(request: Request, next: Next) -> Response {
    let span = info_span!("http", "request_id" = Uuid::now_v7().to_string());

    async move {
        let now: DateTime<Utc> = SystemTime::now().into();
        let now = now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);

        let method = request.method();
        let uri = request.uri();
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str());
        trace!(date=now, %method, %uri, matched_path, headers=?request.headers(), "[request]");

        let response = next.run(request).await;
        trace!(status = %response.status().as_u16(), headers=?response.headers(), "[response]");

        response
    }
    .instrument(span)
    .await
}
