use quantion::{Scan, get_scans};
use serde_json::{Value, json};

use crate::{
    error::ToolError,
    input::Input,
    output::{
        round::{Unit, round_value},
        table::{Cell, Column, Table},
    },
    safety::folders::is_same_file,
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::scans::{
            GET_SCANS_FIELDS, SCAN_QUERY_FIELDS, read_get_scans_options, read_scan_query,
        },
    },
    state::Context,
    tools::{
        Tool,
        table_output::{SaveRequest, read_table_request},
    },
};

pub struct GetScans;

const FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, ""),
    Field::required(
        "query",
        Kind::Object(SCAN_QUERY_FIELDS),
        "Give rt, mz or both",
    ),
    Field::optional("options", Kind::Object(GET_SCANS_FIELDS), ""),
    Field::optional(
        "peaks",
        Kind::Count,
        "Most intense m/z values to return per scan",
    ),
    Field::optional(
        "save_spectra",
        Kind::Text,
        "CSV path for every m/z and intensity",
    ),
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for GetScans {
    fn name(&self) -> &'static str {
        "get_scans"
    }

    fn description(&self) -> &'static str {
        "Read scans by rt, precursor m/z or both. One row per scan with its details; spectra only with peaks or save_spectra."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["get_scans"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let file = input.get_text("file")?;
        let query = input.get_object("query", read_scan_query)?;
        let options = input.get_object("options", read_get_scans_options)?;
        let keys: Vec<String> = options
            .arrays
            .iter()
            .map(|array| array.key.clone())
            .collect();
        let peaks = input.get_count("peaks", 0)?;
        let request = read_table_request(input, context)?;
        let save_spectra = SaveRequest::read(input, context, "save_spectra", request.overwrite())?;
        if let Some(spectra) = &save_spectra
            && request
                .save_path()
                .is_some_and(|path| is_same_file(path, spectra.path()))
        {
            return Err(ToolError::invalid(
                "save_spectra",
                "must not be the same file as save",
            ));
        }
        let full_path = context.folders.get_input_path(&file)?;
        let scans = get_scans(context.files.get_file(&full_path)?, query, options)?;
        let mut output = request.get_output(&get_scan_table(&scans))?;
        if peaks > 0 {
            let shown = output["shown"].as_u64().unwrap_or(0) as usize;
            let spectra: Vec<Value> = scans
                .iter()
                .take(shown)
                .map(|scan| get_top_peaks(scan, peaks))
                .collect();
            output["spectra"] = Value::Array(spectra);
        }
        if let Some(spectra) = save_spectra {
            spectra.write(
                &get_spectra_csv(&scans, &keys),
                &mut output,
                "saved_spectra",
            )?;
        }
        Ok(output)
    }
}

fn get_scan_table(scans: &[Scan]) -> Table {
    let mut table = Table::new(vec![
        Column {
            name: "index",
            unit: Unit::Count,
        },
        Column {
            name: "rt",
            unit: Unit::Rt,
        },
        Column {
            name: "level",
            unit: Unit::Count,
        },
        Column {
            name: "precursor_mz",
            unit: Unit::Mz,
        },
        Column {
            name: "base_peak_mz",
            unit: Unit::Mz,
        },
        Column {
            name: "base_peak_intensity",
            unit: Unit::Intensity,
        },
        Column {
            name: "total_ion_current",
            unit: Unit::Intensity,
        },
        Column {
            name: "polarity",
            unit: Unit::Count,
        },
        Column {
            name: "points",
            unit: Unit::Count,
        },
    ]);
    for scan in scans {
        table.add_row(vec![
            Cell::Count(scan.index as u64),
            Cell::Number(scan.rt),
            Cell::Count(u64::from(scan.level)),
            Cell::Number(scan.precursor_mz),
            Cell::Number(scan.base_peak_mz),
            Cell::Number(scan.base_peak_intensity),
            Cell::Number(scan.total_ion_current),
            Cell::Count(u64::from(scan.polarity)),
            Cell::Count(scan.mz.len() as u64),
        ]);
    }
    table
}

fn get_top_peaks(scan: &Scan, count: usize) -> Value {
    let mut order: Vec<usize> = (0..scan.mz.len().min(scan.intensity.len())).collect();
    order.sort_by(|first, second| scan.intensity[*second].total_cmp(&scan.intensity[*first]));
    order.truncate(count);
    order.sort_unstable();
    let mz: Vec<f64> = order
        .iter()
        .map(|&index| round_value(scan.mz[index], Unit::Mz))
        .collect();
    let intensity: Vec<f64> = order
        .iter()
        .map(|&index| round_value(scan.intensity[index], Unit::Intensity))
        .collect();
    json!({ "index": scan.index, "mz": mz, "intensity": intensity })
}

fn get_spectra_csv(scans: &[Scan], keys: &[String]) -> String {
    let mut text = format!("index,rt,{}\n", keys.join(","));
    for scan in scans {
        let arrays: Vec<&Vec<f64>> = keys.iter().map(|key| &scan[key.as_str()]).collect();
        let length = arrays.iter().map(|values| values.len()).max().unwrap_or(0);
        for point in 0..length {
            let cells: Vec<String> = arrays
                .iter()
                .map(|values| values.get(point).map_or(String::new(), f64::to_string))
                .collect();
            text.push_str(&format!("{},{},{}\n", scan.index, scan.rt, cells.join(",")));
        }
    }
    text
}
