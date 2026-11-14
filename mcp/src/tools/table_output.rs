use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::{
    error::ToolError,
    input::Input,
    output::{save::save_text, table::Table},
    state::Context,
};

pub struct SaveRequest {
    text: String,
    path: PathBuf,
    overwrite: bool,
}

impl SaveRequest {
    pub fn read(
        input: &mut Input,
        context: &Context,
        name: &'static str,
        overwrite: bool,
    ) -> Result<Option<Self>, ToolError> {
        let Some(text) = input.get_optional_text(name)? else {
            return Ok(None);
        };
        let path = context.folders.get_output_path(&text, overwrite)?;
        Ok(Some(Self {
            text,
            path,
            overwrite,
        }))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, content: &str, output: &mut Value, name: &str) -> Result<(), ToolError> {
        save_text(&self.path, content, self.overwrite)?;
        output[name] = json!(self.text);
        Ok(())
    }
}

pub struct TableRequest {
    limit: usize,
    overwrite: bool,
    save: Option<SaveRequest>,
}

impl TableRequest {
    pub fn overwrite(&self) -> bool {
        self.overwrite
    }

    pub fn save_path(&self) -> Option<&Path> {
        self.save.as_ref().map(SaveRequest::path)
    }

    pub fn get_output(&self, table: &Table) -> Result<Value, ToolError> {
        let mut output = table.to_json(self.limit);
        match &self.save {
            Some(save) => save.write(&table.to_csv(), &mut output, "saved")?,
            None if table.row_count() > self.limit => {
                output["hint"] = json!("give save to get every row as CSV");
            }
            None => {}
        }
        Ok(output)
    }
}

pub fn read_table_request(input: &mut Input, context: &Context) -> Result<TableRequest, ToolError> {
    let limit = input.get_count("limit", context.max_rows)?;
    let overwrite = input.get_flag("overwrite", false)?;
    let save = SaveRequest::read(input, context, "save", overwrite)?;
    Ok(TableRequest {
        limit,
        overwrite,
        save,
    })
}
