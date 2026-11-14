use serde_json::{Map, Value, json};

use crate::schema::{Field, Kind};

pub fn get_json_schema(fields: &[Field]) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for field in fields {
        properties.insert(field.name.to_string(), get_field_schema(field));
        if field.required {
            required.push(json!(field.name));
        }
    }
    let mut schema = json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false,
    });
    if !required.is_empty() {
        schema["required"] = Value::Array(required);
    }
    schema
}

fn get_field_schema(field: &Field) -> Value {
    let mut schema = get_kind_schema(&field.kind);
    if !field.description.is_empty() {
        schema["description"] = json!(field.description);
    }
    schema
}

fn get_kind_schema(kind: &Kind) -> Value {
    match kind {
        Kind::Number => json!({ "type": "number" }),
        Kind::Count => json!({ "type": "integer", "minimum": 0 }),
        Kind::Flag => json!({ "type": "boolean" }),
        Kind::Text => json!({ "type": "string" }),
        Kind::Choice(choices) => json!({ "type": "string", "enum": choices }),
        Kind::Object(fields) => get_json_schema(fields),
        Kind::Position => json!({
            "anyOf": [
                { "type": "number" },
                {
                    "type": "object",
                    "properties": { "from": { "type": "number" }, "to": { "type": "number" } },
                    "required": ["from", "to"],
                    "additionalProperties": false,
                },
            ],
        }),
        Kind::TextList => json!({ "type": "array", "items": { "type": "string" }, "minItems": 1 }),
        Kind::ObjectList(fields) => {
            json!({ "type": "array", "items": get_json_schema(fields), "minItems": 1 })
        }
    }
}
