use core::ffi::c_int;

use super::{
    Buf, ParsedFile, clear_buf,
    model::ModelHandle,
    public::{FIELD_SEPARATOR, get_open_file, read_targets_at, read_values, remember, run},
    write_buf,
};
use crate::{
    AnnotateFeaturesOptions, Feature, QuantionError, SampleFile,
    annotations::{annotate_with_library, check_features, check_files, from_spectrum_model_error},
    files::from_fast_error,
    sample_file::FileSource,
    utilities::{
        annotate_features::{FeatureMatch, plan_library_ranges, plan_sample_ranges},
        bridge::{
            Column, QUANTION_PAYLOAD_FEATURE_MATCHES, QUANTION_SECTION_FEATURE_MATCH_COSINE,
            QUANTION_SECTION_FEATURE_MATCH_FILE, QUANTION_SECTION_FEATURE_MATCH_FREQUENCY,
            QUANTION_SECTION_FEATURE_MATCH_METADATA_NAME_BYTES,
            QUANTION_SECTION_FEATURE_MATCH_METADATA_NAME_STARTS,
            QUANTION_SECTION_FEATURE_MATCH_METADATA_VALUE_BYTES,
            QUANTION_SECTION_FEATURE_MATCH_METADATA_VALUE_STARTS,
            QUANTION_SECTION_FEATURE_MATCH_MODEL_SCORE,
            QUANTION_SECTION_FEATURE_MATCH_MS2_FREQUENCY, QUANTION_SECTION_FEATURE_MATCH_MZ,
            QUANTION_SECTION_FEATURE_MATCH_RANK, QUANTION_SECTION_FEATURE_MATCH_RT,
            QUANTION_SECTION_FEATURE_MATCH_SCAN_INDEX, QUANTION_SECTION_FEATURE_MATCH_SCORE,
            build_record_bridge,
        },
        spectrum_model::SpectrumModel,
    },
};

const FUNCTION: &str = "annotate_features";

fn read_options(
    matches: usize,
    ppm_tolerance: f64,
    mz_tolerance: f64,
    rt_range: f64,
    min_relative_intensity: f64,
) -> Result<AnnotateFeaturesOptions, QuantionError> {
    let options = AnnotateFeaturesOptions {
        library: String::new(),
        model: String::new(),
        matches,
        ppm_tolerance,
        mz_tolerance,
        rt_range,
        min_relative_intensity,
    };
    options.check_settings(FUNCTION)?;
    Ok(options)
}

fn read_features(
    mzs: *const f64,
    rts: *const f64,
    count: usize,
) -> Result<Vec<Feature>, QuantionError> {
    if count > 0 && (mzs.is_null() || rts.is_null()) {
        return Err(QuantionError::internal(FUNCTION, "the features are null"));
    }
    let features: Vec<Feature> = read_targets_at(mzs, rts, count)
        .iter()
        .map(|target| Feature::new(target.mz, target.rt))
        .collect();
    check_features(FUNCTION, &features)?;
    Ok(features)
}

fn read_files<'a>(
    library: *mut ParsedFile,
    files: *const *mut ParsedFile,
    count: usize,
) -> Result<Vec<&'a mut SampleFile>, QuantionError> {
    check_files(FUNCTION, count)?;
    if files.is_null() {
        return Err(QuantionError::internal(FUNCTION, "the files are null"));
    }
    let handles = read_values(files, count);
    for (position, handle) in handles.iter().enumerate() {
        if *handle == library {
            return Err(QuantionError::invalid(
                FUNCTION,
                format!("files[{position}]"),
                "must not be the library",
            ));
        }
        if handles[..position].contains(handle) {
            return Err(QuantionError::invalid(
                FUNCTION,
                format!("files[{position}]"),
                "is given twice",
            ));
        }
    }
    handles
        .iter()
        .enumerate()
        .map(|(position, handle)| get_open_file(FUNCTION, *handle, &format!("files[{position}]")))
        .collect()
}

