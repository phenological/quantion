use quantion::find_transitions;
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::targets::{
            FIND_TRANSITIONS_FIELDS, TRANSITION_FIELDS, read_find_transitions_options,
            read_transition,
        },
    },
    state::Context,
    tools::{
        Tool,
        find_targets::FILES,
        table_output::read_table_request,
        target_table::{add_target_rows, get_target_table},
    },
};

pub struct FindTransitions;

const FIELDS: &[Field] = &[
    FILES,
    Field::required(
        "targets",
        Kind::ObjectList(TRANSITION_FIELDS),
        "SRM/MRM transitions to quantify",
    ),
    Field::optional("options", Kind::Object(FIND_TRANSITIONS_FIELDS), ""),
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for FindTransitions {
    fn name(&self) -> &'static str {
        "find_transitions"
    }

    fn description(&self) -> &'static str {
        "Quantify SRM/MRM transitions in one or more samples. One row per file and transition."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["find_transitions"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let files = input.get_text_list("files")?;
        let transitions = input.get_object_list("targets", read_transition)?;
        let options = input.get_object("options", read_find_transitions_options)?;
        let request = read_table_request(input, context)?;
        let ids: Vec<Option<String>> = transitions
            .iter()
            .map(|transition| transition.id.clone())
            .collect();
        let mut table = get_target_table();
        for file in &files {
            let full_path = context.folders.get_input_path(file)?;
            let sample = context.files.get_file(&full_path)?;
            let peaks = find_transitions(sample, &transitions, options.clone())?;
            add_target_rows(&mut table, file, &ids, &peaks);
        }
        request.get_output(&table)
    }
}
