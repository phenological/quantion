use quantion::Peak;

use crate::{
    output::{
        round::Unit,
        table::{Cell, Column, Table},
    },
    tools::peak_table::{get_peak_columns, get_peak_row},
};

pub fn get_target_table() -> Table {
    let mut columns = vec![
        Column {
            name: "file",
            unit: Unit::Text,
        },
        Column {
            name: "id",
            unit: Unit::Text,
        },
    ];
    columns.extend(get_peak_columns());
    Table::new(columns)
}

pub fn add_target_rows(table: &mut Table, file: &str, ids: &[Option<String>], peaks: &[Peak]) {
    for (id, peak) in ids.iter().zip(peaks) {
        let mut row = vec![
            Cell::Text(file.to_string()),
            id.clone().map_or(Cell::Empty, Cell::Text),
        ];
        row.extend(get_peak_row(peak));
        table.add_row(row);
    }
}
