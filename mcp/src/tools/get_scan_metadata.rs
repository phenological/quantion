use quantion::{ScanParam, ScanParamKind, get_scan_metadata};
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    output::{
        round::Unit,
        table::{Cell, Column, Table},
    },
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::scans::{GET_SCAN_METADATA_FIELDS, read_get_scan_metadata_options},
    },
    state::Context,
    tools::{Tool, table_output::read_table_request},
};

pub struct GetScanMetadata;

const FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, ""),
    Field::required("index", Kind::Count, "Scan index from get_scans"),
    Field::optional("options", Kind::Object(GET_SCAN_METADATA_FIELDS), ""),
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for GetScanMetadata {
    fn name(&self) -> &'static str {
        "get_scan_metadata"
    }

    fn description(&self) -> &'static str {
        "Read the attributes, cvParams and userParams of one scan, such as its precursor or collision energy."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["get_scan_metadata"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let file = input.get_text("file")?;
        let index = input.get_count("index", 0_usize)?;
        let options = input.get_object("options", read_get_scan_metadata_options)?;
        let request = read_table_request(input, context)?;
        let full_path = context.folders.get_input_path(&file)?;
        let rows = get_scan_metadata(context.files.get_file(&full_path)?, index, options)?;
        request.get_output(&get_metadata_table(&rows))
    }
}

fn get_metadata_table(rows: &[ScanParam]) -> Table {
    let names = [
        "path",
        "kind",
        "accession",
        "name",
        "value",
        "unit",
        "unit_accession",
    ];
    let mut table = Table::new(
        names
            .iter()
            .map(|name| Column {
                name,
                unit: Unit::Text,
            })
            .collect(),
    );
    for row in rows {
        table.add_row(vec![
            Cell::Text(row.path.clone()),
            Cell::Text(get_kind_name(row.kind).to_string()),
            get_text_cell(&row.accession),
            Cell::Text(row.name.clone()),
            get_text_cell(&row.value),
            get_text_cell(&row.unit),
            get_text_cell(&row.unit_accession),
        ]);
    }
    table
}

fn get_kind_name(kind: ScanParamKind) -> &'static str {
    match kind {
        ScanParamKind::ATTRIBUTE => "ATTRIBUTE",
        ScanParamKind::CV => "CV",
        ScanParamKind::USER => "USER",
    }
}

fn get_text_cell(text: &Option<String>) -> Cell {
    text.clone().map_or(Cell::Empty, Cell::Text)
}
