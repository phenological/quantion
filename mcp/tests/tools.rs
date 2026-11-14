mod common;

use serde_json::json;

use common::{PEAK_RT, Server, TestFolder};

#[test]
fn lists_writes_and_finds_the_peak() {
    let folder = TestFolder::new("tools");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);

    let (files, is_error) = server.call_tool("list_files", json!({}));
    assert!(!is_error, "{files}");
    assert_eq!(files["rows"][0][0], "sample_1.mzML");
    assert_eq!(files["rows"][0][1], "mzML");

    let (converted, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": "sample_1.mzML", "options": { "out": "sample_1.ion" } }),
    );
    assert!(!is_error, "{converted}");
    assert!(converted["size"].as_u64().unwrap_or(0) > 0);
    assert!(folder.path.join("sample_1.ion").exists());

    let trace = json!({ "file": "sample_1.ion", "mz": common::PEAK_MZ });
    let (summary, is_error) = server.call_tool(
        "calculate_eic",
        json!({ "file": "sample_1.ion", "mz": common::PEAK_MZ, "save": "trace.csv", "baseline": {} }),
    );
    assert!(!is_error, "{summary}");
    assert_eq!(summary["points"], 201);
    assert_eq!(summary["apex"]["rt"], PEAK_RT);
    assert_eq!(summary["saved"], "trace.csv");

    let (peaks, is_error) = server.call_tool("find_peaks", json!({ "data": trace }));
    assert!(!is_error, "{peaks}");
    assert_eq!(peaks["total"], 1);
    assert_eq!(peaks["columns"][0], "rt");
    let rt = peaks["rows"][0][0].as_f64().expect("rt");
    assert!((rt - PEAK_RT).abs() < 0.01, "rt {rt}");

    let (peak, is_error) = server.call_tool(
        "get_peak",
        json!({ "data": { "path": "trace.csv" }, "rt": PEAK_RT, "options": { "range": 0.2 } }),
    );
    assert!(!is_error, "{peak}");
    assert!((peak["rt"].as_f64().expect("rt") - PEAK_RT).abs() < 0.01);
}

