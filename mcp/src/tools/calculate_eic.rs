use quantion::{
    CalculateBaselineOptions, DataXY, calculate_baseline, calculate_eic, find_noise_level,
};
use serde_json::{Value, json};

use crate::{
    error::ToolError,
    input::Input,
    output::{
        round::{Unit, round_value},
        table::{Cell, Column, Table},
    },
    schema::{
        Field, Kind, OVERWRITE,
        options::calculate_eic::{
            CALCULATE_BASELINE_FIELDS, EIC_FIELDS, read_calculate_baseline_options,
            read_calculate_eic_options,
        },
    },
    state::Context,
    tools::{Tool, table_output::SaveRequest},
};

pub struct CalculateEic;

const FIELDS: &[Field] = &[
    Field::required("file", Kind::Text, ""),
    Field::required("mz", Kind::Number, "m/z of the ion, not the neutral mass"),
    Field::optional("options", Kind::Object(EIC_FIELDS), ""),
    Field::optional(
        "save",
        Kind::Text,
        "CSV path for every point at full precision",
    ),
    OVERWRITE,
    Field::optional(
        "baseline",
        Kind::Object(CALCULATE_BASELINE_FIELDS),
        "Give it, even as {}, to add the airPLS baseline to the saved CSV",
    ),
];

impl Tool for CalculateEic {
    fn name(&self) -> &'static str {
        "calculate_eic"
    }

    fn description(&self) -> &'static str {
        "Calculate the EIC of one m/z and summarize it: points, rt range, apex and noise. The points go only to a CSV with save."
    }

    fn library_functions(&self) -> &'static [&'static str] {
        &["calculate_eic", "find_noise_level", "calculate_baseline"]
    }

    fn fields(&self) -> &'static [Field] {
        FIELDS
    }

    fn run(&self, input: &mut Input, context: &mut Context) -> Result<Value, ToolError> {
        let file = input.get_text("file")?;
        let mz = input.get_required_number("mz")?;
        let options = input.get_object("options", read_calculate_eic_options)?;
        let overwrite = input.get_flag("overwrite", false)?;
        let save = SaveRequest::read(input, context, "save", overwrite)?;
        let baseline = input.get_optional_object("baseline", read_calculate_baseline_options)?;
        if baseline.is_some() && save.is_none() {
            return Err(ToolError::invalid("baseline", "is only used with save")
                .with_fix("give save to get the baseline in the CSV"));
        }
        let full_path = context.folders.get_input_path(&file)?;
        let eic = calculate_eic(context.files.get_file(&full_path)?, mz, options)?;
        let mut output = get_summary(&eic);
        if let Some(save) = save {
            save.write(
                &get_trace_table(&eic, baseline)?.to_csv(),
                &mut output,
                "saved",
            )?;
        }
        Ok(output)
    }
}

pub fn get_summary(trace: &DataXY) -> Value {
    let Some(apex) =
        (0..trace.y.len()).max_by(|first, second| trace.y[*first].total_cmp(&trace.y[*second]))
    else {
        return json!({ "points": 0 });
    };
    let noise = find_noise_level(&trace.y);
    json!({
        "points": trace.x.len(),
        "rt": {
            "from": round_value(trace.x[0], Unit::Rt),
            "to": round_value(trace.x[trace.x.len() - 1], Unit::Rt),
        },
        "apex": {
            "rt": round_value(trace.x[apex], Unit::Rt),
            "intensity": round_value(trace.y[apex], Unit::Intensity),
        },
        "noise": round_value(noise.intensity, Unit::Intensity),
    })
}

pub fn get_trace_table(
    trace: &DataXY,
    baseline: Option<CalculateBaselineOptions>,
) -> Result<Table, ToolError> {
    let mut columns = vec![
        Column {
            name: "rt",
            unit: Unit::Rt,
        },
        Column {
            name: "intensity",
            unit: Unit::Intensity,
        },
    ];
    let baseline = match baseline {
        Some(options) => {
            columns.push(Column {
                name: "baseline",
                unit: Unit::Intensity,
            });
            Some(calculate_baseline(&trace.y, options)?)
        }
        None => None,
    };
    let mut table = Table::new(columns);
    for index in 0..trace.x.len() {
        let mut row = vec![Cell::Number(trace.x[index]), Cell::Number(trace.y[index])];
        if let Some(baseline) = &baseline {
            row.push(Cell::Number(baseline[index]));
        }
        table.add_row(row);
    }
    Ok(table)
}
