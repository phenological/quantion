use std::{
    env,
    io::{Read, Write},
    process::{Command, Stdio},
};

use serde_json::Value;

use crate::{error::ToolError, protocol::requests::Requests};

pub struct ToolReply {
    pub output: Value,
    pub is_error: bool,
}

pub fn run_tool_in_child(
    name: &str,
    arguments: &Value,
    child_arguments: &[String],
    requests: &Requests,
    id: &Value,
) -> Result<ToolReply, ToolError> {
    let program = env::current_exe().map_err(|error| {
        ToolError::internal(format!("the server program was not found: {error}"))
    })?;
    let mut child = Command::new(program)
        .arg("call")
        .arg(name)
        .args(child_arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| ToolError::internal(format!("{name} could not be started: {error}")))?;
    let input = child.stdin.take();
    let output = child.stdout.take();
    requests.watch_child(id, child);
    if let Some(mut input) = input {
        let _ = input.write_all(arguments.to_string().as_bytes());
    }
    let mut text = String::new();
    if let Some(mut output) = output {
        let _ = output.read_to_string(&mut text);
    }
    let status = match requests.take_child() {
        Some(mut child) => child.wait().ok(),
        None => None,
    };
    if requests.is_cancelled(id) {
        return Err(ToolError::new("cancelled", format!("{name} was stopped")));
    }
    let output: Value = serde_json::from_str(text.trim())
        .map_err(|_| ToolError::internal(format!("{name} stopped without a result")))?;
    match status.and_then(|status| status.code()) {
        Some(0) => Ok(ToolReply {
            output,
            is_error: false,
        }),
        Some(1) if output.get("error").is_some() => Ok(ToolReply {
            output,
            is_error: true,
        }),
        _ => Err(ToolError::internal(format!("{name} stopped unexpectedly"))),
    }
}
