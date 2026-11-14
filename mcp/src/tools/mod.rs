pub mod annotate_features;
pub mod calculate_eic;
pub mod calibrate;
pub mod draw_peaks;
pub mod feature_table;
pub mod find_features;
pub mod find_peaks;
pub mod find_targets;
pub mod find_transitions;
pub mod fit_peak;
pub mod get_peak;
pub mod get_scan_metadata;
pub mod get_scans;
pub mod list_files;
pub mod match_features;
pub mod peak_table;
pub mod predict_rts;
pub mod table_output;
pub mod target_table;
pub mod trace;
pub mod write_ion;
pub mod write_mzml;

use std::panic::{AssertUnwindSafe, catch_unwind};

use serde_json::{Value, json};

use crate::{
    error::ToolError,
    input::Input,
    schema::{Field, write::get_json_schema},
    state::Context,
};

pub trait Tool: Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn library_functions(&self) -> &'static [&'static str];
    fn fields(&self) -> &'static [Field];
    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError>;
}

pub static TOOLS: &[&dyn Tool] = &[
    &list_files::ListFiles,
    &write_ion::WriteIon,
    &write_mzml::WriteMzml,
    &get_scans::GetScans,
    &get_scan_metadata::GetScanMetadata,
    &calculate_eic::CalculateEic,
    &find_peaks::FindPeaks,
    &get_peak::GetPeak,
    &fit_peak::FitPeak,
    &draw_peaks::DrawPeaks,
    &find_targets::FindTargets,
    &find_transitions::FindTransitions,
    &find_features::FindFeatures,
    &match_features::MatchFeatures,
    &annotate_features::AnnotateFeatures,
    &calibrate::Calibrate,
    &predict_rts::PredictRts,
];

pub fn find_tool(name: &str) -> Option<&'static dyn Tool> {
    TOOLS.iter().copied().find(|tool| tool.name() == name)
}

pub const OPENED_BY_EVERY_TOOL: &[&str] = &["parse_ion", "parse_mzml"];

pub const EXPERIMENTAL_WITHOUT_TOOL: &[&str] = &["get_ion_image"];

pub fn get_tool_list() -> Value {
    let tools: Vec<Value> = TOOLS
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name(),
                "description": tool.description(),
                "inputSchema": get_json_schema(tool.fields()),
            })
        })
        .collect();
    json!({ "tools": tools })
}

pub fn get_tool_functions() -> Value {
    let tools: Vec<Value> = TOOLS
        .iter()
        .map(|tool| json!({ "name": tool.name(), "functions": tool.library_functions() }))
        .collect();
    json!({ "tools": tools, "opened_by_every_tool": OPENED_BY_EVERY_TOOL, "experimental_without_tool": EXPERIMENTAL_WITHOUT_TOOL })
}

pub fn run_tool(
    tool: &dyn Tool,
    arguments: Value,
    context: &mut Context,
) -> Result<Value, ToolError> {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let mut input = Input::new(arguments, tool.fields())?;
        let output = tool.run(&mut input, context)?;
        input.finish()?;
        Ok(output)
    }));
    match outcome {
        Ok(result) => result,
        Err(_) => {
            context.files.clear();
            Err(ToolError::internal(format!(
                "{} stopped unexpectedly",
                tool.name()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_public_function_has_a_tool() {
        let surface = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../scripts/check-api/surface/rust.txt"
        ))
        .expect("public API snapshot");
        let functions: Vec<&str> = surface
            .lines()
            .filter_map(|line| line.strip_prefix("pub fn "))
            .filter_map(|line| line.split('(').next())
            .collect();
        assert!(!functions.is_empty());
        for function in functions {
            let has_tool = TOOLS
                .iter()
                .any(|tool| tool.library_functions().contains(&function));
            let is_listed = OPENED_BY_EVERY_TOOL.contains(&function)
                || EXPERIMENTAL_WITHOUT_TOOL.contains(&function);
            assert!(
                has_tool || is_listed,
                "{function} has no tool and is not listed"
            );
            assert!(
                !(has_tool && is_listed),
                "{function} has a tool and is also listed"
            );
        }
    }

    #[test]
    fn tool_names_are_library_names() {
        for tool in TOOLS {
            let is_library_name = tool.library_functions().first() == Some(&tool.name());
            assert!(
                is_library_name || tool.library_functions().is_empty(),
                "{} must be named after its library function",
                tool.name()
            );
        }
    }
}