fn read_model(model: *const ModelHandle) -> Result<SpectrumModel, QuantionError> {
    if model.is_null() {
        return Err(QuantionError::invalid(
            FUNCTION,
            "model",
            "is not open; it was disposed or never opened",
        ));
    }
    SpectrumModel::read(unsafe { &(*model).model })
        .map_err(|error| from_spectrum_model_error(FUNCTION, error))
}

/// Match each feature against the library. `files` holds `file_count` open samples.
///
/// # Safety
/// `library` and every handle in `files` must be valid `ParsedFile`s; `model` a valid model.
/// `mzs` and `rts` must point to `feature_count` values. `out` must be a valid writable `Buf`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn annotate_features(
    library: *mut ParsedFile,
    model: *const ModelHandle,
    files: *const *mut ParsedFile,
    file_count: usize,
    mzs: *const f64,
    rts: *const f64,
    feature_count: usize,
    matches: usize,
    ppm_tolerance: f64,
    mz_tolerance: f64,
    rt_range: f64,
    min_relative_intensity: f64,
    out: *mut Buf,
) -> c_int {
    if out.is_null() {
        return remember(QuantionError::internal(
            FUNCTION,
            "the output buffer is null",
        ));
    }
    clear_buf(out);
    run(FUNCTION, || {
        let options = read_options(
            matches,
            ppm_tolerance,
            mz_tolerance,
            rt_range,
            min_relative_intensity,
        )?;
        let mut samples = read_files(library, files, file_count)?;
        let features = read_features(mzs, rts, feature_count)?;
        let found = if features.is_empty() {
            Vec::new()
        } else {
            let model = read_model(model)?;
            let library = get_open_file(FUNCTION, library, "library")?;
            annotate_with_library(FUNCTION, library, &model, &mut samples, &features, &options)?
        };
        let bytes = build_feature_matches_bridge(&found)
            .ok_or_else(|| QuantionError::internal(FUNCTION, "the matches could not be packed"))?;
        write_buf(out, bytes);
        Ok(())
    })
}

/// Plan the byte ranges `annotate_features` reads from one sample.
///
/// # Safety
/// `file` must be a valid `ParsedFile`. `mzs` and `rts` must point to `feature_count` values.
/// `out` must be a valid writable `Buf`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plan_annotate_samples(
    file: *mut ParsedFile,
    mzs: *const f64,
    rts: *const f64,
    feature_count: usize,
    matches: usize,
    ppm_tolerance: f64,
    mz_tolerance: f64,
    rt_range: f64,
    min_relative_intensity: f64,
    out: *mut Buf,
) -> c_int {
    if out.is_null() {
        return remember(QuantionError::internal(
            FUNCTION,
            "the output buffer is null",
        ));
    }
    clear_buf(out);
    run(FUNCTION, || {
        let options = read_options(
            matches,
            ppm_tolerance,
            mz_tolerance,
            rt_range,
            min_relative_intensity,
        )?;
        let features = read_features(mzs, rts, feature_count)?;
        let file = get_open_file(FUNCTION, file, "files")?;
        if let FileSource::Lazy(ion) | FileSource::Remote(ion) = &mut file.source {
            let ranges = plan_sample_ranges(ion, &features, &options)
                .map_err(|error| from_fast_error(FUNCTION, error))?;
            super::public::write_ranges(out, &ranges);
        }
        Ok(())
    })
}

