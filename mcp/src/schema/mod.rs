pub mod options;
pub mod validate;
pub mod write;

#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub kind: Kind,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Number,
    Count,
    Flag,
    Text,
    Choice(&'static [&'static str]),
    Object(&'static [Field]),
    Position,
    TextList,
    ObjectList(&'static [Field]),
}

impl Field {
    pub const fn required(name: &'static str, kind: Kind, description: &'static str) -> Self {
        Self {
            name,
            kind,
            required: true,
            description,
        }
    }

    pub const fn optional(name: &'static str, kind: Kind, description: &'static str) -> Self {
        Self {
            name,
            kind,
            required: false,
            description,
        }
    }
}

pub const LIMIT: Field = Field::optional("limit", Kind::Count, "");
pub const SAVE: Field = Field::optional("save", Kind::Text, "");
pub const OVERWRITE: Field = Field::optional("overwrite", Kind::Flag, "");

pub fn find_field<'a>(fields: &'a [Field], name: &str) -> Option<&'a Field> {
    fields.iter().find(|field| field.name == name)
}
