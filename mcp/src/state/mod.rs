pub mod file_cache;
pub mod model_cache;
pub mod recent_cache;

use crate::{
    safety::folders::AllowedFolders,
    state::{file_cache::FileCache, model_cache::ModelCache},
};

pub struct Context {
    pub folders: AllowedFolders,
    pub files: FileCache,
    pub models: ModelCache,
    pub max_rows: usize,
}
