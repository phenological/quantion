use quantion::draw_peaks;
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind, OVERWRITE,
        options::fit_peak::{
            DRAW_PEAKS_FIELDS, PEAK_PARAMETERS_FIELDS, read_draw_peaks_options,
            read_peak_parameters,
        },
    },
    state::Context,
    tools::{
        Tool,
        calculate_eic::{get_summary, get_trace_table},
        table_output::SaveRequest,
    },
};

pub struct DrawPeaks;

const FIELDS: &[Field] = &[
    Field::required(
        "params",
        Kind::ObjectList(PEAK_PARAMETERS_FIELDS),
        "Peaks to draw, for example from fit_peak",
    ),
    Field::optional("options", Kind::Object(DRAW_PEAKS_FIELDS), ""),
    Field::optional("save", Kind::Text, "CSV path for every point"),
    OVERWRITE,
];

impl Tool for DrawPeaks {
    fn name(&self) -> &'static str {
        "draw_peaks"
    }

    fn description(&self) -> &'static str {
        "Simulate a chromatogram from peak curves. Overlapping peaks add up. Returns a summary; the points go only to a CSV with save."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["draw_peaks"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let params = input.get_object_list("params", read_peak_parameters)?;
        let options = input.get_object("options", read_draw_peaks_options)?;
        let overwrite = input.get_flag("overwrite", false)?;
        let save = SaveRequest::read(input, context, "save", overwrite)?;
        let data = draw_peaks(&params, options)?;
        let mut output = get_summary(&data);
        if let Some(save) = save {
            save.write(
                &get_trace_table(&data, None)?.to_csv(),
                &mut output,
                "saved",
            )?;
        }
        Ok(output)
    }
}
