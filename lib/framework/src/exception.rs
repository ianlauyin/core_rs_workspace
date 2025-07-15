use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;

pub struct Exception {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub location: Option<String>,
    pub source: Option<Box<Exception>>,
}

pub enum Severity {
    Warn,
    Error,
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
    ($(severity = $severity:expr,)? $(code = $code:expr,)? message = $message:expr $(,source = $source:expr)?) => {{
        let severity = $crate::exception::Severity::Error;
        $(
            let severity = $severity;
        )?
        let code: Option<String> = None;
        $(
            let code = Some($code.to_string());
        )?
        let source: Option<Box<$crate::exception::Exception>> = None;
        $(
            drop(source);
            let source = Some(Box::new($source.into()));
        )?
        $crate::exception::Exception {
            severity,
            code,
            message: $message.to_string(),
            location: Some(format!("{}:{}:{}", file!(), line!(), column!())),
            source,
        }
    }};
}

fn source(source: Option<&(dyn Error + 'static)>) -> Option<Box<Exception>> {
    let mut sources = Vec::new();
    let mut current_source = source;
    while let Some(target) = current_source {
        sources.push(target);
        current_source = target.source();
    }

    let mut result = None;
    for error in sources.into_iter().rev() {
        result = Some(Box::new(Exception {
            severity: Severity::Error,
            code: None,
            message: error.to_string(),
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
            severity: Severity::Error,
            code: None,
            message: error.to_string(),
            location: None,
            source: source(error.source()),
        }
    }
}
