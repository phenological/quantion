use std::path::PathBuf;

use quantion::Target;
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
        options::targets::{TARGET_FIELDS, read_target},
    },
    state::Context,
    tools::{
        Tool,
        predict_rts::{MODEL, get_model_path},
        table_output::read_table_request,
    },
};

pub struct Calibrate;

const ANCHOR_FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, "Anchor sample, such as an LTR or QC"),
    Field::required(
        "targets",
        Kind::ObjectList(TARGET_FIELDS),
        "Targets seen in this sample, with their observed rt",
    ),
];

const FIELDS: &[Field] = &[
    MODEL,
    Field::required("anchors", Kind::ObjectList(ANCHOR_FIELDS), ""),
    LIMIT,
    SAVE,
    OVERWRITE,
];

struct Anchor {
    file: String,
    targets: Vec<Target>,
}

impl Tool for Calibrate {
    fn name(&self) -> &'static str {
        "calibrate"
    }

    fn description(&self) -> &'static str {
        "Turn the rt seen in anchor samples (LTRs or QCs) into reference targets for predict_rts. Each id gets the median rt."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["calibrate"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let model = get_model_path(&input.get_text("model")?, context)?;
        let anchors = input.get_object_list("anchors", read_anchor)?;
        let request = read_table_request(input, context)?;
        let paths = anchors
            .iter()
            .map(|anchor| context.folders.get_input_path(&anchor.file))
            .collect::<Result<Vec<PathBuf>, ToolError>>()?;
        let target_lists: Vec<Vec<Target>> =
            anchors.into_iter().map(|anchor| anchor.targets).collect();
        let warp = context.models.get_warp(&model)?;
        let targets = context.files.with_files(&paths, |samples| {
            Ok(warp.calibrate(samples, &target_lists)?)
        })?;
        let mut table = Table::new(vec![
            Column {
                name: "id",
                unit: Unit::Text,
            },
            Column {
                name: "mz",
                unit: Unit::Mz,
            },
            Column {
                name: "rt",
                unit: Unit::Rt,
            },
        ]);
        for target in &targets {
            table.add_row(vec![
                target.id.clone().map_or(Cell::Empty, Cell::Text),
                Cell::Number(target.mz),
                Cell::Number(target.rt),
            ]);
        }
        request.get_output(&table)
    }
}

fn read_anchor(input: &mut Input) -> Result<Anchor, ToolError> {
    Ok(Anchor {
        file: input.get_text("file")?,
        targets: input.get_object_list("targets", read_target)?,
    })
}
