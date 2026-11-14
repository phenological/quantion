use quantion::find_targets;
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::targets::{
            FIND_TARGETS_FIELDS, TARGET_FIELDS, read_find_targets_options, read_target,
        },
    },
    state::Context,
    tools::{
        Tool,
        table_output::read_table_request,
        target_table::{add_target_rows, get_target_table},
    },
};

pub struct FindTargets;

pub const FILES: Field = Field::required("files", Kind::TextList, "Samples (.ion or .mzML)");

const FIELDS: &[Field] = &[
    FILES,
    Field::required(
        "targets",
        Kind::ObjectList(TARGET_FIELDS),
        "Compounds to quantify",
    ),
    Field::optional("options", Kind::Object(FIND_TARGETS_FIELDS), ""),
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for FindTargets {
    fn name(&self) -> &'static str {
        "find_targets"
    }

    fn description(&self) -> &'static str {
        "Quantify known compounds (targets) in one or more samples. One row per file and target; a target without a peak has every field 0."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["find_targets"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let files = input.get_text_list("files")?;
        let targets = input.get_object_list("targets", read_target)?;
        let options = input.get_object("options", read_find_targets_options)?;
        let request = read_table_request(input, context)?;
        let ids: Vec<Option<String>> = targets.iter().map(|target| target.id.clone()).collect();
        let mut table = get_target_table();
        for file in &files {
            let full_path = context.folders.get_input_path(file)?;
            let sample = context.files.get_file(&full_path)?;
            let peaks = find_targets(sample, &targets, options.clone())?;
            add_target_rows(&mut table, file, &ids, &peaks);
        }
        request.get_output(&table)
    }
}
