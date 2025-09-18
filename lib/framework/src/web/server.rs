use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::MatchedPath;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::debug;
use tracing::info;

use crate::exception::Exception;
use crate::log;
use crate::web::client_info::client_info;
use crate::web::site_directory::SiteDirectory;

pub struct HttpServerConfig {
    pub bind_address: String,
    pub max_forwarded_ips: usize,
    pub site_directory: Option<SiteDirectory>,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        HttpServerConfig {
            bind_address: "0.0.0.0:8080".to_string(),
            max_forwarded_ips: 2,
            site_directory: None,
        }
    }
}

pub async fn start_http_server(
    router: Router,
    mut shutdown_signal: broadcast::Receiver<()>,
    config: HttpServerConfig,
) -> Result<(), Exception> {
    let app = Router::new();
    let app = app.merge(router);
    let app = app.layer(middleware::from_fn(http_server_layer));
    // merge site directory under middleware to avoid log for site
    let app = merge_site_directory(app, &config);
    let app = app.into_make_service_with_connect_info::<SocketAddr>();
    let listener = TcpListener::bind(&config.bind_address).await?;
    info!("http server stated, bind={}", config.bind_address);
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal.recv().await.unwrap();
        })
        .await?;
    info!("http server stopped");

    Ok(())
}

fn merge_site_directory(app: Router, config: &HttpServerConfig) -> Router {
    if let Some(site_directory) = &config.site_directory {
        if let Some(service) = site_directory.into_service() {
            return app.fallback_service(service);
        }
    };
    return app;
}

async fn http_server_layer(mut request: Request, next: Next) -> Response {
    // skip log for health check
    if request.uri().path() == "/health-check" {
        return StatusCode::OK.into_response(); // gce lb health check requires to return 200
    }

    let mut response = None;
    log::start_action("http", None, async {
        let method = request.method().clone();
        let uri = request.uri();
        debug!(method = ?method, "[request]");
        debug!(uri = ?uri, "[request]");
        for (name, value) in request.headers().iter() {
            if name != header::COOKIE {
                debug!("[header] {name}={value:?}");
            }
        }
        let cookies = CookieJar::from_headers(request.headers());
        for cookie in cookies.iter() {
            debug!("[cookie] {}={}", cookie.name(), cookie.value());
        }

        debug!(uri = ?uri, method = ?method, "context");

        let client_info = client_info(&request, 2);
        debug!(client_ip = client_info.client_ip, "context");
        if let Some(ref user_agent) = client_info.user_agent {
            debug!(user_agent, "context");
        }
        request.extensions_mut().insert(Arc::new(client_info));

        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str());
        if let Some(matched_path) = matched_path {
            debug!(matched_path = matched_path, "context");
        }

        if let Some(length) = request
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| str::parse::<usize>(v).ok())
        {
            debug!(request_content_length = length, "stats");
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
