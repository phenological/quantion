use crate::{
    error::{ErrorKind, QuantionError},
    files::{from_fast_error, read_error},
    sample_file::SampleFile,
    utilities::{
        load_model::{Model, ModelError, ModelInput, load_model as read_model},
        structs::Target,
        warp_model::{PredictedRt, WarpError, WarpModel, load_warp as read_warp},
    },
};

pub fn load_model(input: impl Into<ModelInput>) -> Result<Model, QuantionError> {
    const FUNCTION: &str = "load_model";
    read_model(input).map_err(|ModelError::NotRead(reason)| read_error(FUNCTION, reason))
}

pub fn load_warp(model: &Model) -> Result<WarpModel, QuantionError> {
    const FUNCTION: &str = "load_warp";
    read_warp(model).map_err(|error| from_warp_error(FUNCTION, error))
}

impl WarpModel {
    pub fn predict_rts(&self, file: &mut SampleFile, targets: &[Target]) -> Result<Vec<PredictedRt>, QuantionError> {
        const FUNCTION: &str = "predict_rts";
        self.predict_rts_from_reader(&mut file.reader(), targets)
            .map_err(|error| from_warp_error(FUNCTION, error))
    }

    pub fn calibrate(&self, anchor_samples: &mut [SampleFile], target_lists: &[Vec<Target>]) -> Result<Vec<Target>, QuantionError> {
        const FUNCTION: &str = "calibrate";
        let mut readers: Vec<_> = anchor_samples.iter_mut().map(SampleFile::reader).collect();
        self.calibrate_from_readers(&mut readers, target_lists)
            .map_err(|error| from_warp_error(FUNCTION, error))
    }
}

pub(crate) fn from_warp_error(function: &'static str, error: WarpError) -> QuantionError {
    let message = error.to_string();
    match error {
        WarpError::TargetMissing(index) => QuantionError {
            kind: ErrorKind::InvalidArgument,
            function,
            argument: Some(format!("targets[{index}]")),
            message,
        },
        WarpError::IdMissing { list, target } => QuantionError {
            kind: ErrorKind::MissingArgument,
            function,
            argument: Some(format!("target_lists[{list}][{target}].id")),
            message,
        },
        WarpError::ListsDiffer { .. } => QuantionError {
            kind: ErrorKind::InvalidArgument,
            function,
            argument: Some("target_lists".to_string()),
            message,
        },
        WarpError::FileNotRead(error) => from_fast_error(function, error),
        _ => QuantionError::new(ErrorKind::ModelMismatch, function, message),
    }
}
