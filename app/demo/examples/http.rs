use framework::http::HttpClient;
use framework::http::HttpMethod::POST;
use framework::http::HttpRequest;
use framework::http::header;
use framework::log;
use framework::log::ConsoleAppender;
use tracing::debug;
use tracing::warn;

#[tokio::main]
async fn main() {
    log::init_with_action(ConsoleAppender);

    test().await;
}

async fn test() {
    log::start_action("test_http_client", None, async {
        let http_client = HttpClient::default();
        let mut request = HttpRequest::new(POST, "https://www.ubgame.dev".to_string());
        request.body("{some json}".to_owned(), "application/json".to_owned());
        request.headers.insert(header::USER_AGENT, "Rust".to_string());
        let _response = http_client.execute(request).await?;
        // let mut lines = response.lines();
        // while let Some(line) = lines.next().await {
        //     let line = line?;
        //     println!("line={line}");
        // }
        debug!(http_client_hello = 1, "stats");
        warn!("test");

        Ok(())
    })
    .await;
}
