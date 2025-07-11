use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;

pub struct Exception {
    pub code: Option<String>,
    pub message: String,
    pub location: Option<String>,
    pub source: Option<Box<Exception>>,
}

impl Debug for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut index = 0;
        let mut current_source = Some(self);
        while let Some(source) = current_source {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{index}: ")?;
            if let Some(ref code) = source.code {
                write!(f, "[{code}] ")?;
            }
            write!(f, "{}", source.message)?;
            if let Some(ref location) = source.location {
                write!(f, " at {location}")?;
            }
            index += 1;
            current_source = source.source.as_ref().map(|s| s.as_ref());
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! exception {
    (message = $message:expr) => {
        Exception {
            code: None,
            message: $message.to_string(),
            location: Some(format!("{}:{}:{}", file!(), line!(), column!())),
            source: None,
        }
    };
    (code = $code:expr, message = $message:expr) => {
        Exception {
            code: Some($code.to_string()),
            message: $message.to_string(),
            location: Some(format!("{}:{}:{}", file!(), line!(), column!())),
            source: None,
        }
    };
    (message = $message:expr, source = $source:expr) => {
        Exception {
            code: None,
            message: $message.to_string(),
            location: Some(format!("{}:{}:{}", file!(), line!(), column!())),
            source: Some(Box::new($source)),
        }
    };
    (code = $code:expr, message = $message:expr, source = $source:expr) => {
        Exception {
            code: Some($code.to_string()),
            message: $message.to_string(),
            location: Some(format!("{}:{}:{}", file!(), line!(), column!())),
            source: Some(Box::new($source)),
        }
    };
}

fn source(source: Option<&(dyn Error + 'static)>) -> Option<Box<Exception>> {
    let mut sources = Vec::new();
    let mut current_source = source;
    while let Some(target) = current_source {
        sources.push(target);
        current_source = target.source();
    }

    let mut result = None;
    for target in sources.into_iter().rev() {
        result = Some(Box::new(Exception {
            code: None,
            message: target.to_string(),
            location: None,
            source: result,
        }));
    }
    result
}

impl<T> From<T> for Exception
where
    T: Error + 'static,
{
    fn from(error: T) -> Self {
        Exception {
            code: None,
            message: error.to_string(),
            location: None,
            source: source(error.source()),
        }
    }
}
