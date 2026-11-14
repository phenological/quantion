use quantion::{deisotope_features, find_features};
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::features::{
            DEISOTOPE_FIELDS, FIND_FEATURES_FIELDS, read_deisotope_options,
            read_find_features_options,
        },
    },
    state::Context,
    tools::{Tool, feature_table::get_feature_table, table_output::read_table_request},
};

pub struct FindFeatures;

pub const DEISOTOPE: Field = Field::optional(
    "deisotope",
    Kind::Object(DEISOTOPE_FIELDS),
    "Give it, even as {}, to remove 13C isotopes",
);

const FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, ""),
    Field::optional("options", Kind::Object(FIND_FEATURES_FIELDS), ""),
    DEISOTOPE,
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for FindFeatures {
    fn name(&self) -> &'static str {
        "find_features"
    }

    fn description(&self) -> &'static str {
        "Find every feature in one sample without a target list (untargeted). Save the table to annotate it later."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["find_features", "deisotope_features"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let file = input.get_text("file")?;
        let options = input.get_object("options", read_find_features_options)?;
        let deisotope = input.get_optional_object("deisotope", read_deisotope_options)?;
        let request = read_table_request(input, context)?;
        let full_path = context.folders.get_input_path(&file)?;
        let sample = context.files.get_file(&full_path)?;
        let mut features = find_features(sample, options)?;
        if let Some(deisotope) = deisotope {
            features = deisotope_features(features, deisotope)?;
        }
        request.get_output(&get_feature_table(&features))
    }
}
