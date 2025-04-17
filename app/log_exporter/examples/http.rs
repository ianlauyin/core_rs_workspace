use core_ng::http::HttpClient;
use core_ng::http::HttpMethod::POST;
use core_ng::http::HttpRequest;
use core_ng::log;
use core_ng::log::ConsoleAppender;
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
        request.body = Some("{some json}".to_string());
        request.headers.insert("User-Agent", "Rust".to_string());
        let response = http_client.execute(request).await?;
        let _body = response.text().await?;
        // let mut lines = response.lines();
        // while let Some(line) = lines.next().await {
        //     let line = line?;
        //     println!("line={line}");
        // }

        warn!("test");

        Ok(())
    })
    .await;
}
