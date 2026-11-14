use std::fs;

use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    output::{
        round::Unit,
        table::{Cell, Column, Table},
    },
    schema::{Field, Kind},
    state::{Context, file_cache::FileFormat},
    tools::Tool,
};

pub struct ListFiles;

const FIELDS: &[Field] = &[
    Field::optional("folder", Kind::Text, "Defaults to the first allowed folder"),
    Field::optional("limit", Kind::Count, "Most rows to return"),
];

impl Tool for ListFiles {
    fn name(&self) -> &'static str {
        "list_files"
    }

    fn description(&self) -> &'static str {
        "List the .ion and .mzML files in a folder, with their size in bytes."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &[]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let folder = match input.get_optional_text("folder")? {
            Some(folder) => context.folders.get_folder_path(&folder)?,
            None => context.folders.get_main_folder().to_path_buf(),
        };
        let limit = input.get_count("limit", context.max_rows)?;
        let entries = fs::read_dir(&folder).map_err(|_| {
            ToolError::invalid(
                folder.display().to_string(),
                "is not a folder that can be read",
            )
        })?;
        let mut files: Vec<(String, FileFormat, u64)> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                let format = FileFormat::find(&path)?;
                if !context.folders.is_allowed(&path) {
                    return None;
                }
                let metadata = fs::metadata(&path)
                    .ok()
                    .filter(|metadata| metadata.is_file())?;
                let name = entry.file_name().to_str()?.to_string();
                Some((name, format, metadata.len()))
            })
            .collect();
        files.sort_by(|first, second| first.0.cmp(&second.0));
        let mut table = Table::new(vec![
            Column {
                name: "name",
                unit: Unit::Text,
            },
            Column {
                name: "format",
                unit: Unit::Text,
            },
            Column {
                name: "size",
                unit: Unit::Count,
            },
        ]);
        for (name, format, size) in files {
            table.add_row(vec![
                Cell::Text(name),
                Cell::Text(format.name().to_string()),
                Cell::Count(size),
            ]);
        }
        Ok(table.to_json(limit))
    }
}
