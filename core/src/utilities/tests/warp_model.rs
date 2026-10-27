use ionic::{IonReader, ReadOptions};

use crate::utilities::{
    calculate_eic::{EicReader, ScanTime},
    structs::FromTo,
    warp_model::{Target, WarpError, load_warp_model, predict_rts, resample_eic},
};

const MODEL_PATH: &str = "tests/fixtures/shift_model.onnx";
const COVERING_FILE: &str = "tests/fixtures/api/api.ion";
const EMPTY_FILE: &str = "tests/fixtures/test.ion";
const POINTS: usize = 64;
const TARGETS: [Target; 4] = [
    Target { mz: 250.1, rt: 1.2 },
    Target { mz: 250.1, rt: 2.4 },
    Target { mz: 310.2, rt: 3.1 },
    Target { mz: 400.4, rt: 4.0 },
];
const EXPECTED_RTS: [f64; 4] = [1.2629844, 2.4629846, 3.1635001, 4.062953];
const EXPECTED_UNCERTAINTIES: [f64; 4] = [0.05199894, 0.05199894, 0.05262259, 0.05249792];
const TOLERANCE: f64 = 1e-5;
const RESAMPLE_RANGE: FromTo = FromTo { from: 1.0, to: 2.0 };

fn resample(scan_rts: &[f64], input: &[f64]) -> Vec<f32> {
    let mut output = vec![0.0_f32; 5];
    resample_eic(scan_rts, input, RESAMPLE_RANGE, &mut output);
    output
}

fn get_traces(rows: usize, row_step: usize, point_step: usize, cycle: usize) -> Vec<f64> {
    (0..rows)
        .flat_map(|row| {
            (0..POINTS)
                .map(move |point| ((row * row_step + point * point_step) % cycle) as f64 * 10.0)
        })
        .collect()
}

fn open_file(path: &str) -> IonReader {
    IonReader::open(path, &ReadOptions::default()).expect("ion file")
}

#[test]
fn resample_keeps_the_values_at_the_scan_times() {
    let output = resample(&[1.0, 1.25, 1.5, 1.75, 2.0], &[0.0, 10.0, 20.0, 30.0, 40.0]);
    assert_eq!(output, vec![0.0, 10.0, 20.0, 30.0, 40.0]);
}

#[test]
fn resample_takes_the_value_between_two_scans() {
    let output = resample(&[1.0, 1.5, 2.0], &[0.0, 100.0, 0.0]);
    assert_eq!(output, vec![0.0, 50.0, 100.0, 50.0, 0.0]);
}

#[test]
fn resample_holds_the_edge_values_outside_the_scans() {
    let output = resample(&[1.4, 1.6], &[7.0, 9.0]);
    assert_eq!(output, vec![7.0, 7.0, 8.0, 9.0, 9.0]);
}

#[test]
fn a_warp_model_reads_its_settings() {
    let warp = load_warp_model(MODEL_PATH).expect("warp model");

    assert_eq!(warp.points(), POINTS);
    assert_eq!(warp.time_range().from, 0.5);
    assert_eq!(warp.time_range().to, 5.0);
    assert_eq!(warp.context_mz(), &[300.1, 310.2, 320.3]);
}

#[test]
fn predictions_match_onnx() {
    let warp = load_warp_model(MODEL_PATH).expect("warp model");
    let grid: Vec<f64> = (0..POINTS)
        .map(|point| 0.5 + 4.5 * point as f64 / (POINTS - 1) as f64)
        .collect();
    let mut traces = get_traces(4, 7, 3, 11);
    traces.extend(get_traces(3, 5, 1, 13));

    let predicted = warp
        .predict_from_traces(&TARGETS, &grid, &traces)
        .expect("predictions");

    for (prediction, (rt, uncertainty)) in predicted
        .iter()
        .zip(EXPECTED_RTS.iter().zip(EXPECTED_UNCERTAINTIES))
    {
        assert!(
            (prediction.rt - rt).abs() < TOLERANCE,
            "rt {} should be {}",
            prediction.rt,
            rt
        );
        assert!(
            (prediction.uncertainty - uncertainty).abs() < TOLERANCE,
            "uncertainty {} should be {}",
            prediction.uncertainty,
            uncertainty
        );
    }
}

