use std::collections::HashMap;
use std::sync::Arc;

use axum::Extension;
use axum::Router;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderName;
use axum::http::HeaderValue;
use axum::http::header;
use axum::routing::get;
use axum::routing::options;
use axum::routing::post;
use chrono::DateTime;
use chrono::Utc;
use framework::exception;
use framework::exception::Severity;
use framework::json;
use framework::log;
use framework::web::client_info::ClientInfo;
use framework::web::error::HttpError;
use framework::web::error::HttpResult;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;

use crate::AppState;
use crate::kafka::EventMessage;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/event/{app}", options(event_options))
        .route("/event/{app}", post(event_post))
        .route("/test", get(test))
}

#[debug_handler]
async fn test(Extension(client_info): Extension<Arc<ClientInfo>>) -> HttpResult<String> {
    warn!("!!!!!! client_ip = {}", client_info.client_ip);
    Ok("Test endpoint is working".to_string())
}

#[debug_handler]
async fn event_options(headers: HeaderMap) -> HttpResult<HeaderMap> {
    let mut response_headers = HeaderMap::new();

    let origin = headers.get(header::ORIGIN).ok_or_else(|| {
        HttpError::forbidden(exception!(
            severity = Severity::Warn,
            code = "FORDIDDEN",
            message = "access denied"
        ))
    })?;
    response_headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());

    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, PUT, OPTIONS"),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Accept, Content-Type"),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );

    Ok(response_headers)
}

#[debug_handler]
async fn event_post(
    state: State<Arc<AppState>>,
    Path(app): Path<String>,
    headers: HeaderMap,
    body: String,
) -> HttpResult<HeaderMap> {
    // let body = request.body();
    if !body.is_empty() {
        let request: SendEventRequest = json::from_json(&body)?;
        process_events(&state, &app, request, &headers).await?;
    }

    // state.producer.send(&state.topics.event, None, &event).await?;

    let origin = headers.get(header::ORIGIN);
    let mut response_headers = HeaderMap::new();
    if let Some(origin) = origin {
        response_headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
        response_headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            HeaderValue::from_static("true"),
        );
    }
    Ok(response_headers)
}

async fn process_events(
    state: &Arc<AppState>,
    app: &str,
    request: SendEventRequest,
    headers: &HeaderMap,
) -> HttpResult<()> {
    let user_agent = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok());
    let client_ip = headers
        .get(HeaderName::from_static("X-FORWARDED-FOR"))
        .map(|value| value.to_str().unwrap())
        .unwrap_or("hello");

    for event in request.events {
        let mut message = EventMessage {
            id: log::id_generator::random_id(),
            date: event.date,
            app: app.to_string(),
            received_time: Utc::now(),
            result: json::to_json_value(&event.result),
            action: event.action,
            error_code: event.error_code,
            error_message: event.error_message,
            elapsed: event.elapsed_time,
            context: event.context,
            stats: event.stats,
            info: event.info,
        };

        message
            .context
            .insert("user_agent".to_string(), user_agent.unwrap_or("").to_string());
        message.context.insert("client_ip".to_string(), client_ip.to_string());

        state.producer.send(&state.topics.event, None, &message).await?;
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct SendEventRequest {
    events: Vec<Event>,
}

#[derive(Deserialize, Debug)]
struct Event {
    date: DateTime<Utc>,
    result: Result,
    action: String,
    #[serde(rename = "errorCode")]
    error_code: Option<String>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
    context: HashMap<String, String>,
    stats: HashMap<String, f64>,
    info: HashMap<String, String>,
    #[serde(rename = "elapsedTime")]
    elapsed_time: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Result {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "ERROR")]
    Error,
}

/*
 * public class SendEventRequest {
     @NotNull
     @Size(min = 1)
     @Property(name = "events")
     public List<Event> events = new ArrayList<>();

     public enum Result {
         @Property(name = "OK")
         OK,
         @Property(name = "WARN")
         WARN,
         @Property(name = "ERROR")
         ERROR
     }

     public static class Event {
         @NotNull
         @Property(name = "date")
         public ZonedDateTime date;
         @NotNull
         @Property(name = "result")
         public Result result;
         @NotBlank
         @Size(max = 200)
         @Property(name = "action")
         public String action;
         @NotBlank
         @Size(max = 200)
         @Property(name = "errorCode")
         public String errorCode;
         @Size(max = 1000)
         @Property(name = "errorMessage")
         public String errorMessage;
         @NotNull
         @Property(name = "context")
         public Map<String, String> context = new HashMap<>();
         @NotNull
         @Property(name = "stats")
         public Map<String, Double> stats = new HashMap<>();
         @NotNull
         @Property(name = "info")
         public Map<String, String> info = new HashMap<>();
         @NotNull
         @Property(name = "elapsedTime")
         public Long elapsedTime;
     }
 }

 * EventMessage message(SendEventRequest.Event event, String app, Instant now) {
         var message = new EventMessage();
         message.id = LogManager.ID_GENERATOR.next(now);
         message.date = event.date.toInstant();
         message.app = app;
         message.receivedTime = now;
         message.result = JSON.toEnumValue(event.result);
         message.action = event.action;
         message.errorCode = event.errorCode;
         message.errorMessage = event.errorMessage;
         message.context = event.context;
         message.info = event.info;
         message.stats = event.stats;
         message.elapsed = event.elapsedTime;
         return message;
     }
* public Response post(Request request) {
        String origin = request.header("Origin").orElse(null);
        if (origin != null) checkOrigin(origin);    // allow directly call, e.g. mobile app

        processEvents(request, Instant.now());

        Response response = Response.empty();
        if (origin != null) {
            // only need to response CORS headers for browser/ajax
            response.header("Access-Control-Allow-Origin", origin);
            response.header("Access-Control-Allow-Credentials", "true");
        }
        return response;
    }

    private void processEvents(Request request, Instant now) {
        SendEventRequest eventRequest = sendEventRequest(request);
        if (eventRequest == null) return;

        String app = request.pathParam("app");
        String userAgent = request.header(HTTPHeaders.USER_AGENT).orElse(null);
        String clientIP = request.clientIP();
        List<Cookie> cookies = cookies(request);
        for (SendEventRequest.Event event : eventRequest.events) {
            EventMessage message = message(event, app, now);
            addContext(message.context, userAgent, cookies, clientIP);
            eventMessagePublisher.publish(message.id, message);
        }
    }
*/
