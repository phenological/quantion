use std::path::Path;

use quantion::{WarpModel, load_model, load_warp};

use crate::{
    error::ToolError,
    state::recent_cache::{FileKey, RecentCache},
};

const MAX_OPEN_MODELS: usize = 4;

pub struct ModelCache {
    cache: RecentCache<WarpModel>,
}

impl Default for ModelCache {
    fn default() -> Self {
        Self {
            cache: RecentCache::new(MAX_OPEN_MODELS, u64::MAX),
        }
    }
}

impl ModelCache {
    pub fn get_warp(&mut self, path: &Path) -> Result<&WarpModel, ToolError> {
        let key = FileKey::read(path)?;
        let warp = match self.cache.take(&key) {
            Some((warp, _)) => warp,
            None => load_warp(&load_model(path)?)?,
        };
        Ok(self.cache.put(key, warp, 0))
    }
}
