use serde_json::{Value, json};

use crate::output::round::{Unit, round_value};

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Number(f64),
    Count(u64),
    Text(String),
    Empty,
}

pub struct Column {
    pub name: &'static str,
    pub unit: Unit,
}

pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<Cell>>,
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn add_row(&mut self, row: Vec<Cell>) {
        self.rows.push(row);
    }

    pub fn to_json(&self, limit: usize) -> Value {
        let names: Vec<&str> = self.columns.iter().map(|column| column.name).collect();
        let rows: Vec<Value> = self
            .rows
            .iter()
            .take(limit)
            .map(|row| self.get_row_json(row))
            .collect();
        json!({
            "columns": names,
            "rows": rows,
            "total": self.rows.len(),
            "shown": rows.len(),
        })
    }

    pub fn to_csv(&self) -> String {
        let names: Vec<&str> = self.columns.iter().map(|column| column.name).collect();
        let mut text = names.join(",");
        text.push('\n');
        for row in &self.rows {
            let cells: Vec<String> = row.iter().map(get_csv_cell).collect();
            text.push_str(&cells.join(","));
            text.push('\n');
        }
        text
    }

    fn get_row_json(&self, row: &[Cell]) -> Value {
        let cells: Vec<Value> = row
            .iter()
            .zip(&self.columns)
            .map(|(cell, column)| get_cell_json(cell, column.unit))
            .collect();
        Value::Array(cells)
    }
}

pub fn get_cell_json(cell: &Cell, unit: Unit) -> Value {
    match cell {
        Cell::Number(value) => json!(round_value(*value, unit)),
        Cell::Count(value) => json!(value),
        Cell::Text(value) => json!(value),
        Cell::Empty => Value::Null,
    }
}

fn get_csv_cell(cell: &Cell) -> String {
    match cell {
        Cell::Number(value) => value.to_string(),
        Cell::Count(value) => value.to_string(),
        Cell::Text(value) if value.contains([',', '"', '\n']) => {
            format!("\"{}\"", value.replace('"', "\"\""))
        }
        Cell::Text(value) => value.clone(),
        Cell::Empty => String::new(),
    }
}
