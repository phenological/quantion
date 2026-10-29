use std::fmt;

/// **ErrorKind** · Errors
///
/// What went wrong, as a name and a fixed number code.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#ErrorKind)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidArgument = 1,
    MissingArgument = 2,
    UnknownOption = 3,
    RemoteFile = 4,
    FileNotFound = 5,
    ParseFailed = 6,
    ModelMismatch = 7,
    FileExists = 8,
    UploadFailed = 9,
    Internal = 99,
}

impl ErrorKind {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn name(self) -> &'static str {
        match self {
            ErrorKind::InvalidArgument => "invalid_argument",
            ErrorKind::MissingArgument => "missing_argument",
            ErrorKind::UnknownOption => "unknown_option",
            ErrorKind::RemoteFile => "remote_file",
            ErrorKind::FileNotFound => "file_not_found",
            ErrorKind::ParseFailed => "parse_failed",
            ErrorKind::ModelMismatch => "model_mismatch",
            ErrorKind::FileExists => "file_exists",
            ErrorKind::UploadFailed => "upload_failed",
            ErrorKind::Internal => "internal",
        }
    }
}

/// **QuantionError** · Errors
///
/// The error every function raises, with its kind, code, function, argument and message.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#QuantionError)
#[derive(Clone, Debug, PartialEq)]
pub struct QuantionError {
    pub kind: ErrorKind,
    pub function: &'static str,
    pub argument: Option<String>,
    pub message: String,
}

impl QuantionError {
    pub(crate) fn new(kind: ErrorKind, function: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            function,
            argument: None,
            message: message.into(),
        }
    }

    pub(crate) fn invalid(function: &'static str, argument: impl Into<String>, message: impl Into<String>) -> Self {
        let argument = argument.into();
        let message = format!("{argument} {}", message.into());
        Self {
            kind: ErrorKind::InvalidArgument,
            function,
            argument: Some(argument),
            message,
        }
    }

    pub(crate) fn missing(function: &'static str, argument: impl Into<String>) -> Self {
        let argument = argument.into();
        let message = format!("{argument} is required");
        Self {
            kind: ErrorKind::MissingArgument,
            function,
            argument: Some(argument),
            message,
        }
    }

    pub(crate) fn remote_file(function: &'static str) -> Self {
        Self::new(
            ErrorKind::RemoteFile,
            function,
            "this needs the whole file: open it from a path or bytes, not a URL",
        )
    }

    pub(crate) fn internal(function: &'static str, message: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::Internal,
            function,
            format!("{} (this is a bug in quantion, please report it)", message.into()),
        )
    }
}

impl fmt::Display for QuantionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.function, self.message)
    }
}

impl std::error::Error for QuantionError {}

pub(crate) fn check_finite(function: &'static str, argument: &str, value: f64) -> Result<(), QuantionError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(QuantionError::invalid(function, argument, format!("must be a finite number, got {value}")))
    }
}

pub(crate) fn check_positive(function: &'static str, argument: &str, value: f64) -> Result<(), QuantionError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(QuantionError::invalid(function, argument, format!("must be greater than 0, got {value}")))
    }
}

pub(crate) fn check_not_negative(function: &'static str, argument: &str, value: f64) -> Result<(), QuantionError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(QuantionError::invalid(function, argument, format!("must be 0 or more, got {value}")))
    }
}
