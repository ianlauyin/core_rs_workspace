use std::error::Error;
use std::fmt::Display;
use std::fmt::{self};

#[derive(Debug)]
struct AppError {
    message: String,
    error_code: Option<String>,
}

trait ErrorCode: Display + fmt::Debug + Send + Sync + 'static {
    fn error_code(&self) -> Option<String>;
}

impl ErrorCode for AppError {
    fn error_code(&self) -> Option<String> {
        self.error_code.clone()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = &self.error_code {
            write!(f, "{} (code: {})", self.message, code)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl Error for AppError {}

pub fn main() {
    let e = test().unwrap_err();
    let app = e.downcast_ref::<Box<dyn ErrorCode>>();
    dbg!(app.unwrap().error_code());
}

fn test() -> anyhow::Result<()> {
    // Simulate an error
    let error = AppError {
        message: "An error occurred".to_string(),
        error_code: Some("E001".to_string()),
    };
    let error = Box::new(error) as Box<dyn ErrorCode + Send + Sync>;
    Err(anyhow::anyhow!(error))
}
