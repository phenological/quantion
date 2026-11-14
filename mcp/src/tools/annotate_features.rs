use std::path::PathBuf;

use quantion::{Feature, annotate_features};
use serde_json::Value;

use crate::{
    error::ToolError,
    input::Input,
    schema::{
        Field, Kind, LIMIT, OVERWRITE, SAVE,
        options::annotate_features::{ANNOTATE_FEATURES_FIELDS, read_annotate_features_options},
    },
    state::Context,
    tools::{
        Tool,
        feature_table::{get_annotation_table, read_feature_file},
        table_output::read_table_request,
    },
};

pub struct AnnotateFeatures;

const FEATURE_FIELDS: &[Field] = &[
    Field::required("mz", Kind::Number, ""),
    Field::required("rt", Kind::Number, ""),
];

const FIELDS: &[Field] = &[
    Field::required("files", Kind::TextList, "Samples with MS2 scans"),
    Field::optional(
        "features",
        Kind::Text,
        "CSV saved by find_features or match_features",
    ),
    Field::optional(
        "feature_list",
        Kind::ObjectList(FEATURE_FIELDS),
        "Features given inline, instead of features",
    ),
    Field::required("library", Kind::Text, "Spectral library (.ion)"),
    Field::required("model", Kind::Text, "Spectrum model (.onnx)"),
    Field::optional("options", Kind::Object(ANNOTATE_FEATURES_FIELDS), ""),
    LIMIT,
    SAVE,
    OVERWRITE,
];

const LIBRARY_FIX: &str = "download library_positive.ion and model.onnx from https://github.com/josoriom/cipher/releases into an allowed folder";

impl Tool for AnnotateFeatures {
    fn name(&self) -> &'static str {
        "annotate_features"
    }

    fn description(&self) -> &'static str {
        "Name features by matching their MS2 spectra against a spectral library (experimental). One row per feature."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["annotate_features"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let files = input.get_text_list("files")?;
        let feature_file = input.get_optional_text("features")?;
        let feature_list = input.get_optional_object_list("feature_list", read_feature)?;
        let library = get_model_path(&input.get_text("library")?, context)?;
        let model = get_model_path(&input.get_text("model")?, context)?;
        let options = input.get_object("options", |options| {
            read_annotate_features_options(options, &library, &model)
        })?;
        let request = read_table_request(input, context)?;
        let features = match (feature_file, feature_list) {
            (Some(path), None) => {
                read_feature_file(&context.folders.get_input_path(&path)?, &path)?
            }
            (None, Some(features)) => features,
            _ => {
                return Err(
                    ToolError::invalid("features", "or feature_list is needed, not both")
                        .with_fix("give features as a CSV path, or feature_list as [{mz, rt}]"),
                );
            }
        };
        let paths = files
            .iter()
            .map(|file| context.folders.get_input_path(file))
            .collect::<Result<Vec<PathBuf>, ToolError>>()?;
        let annotations = context.files.with_files(&paths, |samples| {
            Ok(annotate_features(samples, &features, options)?)
        })?;
        request.get_output(&get_annotation_table(&annotations))
    }
}

fn read_feature(input: &mut Input) -> Result<Feature, ToolError> {
    Ok(Feature::new(
        input.get_required_number("mz")?,
        input.get_required_number("rt")?,
    ))
}

fn get_model_path(text: &str, context: &Context) -> Result<PathBuf, ToolError> {
    context
        .folders
        .get_input_path(text)
        .map_err(|error| error.with_fix(LIBRARY_FIX))
}
