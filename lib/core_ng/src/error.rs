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
        Exception::new(
            None,
            $message.to_string(),
            Some(format!("{}:{}:{}", file!(), line!(), column!())),
            None,
        )
    };
    (code = $code:expr, message = $message:expr) => {
        Exception::new(
            Some($code.to_string()),
            $message.to_string(),
            Some(format!("{}:{}:{}", file!(), line!(), column!())),
            None,
        )
    };
    (message = $message:expr, source = $source:expr) => {
        Exception::new(
            None,
            $message.to_string(),
            Some(format!("{}:{}:{}", file!(), line!(), column!())),
            Some($source),
        )
    };
    (code = $code:expr, message = $message:expr, source = $source:expr) => {
        Exception::new(
            Some($code.to_string()),
            $message.to_string(),
            Some(format!("{}:{}:{}", file!(), line!(), column!())),
            Some($source),
        )
    };
}

impl Exception {
    pub fn new(
        code: Option<String>,
        message: String,
        location: Option<String>,
        source: Option<&(dyn Error + 'static)>,
    ) -> Self {
        Exception {
            code,
            message,
            location,
            source: Self::from(source),
        }
    }

    fn from(source: Option<&(dyn Error + 'static)>) -> Option<Box<Exception>> {
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
}

impl<T> From<T> for Exception
where
    T: Error + 'static,
{
    fn from(error: T) -> Self {
        Exception::new(None, error.to_string(), None, Some(&error))
    }
}
