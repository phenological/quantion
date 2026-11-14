use std::fmt;

/// **ErrorKind** · Errors
///
/// What went wrong, as a name and a fixed number code.
///
/// Fields:
/// - `InvalidArgument`: A value is out of range or has the wrong type.
/// - `MissingArgument`: A required value is missing.
/// - `UnknownOption`: An option name that does not exist, often a typo.
/// - `RemoteFile`: The function needs the whole file, but a URL was given. Download the file first.
/// - `FileNotFound`: The path does not exist.
/// - `ParseFailed`: The file is not a valid Ionic (.ion), mzML or ONNX file.
/// - `ModelMismatch`: The model has no warp, or a target is not in it.
/// - `FileExists`: The output file already exists and overwrite is false.
/// - `UploadFailed`: The upload was rejected. The message includes the HTTP status.
/// - `Internal`: A bug in Quant·ion. Please report it as an issue.
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
/// The error every function raises. It says what went wrong (kind and code), where (function and argument) and why (message).
///
/// Fields:
/// - `kind`: The type of problem, for example an invalid input or a file that cannot be read.
/// - `code`: A fixed number for the kind, useful in scripts.
/// - `function`: The function that failed.
/// - `argument`: The input that caused it, such as targets\[3\].range. Optional.
/// - `message`: Why it failed, in plain words. It starts with the function name.
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

    pub(crate) fn invalid(
        function: &'static str,
        argument: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
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
            format!(
                "{} (this is a bug in quantion, please report it)",
                message.into()
            ),
        )
    }
}

impl fmt::Display for QuantionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.function, self.message)
    }
}

impl std::error::Error for QuantionError {}

pub(crate) fn check_finite(
    function: &'static str,
    argument: &str,
    value: f64,
) -> Result<(), QuantionError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(QuantionError::invalid(
            function,
            argument,
            format!("must be a finite number, got {value}"),
        ))
    }
}

pub(crate) fn check_positive(
    function: &'static str,
    argument: &str,
    value: f64,
) -> Result<(), QuantionError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(QuantionError::invalid(
            function,
            argument,
            format!("must be greater than 0, got {value}"),
        ))
    }
}

pub(crate) fn check_not_negative(
    function: &'static str,
    argument: &str,
    value: f64,
) -> Result<(), QuantionError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(QuantionError::invalid(
            function,
            argument,
            format!("must be 0 or more, got {value}"),
        ))
    }
}

pub(crate) fn check_mz_and_rt(
    function: &'static str,
    item: &str,
    mz: f64,
    rt: f64,
) -> Result<(), QuantionError> {
    check_positive(function, &format!("{item}.mz"), mz)?;
    check_finite(function, &format!("{item}.rt"), rt)
}

pub(crate) fn check_tolerances(
    function: &'static str,
    prefix: &str,
    ppm_tolerance: f64,
    mz_tolerance: f64,
) -> Result<(), QuantionError> {
    check_not_negative(function, &format!("{prefix}ppm_tolerance"), ppm_tolerance)?;
    check_not_negative(function, &format!("{prefix}mz_tolerance"), mz_tolerance)?;
    if ppm_tolerance == 0.0 && mz_tolerance == 0.0 {
        return Err(QuantionError::invalid(
            function,
            format!("{prefix}ppm_tolerance"),
            "and mz_tolerance cannot both be 0",
        ));
    }
    Ok(())
}

pub(crate) fn check_threads(function: &'static str, threads: usize) -> Result<(), QuantionError> {
    if threads == 0 {
        return Err(QuantionError::invalid(
            function,
            "threads",
            "must be 1 or more, got 0",
        ));
    }
    Ok(())
}
