use std::path::PathBuf;

use quantion::{IonOutput, WriteIonOptions, WriteMzmlOptions};

use crate::{
    error::ToolError,
    input::Input,
    safety::folders::AllowedFolders,
    schema::{Field, Kind, OVERWRITE},
    state::file_cache::FileFormat,
};

const OUT: Field = Field::required("out", Kind::Text, "");

pub const WRITE_ION_FIELDS: &[Field] = &[
    OUT,
    OVERWRITE,
    Field::optional("level", Kind::Count, "Compression level, 0 to 22"),
    Field::optional("f32_compress", Kind::Flag, ""),
    Field::optional("block_size", Kind::Count, ""),
    Field::optional(
        "mz_window",
        Kind::Count,
        "Storage window in Da, 0 keeps spectra whole",
    ),
];

pub const WRITE_MZML_FIELDS: &[Field] = &[OUT, OVERWRITE];

pub struct Output<T> {
    pub options: T,
    pub text: String,
}

pub fn read_write_ion_options(
    input: &mut Input,
    folders: &AllowedFolders,
) -> Result<Output<WriteIonOptions>, ToolError> {
    let default = WriteIonOptions::default();
    let (out, overwrite, text) = read_output(input, folders, FileFormat::Ion)?;
    let options = WriteIonOptions {
        out: Some(IonOutput::Path(out)),
        overwrite,
        level: input.get_count("level", default.level)?,
        f32_compress: input.get_flag("f32_compress", default.f32_compress)?,
        block_size: input.get_count("block_size", default.block_size)?,
        mz_window: input.get_count("mz_window", default.mz_window)?,
        headers: Vec::new(),
    };
    Ok(Output { options, text })
}

pub fn read_write_mzml_options(
    input: &mut Input,
    folders: &AllowedFolders,
) -> Result<Output<WriteMzmlOptions>, ToolError> {
    let (out, overwrite, text) = read_output(input, folders, FileFormat::Mzml)?;
    let options = WriteMzmlOptions {
        out: Some(out),
        overwrite,
    };
    Ok(Output { options, text })
}

fn read_output(
    input: &mut Input,
    folders: &AllowedFolders,
    format: FileFormat,
) -> Result<(PathBuf, bool, String), ToolError> {
    let overwrite = input.get_flag("overwrite", false)?;
    let text = input.get_text("out")?;
    let path = folders.get_output_path(&text, overwrite)?;
    if FileFormat::find(&path) != Some(format) {
        return Err(ToolError::invalid(
            "options.out",
            format!("must end in .{}", format.name()),
        ));
    }
    let mut part = path.clone().into_os_string();
    part.push(".part");
    if PathBuf::from(part).symlink_metadata().is_ok() {
        return Err(ToolError::file_exists(&format!("{text}.part"))
            .with_fix("remove the unfinished .part file, or choose another name"));
    }
    Ok((path, overwrite, text))
}
