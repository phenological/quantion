use std::path::Path;

use quantion::AnnotateFeaturesOptions;

use crate::{
    error::ToolError,
    input::Input,
    schema::{Field, Kind},
};

pub const ANNOTATE_FEATURES_FIELDS: &[Field] = &[
    Field::optional("matches", Kind::Count, ""),
    Field::optional("ppm_tolerance", Kind::Number, ""),
    Field::optional("mz_tolerance", Kind::Number, ""),
    Field::optional(
        "rt_range",
        Kind::Number,
        "Largest apex to MS2 distance, in minutes",
    ),
    Field::optional(
        "min_relative_intensity",
        Kind::Number,
        "MS2 peaks below this share of the base peak are removed",
    ),
];

pub fn read_annotate_features_options(
    input: &mut Input,
    library: &Path,
    model: &Path,
) -> Result<AnnotateFeaturesOptions, ToolError> {
    let default = AnnotateFeaturesOptions::default();
    Ok(AnnotateFeaturesOptions {
        library: get_path_text(library)?,
        model: get_path_text(model)?,
        matches: input.get_count("matches", default.matches)?,
        ppm_tolerance: input.get_number("ppm_tolerance", default.ppm_tolerance)?,
        mz_tolerance: input.get_number("mz_tolerance", default.mz_tolerance)?,
        rt_range: input.get_number("rt_range", default.rt_range)?,
        min_relative_intensity: input
            .get_number("min_relative_intensity", default.min_relative_intensity)?,
    })
}

fn get_path_text(path: &Path) -> Result<String, ToolError> {
    path.to_str().map(str::to_string).ok_or_else(|| {
        ToolError::invalid(
            path.display().to_string(),
            "must be a path in plain text (UTF-8)",
        )
    })
}