/// Plan the byte ranges `annotate_features` reads from the library, for every sample at once.
///
/// # Safety
/// `library` and every handle in `files` must be valid `ParsedFile`s; `model` a valid model.
/// `mzs` and `rts` must point to `feature_count` values. `out` must be a valid writable `Buf`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plan_annotate_library(
    library: *mut ParsedFile,
    model: *const ModelHandle,
    files: *const *mut ParsedFile,
    file_count: usize,
    mzs: *const f64,
    rts: *const f64,
    feature_count: usize,
    matches: usize,
    ppm_tolerance: f64,
    mz_tolerance: f64,
    rt_range: f64,
    min_relative_intensity: f64,
    out: *mut Buf,
) -> c_int {
    if out.is_null() {
        return remember(QuantionError::internal(
            FUNCTION,
            "the output buffer is null",
        ));
    }
    clear_buf(out);
    run(FUNCTION, || {
        let options = read_options(
            matches,
            ppm_tolerance,
            mz_tolerance,
            rt_range,
            min_relative_intensity,
        )?;
        let mut samples = read_files(library, files, file_count)?;
        let features = read_features(mzs, rts, feature_count)?;
        let model = read_model(model)?;
        let library = get_open_file(FUNCTION, library, "library")?;
        if let FileSource::Lazy(ion) | FileSource::Remote(ion) = &mut library.source {
            let ranges = plan_library_ranges(
                ion,
                &mut samples,
                &features,
                &options,
                model.vector_accession,
            )
            .map_err(|error| from_fast_error(FUNCTION, error))?;
            super::public::write_ranges(out, &ranges);
        }
        Ok(())
    })
}

fn get_number(value: Option<f64>) -> f64 {
    value.unwrap_or(f64::NAN)
}

fn get_count(value: Option<usize>) -> f64 {
    value.map_or(f64::NAN, |count| count as f64)
}

fn build_feature_matches_bridge(matches: &[FeatureMatch]) -> Option<Box<[u8]>> {
    let numbers =
        |read: fn(&FeatureMatch) -> f64| -> Vec<f64> { matches.iter().map(read).collect() };
    let mz = numbers(|found| found.mz);
    let rt = numbers(|found| found.rt);
    let rank = numbers(|found| get_count(found.rank));
    let frequency = numbers(|found| found.frequency);
    let ms2_frequency = numbers(|found| found.ms2_frequency);
    let score = numbers(|found| get_number(found.score));
    let model_score = numbers(|found| get_number(found.model_score));
    let cosine = numbers(|found| get_number(found.cosine));
    let file = numbers(|found| get_count(found.file));
    let scan_index = numbers(|found| get_count(found.scan_index));
    let names: Vec<String> = matches
        .iter()
        .map(|found| {
            found
                .metadata
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(&FIELD_SEPARATOR.to_string())
        })
        .collect();
    let values: Vec<String> = matches
        .iter()
        .map(|found| {
            found
                .metadata
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .join(&FIELD_SEPARATOR.to_string())
        })
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let values: Vec<&str> = values.iter().map(String::as_str).collect();
    build_record_bridge(
        QUANTION_PAYLOAD_FEATURE_MATCHES,
        matches.len() as u64,
        &[
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_MZ,
                values: &mz,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_RT,
                values: &rt,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_RANK,
                values: &rank,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_FREQUENCY,
                values: &frequency,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_MS2_FREQUENCY,
                values: &ms2_frequency,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_SCORE,
                values: &score,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_MODEL_SCORE,
                values: &model_score,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_COSINE,
                values: &cosine,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_FILE,
                values: &file,
            },
            Column::Numbers {
                id: QUANTION_SECTION_FEATURE_MATCH_SCAN_INDEX,
                values: &scan_index,
            },
            Column::Text {
                starts_id: QUANTION_SECTION_FEATURE_MATCH_METADATA_NAME_STARTS,
                bytes_id: QUANTION_SECTION_FEATURE_MATCH_METADATA_NAME_BYTES,
                values: &names,
            },
            Column::Text {
                starts_id: QUANTION_SECTION_FEATURE_MATCH_METADATA_VALUE_STARTS,
                bytes_id: QUANTION_SECTION_FEATURE_MATCH_METADATA_VALUE_BYTES,
                values: &values,
            },
        ],
    )
}
