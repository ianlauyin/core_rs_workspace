use anyhow::Result;
use axum::Router;
use axum::extract::MatchedPath;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::debug;
use tracing::info;

use crate::log;

pub async fn start_http_server(router: Router, mut shutdown_signal: broadcast::Receiver<()>) -> Result<()> {
    let app = Router::new();
    let app = app.route("/health-check", get(health_check));
    let app = app.merge(router);
    let app = app.layer(middleware::from_fn(action_log_layer));

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

async fn action_log_layer(request: Request, next: Next) -> Response {
    let mut response = None;
    log::start_action("http", None, async {
        let method = request.method();
        let uri = request.uri();
        debug!(method = ?method, "[request]");
        debug!(uri = ?uri, "[request]");
        for (name, value) in request.headers().iter() {
            debug!("[header] {name}={value:?}");
        }
        debug!(uri = ?uri, method = ?method, "context");

        if let Some(user_agent) = request.headers().get("user-agent") {
            if let Ok(user_agent) = user_agent.to_str() {
                debug!(user_agent, "context");
            }
        }

        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str());
        if let Some(matched_path) = matched_path {
            debug!(matched_path = matched_path, "context");
        }
        let http_response = next.run(request).await;
        let status = http_response.status().as_u16();
        debug!(status, "[response]");
        debug!(response_status = status, "context");
        for (name, value) in http_response.headers().iter() {
            debug!("[header] {name}={value:?}");
        }
        response = Some(http_response);
        Ok(())
    })
    .await;
    if let Some(response) = response {
        response
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
