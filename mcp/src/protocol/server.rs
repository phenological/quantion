use std::{
    io,
    sync::mpsc::{Receiver, channel},
    thread,
};

use serde_json::{Value, json};

use crate::{
    protocol::{
        child::{ToolReply, run_tool_in_child},
        message::{
            ErrorCode, Message, ProtocolError, get_error_response, get_result_response,
            read_message,
        },
        requests::Requests,
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

pub fn run_server(context: Context) -> io::Result<()> {
    let (sender, receiver) = channel::<Job>();
    let requests = Requests::default();
    let worker_requests = requests.clone();
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
                requests.add(&id);
                if sender.send(Job { id, method, params }).is_err() {
                    break;
                }
            }
            Ok(Message::Notification { method, params }) => {
                if method == "notifications/cancelled" {
                    requests.cancel(&params["requestId"]);
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

fn run_worker(receiver: Receiver<Job>, mut context: Context, requests: Requests) -> io::Result<()> {
    let tool_list = get_tool_list();
    for job in receiver {
        let call = Call {
            id: &job.id,
            requests: &requests,
        };
        let response =
            match handle_request(&job.method, job.params, &mut context, &tool_list, &call) {
                Ok(result) => get_result_response(&job.id, result),
                Err(error) => get_error_response(&job.id, &error),
            };
        let was_cancelled = requests.close(&job.id);
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
    call: &Call,
) -> Result<Value, ProtocolError> {
    match method {
        "initialize" => Ok(get_initialize_result(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tool_list.clone()),
        "tools/call" => call_tool(params, context, call),
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

struct Call<'a> {
    id: &'a Value,
    requests: &'a Requests,
}

fn call_tool(
    mut params: Value,
    context: &mut Context,
    call: &Call,
) -> Result<Value, ProtocolError> {
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
    let result = if tool.runs_in_child() {
        run_tool_in_child(
            tool.name(),
            &arguments,
            &context.child_arguments,
            call.requests,
            call.id,
        )
    } else {
        run_tool(tool, arguments, context).map(|output| ToolReply {
            output,
            is_error: false,
        })
    };
    let reply = result.unwrap_or_else(|error| ToolReply {
        output: error.to_json(),
        is_error: true,
    });
    Ok(json!({
        "content": [{ "type": "text", "text": reply.output.to_string() }],
        "isError": reply.is_error,
    }))
}
