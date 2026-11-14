use std::{fs, path::Path};

use quantion::{DataXY, calculate_eic};

use crate::{
    error::ToolError, input::Input, schema::options::calculate_eic::read_calculate_eic_options,
    state::Context,
};

pub const TRACE_COLUMNS: [&str; 2] = ["rt", "intensity"];

pub fn read_data(input: &mut Input, context: &mut Context) -> Result<DataXY, ToolError> {
    input.get_object("data", |data| read_data_source(data, context))
}

fn read_data_source(input: &mut Input, context: &mut Context) -> Result<DataXY, ToolError> {
    let file = input.get_optional_text("file")?;
    let mz = input.get_optional_number("mz")?;
    let options = input.get_object("eic", read_calculate_eic_options)?;
    let path = input.get_optional_text("path")?;
    match (file, mz, path) {
        (Some(file), Some(mz), None) => {
            let full_path = context.folders.get_input_path(&file)?;
            let sample = context.files.get_file(&full_path)?;
            Ok(calculate_eic(sample, mz, options)?)
        }
        (None, None, Some(path)) => {
            let full_path = context.folders.get_input_path(&path)?;
            read_trace_file(&full_path, &path)
        }
        _ => Err(ToolError::invalid("data", "needs file and mz, or path")
            .with_fix("give {file, mz} for an EIC, or {path} for a CSV saved by calculate_eic")),
    }
}

fn read_trace_file(full_path: &Path, path: &str) -> Result<DataXY, ToolError> {
    let text = fs::read_to_string(full_path).map_err(|_| ToolError::file_not_found(path))?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split(',').collect();
    if header.len() < 2 || header[..2] != TRACE_COLUMNS {
        return Err(
            ToolError::invalid(path, "must start with the columns rt,intensity")
                .with_fix("use a CSV saved by calculate_eic"),
        );
    }
    let mut data = DataXY {
        x: Vec::new(),
        y: Vec::new(),
    };
    for (index, line) in lines.enumerate().filter(|(_, line)| !line.is_empty()) {
        let mut cells = line.split(',');
        data.x.push(read_cell(cells.next(), path, index)?);
        data.y.push(read_cell(cells.next(), path, index)?);
    }
    Ok(data)
}

fn read_cell(cell: Option<&str>, path: &str, index: usize) -> Result<f64, ToolError> {
    cell.and_then(|text| text.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .ok_or_else(|| ToolError::invalid(path, format!("has a bad number on line {}", index + 2)))
}
