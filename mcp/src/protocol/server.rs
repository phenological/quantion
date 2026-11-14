use std::{
    collections::HashSet,
    io,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, channel},
    },
    thread,
};

use serde_json::{Value, json};

use crate::{
    protocol::{
        message::{
            ErrorCode, Message, ProtocolError, get_error_response, get_result_response,
            read_message,
        },
        transport::{Line, read_line, write_message},
    },
    state::Context,
    tools::{find_tool, get_tool_list, run_tool},
};

pub const SUPPORTED_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "Quant·ion processes LCMS samples (.ion and .mzML) on this computer. \
Tools and options have the same names, units and defaults as the Quant·ion library (https://phenological.github.io/quantion/). \
rt is in minutes. m/z is the m/z of the ion, not the neutral mass. mz_tolerance is in Da and ppm_tolerance in ppm. threads is the number of CPU cores. \
Write .mzML samples to .ion with write_ion first: .ion opens fast and uses little memory. \
Relative paths start in the first allowed folder. \
Tables return limit rows, rounded. save writes every row at full precision as CSV, and overwrite allows replacing a file.";

struct Job {
    id: Value,
    method: String,
    params: Value,
}

#[derive(Default)]
struct RequestState {
    pending: HashSet<String>,
    cancelled: HashSet<String>,
}

type SharedRequests = Arc<Mutex<RequestState>>;

pub fn run_server(context: Context) -> io::Result<()> {
    let (sender, receiver) = channel::<Job>();
    let requests: SharedRequests = Arc::new(Mutex::new(RequestState::default()));
    let worker_requests = Arc::clone(&requests);
    let worker = thread::spawn(move || run_worker(receiver, context, worker_requests));
    let mut input = io::stdin().lock();
    while let Some(line) = read_line(&mut input)? {
        let line = match line {
            Line::Text(line) => line,
            Line::Unreadable(reason) => {
                let error = ProtocolError::new(ErrorCode::ParseError, reason);
                write_message(&get_error_response(&Value::Null, &error))?;
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        match read_message(&line) {
            Ok(Message::Request { id, method, .. }) if method == "ping" => {
                write_message(&get_result_response(&id, json!({})))?;
            }
            Ok(Message::Request { id, method, params }) => {
                add_pending(&requests, &id);
                if sender.send(Job { id, method, params }).is_err() {
                    break;
                }
            }
            Ok(Message::Notification { method, params }) => {
                if method == "notifications/cancelled" {
                    add_cancelled(&requests, &params);
                }
            }
            Ok(Message::Response) => {}
            Err(error) => write_message(&get_error_response(&Value::Null, &error))?,
        }
    }
    drop(sender);
    worker
        .join()
        .map_err(|_| io::Error::other("the worker thread stopped unexpectedly"))?
}

fn run_worker(
    receiver: Receiver<Job>,
    mut context: Context,
    requests: SharedRequests,
) -> io::Result<()> {
    let tool_list = get_tool_list();
    for job in receiver {
        let response = match handle_request(&job.method, job.params, &mut context, &tool_list) {
            Ok(result) => get_result_response(&job.id, result),
            Err(error) => get_error_response(&job.id, &error),
        };
        let was_cancelled = close_request(&requests, &job.id);
        if !was_cancelled {
            write_message(&response)?;
        }
    }
    Ok(())
}

fn handle_request(
    method: &str,
    params: Value,
    context: &mut Context,
    tool_list: &Value,
) -> Result<Value, ProtocolError> {
    match method {
        "initialize" => Ok(get_initialize_result(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tool_list.clone()),
        "tools/call" => call_tool(params, context),
        _ => Err(ProtocolError::new(
            ErrorCode::MethodNotFound,
            format!("{method} is not supported"),
        )),
    }
}

fn get_initialize_result(params: &Value) -> Value {
    let requested = params["protocolVersion"].as_str().unwrap_or_default();
    let version = SUPPORTED_VERSIONS
        .iter()
        .find(|supported| **supported == requested)
        .copied()
        .unwrap_or(SUPPORTED_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "quantion-mcp", "version": env!("CARGO_PKG_VERSION") },
        "instructions": INSTRUCTIONS,
    })
}

fn call_tool(mut params: Value, context: &mut Context) -> Result<Value, ProtocolError> {
    let Some(name) = params["name"].as_str() else {
        return Err(ProtocolError::new(
            ErrorCode::InvalidParams,
            "name is required",
        ));
    };
    let Some(tool) = find_tool(name) else {
        return Err(ProtocolError::new(
            ErrorCode::InvalidParams,
            format!("{name} is not a tool"),
        ));
    };
    let arguments = params["arguments"].take();
    let (output, is_error) = match run_tool(tool, arguments, context) {
        Ok(output) => (output, false),
        Err(error) => (error.to_json(), true),
    };
    Ok(json!({
        "content": [{ "type": "text", "text": output.to_string() }],
        "isError": is_error,
    }))
}

fn add_pending(requests: &SharedRequests, id: &Value) {
    if let Ok(mut requests) = requests.lock() {
        requests.pending.insert(id.to_string());
    }
}

fn add_cancelled(requests: &SharedRequests, params: &Value) {
    let id = params["requestId"].to_string();
    if let Ok(mut requests) = requests.lock()
        && requests.pending.contains(&id)
    {
        requests.cancelled.insert(id);
    }
}

fn close_request(requests: &SharedRequests, id: &Value) -> bool {
    let id = id.to_string();
    requests
        .lock()
        .map(|mut requests| {
            requests.pending.remove(&id);
            requests.cancelled.remove(&id)
        })
        .unwrap_or(false)
}
