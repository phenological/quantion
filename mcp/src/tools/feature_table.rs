use std::{fs, path::Path};

use quantion::{Feature, FeatureAnnotation};

use crate::{
    error::ToolError,
    output::{
        round::Unit,
        table::{Cell, Column, Table},
    },
};

const FEATURE_COLUMNS: [&str; 8] = [
    "mz",
    "rt",
    "from",
    "to",
    "intensity",
    "integral",
    "n_points",
    "frequency",
];

pub fn get_feature_table(features: &[Feature]) -> Table {
    let units = [
        Unit::Mz,
        Unit::Rt,
        Unit::Rt,
        Unit::Rt,
        Unit::Intensity,
        Unit::Intensity,
        Unit::Count,
        Unit::Ratio,
    ];
    let columns = FEATURE_COLUMNS
        .iter()
        .zip(units)
        .map(|(name, unit)| Column { name, unit })
        .collect();
    let mut table = Table::new(columns);
    for feature in features {
        table.add_row(vec![
            Cell::Number(feature.mz),
            Cell::Number(feature.rt),
            Cell::Number(feature.from),
            Cell::Number(feature.to),
            Cell::Number(feature.intensity),
            Cell::Number(feature.integral),
            Cell::Count(feature.n_points as u64),
            feature.frequency.map_or(Cell::Empty, Cell::Number),
        ]);
    }
    table
}

pub fn get_annotation_table(annotations: &[FeatureAnnotation]) -> Table {
    let mut table = Table::new(vec![
        Column {
            name: "mz",
            unit: Unit::Mz,
        },
        Column {
            name: "rt",
            unit: Unit::Rt,
        },
        Column {
            name: "label",
            unit: Unit::Text,
        },
        Column {
            name: "id",
            unit: Unit::Text,
        },
        Column {
            name: "database",
            unit: Unit::Text,
        },
        Column {
            name: "frequency",
            unit: Unit::Ratio,
        },
        Column {
            name: "score",
            unit: Unit::Ratio,
        },
        Column {
            name: "cosine",
            unit: Unit::Ratio,
        },
    ]);
    for annotation in annotations {
        table.add_row(vec![
            Cell::Number(annotation.mz),
            Cell::Number(annotation.rt),
            get_text_cell(&annotation.label),
            get_text_cell(&annotation.id),
            get_text_cell(&annotation.database),
            Cell::Number(annotation.frequency),
            annotation.score.map_or(Cell::Empty, Cell::Number),
            annotation.cosine.map_or(Cell::Empty, Cell::Number),
        ]);
    }
    table
}

fn get_text_cell(text: &Option<String>) -> Cell {
    text.clone().map_or(Cell::Empty, Cell::Text)
}

pub fn read_feature_file(full_path: &Path, path: &str) -> Result<Vec<Feature>, ToolError> {
    let text = fs::read_to_string(full_path).map_err(|_| ToolError::file_not_found(path))?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines
        .next()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .collect();
    let find_column = |name: &str| header.iter().position(|column| *column == name);
    let (Some(mz_column), Some(rt_column)) = (find_column("mz"), find_column("rt")) else {
        return Err(ToolError::invalid(path, "must have the columns mz and rt")
            .with_fix("use a CSV saved by find_features or match_features"));
    };
    let mut features = Vec::new();
    for (index, line) in lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
    {
        let cells: Vec<&str> = line.split(',').collect();
        let read = |column: Option<usize>| read_cell(&cells, column, path, index);
        let mut feature = Feature::new(
            read(Some(mz_column))?.unwrap_or_default(),
            read(Some(rt_column))?.unwrap_or_default(),
        );
        feature.from = read(find_column("from"))?.unwrap_or_default();
        feature.to = read(find_column("to"))?.unwrap_or_default();
        feature.intensity = read(find_column("intensity"))?.unwrap_or_default();
        feature.integral = read(find_column("integral"))?.unwrap_or_default();
        feature.n_points = read(find_column("n_points"))?.unwrap_or_default() as usize;
        feature.frequency = read(find_column("frequency"))?;
        features.push(feature);
    }
    if features.is_empty() {
        return Err(ToolError::invalid(path, "has no features"));
    }
    Ok(features)
}

fn read_cell(
    cells: &[&str],
    column: Option<usize>,
    path: &str,
    index: usize,
) -> Result<Option<f64>, ToolError> {
    let Some(text) = column
        .and_then(|column| cells.get(column))
        .map(|text| text.trim())
    else {
        return Ok(None);
    };
    if text.is_empty() {
        return Ok(None);
    }
    match text.parse::<f64>() {
        Ok(value) if value.is_finite() => Ok(Some(value)),
        _ => Err(ToolError::invalid(
            path,
            format!("has a bad number on line {}", index + 2),
        )),
    }
}
