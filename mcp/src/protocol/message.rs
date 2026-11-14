use serde_json::{Value, json};

#[derive(Debug, PartialEq)]
pub enum Message {
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    Notification {
        method: String,
        params: Value,
    },
    Response,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
}

#[derive(Debug, PartialEq)]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
}

impl ProtocolError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn read_message(line: &str) -> Result<Message, ProtocolError> {
    let value: Value = serde_json::from_str(line)
        .map_err(|_| ProtocolError::new(ErrorCode::ParseError, "the message is not valid JSON"))?;
    let Value::Object(mut message) = value else {
        return Err(ProtocolError::new(
            ErrorCode::InvalidRequest,
            "the message must be one JSON object; batches are not supported",
        ));
    };
    if message.get("jsonrpc") != Some(&json!("2.0")) {
        return Err(ProtocolError::new(
            ErrorCode::InvalidRequest,
            "jsonrpc must be \"2.0\"",
        ));
    }
    let id = message.remove("id");
    let params = message.remove("params").unwrap_or(Value::Null);
    let method = match message.remove("method") {
        Some(Value::String(method)) => method,
        Some(_) => {
            return Err(ProtocolError::new(
                ErrorCode::InvalidRequest,
                "method must be text",
            ));
        }
        None if id.is_some() => return Ok(Message::Response),
        None => {
            return Err(ProtocolError::new(
                ErrorCode::InvalidRequest,
                "method is required",
            ));
        }
    };
    match id {
        None => Ok(Message::Notification { method, params }),
        Some(id) if id.is_string() || id.is_i64() || id.is_u64() => {
            Ok(Message::Request { id, method, params })
        }
        Some(_) => Err(ProtocolError::new(
            ErrorCode::InvalidRequest,
            "id must be text or a whole number",
        )),
    }
}

pub fn get_result_response(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

pub fn get_error_response(id: &Value, error: &ProtocolError) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": error.code as i32, "message": error.message },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_requests_and_notifications() {
        let request = read_message(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).unwrap();
        assert_eq!(
            request,
            Message::Request {
                id: json!(1),
                method: "ping".into(),
                params: Value::Null
            }
        );
        let notification =
            read_message(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).unwrap();
        assert!(matches!(notification, Message::Notification { .. }));
    }

    #[test]
    fn rejects_bad_messages() {
        assert_eq!(read_message("{").unwrap_err().code, ErrorCode::ParseError);
        assert_eq!(
            read_message("[]").unwrap_err().code,
            ErrorCode::InvalidRequest
        );
        assert_eq!(
            read_message(r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#)
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        assert_eq!(
            read_message(r#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#)
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
    }
}
