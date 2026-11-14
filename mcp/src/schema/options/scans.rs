use quantion::{GetScanMetadataOptions, GetScansOptions, ScanArray, ScanQuery};

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind,
        options::calculate_eic::{TIME_UNITS, read_time_unit},
    },
};

pub const SCAN_QUERY_FIELDS: &[Field] = &[
    Field::optional(
        "rt",
        Kind::Position,
        "One rt for the closest scan, or {from, to}",
    ),
    Field::optional(
        "mz",
        Kind::Position,
        "Precursor m/z, one value for the closest scan, or {from, to}",
    ),
];

pub const SCAN_ARRAY_FIELDS: &[Field] = &[
    Field::required("key", Kind::Text, ""),
    Field::required("accession", Kind::Text, ""),
];

pub const GET_SCANS_FIELDS: &[Field] = &[
    Field::optional("level", Kind::Count, "MS level, 0 reads every level"),
    Field::optional("time_unit", Kind::Choice(TIME_UNITS), ""),
    Field::optional(
        "arrays",
        Kind::ObjectList(SCAN_ARRAY_FIELDS),
        "Data arrays to read; default mz and intensity",
    ),
];

pub const GET_SCAN_METADATA_FIELDS: &[Field] = &[Field::optional(
    "tags",
    Kind::TextList,
    "Tags to keep, such as scan or precursor",
)];

pub fn read_scan_query(input: &mut Input) -> Result<ScanQuery, ToolError> {
    Ok(ScanQuery {
        rt: input.get_position("rt")?,
        mz: input.get_position("mz")?,
    })
}

pub fn read_get_scans_options(input: &mut Input) -> Result<GetScansOptions, ToolError> {
    let default = GetScansOptions::default();
    Ok(GetScansOptions {
        level: input.get_count("level", default.level)?,
        time_unit: read_time_unit(input, default.time_unit)?,
        arrays: input
            .get_optional_object_list("arrays", read_scan_array)?
            .unwrap_or(default.arrays),
    })
}

pub fn read_get_scan_metadata_options(
    input: &mut Input,
) -> Result<GetScanMetadataOptions, ToolError> {
    let default = GetScanMetadataOptions::default();
    Ok(GetScanMetadataOptions {
        tags: input
            .get_optional_text_list("tags")?
            .unwrap_or(default.tags),
    })
}

fn read_scan_array(input: &mut Input) -> Result<ScanArray, ToolError> {
    Ok(ScanArray {
        key: input.get_text("key")?,
        accession: input.get_text("accession")?,
    })
}
