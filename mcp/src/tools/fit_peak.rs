use quantion::{PeakParameters, fit_peak};
use serde_json::{Value, json};

use crate::{
    error::ToolError,
    input::Input,
    output::round::{Unit, round_value},
    schema::{
        Field, Kind,
        options::{
            calculate_eic::DATA_FIELDS,
            fit_peak::{FIT_PEAK_FIELDS, read_fit_peak_options},
        },
    },
    state::Context,
    tools::{Tool, trace::read_data},
};

pub struct FitPeak;

const FIELDS: &[Field] = &[
    Field::required("data", Kind::Object(DATA_FIELDS), ""),
    Field::required("rt", Kind::Number, "rt of the peak, within 0.05 min"),
    Field::optional("options", Kind::Object(FIT_PEAK_FIELDS), ""),
];

impl Tool for FitPeak {
    fn name(&self) -> &'static str {
        "fit_peak"
    }

    fn description(&self) -> &'static str {
        "Fit a Gaussian or EMG curve to the peak at rt: height, center, fwhm, tail and r2. null when no peak is found or the fit fails."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["fit_peak"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let data = read_data(input, context)?;
        let rt = input.get_required_number("rt")?;
        let options = input.get_object("options", read_fit_peak_options)?;
        let parameters = fit_peak(&data, rt, options)?;
        Ok(parameters.as_ref().map_or(Value::Null, get_parameters_json))
    }
}

pub fn get_parameters_json(parameters: &PeakParameters) -> Value {
    json!({
        "height": round_value(parameters.height, Unit::Intensity),
        "center": round_value(parameters.center, Unit::Rt),
        "fwhm": round_value(parameters.fwhm, Unit::Rt),
        "tail": round_value(parameters.tail, Unit::Rt),
        "r2": round_value(parameters.r2, Unit::Ratio),
    })
}
