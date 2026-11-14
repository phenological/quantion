use quantion::{FromTo, Position};
use serde_json::{Map, Value};

use crate::{
    error::ToolError,
    schema::{
        Field, Kind, find_field,
        validate::{get_item_name, get_name, validate_object},
    },
};

pub struct Input {
    values: Map<String, Value>,
    fields: &'static [Field],
    prefix: String,
    read_names: Vec<&'static str>,
}

impl Input {
    pub fn new(value: Value, fields: &'static [Field]) -> Result<Self, ToolError> {
        let value = match value {
            Value::Null => Value::Object(Map::new()),
            value => value,
        };
        validate_object(&value, fields, "")?;
        Ok(Self::from_valid(value, fields, String::new()))
    }

    fn from_valid(value: Value, fields: &'static [Field], prefix: String) -> Self {
        let values = match value {
            Value::Object(values) => values,
            _ => Map::new(),
        };
        Self {
            values,
            fields,
            prefix,
            read_names: Vec::new(),
        }
    }

    pub fn get_number(&mut self, name: &'static str, default: f64) -> Result<f64, ToolError> {
        Ok(self.get_optional_number(name)?.unwrap_or(default))
    }

    pub fn get_optional_number(&mut self, name: &'static str) -> Result<Option<f64>, ToolError> {
        Ok(self.take(name)?.and_then(|value| value.as_f64()))
    }

    pub fn get_required_number(&mut self, name: &'static str) -> Result<f64, ToolError> {
        let argument = self.get_argument(name);
        self.get_optional_number(name)?
            .ok_or_else(|| ToolError::missing(argument))
    }

    pub fn get_count<T: TryFrom<u64>>(
        &mut self,
        name: &'static str,
        default: T,
    ) -> Result<T, ToolError> {
        let argument = self.get_argument(name);
        match self.take(name)?.and_then(|value| value.as_u64()) {
            Some(count) => {
                T::try_from(count).map_err(|_| ToolError::invalid(argument, "is too large"))
            }
            None => Ok(default),
        }
    }

    pub fn get_flag(&mut self, name: &'static str, default: bool) -> Result<bool, ToolError> {
        Ok(self
            .take(name)?
            .and_then(|value| value.as_bool())
            .unwrap_or(default))
    }

    pub fn get_optional_text(&mut self, name: &'static str) -> Result<Option<String>, ToolError> {
        Ok(self
            .take(name)?
            .and_then(|value| value.as_str().map(str::to_string)))
    }

    pub fn get_text(&mut self, name: &'static str) -> Result<String, ToolError> {
        let argument = self.get_argument(name);
        self.get_optional_text(name)?
            .ok_or_else(|| ToolError::missing(argument))
    }

    pub fn get_position(&mut self, name: &'static str) -> Result<Position, ToolError> {
        let argument = self.get_argument(name);
        match self.take(name)? {
            None => Ok(Position::Any),
            Some(Value::Number(number)) => number
                .as_f64()
                .map(Position::Closest)
                .ok_or_else(|| ToolError::invalid(argument, "must be a number")),
            Some(range) => match (range["from"].as_f64(), range["to"].as_f64()) {
                (Some(from), Some(to)) => Ok(Position::Range(FromTo { from, to })),
                _ => Err(ToolError::internal(format!(
                    "{argument} passed validation without from and to"
                ))),
            },
        }
    }

    pub fn get_object<T>(
        &mut self,
        name: &'static str,
        read: impl FnOnce(&mut Input) -> Result<T, ToolError>,
    ) -> Result<T, ToolError> {
        let fields = self.get_object_fields(name)?;
        let value = self.take(name)?.unwrap_or(Value::Null);
        read_child(value, fields, self.get_argument(name), read)
    }

    pub fn get_optional_object<T>(
        &mut self,
        name: &'static str,
        read: impl FnOnce(&mut Input) -> Result<T, ToolError>,
    ) -> Result<Option<T>, ToolError> {
        let fields = self.get_object_fields(name)?;
        match self.take(name)? {
            Some(value) => Ok(Some(read_child(
                value,
                fields,
                self.get_argument(name),
                read,
            )?)),
            None => Ok(None),
        }
    }

    pub fn get_text_list(&mut self, name: &'static str) -> Result<Vec<String>, ToolError> {
        let argument = self.get_argument(name);
        self.get_optional_text_list(name)?
            .ok_or_else(|| ToolError::missing(argument))
    }

    pub fn get_optional_text_list(
        &mut self,
        name: &'static str,
    ) -> Result<Option<Vec<String>>, ToolError> {
        let Some(Value::Array(items)) = self.take(name)? else {
            return Ok(None);
        };
        Ok(Some(
            items
                .into_iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect(),
        ))
    }

    pub fn get_object_list<T>(
        &mut self,
        name: &'static str,
        read: impl FnMut(&mut Input) -> Result<T, ToolError>,
    ) -> Result<Vec<T>, ToolError> {
        let argument = self.get_argument(name);
        self.get_optional_object_list(name, read)?
            .ok_or_else(|| ToolError::missing(argument))
    }

    pub fn get_optional_object_list<T>(
        &mut self,
        name: &'static str,
        mut read: impl FnMut(&mut Input) -> Result<T, ToolError>,
    ) -> Result<Option<Vec<T>>, ToolError> {
        let argument = self.get_argument(name);
        let Some(Kind::ObjectList(fields)) = find_field(self.fields, name).map(|field| field.kind)
        else {
            return Err(ToolError::internal(format!(
                "{argument} is not declared as a list of objects"
            )));
        };
        let Some(Value::Array(items)) = self.take(name)? else {
            return Ok(None);
        };
        items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                read_child(item, fields, get_item_name(&argument, index), &mut read)
            })
            .collect::<Result<Vec<T>, ToolError>>()
            .map(Some)
    }

    fn get_object_fields(&self, name: &'static str) -> Result<&'static [Field], ToolError> {
        match find_field(self.fields, name).map(|field| field.kind) {
            Some(Kind::Object(fields)) => Ok(fields),
            _ => Err(ToolError::internal(format!(
                "{} is not declared as an object",
                self.get_argument(name)
            ))),
        }
    }

    pub fn finish(self) -> Result<(), ToolError> {
        for field in self.fields {
            if !self.read_names.contains(&field.name) {
                return Err(ToolError::internal(format!(
                    "{} is declared but never read",
                    get_name(&self.prefix, field.name)
                )));
            }
        }
        Ok(())
    }

    fn take(&mut self, name: &'static str) -> Result<Option<Value>, ToolError> {
        if find_field(self.fields, name).is_none() {
            return Err(ToolError::internal(format!(
                "{} is read but not declared",
                self.get_argument(name)
            )));
        }
        self.read_names.push(name);
        Ok(self.values.remove(name).filter(|value| !value.is_null()))
    }

    fn get_argument(&self, name: &str) -> String {
        get_name(&self.prefix, name)
    }
}

fn read_child<T>(
    value: Value,
    fields: &'static [Field],
    prefix: String,
    read: impl FnOnce(&mut Input) -> Result<T, ToolError>,
) -> Result<T, ToolError> {
    let mut input = Input::from_valid(value, fields, prefix);
    let output = read(&mut input)?;
    input.finish()?;
    Ok(output)
}
