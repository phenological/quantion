use quantion::{FindTargetsOptions, FindTransitionsOptions, Target, Transition};

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind,
        options::{
            calculate_eic::{EIC_FIELDS, read_calculate_eic_options},
            features::THREADS,
            find_peaks::{FIND_PEAKS_FIELDS, read_find_peaks_options},
        },
    },
};

pub const TARGET_FIELDS: &[Field] = &[
    Field::optional("id", Kind::Text, ""),
    Field::required("mz", Kind::Number, "m/z of the ion"),
    Field::required("rt", Kind::Number, "Expected retention time"),
    Field::optional(
        "range",
        Kind::Number,
        "Search rt ± range. Missing searches the whole EIC",
    ),
];

pub const TRANSITION_FIELDS: &[Field] = &[
    Field::optional("id", Kind::Text, ""),
    Field::required("precursor", Kind::Number, ""),
    Field::required("product", Kind::Number, ""),
    Field::required("rt", Kind::Number, "Expected retention time"),
    Field::required(
        "range",
        Kind::Number,
        "Search rt ± range, 0 searches the whole trace",
    ),
];

pub const FIND_TARGETS_FIELDS: &[Field] = &[
    Field::optional("eic", Kind::Object(EIC_FIELDS), ""),
    Field::optional("find_peaks", Kind::Object(FIND_PEAKS_FIELDS), ""),
    THREADS,
];

pub const FIND_TRANSITIONS_FIELDS: &[Field] = &[
    Field::optional("find_peaks", Kind::Object(FIND_PEAKS_FIELDS), ""),
    THREADS,
];

pub fn read_target(input: &mut Input) -> Result<Target, ToolError> {
    Ok(Target {
        id: input.get_optional_text("id")?,
        mz: input.get_required_number("mz")?,
        rt: input.get_required_number("rt")?,
        range: input.get_optional_number("range")?,
    })
}

pub fn read_transition(input: &mut Input) -> Result<Transition, ToolError> {
    Ok(Transition {
        id: input.get_optional_text("id")?,
        precursor: input.get_required_number("precursor")?,
        product: input.get_required_number("product")?,
        rt: input.get_required_number("rt")?,
        range: input.get_required_number("range")?,
    })
}

pub fn read_find_targets_options(input: &mut Input) -> Result<FindTargetsOptions, ToolError> {
    let default = FindTargetsOptions::default();
    Ok(FindTargetsOptions {
        eic: input.get_object("eic", read_calculate_eic_options)?,
        find_peaks: input.get_object("find_peaks", read_find_peaks_options)?,
        threads: input.get_count("threads", default.threads)?,
    })
}

pub fn read_find_transitions_options(
    input: &mut Input,
) -> Result<FindTransitionsOptions, ToolError> {
    let default = FindTransitionsOptions::default();
    Ok(FindTransitionsOptions {
        find_peaks: input.get_object("find_peaks", read_find_peaks_options)?,
        threads: input.get_count("threads", default.threads)?,
    })
}
