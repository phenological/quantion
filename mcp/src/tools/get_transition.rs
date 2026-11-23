use quantion::{TransitionQuery, get_transition};
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{Field, Kind, OVERWRITE},
    state::Context,
    tools::{
        Tool,
        calculate_eic::{get_summary, get_trace_table},
        table_output::SaveRequest,
    },
};

pub struct GetTransition;

pub const QUERY_FIELDS: [Field; 3] = [
    Field::optional(
        "index",
        Kind::Count,
        "From 0, or give precursor and product",
    ),
    Field::optional("precursor", Kind::Number, "Q1 m/z"),
    Field::optional("product", Kind::Number, "Q3 m/z"),
];

const FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, ""),
    QUERY_FIELDS[0],
    QUERY_FIELDS[1],
    QUERY_FIELDS[2],
    Field::optional("save", Kind::Text, "CSV path for every point"),
    OVERWRITE,
];

impl Tool for GetTransition {
    fn name(&self) -> &'static str {
        "get_transition"
    }

    fn description(&self) -> &'static str {
        "Summarize one stored chromatogram (TIC, SRM or MRM): points, rt range, apex and noise. Points go to a CSV with save."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["get_transition"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let file = input.get_text("file")?;
        let query = read_transition_query(input)?;
        let overwrite = input.get_flag("overwrite", false)?;
        let save = SaveRequest::read(input, context, "save", overwrite)?;
        let full_path = context.folders.get_input_path(&file)?;
        let trace = get_transition(context.files.get_file(&full_path)?, query)?;
        let mut output = get_summary(&trace);
        if let Some(save) = save {
            save.write(
                &get_trace_table(&trace, None)?.to_csv(),
                &mut output,
                "saved",
            )?;
        }
        Ok(output)
    }
}

pub fn read_transition_query(input: &mut Input) -> Result<TransitionQuery, ToolError> {
    let index = input.get_optional_number("index")?;
    let precursor = input.get_optional_number("precursor")?;
    let product = input.get_optional_number("product")?;
    match (index, precursor, product) {
        (Some(index), None, None) => Ok(TransitionQuery::Index(index as usize)),
        (None, Some(precursor), Some(product)) => {
            Ok(TransitionQuery::Transition { precursor, product })
        }
        (Some(_), _, _) => Err(ToolError::invalid(
            "index",
            "cannot be given with precursor or product",
        )
        .with_fix("give index, or precursor and product")),
        _ => Err(ToolError::missing("precursor and product")
            .with_fix("give index, or both precursor and product")),
    }
}
