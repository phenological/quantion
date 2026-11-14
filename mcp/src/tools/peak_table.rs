use quantion::Peak;
use serde_json::{Map, Value};

use crate::output::{
    round::Unit,
    table::{Cell, Column, Table, get_cell_json},
};

pub fn get_peak_columns() -> Vec<Column> {
    vec![
        Column {
            name: "rt",
            unit: Unit::Rt,
        },
        Column {
            name: "from",
            unit: Unit::Rt,
        },
        Column {
            name: "to",
            unit: Unit::Rt,
        },
        Column {
            name: "integral",
            unit: Unit::Intensity,
        },
        Column {
            name: "intensity",
            unit: Unit::Intensity,
        },
        Column {
            name: "n_points",
            unit: Unit::Count,
        },
    ]
}

pub fn get_peak_row(peak: &Peak) -> Vec<Cell> {
    vec![
        Cell::Number(peak.rt),
        Cell::Number(peak.from),
        Cell::Number(peak.to),
        Cell::Number(peak.integral),
        Cell::Number(peak.intensity),
        Cell::Count(peak.n_points as u64),
    ]
}

pub fn get_peak_table(peaks: &[Peak]) -> Table {
    let mut table = Table::new(get_peak_columns());
    for peak in peaks {
        table.add_row(get_peak_row(peak));
    }
    table
}

pub fn get_peak_json(peak: &Peak) -> Value {
    let mut output = Map::new();
    for (column, cell) in get_peak_columns().iter().zip(get_peak_row(peak)) {
        output.insert(column.name.to_string(), get_cell_json(&cell, column.unit));
    }
    Value::Object(output)
}
