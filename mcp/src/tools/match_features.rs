use quantion::{deisotope_features, match_features};
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::features::{
            MATCH_FEATURES_FIELDS, read_deisotope_options, read_match_features_options,
        },
    },
    state::Context,
    tools::{
        Tool, feature_table::get_feature_table, find_features::DEISOTOPE,
        table_output::read_table_request,
    },
};

pub struct MatchFeatures;

const FIELDS: &[Field] = &[
    Field::optional(
        "folder",
        Kind::Text,
        "Folder with one .ion or .mzML file per sample. Defaults to the first allowed folder",
    ),
    Field::optional("options", Kind::Object(MATCH_FEATURES_FIELDS), ""),
    DEISOTOPE,
    LIMIT,
    SAVE,
    OVERWRITE,
];

impl Tool for MatchFeatures {
    fn name(&self) -> &'static str {
        "match_features"
    }

    fn description(&self) -> &'static str {
        "Find the features in every sample of a folder and match them across samples. Values are medians; frequency is the share of samples with the feature."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["match_features", "deisotope_features"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let folder = match input.get_optional_text("folder")? {
            Some(folder) => context.folders.get_folder_path(&folder)?,
            None => context.folders.get_main_folder().to_path_buf(),
        };
        let options = input.get_object("options", read_match_features_options)?;
        let deisotope = input.get_optional_object("deisotope", read_deisotope_options)?;
        let request = read_table_request(input, context)?;
        context.folders.check_samples(&folder)?;
        let mut features = match_features(&folder, options)?;
        if let Some(deisotope) = deisotope {
            features = deisotope_features(features, deisotope)?;
        }
        request.get_output(&get_feature_table(&features))
    }
}
