use std::path::PathBuf;

use crate::state::file_cache::BYTES_PER_MEGABYTE;

pub const HELP: &str = "quantion-mcp: MCP server for Quant·ion LCMS processing

Usage:
  quantion-mcp --allow <folder>...           start the server on stdin and stdout
  quantion-mcp call <tool> [--input <file>]  run one tool; the JSON input comes from the file or stdin
  quantion-mcp tools                         print the tool list, then the library functions of each tool, as JSON
  quantion-mcp --version | --help

Options:
  --allow <folder>       folder the tools may read and write; repeat for more (required for the server; call defaults to the current folder)
  --max-rows <count>     rows returned when a tool has no limit (default: 50)
  --memory-limit <MB>    memory for open mzML files (default: 2048)
";

#[derive(Debug, PartialEq)]
pub enum Command {
    Serve(ServerOptions),
    Call {
        tool: String,
        input: Option<PathBuf>,
        options: ServerOptions,
    },
    Tools,
    Version,
    Help,
}

#[derive(Debug, PartialEq)]
pub struct ServerOptions {
    pub allowed_folders: Vec<PathBuf>,
    pub max_rows: usize,
    pub memory_limit: u64,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            allowed_folders: Vec::new(),
            max_rows: 50,
            memory_limit: 2048 * BYTES_PER_MEGABYTE,
        }
    }
}

pub fn read_command(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut arguments = arguments.into_iter().peekable();
    let mut options = ServerOptions::default();
    let mut tool = None;
    let mut input = None;
    let mut is_call = false;
    match arguments.peek().map(String::as_str) {
        Some("call") => {
            arguments.next();
            is_call = true;
            tool = Some(arguments.next().ok_or("call needs a tool name")?);
        }
        Some("tools") => return read_alone(arguments, Command::Tools),
        Some("--version") => return read_alone(arguments, Command::Version),
        Some("--help") => return read_alone(arguments, Command::Help),
        _ => {}
    }
    while let Some(argument) = arguments.next() {
        let mut get_value = || arguments.next().ok_or(format!("{argument} needs a value"));
        match argument.as_str() {
            "--allow" => options.allowed_folders.push(PathBuf::from(get_value()?)),
            "--max-rows" => options.max_rows = read_positive(&argument, &get_value()?)?,
            "--memory-limit" => {
                let megabytes: u64 = read_positive(&argument, &get_value()?)?;
                options.memory_limit = megabytes
                    .checked_mul(BYTES_PER_MEGABYTE)
                    .ok_or("--memory-limit is too large")?;
            }
            "--input" if is_call => input = Some(PathBuf::from(get_value()?)),
            _ => return Err(format!("{argument} is not an option; see --help")),
        }
    }
    match tool {
        Some(tool) => Ok(Command::Call {
            tool,
            input,
            options,
        }),
        None => Ok(Command::Serve(options)),
    }
}

fn read_alone(
    mut arguments: impl Iterator<Item = String>,
    command: Command,
) -> Result<Command, String> {
    arguments.next();
    match arguments.next() {
        Some(argument) => Err(format!("{argument} is not an option here; see --help")),
        None => Ok(command),
    }
}

fn read_positive<T: std::str::FromStr + PartialEq + Default>(
    name: &str,
    text: &str,
) -> Result<T, String> {
    match text.parse::<T>() {
        Ok(number) if number != T::default() => Ok(number),
        _ => Err(format!("{name} must be a whole number above 0, got {text}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Result<Command, String> {
        read_command(text.split_whitespace().map(str::to_string))
    }

    #[test]
    fn reads_server_options() {
        let Command::Serve(options) = read("--allow data --allow more --max-rows 10").unwrap()
        else {
            panic!("expected serve");
        };
        assert_eq!(
            options.allowed_folders,
            vec![PathBuf::from("data"), PathBuf::from("more")]
        );
        assert_eq!(options.max_rows, 10);
    }

    #[test]
    fn reads_call() {
        let command = read("call find_peaks --input request.json").unwrap();
        assert!(matches!(command, Command::Call { ref tool, .. } if tool == "find_peaks"));
    }

    #[test]
    fn rejects_unknown_options() {
        assert!(read("--alow data").is_err());
        assert!(read("--input request.json").is_err());
        assert!(read("--allow").is_err());
        assert!(read("--version extra").is_err());
        assert!(read("--max-rows 0").is_err());
        assert!(read("--memory-limit 0").is_err());
    }
}
