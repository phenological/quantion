mod arguments;
mod error;
mod input;
mod output;
mod protocol;
mod safety;
mod schema;
mod state;
mod tools;

use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use serde_json::Value;

use crate::{
    arguments::{Command, HELP, ServerOptions, read_command},
    error::ToolError,
    protocol::server::run_server,
    safety::folders::AllowedFolders,
    state::{Context, file_cache::FileCache, model_cache::ModelCache},
    tools::{find_tool, get_tool_functions, get_tool_list, run_tool},
};

fn main() -> ExitCode {
    let command = match read_command(env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => return stop_with_error(&message),
    };
    match command {
        Command::Serve(options) => serve(options),
        Command::Call {
            tool,
            input,
            options,
        } => call(&tool, input.as_deref(), options),
        Command::Tools => print_text(&format!("{}\n{}\n", get_tool_list(), get_tool_functions())),
        Command::Version => print_text(&format!("quantion-mcp {}\n", env!("CARGO_PKG_VERSION"))),
        Command::Help => print_text(HELP),
    }
}

fn serve(options: ServerOptions) -> ExitCode {
    if options.allowed_folders.is_empty() {
        return stop_with_error(
            "give at least one --allow folder, the folder the tools may read and write",
        );
    }
    let context = match get_context(options) {
        Ok(context) => context,
        Err(message) => return stop_with_error(&message),
    };
    match run_server(context) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => stop_with_error(&error.to_string()),
    }
}

fn call(name: &str, input: Option<&Path>, options: ServerOptions) -> ExitCode {
    let Some(tool) = find_tool(name) else {
        return stop_with_error(&format!("{name} is not a tool; run quantion-mcp tools"));
    };
    let mut context = match get_context(options) {
        Ok(context) => context,
        Err(message) => return stop_with_error(&message),
    };
    let arguments = match read_input(input) {
        Ok(arguments) => arguments,
        Err(message) => return stop_with_error(&message),
    };
    match run_tool(tool, arguments, &mut context) {
        Ok(output) => print_text(&format!("{output}\n")),
        Err(error) => {
            print_text(&format!("{}\n", error.to_json()));
            ExitCode::FAILURE
        }
    }
}

fn get_context(options: ServerOptions) -> Result<Context, String> {
    let folders = if options.allowed_folders.is_empty() {
        vec![env::current_dir().map_err(|error| error.to_string())?]
    } else {
        options.allowed_folders
    };
    Ok(Context {
        folders: AllowedFolders::new(&folders)?,
        files: FileCache::new(options.memory_limit),
        models: ModelCache::default(),
        max_rows: options.max_rows,
    })
}

fn read_input(input: Option<&Path>) -> Result<Value, String> {
    let text = match input {
        Some(path) => fs::read_to_string(path).map_err(|_| get_missing_input(path))?,
        None => {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .map_err(|error| error.to_string())?;
            text
        }
    };
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text).map_err(|error| format!("the input is not valid JSON: {error}"))
}

fn get_missing_input(path: &Path) -> String {
    ToolError::file_not_found(&PathBuf::from(path).display().to_string())
        .message()
        .to_string()
}

fn print_text(text: &str) -> ExitCode {
    let mut output = io::stdout().lock();
    match output
        .write_all(text.as_bytes())
        .and_then(|_| output.flush())
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn stop_with_error(message: &str) -> ExitCode {
    eprintln!("quantion-mcp: {message}");
    ExitCode::FAILURE
}
