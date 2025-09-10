use framework::exception;
use framework::exception::Exception;
use framework::http::HttpClient;
use framework::http::HttpRequest;

pub struct Opensearch {
    uri: String,
    client: HttpClient,
}

impl Opensearch {
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_owned(),
            client: HttpClient::default(),
        }
    }

    pub async fn put_index_template(&self, name: &str, template: String) -> Result<(), Exception> {
        let mut request = HttpRequest::new(
            framework::http::HttpMethod::PUT,
            format!("{}/_index_template/{name}", self.uri),
        );
        request.body(template, "application/json");
        let response = self.client.execute(request).await?;
        if response.status != 200 {
            return Err(exception!(
                message = format!("failed to create index template, name={name}")
            ));
        }
        Ok(())
    }
}
