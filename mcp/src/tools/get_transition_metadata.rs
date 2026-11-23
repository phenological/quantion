use quantion::{GetTransitionMetadataOptions, get_transition_metadata};
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{Field, Kind, LIMIT, OVERWRITE, SAVE},
    state::Context,
    tools::{
        Tool,
        get_scan_metadata::get_metadata_table,
        get_transition::{QUERY_FIELDS, read_transition_query},
        table_output::read_table_request,
    },
};

pub struct GetTransitionMetadata;

const OPTION_FIELDS: &[Field] = &[Field::optional(
    "tags",
    Kind::TextList,
    "Tags to keep, such as precursor",
)];

const FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, ""),
    QUERY_FIELDS[0],
    QUERY_FIELDS[1],
    QUERY_FIELDS[2],
    Field::optional("options", Kind::Object(OPTION_FIELDS), ""),
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for GetTransitionMetadata {
    fn name(&self) -> &'static str {
        "get_transition_metadata"
    }

    fn description(&self) -> &'static str {
        "Read the params of one stored chromatogram."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["get_transition_metadata"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let file = input.get_text("file")?;
        let query = read_transition_query(input)?;
        let options = input.get_object("options", read_options)?;
        let request = read_table_request(input, context)?;
        let full_path = context.folders.get_input_path(&file)?;
        let rows = get_transition_metadata(context.files.get_file(&full_path)?, query, options)?;
        request.get_output(&get_metadata_table(&rows))
    }
}

fn read_options(input: &mut Input) -> Result<GetTransitionMetadataOptions, ToolError> {
    let default = GetTransitionMetadataOptions::default();
    Ok(GetTransitionMetadataOptions {
        tags: input
            .get_optional_text_list("tags")?
            .unwrap_or(default.tags),
    })
}
