use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{error::ToolError, state::file_cache::FileFormat};

pub struct AllowedFolders {
    folders: Vec<PathBuf>,
}

impl AllowedFolders {
    pub fn new(folders: &[PathBuf]) -> Result<Self, String> {
        let mut allowed = Vec::with_capacity(folders.len());
        for folder in folders {
            let full_path = folder
                .canonicalize()
                .map_err(|_| format!("{} does not exist", folder.display()))?;
            if !full_path.is_dir() {
                return Err(format!("{} is not a folder", folder.display()));
            }
            allowed.push(full_path);
        }
        if allowed.is_empty() {
            return Err("at least one allowed folder is needed".to_string());
        }
        Ok(Self { folders: allowed })
    }

    pub fn get_main_folder(&self) -> &Path {
        &self.folders[0]
    }

    pub fn get_input_path(&self, text: &str) -> Result<PathBuf, ToolError> {
        let path = self.resolve(text)?;
        let full_path = path
            .canonicalize()
            .map_err(|_| ToolError::file_not_found(text))?;
        self.check_allowed(&full_path, text)?;
        Ok(full_path)
    }

    pub fn get_folder_path(&self, text: &str) -> Result<PathBuf, ToolError> {
        let full_path = self.get_input_path(text)?;
        if !full_path.is_dir() {
            return Err(ToolError::invalid(text, "is not a folder"));
        }
        Ok(full_path)
    }

    pub fn check_samples(&self, folder: &Path) -> Result<(), ToolError> {
        let entries = fs::read_dir(folder).map_err(|_| {
            ToolError::invalid(
                folder.display().to_string(),
                "is not a folder that can be read",
            )
        })?;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if FileFormat::find(&path).is_none() {
                continue;
            }
            let text = path.display().to_string();
            let full_path = path
                .canonicalize()
                .map_err(|_| ToolError::file_not_found(&text))?;
            self.check_allowed(&full_path, &text)?;
        }
        Ok(())
    }

    pub fn get_output_path(&self, text: &str, overwrite: bool) -> Result<PathBuf, ToolError> {
        let path = self.resolve(text)?;
        let Some(name) = path.file_name() else {
            return Err(ToolError::invalid(text, "must end with a file name"));
        };
        let parent = path.parent().unwrap_or(Path::new(""));
        let full_parent = parent.canonicalize().map_err(|_| {
            ToolError::file_not_found(&parent.display().to_string())
                .with_fix("create the folder first, or write to an allowed folder")
        })?;
        self.check_allowed(&full_parent, text)?;
        let full_path = full_parent.join(name);
        if full_path.symlink_metadata().is_ok() {
            if !overwrite {
                return Err(ToolError::file_exists(text));
            }
            let target = full_path
                .canonicalize()
                .map_err(|_| ToolError::file_not_found(text))?;
            self.check_allowed(&target, text)?;
            return Ok(target);
        }
        Ok(full_path)
    }

    fn resolve(&self, text: &str) -> Result<PathBuf, ToolError> {
        if text.contains("://") {
            return Err(
                ToolError::invalid(text, "is a URL, but only local paths are allowed")
                    .with_fix("download the file into an allowed folder first"),
            );
        }
        let path = Path::new(text);
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(self.get_main_folder().join(path))
        }
    }

    pub fn is_allowed(&self, path: &Path) -> bool {
        path.canonicalize()
            .is_ok_and(|full_path| self.check_allowed(&full_path, "").is_ok())
    }

    fn check_allowed(&self, full_path: &Path, text: &str) -> Result<(), ToolError> {
        if self
            .folders
            .iter()
            .any(|folder| full_path.starts_with(folder))
        {
            Ok(())
        } else {
            Err(ToolError::folder_not_allowed(text))
        }
    }
}

pub fn is_same_file(first: &Path, second: &Path) -> bool {
    if first == second {
        return true;
    }
    let same_folder = first.parent() == second.parent();
    let same_name = match (first.file_name(), second.file_name()) {
        (Some(first), Some(second)) => first.eq_ignore_ascii_case(second),
        _ => false,
    };
    same_folder && same_name
}
