use quantion::QuantionError;
use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq)]
pub struct ToolError {
    kind: &'static str,
    argument: Option<String>,
    message: String,
    fix: Option<String>,
}

impl ToolError {
    pub fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            argument: None,
            message: message.into(),
            fix: None,
        }
    }

    pub fn invalid(argument: impl Into<String>, message: impl Into<String>) -> Self {
        let argument = argument.into();
        let message = format!("{argument} {}", message.into());
        Self::new("invalid_argument", message).with_argument(argument)
    }

    pub fn missing(argument: impl Into<String>) -> Self {
        let argument = argument.into();
        let message = format!("{argument} is required");
        Self::new("missing_argument", message).with_argument(argument)
    }

    pub fn unknown_option(argument: impl Into<String>) -> Self {
        let argument = argument.into();
        let message = format!("{argument} is not an option");
        Self::new("unknown_option", message).with_argument(argument)
    }

    pub fn file_not_found(path: &str) -> Self {
        Self::new("file_not_found", format!("{path} does not exist")).with_argument(path)
    }

    pub fn file_exists(path: &str) -> Self {
        Self::new("file_exists", format!("{path} already exists"))
            .with_argument(path)
            .with_fix("choose another name, or set overwrite to true")
    }

    pub fn folder_not_allowed(path: &str) -> Self {
        Self::new(
            "folder_not_allowed",
            format!("{path} is outside the allowed folders"),
        )
        .with_argument(path)
        .with_fix("use a path inside a folder given with --allow")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
            .with_fix("this is a bug in quantion-mcp, please report it as an issue")
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn with_argument(mut self, argument: impl Into<String>) -> Self {
        self.argument = Some(argument.into());
        self
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    pub fn to_json(&self) -> Value {
        let mut error = json!({ "kind": self.kind, "message": self.message });
        if let Some(argument) = &self.argument {
            error["argument"] = json!(argument);
        }
        if let Some(fix) = &self.fix {
            error["fix"] = json!(fix);
        }
        json!({ "error": error })
    }
}

impl From<QuantionError> for ToolError {
    fn from(error: QuantionError) -> Self {
        Self {
            kind: error.kind.name(),
            argument: error.argument,
            message: error.message,
            fix: None,
        }
    }
}
