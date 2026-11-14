use quantion::find_peaks;
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::{
            calculate_eic::DATA_FIELDS,
            find_peaks::{FIND_PEAKS_FIELDS, read_find_peaks_options},
        },
    },
    state::Context,
    tools::{Tool, peak_table::get_peak_table, table_output::read_table_request, trace::read_data},
};

pub struct FindPeaks;

const FIELDS: &[Field] = &[
    Field::required("data", Kind::Object(DATA_FIELDS), ""),
    Field::optional("options", Kind::Object(FIND_PEAKS_FIELDS), ""),
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for FindPeaks {
    fn name(&self) -> &'static str {
        "find_peaks"
    }

    fn description(&self) -> &'static str {
        "Find every peak in one trace, such as an EIC. Returns a peak table in rt order."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["find_peaks"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let data = read_data(input, context)?;
        let options = input.get_object("options", read_find_peaks_options)?;
        let request = read_table_request(input, context)?;
        let peaks = find_peaks(&data, options)?;
        request.get_output(&get_peak_table(&peaks))
    }
}
