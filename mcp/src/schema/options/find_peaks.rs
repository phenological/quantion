use quantion::{FindPeaksOptions, PeakShape};

use crate::{
    error::ToolError,
    input::Input,
    schema::{Field, Kind},
};

pub const PEAK_SHAPES: &[&str] = &["EMG", "GAUSSIAN"];

pub const FIND_PEAKS_FIELDS: &[Field] = &[
    Field::optional("min_integral", Kind::Number, ""),
    Field::optional("min_intensity", Kind::Number, ""),
    Field::optional("min_peak_width_points", Kind::Count, ""),
    Field::optional("noise", Kind::Number, ""),
    Field::optional("auto_noise", Kind::Flag, ""),
    Field::optional("auto_baseline", Kind::Flag, ""),
    Field::optional("lambda", Kind::Number, ""),
    Field::optional("max_iterations", Kind::Count, ""),
    Field::optional("allow_overlap", Kind::Flag, ""),
    Field::optional("min_snr", Kind::Number, ""),
    Field::optional("min_r2", Kind::Number, ""),
    Field::optional("shape", Kind::Choice(PEAK_SHAPES), ""),
    Field::optional("kernel_size", Kind::Count, ""),
];

pub fn read_find_peaks_options(input: &mut Input) -> Result<FindPeaksOptions, ToolError> {
    let default = FindPeaksOptions::default();
    Ok(FindPeaksOptions {
        min_integral: input
            .get_optional_number("min_integral")?
            .or(default.min_integral),
        min_intensity: input.get_number("min_intensity", default.min_intensity)?,
        min_peak_width_points: input
            .get_count("min_peak_width_points", default.min_peak_width_points)?,
        noise: input.get_optional_number("noise")?.or(default.noise),
        auto_noise: input.get_flag("auto_noise", default.auto_noise)?,
        auto_baseline: input.get_flag("auto_baseline", default.auto_baseline)?,
        lambda: input.get_number("lambda", default.lambda)?,
        max_iterations: input.get_count("max_iterations", default.max_iterations)?,
        allow_overlap: input.get_flag("allow_overlap", default.allow_overlap)?,
        min_snr: input.get_number("min_snr", default.min_snr)?,
        min_r2: input.get_number("min_r2", default.min_r2)?,
        shape: read_peak_shape(input, default.shape)?,
        kernel_size: input.get_count("kernel_size", default.kernel_size)?,
    })
}

pub fn read_peak_shape(input: &mut Input, default: PeakShape) -> Result<PeakShape, ToolError> {
    match input.get_optional_text("shape")?.as_deref() {
        Some("EMG") => Ok(PeakShape::EMG),
        Some("GAUSSIAN") => Ok(PeakShape::GAUSSIAN),
        Some(other) => Err(ToolError::invalid(
            "shape",
            format!("must be EMG or GAUSSIAN, got {other}"),
        )),
        None => Ok(default),
    }
}
