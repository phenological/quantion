use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use serde_json::{Value, json};

use crate::error::ToolError;

pub fn save_text(path: &Path, text: &str, overwrite: bool) -> Result<(), ToolError> {
    let display = path.display().to_string();
    let mut file = open_output(path, overwrite).map_err(|error| match error.kind() {
        std::io::ErrorKind::AlreadyExists => ToolError::file_exists(&display),
        _ => ToolError::new(
            "write_failed",
            format!("{display} could not be written: {error}"),
        ),
    })?;
    file.write_all(text.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|error| {
            ToolError::new(
                "write_failed",
                format!("{display} could not be written: {error}"),
            )
        })
}

fn open_output(path: &Path, overwrite: bool) -> std::io::Result<File> {
    if overwrite {
        File::create(path)
    } else {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
}

pub fn get_written_output(text: &str, path: &Path) -> Value {
    let size = path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
    json!({ "out": text, "size": size })
}
