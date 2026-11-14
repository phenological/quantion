use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::error::ToolError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileKey {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

impl FileKey {
    pub fn read(path: &Path) -> Result<Self, ToolError> {
        let metadata = path
            .metadata()
            .map_err(|_| ToolError::file_not_found(&path.display().to_string()))?;
        Ok(Self {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

struct CacheEntry<T> {
    key: FileKey,
    value: T,
    memory: u64,
}

pub struct RecentCache<T> {
    entries: Vec<CacheEntry<T>>,
    max_entries: usize,
    memory_limit: u64,
}

impl<T> RecentCache<T> {
    pub fn new(max_entries: usize, memory_limit: u64) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
            memory_limit,
        }
    }

    pub fn memory_limit(&self) -> u64 {
        self.memory_limit
    }

    pub fn take(&mut self, key: &FileKey) -> Option<(T, u64)> {
        let position = self
            .entries
            .iter()
            .position(|entry| entry.key.path == key.path)?;
        let entry = self.entries.remove(position);
        (entry.key == *key).then_some((entry.value, entry.memory))
    }

    pub fn put(&mut self, key: FileKey, value: T, memory: u64) -> &mut T {
        self.remove(&key.path);
        self.free_memory_for(memory);
        self.entries.push(CacheEntry { key, value, memory });
        let last = self.entries.len() - 1;
        &mut self.entries[last].value
    }

    pub fn free_memory_for(&mut self, memory: u64) {
        while !self.entries.is_empty()
            && (self.entries.len() >= self.max_entries
                || self.get_used_memory().saturating_add(memory) > self.memory_limit)
        {
            self.entries.remove(0);
        }
    }

    pub fn remove(&mut self, path: &Path) {
        self.entries.retain(|entry| entry.key.path != path);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn get_used_memory(&self) -> u64 {
        self.entries.iter().map(|entry| entry.memory).sum()
    }
}
