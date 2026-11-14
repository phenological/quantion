mod common;

use serde_json::json;

use common::{Server, TestFolder};

const BYTES_PER_TOKEN: usize = 4;
const TOOL_LIST_TOKENS: usize = 5_000;
const REPLY_TOKENS: usize = 400;

#[test]
fn tool_list_stays_small() {
    let folder = TestFolder::new("budget-list");
    let mut server = Server::start(&folder.path);
    let reply = server.request("tools/list", json!({}));
    let size = reply["result"].to_string().len();
    assert!(
        size <= TOOL_LIST_TOKENS * BYTES_PER_TOKEN,
        "tools/list is about {} tokens, more than {TOOL_LIST_TOKENS}",
        size / BYTES_PER_TOKEN
    );
}

#[test]
fn replies_stay_small() {
    let folder = TestFolder::new("budget-reply");
    folder.write_sample("sample_1.mzML");
    let mut server = Server::start(&folder.path);
    let trace = json!({ "file": "sample_1.mzML", "mz": common::PEAK_MZ });
    for (tool, arguments) in [
        ("list_files", json!({})),
        (
            "calculate_eic",
            json!({ "file": "sample_1.mzML", "mz": common::PEAK_MZ }),
        ),
        ("find_peaks", json!({ "data": trace })),
        ("get_peak", json!({ "data": trace, "rt": common::PEAK_RT })),
    ] {
        let (output, is_error) = server.call_tool(tool, arguments);
        assert!(!is_error, "{tool}: {output}");
        let size = output.to_string().len();
        assert!(
            size <= REPLY_TOKENS * BYTES_PER_TOKEN,
            "{tool} reply is about {} tokens, more than {REPLY_TOKENS}",
            size / BYTES_PER_TOKEN
        );
    }
}
