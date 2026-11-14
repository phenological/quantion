use quantion::get_peak;
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind,
        options::{
            calculate_eic::DATA_FIELDS,
            get_peak::{GET_PEAK_FIELDS, read_get_peak_options},
        },
    },
    state::Context,
    tools::{Tool, peak_table::get_peak_json, trace::read_data},
};

pub struct GetPeak;

const FIELDS: &[Field] = &[
    Field::required("data", Kind::Object(DATA_FIELDS), ""),
    Field::required("rt", Kind::Number, "Expected retention time"),
    Field::optional("options", Kind::Object(GET_PEAK_FIELDS), ""),
];

impl Tool for GetPeak {
    fn name(&self) -> &'static str {
        "get_peak"
    }

    fn description(&self) -> &'static str {
        "Find the one peak closest to rt in a trace. An empty peak has every field 0."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["get_peak"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let data = read_data(input, context)?;
        let rt = input.get_required_number("rt")?;
        let options = input.get_object("options", read_get_peak_options)?;
        let peak = get_peak(&data, rt, options)?;
        Ok(get_peak_json(&peak))
    }
}
