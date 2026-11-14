use quantion::{
    DeisotopeFeaturesOptions, FindFeaturesOptions, GridOptions, GroupingOptions,
    MatchFeaturesOptions,
};

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind,
        options::{
            calculate_eic::{EIC_FIELDS, read_calculate_eic_options},
            find_peaks::{FIND_PEAKS_FIELDS, read_find_peaks_options},
        },
    },
};

pub const THREADS: Field = Field::optional("threads", Kind::Count, "");

pub const GRID_FIELDS: &[Field] = &[
    Field::optional("start", Kind::Number, ""),
    Field::optional("end", Kind::Number, ""),
    Field::optional("step_size", Kind::Number, ""),
];

pub const GROUPING_FIELDS: &[Field] = &[
    Field::optional("ppm_tolerance", Kind::Number, ""),
    Field::optional("mz_tolerance", Kind::Number, ""),
    Field::optional("rt_tolerance", Kind::Number, ""),
    Field::optional(
        "min_frequency",
        Kind::Number,
        "Smallest share of samples, 0 to 1",
    ),
];

pub const DEISOTOPE_FIELDS: &[Field] = &[
    Field::optional("rt_tolerance", Kind::Number, ""),
    Field::optional("mz_tolerance", Kind::Number, ""),
];

pub const FIND_FEATURES_FIELDS: &[Field] = &[
    Field::optional("eic", Kind::Object(EIC_FIELDS), ""),
    Field::optional("grid", Kind::Object(GRID_FIELDS), ""),
    Field::optional("find_peaks", Kind::Object(FIND_PEAKS_FIELDS), ""),
    THREADS,
];

pub const MATCH_FEATURES_FIELDS: &[Field] = &[
    Field::optional("eic", Kind::Object(EIC_FIELDS), ""),
    Field::optional("grid", Kind::Object(GRID_FIELDS), ""),
    Field::optional("grouping", Kind::Object(GROUPING_FIELDS), ""),
    Field::optional("find_peaks", Kind::Object(FIND_PEAKS_FIELDS), ""),
    THREADS,
];

pub fn read_grid_options(input: &mut Input) -> Result<GridOptions, ToolError> {
    let default = GridOptions::default();
    Ok(GridOptions {
        start: input.get_number("start", default.start)?,
        end: input.get_number("end", default.end)?,
        step_size: input.get_number("step_size", default.step_size)?,
    })
}

pub fn read_grouping_options(input: &mut Input) -> Result<GroupingOptions, ToolError> {
    let default = GroupingOptions::default();
    Ok(GroupingOptions {
        ppm_tolerance: input.get_number("ppm_tolerance", default.ppm_tolerance)?,
        mz_tolerance: input.get_number("mz_tolerance", default.mz_tolerance)?,
        rt_tolerance: input.get_number("rt_tolerance", default.rt_tolerance)?,
        min_frequency: input.get_number("min_frequency", default.min_frequency)?,
    })
}

pub fn read_deisotope_options(input: &mut Input) -> Result<DeisotopeFeaturesOptions, ToolError> {
    let default = DeisotopeFeaturesOptions::default();
    Ok(DeisotopeFeaturesOptions {
        rt_tolerance: input.get_number("rt_tolerance", default.rt_tolerance)?,
        mz_tolerance: input.get_number("mz_tolerance", default.mz_tolerance)?,
    })
}

pub fn read_find_features_options(input: &mut Input) -> Result<FindFeaturesOptions, ToolError> {
    let default = FindFeaturesOptions::default();
    Ok(FindFeaturesOptions {
        eic: input.get_object("eic", read_calculate_eic_options)?,
        grid: input.get_object("grid", read_grid_options)?,
        find_peaks: input.get_object("find_peaks", read_find_peaks_options)?,
        threads: input.get_count("threads", default.threads)?,
    })
}

pub fn read_match_features_options(input: &mut Input) -> Result<MatchFeaturesOptions, ToolError> {
    let default = MatchFeaturesOptions::default();
    Ok(MatchFeaturesOptions {
        eic: input.get_object("eic", read_calculate_eic_options)?,
        grid: input.get_object("grid", read_grid_options)?,
        grouping: input.get_object("grouping", read_grouping_options)?,
        find_peaks: input.get_object("find_peaks", read_find_peaks_options)?,
        threads: input.get_count("threads", default.threads)?,
    })
}
