use ionic::{
    IonReader, ReadOptions, WriteOptions,
    mzml::structs::{
        BinaryDataArray, BinaryDataArrayList, CvParam, MzML, NumericArray, NumericType, Precursor,
        PrecursorList, Run, Scan, ScanList, SelectedIon, SelectedIonList, Spectrum, SpectrumList,
        UserParam,
    },
};

use crate::{
    AnnotateFeaturesOptions, ErrorKind, FeatureAnnotation, ParseIonOptions, Polarity,
    QuantionError, SampleFile, annotate_features,
    annotations::annotate_with_library,
    parse_ion,
    sample_file::FileSource,
    utilities::{find_features::Feature, ion::write_mzml_to_ion, spectrum_model::SpectrumModel},
};

const FOLDER: &str = "tests/fixtures/annotate";
const SAMPLES: [&str; 3] = ["sample_peak_and_ms2", "sample_peak_only", "sample_no_peak"];

fn get_files() -> Vec<SampleFile> {
    SAMPLES
        .iter()
        .map(|name| {
            parse_ion(
                format!("{FOLDER}/{name}.ion").as_str(),
                ParseIonOptions::default(),
            )
            .expect("sample")
        })
        .collect()
}

fn get_options(model: &str) -> AnnotateFeaturesOptions {
    AnnotateFeaturesOptions {
        library: format!("{FOLDER}/library.ion"),
        model: format!("{FOLDER}/{model}.onnx"),
        ..Default::default()
    }
}

fn annotate(
    features: &[Feature],
    options: AnnotateFeaturesOptions,
) -> Result<Vec<FeatureAnnotation>, QuantionError> {
    annotate_features(&mut get_files(), features, options)
}

fn get_features() -> Vec<Feature> {
    vec![Feature::new(195.0877, 2.0)]
}

#[test]
fn the_best_compound_names_the_feature() {
    let annotations = annotate(&get_features(), get_options("model")).expect("annotations");
    let first = &annotations[0];
    assert_eq!(first.label.as_deref(), Some("Caffeine"));
    assert_eq!(first.id.as_deref(), Some("ENTRY_1"));
    assert_eq!(first.database.as_deref(), Some("SOURCE_A"));
    assert!((first.frequency - 1.0 / 3.0).abs() < 1e-12);
    assert!(first.score.expect("score") > 0.99);
    assert!(first.cosine.expect("cosine") > 0.99);
}