#[test]
fn predict_rts_gives_one_prediction_per_target_from_a_file() {
    let warp = load_warp_model(MODEL_PATH).expect("warp model");
    let mut ion = open_file(COVERING_FILE);

    let predicted =
        predict_rts(&warp, &mut EicReader::Ion(&mut ion), &TARGETS).expect("predictions");

    assert_eq!(predicted.len(), TARGETS.len());
    assert!(
        predicted
            .iter()
            .all(|prediction| prediction.rt.is_finite() && prediction.uncertainty.is_finite())
    );
}

#[test]
fn a_target_without_mz_is_refused() {
    let warp = load_warp_model(MODEL_PATH).expect("warp model");
    let mut ion = open_file(COVERING_FILE);
    let targets = [
        TARGETS[0],
        Target {
            mz: f64::NAN,
            rt: 2.0,
        },
    ];

    let result = predict_rts(&warp, &mut EicReader::Ion(&mut ion), &targets);

    assert!(matches!(result, Err(WarpError::TargetMissing(1))));
}

#[test]
fn a_file_without_scans_in_the_model_range_is_refused() {
    let warp = load_warp_model(MODEL_PATH).expect("warp model");
    let mut ion = open_file(EMPTY_FILE);

    let result = predict_rts(&warp, &mut EicReader::Ion(&mut ion), &TARGETS);

    assert!(matches!(
        result,
        Err(WarpError::ScansNotCovered { found: None, .. })
    ));
}

#[test]
fn a_file_whose_scans_cover_part_of_the_model_range_is_refused() {
    let warp = load_warp_model(MODEL_PATH).expect("warp model");
    let scans: Vec<ScanTime> = [1.0, 2.0, 3.0, 4.0]
        .iter()
        .enumerate()
        .map(|(index, rt)| ScanTime { index, rt: *rt })
        .collect();

    let error = warp.check_scans(&scans).expect_err("scans from 1 to 4 min");

    assert_eq!(
        error.to_string(),
        "the file's scans cover 1 to 4 min, the warp model needs 0.5 to 5 min"
    );
}

#[test]
fn no_targets_give_no_predictions_without_reading_the_file() {
    let warp = load_warp_model(MODEL_PATH).expect("warp model");
    let mut ion = open_file(EMPTY_FILE);

    let predicted = predict_rts(&warp, &mut EicReader::Ion(&mut ion), &[]).expect("predictions");

    assert!(predicted.is_empty());
}

#[test]
fn a_missing_model_file_is_refused() {
    let result = load_warp_model("tests/fixtures/missing.onnx");

    assert!(matches!(result, Err(WarpError::ModelNotRead(_))));
}

#[cfg(not(feature = "http"))]
#[test]
fn a_url_needs_the_http_feature() {
    let result = load_warp_model("http://127.0.0.1:9/shift_model.onnx");

    assert!(
        matches!(result, Err(WarpError::ModelNotRead(reason)) if reason.contains("http feature"))
    );
}

#[cfg(feature = "http")]
mod from_url {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread,
    };

    use super::*;

    fn serve(bytes: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let address = listener.local_addr().expect("address");
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().expect("clone"));
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap_or(0) > 0 && line != "\r\n" {
                    line.clear();
                }
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    bytes.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&bytes);
            }
        });
        format!("http://{address}/shift_model.onnx")
    }

    #[test]
    fn a_model_from_a_url_predicts_like_one_from_a_path() {
        let url = serve(std::fs::read(MODEL_PATH).expect("model file"));
        let from_url = load_warp_model(url.as_str()).expect("warp model from url");
        let from_path = load_warp_model(MODEL_PATH).expect("warp model from path");
        let mut ion = open_file(COVERING_FILE);
        let mut reader = EicReader::Ion(&mut ion);

        let url_predictions =
            predict_rts(&from_url, &mut reader, &TARGETS).expect("url predictions");
        let path_predictions =
            predict_rts(&from_path, &mut reader, &TARGETS).expect("path predictions");

        assert_eq!(url_predictions, path_predictions);
    }
}
