use quantion::{CalculateBaselineOptions, CalculateEicOptions, TimeUnit};

use crate::{
    error::ToolError,
    input::Input,
    schema::{Field, Kind},
};

pub const TIME_UNITS: &[&str] = &["MINUTES", "SECONDS"];

pub const EIC_FIELDS: &[Field] = &[
    Field::optional("from", Kind::Number, ""),
    Field::optional("to", Kind::Number, ""),
    Field::optional("ppm_tolerance", Kind::Number, ""),
    Field::optional("mz_tolerance", Kind::Number, ""),
    Field::optional("time_unit", Kind::Choice(TIME_UNITS), ""),
];

pub const DATA_FIELDS: &[Field] = &[
    Field::optional("file", Kind::Text, "Sample for an EIC. Give it with mz"),
    Field::optional("mz", Kind::Number, "m/z of the ion"),
    Field::optional("eic", Kind::Object(EIC_FIELDS), ""),
    Field::optional(
        "path",
        Kind::Text,
        "CSV saved by calculate_eic, instead of file and mz",
    ),
];

pub const CALCULATE_BASELINE_FIELDS: &[Field] = &[
    Field::optional("lambda", Kind::Number, "Smoothness"),
    Field::optional("max_iterations", Kind::Count, ""),
];

pub fn read_calculate_baseline_options(
    input: &mut Input,
) -> Result<CalculateBaselineOptions, ToolError> {
    let default = CalculateBaselineOptions::default();
    Ok(CalculateBaselineOptions {
        lambda: input.get_number("lambda", default.lambda)?,
        max_iterations: input.get_count("max_iterations", default.max_iterations)?,
    })
}

pub fn read_calculate_eic_options(input: &mut Input) -> Result<CalculateEicOptions, ToolError> {
    let default = CalculateEicOptions::default();
    Ok(CalculateEicOptions {
        from: input.get_number("from", default.from)?,
        to: input.get_number("to", default.to)?,
        ppm_tolerance: input.get_number("ppm_tolerance", default.ppm_tolerance)?,
        mz_tolerance: input.get_number("mz_tolerance", default.mz_tolerance)?,
        time_unit: read_time_unit(input, default.time_unit)?,
    })
}

pub fn read_time_unit(input: &mut Input, default: TimeUnit) -> Result<TimeUnit, ToolError> {
    match input.get_optional_text("time_unit")?.as_deref() {
        Some("MINUTES") => Ok(TimeUnit::MINUTES),
        Some("SECONDS") => Ok(TimeUnit::SECONDS),
        Some(other) => Err(ToolError::invalid(
            "time_unit",
            format!("must be MINUTES or SECONDS, got {other}"),
        )),
        None => Ok(default),
    }
}