#[test]
fn each_file_keeps_its_best_matches_in_order() {
    let annotations = annotate(&get_features(), get_options("model")).expect("annotations");
    let files = &annotations[0].files;
    assert_eq!(
        files.iter().map(|file| file.file).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert!(files[0].scan_index.is_some());
    assert_eq!(files[1].scan_index, None);
    assert_eq!(files[2].scan_index, None);
    let labels: Vec<&str> = files[0]
        .matches
        .iter()
        .map(|found| found.label.as_str())
        .collect();
    assert_eq!(labels, vec!["Caffeine", "Compound Y"]);
    assert!(files[0].matches[0].score >= files[0].matches[1].score);
    let best = &files[0].matches[0];
    assert_eq!(best.mz, vec![110.0713, 123.0427, 138.0662]);
    assert_eq!(
        best.metadata.get("family").map(String::as_str),
        Some("FAMILY_C")
    );
    assert_eq!(
        best.metadata.get("note").map(String::as_str),
        Some("synthetic 1")
    );
    assert!(!best.metadata.contains_key("entry"));
    assert!(!best.metadata.contains_key("source"));
    assert!(!best.metadata.contains_key("title"));
}

#[test]
fn matches_limits_the_list_of_each_file() {
    let options = AnnotateFeaturesOptions {
        matches: 1,
        ..get_options("model")
    };
    let annotations = annotate(&get_features(), options).expect("annotations");
    assert_eq!(annotations[0].files[0].matches.len(), 1);
    assert_eq!(annotations[0].files[0].matches[0].label, "Caffeine");
}

#[test]
fn a_feature_without_candidates_has_no_label() {
    let features = [Feature::new(350.05, 2.0), Feature::new(500.0, 2.0)];
    let annotations = annotate(&features, get_options("model")).expect("annotations");
    for annotation in &annotations {
        assert_eq!(annotation.label, None);
        assert_eq!(annotation.id, None);
        assert_eq!(annotation.score, None);
        assert_eq!(annotation.frequency, 0.0);
        assert!(annotation.files.iter().all(|file| file.matches.is_empty()));
    }
    assert!(annotations[0].files[0].scan_index.is_some());
    assert_eq!(annotations[1].files[0].scan_index, None);
}

#[test]
fn no_features_give_no_annotations() {
    assert!(
        annotate(&[], get_options("model"))
            .expect("annotations")
            .is_empty()
    );
}

#[test]
fn no_files_are_refused() {
    let error =
        annotate_features(&mut [], &get_features(), get_options("model")).expect_err("no files");
    assert_eq!(error.argument.as_deref(), Some("files"));
}

#[test]
fn invalid_options_are_refused() {
    let cases = [
        (
            AnnotateFeaturesOptions {
                matches: 0,
                ..get_options("model")
            },
            "matches",
        ),
        (
            AnnotateFeaturesOptions {
                rt_range: 0.0,
                ..get_options("model")
            },
            "rt_range",
        ),
        (
            AnnotateFeaturesOptions {
                min_relative_intensity: 2.0,
                ..get_options("model")
            },
            "min_relative_intensity",
        ),
        (
            AnnotateFeaturesOptions {
                library: String::new(),
                ..get_options("model")
            },
            "library",
        ),
    ];
    for (options, argument) in cases {
        let error = annotate(&get_features(), options).expect_err(argument);
        assert_eq!(error.kind, ErrorKind::InvalidArgument);
        assert_eq!(error.argument.as_deref(), Some(argument));
    }
}

#[test]
fn a_model_without_a_key_is_refused() {
    let error =
        annotate(&get_features(), get_options("model_without_label")).expect_err("missing key");
    assert_eq!(error.argument.as_deref(), Some("model"));
    assert!(error.message.contains("label"), "{}", error.message);
}

#[test]
fn a_model_with_an_invalid_key_is_refused() {
    let error = annotate(&get_features(), get_options("model_with_bad_max_peaks"))
        .expect_err("invalid key");
    assert_eq!(error.argument.as_deref(), Some("model"));
    assert!(error.message.contains("max_peaks"), "{}", error.message);
}

const POSITIVE_SCAN: &str = "MS:1000130";
const NEGATIVE_SCAN: &str = "MS:1000129";
const MS_LEVEL: &str = "MS:1000511";
const CENTROID: &str = "MS:1000127";
const SCAN_START_TIME: &str = "MS:1000016";
const MINUTE: &str = "UO:0000031";
const SELECTED_ION_MZ: &str = "MS:1000744";
const MZ_ARRAY: &str = "MS:1000514";
const INTENSITY_ARRAY: &str = "MS:1000515";
const VECTOR_ARRAY: &str = "MS:1000787";
const FLOAT_64: &str = "MS:1000523";
const NO_COMPRESSION: &str = "MS:1000576";

const FEATURE_MZ: f64 = 195.0877;
const FEATURE_RT: f64 = 2.0;
const PRECURSOR_MZ: f64 = 195.0876;
const MS1_SCANS: usize = 100;
const VECTOR_WIDTH: usize = 64;
const FRAGMENT_MZ: [f64; 3] = [110.0713, 123.0427, 138.0662];
const FRAGMENT_INTENSITY: [f64; 3] = [1000.0, 500.0, 250.0];

fn cv_param(accession: &str, value: Option<&str>) -> CvParam {
    CvParam {
        accession: Some(accession.to_string()),
        name: accession.to_string(),
        value: value.map(str::to_string),
        ..Default::default()
    }
}

fn rt_param(rt: f64) -> CvParam {
    CvParam {
        accession: Some(SCAN_START_TIME.to_string()),
        name: SCAN_START_TIME.to_string(),
        value: Some(rt.to_string()),
        unit_accession: Some(MINUTE.to_string()),
        ..Default::default()
    }
}

fn user_param(name: &str, value: &str) -> UserParam {
    UserParam {
        name: name.to_string(),
        value: Some(value.to_string()),
        ..Default::default()
    }
}

fn binary_array(accession: &str, values: Vec<f64>) -> BinaryDataArray {
    BinaryDataArray {
        array_length: Some(values.len()),
        cv_params: vec![
            cv_param(accession, None),
            cv_param(FLOAT_64, None),
            cv_param(NO_COMPRESSION, None),
        ],
        numeric_type: Some(NumericType::Float64),
        binary: Some(NumericArray::F64(values)),
        ..Default::default()
    }
}

fn scan_list(rt: f64) -> ScanList {
    ScanList {
        count: Some(1),
        scans: vec![Scan {
            cv_params: vec![rt_param(rt)],
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn precursor_list(mz: f64) -> PrecursorList {
    PrecursorList {
        count: Some(1),
        precursors: vec![Precursor {
            selected_ion_list: Some(SelectedIonList {
                count: Some(1),
                selected_ions: vec![SelectedIon {
                    cv_params: vec![cv_param(SELECTED_ION_MZ, Some(&mz.to_string()))],
                    ..Default::default()
                }],
            }),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn ms1_spectrum(index: usize, rt: f64, polarity: &str) -> Spectrum {
    let distance = (rt - FEATURE_RT) / 0.05;
    let intensity = 100000.0 * (-0.5 * distance * distance).exp();
    Spectrum {
        id: format!("scan={}", index + 1),
        index: Some(index as u32),
        default_array_length: Some(1),
        ms_level: Some(1),
        cv_params: vec![
            cv_param(MS_LEVEL, Some("1")),
            cv_param(polarity, None),
            cv_param(CENTROID, None),
        ],
        scan_list: Some(scan_list(rt)),
        binary_data_array_list: Some(BinaryDataArrayList {
            count: Some(2),
            binary_data_arrays: vec![
                binary_array(MZ_ARRAY, vec![FEATURE_MZ]),
                binary_array(INTENSITY_ARRAY, vec![intensity]),
            ],
        }),
        ..Default::default()
    }
}

fn ms2_spectrum(index: usize, rt: f64, polarity: &str) -> Spectrum {
    Spectrum {
        id: format!("scan={}", index + 1),
        index: Some(index as u32),
        default_array_length: Some(FRAGMENT_MZ.len()),
        ms_level: Some(2),
        cv_params: vec![
            cv_param(MS_LEVEL, Some("2")),
            cv_param(polarity, None),
            cv_param(CENTROID, None),
        ],
        scan_list: Some(scan_list(rt)),
        precursor_list: Some(precursor_list(FEATURE_MZ)),
        binary_data_array_list: Some(BinaryDataArrayList {
            count: Some(2),
            binary_data_arrays: vec![
                binary_array(MZ_ARRAY, FRAGMENT_MZ.to_vec()),
                binary_array(INTENSITY_ARRAY, FRAGMENT_INTENSITY.to_vec()),
            ],
        }),
        ..Default::default()
    }
}

fn library_spectrum(index: usize, polarity: &str, title: &str) -> Spectrum {
    let vector = vec![1.0 / (VECTOR_WIDTH as f64).sqrt(); VECTOR_WIDTH];
    Spectrum {
        id: format!("scan={}", index + 1),
        index: Some(index as u32),
        default_array_length: Some(FRAGMENT_MZ.len()),
        ms_level: Some(2),
        cv_params: vec![
            cv_param(MS_LEVEL, Some("2")),
            cv_param(polarity, None),
            cv_param(CENTROID, None),
        ],
        user_params: vec![
            user_param("family", "FAMILY_N"),
            user_param("entry", "ENTRY_N"),
            user_param("source", "SOURCE_N"),
            user_param("title", title),
        ],
        scan_list: Some(scan_list(FEATURE_RT)),
        precursor_list: Some(precursor_list(PRECURSOR_MZ)),
        binary_data_array_list: Some(BinaryDataArrayList {
            count: Some(3),
            binary_data_arrays: vec![
                binary_array(MZ_ARRAY, FRAGMENT_MZ.to_vec()),
                binary_array(INTENSITY_ARRAY, FRAGMENT_INTENSITY.to_vec()),
                binary_array(VECTOR_ARRAY, vector),
            ],
        }),
        ..Default::default()
    }
}

fn to_sample(spectra: Vec<Spectrum>) -> SampleFile {
    let mzml = MzML {
        run: Run {
            id: "test".to_string(),
            spectrum_list: Some(SpectrumList {
                count: Some(spectra.len()),
                spectra,
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bytes = Vec::new();
    write_mzml_to_ion(
        &mzml,
        WriteOptions {
            compression_level: 0,
            force_f32: false,
            ..Default::default()
        },
        &mut bytes,
    )
    .expect("ion bytes");
    let ion = IonReader::from_bytes(&bytes, &ReadOptions::default()).expect("ion reader");
    SampleFile::new(FileSource::Lazy(Box::new(ion)))
}

struct ScanPlan {
    rt: f64,
    polarity: &'static str,
    is_ms2: bool,
}

fn build_sample(
    ms1_polarity: &'static str,
    ms2_scans: &[(f64, &'static str)],
) -> (SampleFile, Vec<usize>) {
    let mut plans: Vec<ScanPlan> = (0..MS1_SCANS)
        .map(|index| ScanPlan {
            rt: 1.5 + index as f64 * 0.01,
            polarity: ms1_polarity,
            is_ms2: false,
        })
        .collect();
    for (rt, polarity) in ms2_scans {
        plans.push(ScanPlan {
            rt: *rt,
            polarity,
            is_ms2: true,
        });
    }
    plans.sort_by(|left, right| left.rt.total_cmp(&right.rt));
    let mut spectra = Vec::with_capacity(plans.len());
    let mut ms2_indexes = Vec::new();
    for (index, plan) in plans.iter().enumerate() {
        if plan.is_ms2 {
            ms2_indexes.push(index);
            spectra.push(ms2_spectrum(index, plan.rt, plan.polarity));
        } else {
            spectra.push(ms1_spectrum(index, plan.rt, plan.polarity));
        }
    }
    (to_sample(spectra), ms2_indexes)
}

fn sample_with_polarity(polarity: &'static str) -> SampleFile {
    build_sample(polarity, &[(FEATURE_RT, polarity)]).0
}

fn library_with_polarity(polarity: &'static str) -> SampleFile {
    to_sample(vec![library_spectrum(0, polarity, "Negative Compound")])
}

fn annotate_in_memory(
    sample: SampleFile,
    library: SampleFile,
    polarity: Polarity,
) -> Result<Vec<FeatureAnnotation>, QuantionError> {
    let model = crate::load_model(format!("{FOLDER}/model.onnx").as_str()).expect("model file");
    let model = SpectrumModel::read(&model).expect("model");
    let mut library = library;
    let mut sample = sample;
    let mut files = vec![&mut sample];
    annotate_with_library(
        "annotate_features",
        &mut library,
        &model,
        &mut files,
        &get_features(),
        &AnnotateFeaturesOptions {
            polarity,
            ..Default::default()
        },
    )
}

#[test]
fn a_negative_sample_in_positive_mode_is_refused() {
    let error = annotate_in_memory(
        sample_with_polarity(NEGATIVE_SCAN),
        library_with_polarity(NEGATIVE_SCAN),
        Polarity::POSITIVE,
    )
    .expect_err("wrong polarity");
    assert_eq!(error.kind, ErrorKind::InvalidArgument);
    assert_eq!(error.argument.as_deref(), Some("polarity"));
    assert!(error.message.contains("negative"), "{}", error.message);
}

#[test]
fn a_library_without_the_wanted_polarity_is_refused() {
    let error = annotate_in_memory(
        sample_with_polarity(NEGATIVE_SCAN),
        library_with_polarity(POSITIVE_SCAN),
        Polarity::NEGATIVE,
    )
    .expect_err("wrong library");
    assert_eq!(error.argument.as_deref(), Some("library"));
    assert!(error.message.contains("negative"), "{}", error.message);
}
