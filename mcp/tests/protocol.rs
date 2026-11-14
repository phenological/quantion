mod common;

use std::process::Command;

use serde_json::json;

use common::{Server, TestFolder};

#[test]
fn negotiates_the_version() {
    let folder = TestFolder::new("version");
    let mut server = Server::start(&folder.path);
    let reply = server.request("initialize", json!({ "protocolVersion": "1999-01-01" }));
    assert_eq!(reply["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(reply["result"]["serverInfo"]["name"], "quantion-mcp");
}

#[test]
fn answers_protocol_errors() {
    let folder = TestFolder::new("errors");
    let mut server = Server::start(&folder.path);

    let reply = server.request("resources/list", json!({}));
    assert_eq!(reply["error"]["code"], -32601);

    let reply = server.request(
        "tools/call",
        json!({ "name": "no_such_tool", "arguments": {} }),
    );
    assert_eq!(reply["error"]["code"], -32602);

    server.send_line("{ not json");
    assert_eq!(server.read_reply()["error"]["code"], -32700);

    server.send_line("[]");
    assert_eq!(server.read_reply()["error"]["code"], -32600);

    let reply = server.request("ping", json!({}));
    assert_eq!(reply["result"], json!({}));
}

#[test]
fn ignores_unknown_cancels_and_responses() {
    let folder = TestFolder::new("cancel");
    let mut server = Server::start(&folder.path);
    server.send_line(&json!({ "jsonrpc": "2.0", "method": "notifications/cancelled", "params": { "requestId": 99 } }).to_string());
    server.send_line(&json!({ "jsonrpc": "2.0", "id": 7, "result": {} }).to_string());
    let reply = server.request("tools/list", json!({}));
    assert!(
        reply["result"]["tools"]
            .as_array()
            .is_some_and(|tools| !tools.is_empty())
    );
}

#[test]
fn closes_cleanly_when_input_ends() {
    let folder = TestFolder::new("close");
    let output = Command::new(env!("CARGO_BIN_EXE_quantion-mcp"))
        .arg("--allow")
        .arg(&folder.path)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("server runs");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn rejects_bad_command_lines() {
    let output = Command::new(env!("CARGO_BIN_EXE_quantion-mcp"))
        .arg("--alow")
        .output()
        .expect("binary runs");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--alow is not an option"));
}

#[test]
fn runs_one_tool_from_the_command_line() {
    let folder = TestFolder::new("call");
    folder.write_sample("sample_1.mzML");
    let input = folder.path.join("request.json");
    std::fs::write(&input, json!({ "limit": 5 }).to_string()).expect("request");
    let output = Command::new(env!("CARGO_BIN_EXE_quantion-mcp"))
        .args(["call", "list_files", "--input"])
        .arg(&input)
        .arg("--allow")
        .arg(&folder.path)
        .output()
        .expect("binary runs");
    assert!(output.status.success());
    let reply: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON output");
    assert_eq!(reply["total"], 1);
    assert_eq!(reply["rows"][0][0], "sample_1.mzML");
}

#[test]
fn needs_an_allowed_folder() {
    let output = Command::new(env!("CARGO_BIN_EXE_quantion-mcp"))
        .stdin(std::process::Stdio::null())
        .output()
        .expect("binary runs");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--allow"));
}

#[test]
fn answers_bad_bytes_and_keeps_running() {
    let folder = TestFolder::new("bad-bytes");
    let mut server = Server::start(&folder.path);
    server.send_bytes(b"\xff\xfe\n");
    assert_eq!(server.read_reply()["error"]["code"], -32700);
    let reply = server.request("ping", json!({}));
    assert_eq!(reply["result"], json!({}));
}
