use std::path::{Path, PathBuf};

use quantion::{ParseIonOptions, SampleFile, parse_ion, parse_mzml};

use crate::{
    error::ToolError,
    state::recent_cache::{FileKey, RecentCache},
};

const MAX_OPEN_FILES: usize = 16;
pub const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileFormat {
    Ion,
    Mzml,
}

impl FileFormat {
    pub fn find(path: &Path) -> Option<FileFormat> {
        match path.extension()?.to_str()? {
            "ion" => Some(FileFormat::Ion),
            "mzML" => Some(FileFormat::Mzml),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FileFormat::Ion => "ion",
            FileFormat::Mzml => "mzML",
        }
    }
}

pub struct FileCache {
    cache: RecentCache<SampleFile>,
}

struct TakenFile {
    key: FileKey,
    file: SampleFile,
    memory: u64,
}

impl FileCache {
    pub fn new(memory_limit: u64) -> Self {
        Self {
            cache: RecentCache::new(MAX_OPEN_FILES, memory_limit),
        }
    }

    pub fn get_file(&mut self, path: &Path) -> Result<&mut SampleFile, ToolError> {
        let taken = self.take_file(path, 0)?;
        Ok(self.cache.put(taken.key, taken.file, taken.memory))
    }

    pub fn with_files<T>(
        &mut self,
        paths: &[PathBuf],
        work: impl FnOnce(&mut [SampleFile]) -> Result<T, ToolError>,
    ) -> Result<T, ToolError> {
        self.check_total_memory(paths)?;
        let mut keys = Vec::with_capacity(paths.len());
        let mut files = Vec::with_capacity(paths.len());
        let mut held = 0;
        for path in paths {
            let taken = self.take_file(path, held)?;
            held += taken.memory;
            keys.push((taken.key, taken.memory));
            files.push(taken.file);
        }
        let result = work(&mut files);
        for ((key, memory), file) in keys.into_iter().zip(files) {
            self.cache.put(key, file, memory);
        }
        result
    }

    pub fn forget(&mut self, path: &Path) {
        self.cache.remove(path);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    fn take_file(&mut self, path: &Path, held: u64) -> Result<TakenFile, ToolError> {
        let key = FileKey::read(path)?;
        if let Some((file, memory)) = self.cache.take(&key) {
            return Ok(TakenFile { key, file, memory });
        }
        let format = get_format(path)?;
        let memory = get_memory(format, &key);
        self.check_memory(&path.display().to_string(), memory)?;
        self.cache.free_memory_for(held + memory);
        let file = open_file(path, format)?;
        Ok(TakenFile { key, file, memory })
    }

    fn check_total_memory(&self, paths: &[PathBuf]) -> Result<(), ToolError> {
        let mut memory = 0;
        for path in paths {
            memory += get_memory(get_format(path)?, &FileKey::read(path)?);
        }
        self.check_memory("these files together", memory)
    }

    fn check_memory(&self, name: &str, memory: u64) -> Result<(), ToolError> {
        let limit = self.cache.memory_limit();
        if memory <= limit {
            return Ok(());
        }
        Err(ToolError::new(
            "file_too_large",
            format!(
                "{name}: {} MB are needed in memory, more than the limit of {} MB",
                memory / BYTES_PER_MEGABYTE,
                limit / BYTES_PER_MEGABYTE
            ),
        )
        .with_fix("start the server with a higher --memory-limit, then save the mzML files as .ion with write_ion, which use little memory"))
    }
}

pub fn get_format(path: &Path) -> Result<FileFormat, ToolError> {
    FileFormat::find(path).ok_or_else(|| {
        ToolError::invalid(path.display().to_string(), "must be a .ion or .mzML file")
    })
}

fn get_memory(format: FileFormat, key: &FileKey) -> u64 {
    match format {
        FileFormat::Ion => 0,
        FileFormat::Mzml => key.size(),
    }
}

fn open_file(path: &Path, format: FileFormat) -> Result<SampleFile, ToolError> {
    let file = match format {
        FileFormat::Ion => parse_ion(path, ParseIonOptions::default())?,
        FileFormat::Mzml => parse_mzml(path)?,
    };
    Ok(file)
}
