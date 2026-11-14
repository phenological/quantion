use serde_json::{Map, Value};

use crate::{
    error::ToolError,
    schema::{Field, Kind, find_field},
};

const POSITION_FIELDS: [&str; 2] = ["from", "to"];

pub fn validate_object(value: &Value, fields: &[Field], prefix: &str) -> Result<(), ToolError> {
    let Value::Object(values) = value else {
        return Err(ToolError::invalid(
            get_name(prefix, "input"),
            "must be an object",
        ));
    };
    check_known_names(values, fields, prefix)?;
    for field in fields {
        let name = get_name(prefix, field.name);
        match values.get(field.name) {
            Some(Value::Null) | None if field.required => return Err(ToolError::missing(name)),
            Some(Value::Null) | None => {}
            Some(value) => validate_value(value, &field.kind, &name)?,
        }
    }
    Ok(())
}

fn check_known_names(
    values: &Map<String, Value>,
    fields: &[Field],
    prefix: &str,
) -> Result<(), ToolError> {
    for name in values.keys() {
        if find_field(fields, name).is_none() {
            let names: Vec<&str> = fields.iter().map(|field| field.name).collect();
            return Err(get_unknown_name_error(
                &get_name(prefix, name),
                name,
                &names,
            ));
        }
    }
    Ok(())
}

fn validate_value(value: &Value, kind: &Kind, name: &str) -> Result<(), ToolError> {
    match kind {
        Kind::Number => check_number(value, name),
        Kind::Count => check_count(value, name),
        Kind::Flag if value.is_boolean() => Ok(()),
        Kind::Flag => Err(ToolError::invalid(name, "must be true or false")),
        Kind::Text => check_text(value, name),
        Kind::Choice(choices) => check_choice(value, choices, name),
        Kind::Object(fields) => validate_object(value, fields, name),
        Kind::Position => check_position(value, name),
        Kind::TextList => check_list(value, name, check_text),
        Kind::ObjectList(fields) => check_list(value, name, |item, item_name| {
            validate_object(item, fields, item_name)
        }),
    }
}

fn check_list(
    value: &Value,
    name: &str,
    check_item: impl Fn(&Value, &str) -> Result<(), ToolError>,
) -> Result<(), ToolError> {
    let Value::Array(items) = value else {
        return Err(ToolError::invalid(name, "must be a list"));
    };
    if items.is_empty() {
        return Err(ToolError::invalid(name, "must not be empty"));
    }
    for (index, item) in items.iter().enumerate() {
        check_item(item, &get_item_name(name, index))?;
    }
    Ok(())
}

pub fn get_item_name(name: &str, index: usize) -> String {
    format!("{name}[{index}]")
}

fn check_number(value: &Value, name: &str) -> Result<(), ToolError> {
    match value.as_f64() {
        Some(number) if number.is_finite() => Ok(()),
        _ => Err(ToolError::invalid(name, "must be a number")),
    }
}

fn check_count(value: &Value, name: &str) -> Result<(), ToolError> {
    match value.as_u64() {
        Some(_) => Ok(()),
        None => Err(ToolError::invalid(
            name,
            "must be a whole number, 0 or more",
        )),
    }
}

fn check_text(value: &Value, name: &str) -> Result<(), ToolError> {
    match value.as_str() {
        Some(text) if !text.is_empty() => Ok(()),
        Some(_) => Err(ToolError::invalid(name, "must not be empty")),
        None => Err(ToolError::invalid(name, "must be text")),
    }
}

fn check_choice(value: &Value, choices: &[&str], name: &str) -> Result<(), ToolError> {
    let text = value.as_str().unwrap_or_default();
    if choices.contains(&text) {
        return Ok(());
    }
    Err(
        ToolError::invalid(name, format!("must be one of {}", choices.join(", ")))
            .with_fix(get_suggestion(text, choices)),
    )
}

fn check_position(value: &Value, name: &str) -> Result<(), ToolError> {
    if value.is_number() {
        return check_number(value, name);
    }
    let Value::Object(values) = value else {
        return Err(ToolError::invalid(
            name,
            "must be a number or a range {from, to}",
        ));
    };
    for key in values.keys() {
        if !POSITION_FIELDS.contains(&key.as_str()) {
            return Err(get_unknown_name_error(
                &get_name(name, key),
                key,
                &POSITION_FIELDS,
            ));
        }
    }
    for key in POSITION_FIELDS {
        match values.get(key) {
            Some(value) => check_number(value, &get_name(name, key))?,
            None => return Err(ToolError::missing(get_name(name, key))),
        }
    }
    Ok(())
}

fn get_unknown_name_error(argument: &str, name: &str, names: &[&str]) -> ToolError {
    ToolError::unknown_option(argument).with_fix(get_suggestion(name, names))
}

fn get_suggestion(text: &str, choices: &[&str]) -> String {
    match find_closest(text, choices) {
        Some(closest) => format!("use {closest}"),
        None => format!("use one of {}", choices.join(", ")),
    }
}

fn find_closest<'a>(text: &str, choices: &[&'a str]) -> Option<&'a str> {
    let text = text.to_lowercase();
    choices
        .iter()
        .map(|choice| (*choice, get_edit_distance(&text, &choice.to_lowercase())))
        .filter(|(choice, distance)| *distance <= (choice.len() / 3).max(1))
        .min_by_key(|(_, distance)| *distance)
        .map(|(choice, _)| choice)
}

fn get_edit_distance(first: &str, second: &str) -> usize {
    let second: Vec<char> = second.chars().collect();
    let mut previous: Vec<usize> = (0..=second.len()).collect();
    let mut current = vec![0; second.len() + 1];
    for (row, first_char) in first.chars().enumerate() {
        current[0] = row + 1;
        for (column, second_char) in second.iter().enumerate() {
            let cost = usize::from(first_char != *second_char);
            current[column + 1] = (previous[column] + cost)
                .min(previous[column + 1] + 1)
                .min(current[column] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[second.len()]
}

pub fn get_name(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}.{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_closest_name() {
        assert_eq!(
            find_closest("min_snrr", &["min_snr", "min_r2"]),
            Some("min_snr")
        );
        assert_eq!(find_closest("emg", &["EMG", "GAUSSIAN"]), Some("EMG"));
        assert_eq!(find_closest("color", &["min_snr", "min_r2"]), None);
    }
}
