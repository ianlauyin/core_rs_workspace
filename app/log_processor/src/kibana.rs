use framework::exception;
use framework::exception::Exception;
use framework::http::HttpClient;
use framework::http::HttpMethod::POST;
use framework::http::HttpRequest;
use tracing::info;

pub async fn import(kibana_uri: &str) -> Result<(), Exception> {
    let http_client = HttpClient::default();
    let mut request = HttpRequest::new(
        POST,
        format!("{kibana_uri}/api/saved_objects/_bulk_create?overwrite=true"),
    );
    request.headers.insert("kbn-xsrf", "true".to_string());
    request.headers.insert("Content-Type", "application/json".to_string());
    request.body = Some("hello".to_string());

    let response = http_client.execute(request).await?;
    if response.status_code() == 200 {
        info!("kibana objects are imported")
    } else {
        return Err(exception!(
            message = format!("failed to import kibana objects, status={}", response.status_code())
        ));
    }

    Ok(())
}