#[test]
fn rejects_bad_input_with_a_fix() {
    let folder = TestFolder::new("bad-input");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);
    let trace = json!({ "file": "sample_1.mzML", "mz": common::PEAK_MZ });

    let (error, is_error) = server.call_tool(
        "find_peaks",
        json!({ "data": trace, "options": { "min_snrr": 3 } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "unknown_option");
    assert_eq!(error["error"]["argument"], "options.min_snrr");
    assert_eq!(error["error"]["fix"], "use min_snr");

    let (error, is_error) = server.call_tool(
        "find_peaks",
        json!({ "data": trace, "options": { "shape": "emg" } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["fix"], "use EMG");

    let (error, is_error) =
        server.call_tool("find_peaks", json!({ "data": { "file": "sample_1.mzML" } }));
    assert!(is_error);
    assert_eq!(error["error"]["argument"], "data");

    let (error, is_error) = server.call_tool(
        "find_peaks",
        json!({ "data": trace, "options": { "min_r2": 2 } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "invalid_argument");

    let (error, is_error) = server.call_tool("get_peak", json!({ "data": trace }));
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "missing_argument");
}

#[test]
fn finds_targets_in_many_files() {
    let folder = TestFolder::new("targets");
    folder.write_sample("sample_1.mzML");
    folder.write_sample("sample_2.mzML");
    let mut server = Server::start(&folder.path);
    let (table, is_error) = server.call_tool(
        "find_targets",
        json!({
            "files": ["sample_1.mzML", "sample_2.mzML"],
            "targets": [
                { "id": "compound_1", "mz": common::PEAK_MZ, "rt": PEAK_RT, "range": 0.2 },
                { "id": "compound_2", "mz": 600.3, "rt": PEAK_RT, "range": 0.2 }
            ],
            "save": "targets.csv"
        }),
    );
    assert!(!is_error, "{table}");
    assert_eq!(table["total"], 4);
    assert_eq!(table["columns"][0], "file");
    assert_eq!(table["rows"][0][0], "sample_1.mzML");
    assert_eq!(table["rows"][0][1], "compound_1");
    assert!((table["rows"][0][2].as_f64().unwrap() - PEAK_RT).abs() < 0.01);
    assert_eq!(table["rows"][1][2], 0.0);
    assert_eq!(table["rows"][3][0], "sample_2.mzML");
    assert!(folder.path.join("targets.csv").exists());
}

#[test]
fn finds_and_matches_features() {
    let folder = TestFolder::new("features");
    folder.write_sample("sample_1.mzML");
    folder.write_sample("sample_2.mzML");
    let mut server = Server::start(&folder.path);
    let options = json!({ "grid": { "start": 299.0, "end": 301.0 } });

    let (features, is_error) = server.call_tool(
        "find_features",
        json!({ "file": "sample_1.mzML", "options": options, "deisotope": {}, "save": "features.csv" }),
    );
    assert!(!is_error, "{features}");
    assert_eq!(features["columns"][0], "mz");
    let found = features["rows"].as_array().unwrap().iter().any(|row| {
        (row[0].as_f64().unwrap() - common::PEAK_MZ).abs() < 0.01
            && (row[1].as_f64().unwrap() - PEAK_RT).abs() < 0.01
    });
    assert!(found, "{features}");

    let (matched, is_error) = server.call_tool("match_features", json!({ "options": options }));
    assert!(!is_error, "{matched}");
    let frequency = matched["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| (row[0].as_f64().unwrap() - common::PEAK_MZ).abs() < 0.01)
        .map(|row| row[7].as_f64().unwrap());
    assert_eq!(frequency, Some(1.0), "{matched}");
}

#[test]
fn annotation_needs_a_local_library() {
    let folder = TestFolder::new("annotate");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);
    let (error, is_error) = server.call_tool(
        "annotate_features",
        json!({
            "files": ["sample_1.mzML"],
            "feature_list": [{ "mz": common::PEAK_MZ, "rt": PEAK_RT }],
            "library": "library_positive.ion",
            "model": "model.onnx"
        }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "file_not_found");
    assert!(
        error["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("josoriom/cipher")
    );

    std::fs::write(folder.path.join("library.ion"), b"").unwrap();
    std::fs::write(folder.path.join("model.onnx"), b"").unwrap();
    let (error, is_error) = server.call_tool(
        "annotate_features",
        json!({
            "files": ["sample_1.mzML"],
            "library": "library.ion",
            "model": "model.onnx",
            "features": "features.csv",
            "feature_list": [{ "mz": 1, "rt": 1 }]
        }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["argument"], "features");
}

#[test]
fn reads_scans_and_their_metadata() {
    let folder = TestFolder::new("scans");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);

    let (scans, is_error) = server.call_tool(
        "get_scans",
        json!({
            "file": "sample_1.mzML",
            "query": { "rt": { "from": 1.995, "to": 2.005 } },
            "peaks": 1,
            "save_spectra": "spectra.csv"
        }),
    );
    assert!(!is_error, "{scans}");
    assert_eq!(scans["total"], 1);
    assert_eq!(scans["columns"][0], "index");
    assert_eq!(scans["rows"][0][8], 3);
    assert_eq!(scans["spectra"][0]["mz"][0], common::PEAK_MZ);
    let spectra = std::fs::read_to_string(folder.path.join("spectra.csv")).unwrap();
    assert_eq!(spectra.lines().count(), 4);

    let (closest, is_error) = server.call_tool(
        "get_scans",
        json!({ "file": "sample_1.mzML", "query": { "rt": PEAK_RT } }),
    );
    assert!(!is_error, "{closest}");
    assert_eq!(closest["total"], 1);
    let index = closest["rows"][0][0].as_u64().unwrap();

    let (metadata, is_error) = server.call_tool(
        "get_scan_metadata",
        json!({ "file": "sample_1.mzML", "index": index, "options": { "tags": ["spectrum"] } }),
    );
    assert!(!is_error, "{metadata}");
    let has_level = metadata["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row[3] == "ms level" && row[4] == "1");
    assert!(has_level, "{metadata}");
}

#[test]
fn fits_and_draws_peaks() {
    let folder = TestFolder::new("fit");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);

    let (fit, is_error) = server.call_tool(
        "fit_peak",
        json!({
            "data": { "file": "sample_1.mzML", "mz": common::PEAK_MZ },
            "rt": PEAK_RT,
            "options": { "shape": "GAUSSIAN" }
        }),
    );
    assert!(!is_error, "{fit}");
    assert!(
        (fit["center"].as_f64().unwrap() - PEAK_RT).abs() < 0.01,
        "{fit}"
    );
    assert!(fit["r2"].as_f64().unwrap() > 0.9, "{fit}");

    let (drawn, is_error) = server.call_tool(
        "draw_peaks",
        json!({
            "params": [{ "height": 1000.0, "center": 2.0, "fwhm": 0.05 }],
            "options": { "from": 1.5, "to": 2.5, "points": 101 },
            "save": "drawn.csv"
        }),
    );
    assert!(!is_error, "{drawn}");
    assert_eq!(drawn["points"], 101);
    assert_eq!(drawn["apex"]["rt"], 2.0);
    assert!(folder.path.join("drawn.csv").exists());
}

#[test]
fn rt_models_need_a_local_model() {
    let folder = TestFolder::new("models");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);
    let targets = json!([{ "id": "compound_1", "mz": common::PEAK_MZ, "rt": PEAK_RT }]);

    let (error, is_error) = server.call_tool(
        "predict_rts",
        json!({ "model": "model.onnx", "files": ["sample_1.mzML"], "targets": targets }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "file_not_found");
    assert!(error["error"]["fix"].as_str().unwrap().contains("deepwarp"));

    let (error, is_error) = server.call_tool(
        "calibrate",
        json!({ "model": "model.onnx", "anchors": [{ "file": "sample_1.mzML", "targets": targets }] }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "file_not_found");

    std::fs::write(folder.path.join("model.onnx"), b"not a model").unwrap();
    let (error, is_error) = server.call_tool(
        "predict_rts",
        json!({ "model": "model.onnx", "files": ["sample_1.mzML"], "targets": targets }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "parse_failed");
}

#[test]
fn reads_chosen_scan_arrays() {
    let folder = TestFolder::new("arrays");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);
    let (scans, is_error) = server.call_tool(
        "get_scans",
        json!({
            "file": "sample_1.mzML",
            "query": { "rt": PEAK_RT },
            "options": { "arrays": [{ "key": "intensity", "accession": "MS:1000515" }] },
            "save_spectra": "arrays.csv"
        }),
    );
    assert!(!is_error, "{scans}");
    let text = std::fs::read_to_string(folder.path.join("arrays.csv")).unwrap();
    assert!(text.starts_with("index,rt,intensity\n"), "{text}");
    assert_eq!(text.lines().count(), 4);
}
