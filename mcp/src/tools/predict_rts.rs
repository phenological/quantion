use std::path::PathBuf;

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
    tools::{Tool, find_targets::FILES, table_output::read_table_request},
};

pub struct PredictRts;

pub const MODEL: Field = Field::required("model", Kind::Text, "deepwarp model (.onnx)");

const FIELDS: &[Field] = &[
    MODEL,
    FILES,
    Field::required(
        "targets",
        Kind::ObjectList(TARGET_FIELDS),
        "Targets with reference rt, for example from calibrate",
    ),
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for PredictRts {
    fn name(&self) -> &'static str {
        "predict_rts"
    }

    fn description(&self) -> &'static str {
        "Predict where each target elutes in each sample with a deepwarp model, to correct rt drift. One row per file and target."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["predict_rts", "load_model", "load_warp"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let model = get_model_path(&input.get_text("model")?, context)?;
        let files = input.get_text_list("files")?;
        let targets = input.get_object_list("targets", read_target)?;
        let request = read_table_request(input, context)?;
        let mut table = Table::new(vec![
            Column {
                name: "file",
                unit: Unit::Text,
            },
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
            Column {
                name: "predicted_rt",
                unit: Unit::Rt,
            },
            Column {
                name: "uncertainty",
                unit: Unit::Rt,
            },
        ]);
        let warp = context.models.get_warp(&model)?;
        for file in &files {
            let full_path = context.folders.get_input_path(file)?;
            let sample = context.files.get_file(&full_path)?;
            let predictions = warp.predict_rts(sample, &targets)?;
            for (target, prediction) in targets.iter().zip(predictions) {
                table.add_row(vec![
                    Cell::Text(file.clone()),
                    target.id.clone().map_or(Cell::Empty, Cell::Text),
                    Cell::Number(target.mz),
                    Cell::Number(target.rt),
                    Cell::Number(prediction.rt),
                    Cell::Number(prediction.uncertainty),
                ]);
            }
        }
        request.get_output(&table)
    }
}

pub fn get_model_path(text: &str, context: &Context) -> Result<PathBuf, ToolError> {
    context.folders.get_input_path(text).map_err(|error| {
        error.with_fix("give a local .onnx deepwarp model inside an allowed folder; see https://github.com/josoriom/deepwarp")
    })
}
