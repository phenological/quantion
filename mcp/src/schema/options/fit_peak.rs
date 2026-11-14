use quantion::{DrawPeaksOptions, FitPeakOptions, PeakParameters};

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind,
        options::find_peaks::{PEAK_SHAPES, read_peak_shape},
    },
};

pub const FIT_PEAK_FIELDS: &[Field] = &[
    Field::optional("shape", Kind::Choice(PEAK_SHAPES), ""),
    Field::optional(
        "intensity",
        Kind::Number,
        "Apex intensity to start from, 0 finds it",
    ),
];

pub const DRAW_PEAKS_FIELDS: &[Field] = &[
    Field::optional("from", Kind::Number, "0 with to 0 covers every peak"),
    Field::optional("to", Kind::Number, ""),
    Field::optional("points", Kind::Count, ""),
];

pub const PEAK_PARAMETERS_FIELDS: &[Field] = &[
    Field::required("height", Kind::Number, ""),
    Field::required("center", Kind::Number, "Apex rt"),
    Field::required("fwhm", Kind::Number, "In minutes"),
    Field::optional("tail", Kind::Number, "0 is Gaussian"),
    Field::optional("r2", Kind::Number, ""),
];

pub fn read_fit_peak_options(input: &mut Input) -> Result<FitPeakOptions, ToolError> {
    let default = FitPeakOptions::default();
    Ok(FitPeakOptions {
        shape: read_peak_shape(input, default.shape)?,
        intensity: input.get_number("intensity", default.intensity)?,
    })
}

pub fn read_draw_peaks_options(input: &mut Input) -> Result<DrawPeaksOptions, ToolError> {
    let default = DrawPeaksOptions::default();
    Ok(DrawPeaksOptions {
        from: input.get_number("from", default.from)?,
        to: input.get_number("to", default.to)?,
        points: input.get_count("points", default.points)?,
    })
}

pub fn read_peak_parameters(input: &mut Input) -> Result<PeakParameters, ToolError> {
    Ok(PeakParameters {
        height: input.get_required_number("height")?,
        center: input.get_required_number("center")?,
        fwhm: input.get_required_number("fwhm")?,
        tail: input.get_number("tail", 0.0)?,
        r2: input.get_number("r2", 0.0)?,
    })
}
