use quantion::GetPeakOptions;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind,
        options::find_peaks::{FIND_PEAKS_FIELDS, read_find_peaks_options},
    },
};

pub const GET_PEAK_FIELDS: &[Field] = &[
    Field::optional(
        "range",
        Kind::Number,
        "Half-width of the search window around rt, in minutes. 0 searches the whole trace",
    ),
    Field::optional("find_peaks", Kind::Object(FIND_PEAKS_FIELDS), ""),
];

pub fn read_get_peak_options(input: &mut Input) -> Result<GetPeakOptions, ToolError> {
    let default = GetPeakOptions::default();
    Ok(GetPeakOptions {
        range: input.get_number("range", default.range)?,
        find_peaks: input.get_object("find_peaks", read_find_peaks_options)?,
    })
}
