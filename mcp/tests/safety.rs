mod common;

use std::fs;

use serde_json::json;

use common::{Server, TestFolder};

#[test]
fn stays_inside_the_allowed_folders() {
    let folder = TestFolder::new("safety-allowed");
    let outside = TestFolder::new("safety-outside");
    folder.write_sample("sample_1.mzML");
    outside.write_sample("secret.mzML");
    let mut server = Server::start(&folder.path);
    let outside_file = outside.path.join("secret.mzML").display().to_string();

    let (error, is_error) = server.call_tool(
        "list_files",
        json!({ "folder": outside.path.display().to_string() }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "folder_not_allowed");

    let (error, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": outside_file, "options": { "out": "copy.ion" } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "folder_not_allowed");

    let escape = format!(
        "../{}/secret.mzML",
        outside.path.file_name().unwrap().to_str().unwrap()
    );
    let (error, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": escape, "options": { "out": "copy.ion" } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "folder_not_allowed");

    let output = outside.path.join("copy.ion").display().to_string();
    let (error, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": "sample_1.mzML", "options": { "out": output } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "folder_not_allowed");
    assert!(!outside.path.join("copy.ion").exists());

    let (error, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": "https://example.com/sample.ion", "options": { "out": "copy.ion" } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "invalid_argument");
}

#[cfg(unix)]
#[test]
fn does_not_follow_links_out_of_the_allowed_folders() {
    let folder = TestFolder::new("safety-link");
    let outside = TestFolder::new("safety-link-outside");
    folder.write_sample("sample_1.mzML");
    outside.write_sample("secret.mzML");
    std::os::unix::fs::symlink(
        outside.path.join("secret.mzML"),
        folder.path.join("link.mzML"),
    )
    .unwrap();
    std::os::unix::fs::symlink(&outside.path, folder.path.join("outside")).unwrap();
    let mut server = Server::start(&folder.path);

    let (error, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": "link.mzML", "options": { "out": "copy.ion" } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "folder_not_allowed");

    let (error, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": "sample_1.mzML", "options": { "out": "outside/copy.ion" } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "folder_not_allowed");
    assert!(!outside.path.join("copy.ion").exists());
}

#[test]
fn never_overwrites_without_permission() {
    let folder = TestFolder::new("safety-overwrite");
    folder.write_sample("sample_1.mzML");
    fs::write(folder.path.join("trace.csv"), "keep me").unwrap();
    let mut server = Server::start(&folder.path);

    let (error, is_error) = server.call_tool(
        "calculate_eic",
        json!({ "file": "sample_1.mzML", "mz": common::PEAK_MZ, "save": "trace.csv" }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "file_exists");
    assert_eq!(
        fs::read_to_string(folder.path.join("trace.csv")).unwrap(),
        "keep me"
    );

    let (_, is_error) = server.call_tool(
        "calculate_eic",
        json!({ "file": "sample_1.mzML", "mz": common::PEAK_MZ, "save": "trace.csv", "overwrite": true }),
    );
    assert!(!is_error);
    assert!(
        fs::read_to_string(folder.path.join("trace.csv"))
            .unwrap()
            .starts_with("rt,intensity")
    );

    let (error, is_error) = server.call_tool(
        "write_ion", json!({ "file": "sample_1.mzML", "options": { "out": "sample_1.mzML", "overwrite": true } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["argument"], "options.out");
}

#[cfg(unix)]
#[test]
fn match_features_refuses_links_out_of_the_allowed_folders() {
    let folder = TestFolder::new("safety-match");
    let outside = TestFolder::new("safety-match-outside");
    folder.write_sample("sample_1.mzML");
    outside.write_sample("secret.mzML");
    std::os::unix::fs::symlink(
        outside.path.join("secret.mzML"),
        folder.path.join("link.mzML"),
    )
    .unwrap();
    let mut server = Server::start(&folder.path);
    let (error, is_error) = server.call_tool("match_features", json!({}));
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "folder_not_allowed");

    let (files, is_error) = server.call_tool("list_files", json!({}));
    assert!(!is_error);
    assert_eq!(files["total"], 1);
}

#[test]
fn refuses_unsafe_or_unclear_outputs() {
    let folder = TestFolder::new("safety-outputs");
    folder.write_sample("sample_1.mzML");
    fs::write(folder.path.join("sample_1.ion.part"), "unfinished").unwrap();
    let mut server = Server::start(&folder.path);

    let (error, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": "sample_1.mzML", "options": { "out": "sample_1.ion" } }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["kind"], "file_exists");
    assert_eq!(
        fs::read_to_string(folder.path.join("sample_1.ion.part")).unwrap(),
        "unfinished"
    );

    let (error, is_error) = server.call_tool(
        "get_scans",
        json!({ "file": "sample_1.mzML", "query": { "rt": 2.0 }, "save": "same.csv", "save_spectra": "same.csv" }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["argument"], "save_spectra");
    assert!(!folder.path.join("same.csv").exists());

    let (error, is_error) = server.call_tool(
        "calculate_eic",
        json!({ "file": "sample_1.mzML", "mz": common::PEAK_MZ, "baseline": {} }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["argument"], "baseline");

    let (written, is_error) = server.call_tool(
        "write_ion",
        json!({ "file": "sample_1.mzML", "options": { "out": "copy.ion" } }),
    );
    assert!(!is_error, "{written}");
    assert_eq!(written["out"], "copy.ion");
}

#[test]
fn treats_names_that_differ_only_in_case_as_one_file() {
    let folder = TestFolder::new("safety-case");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);
    let (error, is_error) = server.call_tool(
        "get_scans",
        json!({ "file": "sample_1.mzML", "query": { "rt": 2.0 }, "save": "same.csv", "save_spectra": "SAME.csv" }),
    );
    assert!(is_error);
    assert_eq!(error["error"]["argument"], "save_spectra");
    assert!(!folder.path.join("same.csv").exists());
}
