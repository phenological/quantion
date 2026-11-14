use quantion::write_mzml;
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    output::save::get_written_output,
    safety::folders::is_same_file,
    schema::{
        Field, Kind,
        options::write_ion::{WRITE_MZML_FIELDS, read_write_mzml_options},
    },
    state::Context,
    tools::Tool,
};

pub struct WriteMzml;

const FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, ""),
    Field::required("options", Kind::Object(WRITE_MZML_FIELDS), ""),
];

impl Tool for WriteMzml {
    fn name(&self) -> &'static str {
        "write_mzml"
    }

    fn description(&self) -> &'static str {
        "Save a sample as mzML."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["write_mzml"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let file = input.get_text("file")?;
        let output = input.get_object("options", |options| {
            read_write_mzml_options(options, &context.folders)
        })?;
        let Some(out) = output.options.out.clone() else {
            return Err(ToolError::internal("write_mzml has no output path"));
        };
        let full_path = context.folders.get_input_path(&file)?;
        if is_same_file(&full_path, &out) {
            return Err(ToolError::invalid(
                "options.out",
                "must not be the same file as file",
            ));
        }
        context.files.forget(&out);
        write_mzml(context.files.get_file(&full_path)?, output.options)?;
        Ok(get_written_output(&output.text, &out))
    }
}
