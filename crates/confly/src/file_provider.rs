use color_eyre::Result;
use color_eyre::eyre::{OptionExt, eyre};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub enum FileProvider {
    FileSystem,
    Memory(Arc<RwLock<HashMap<PathBuf, String>>>),
}

impl FileProvider {
    pub fn read(&self, path: impl AsRef<Path>) -> Result<String> {
        let content = match self {
            FileProvider::FileSystem => std::fs::read_to_string(path)?,
            FileProvider::Memory(dir) => dir
                .read()
                .map_err(|e| eyre!("{e}"))?
                .get(&path.as_ref().to_path_buf())
                .cloned()
                .ok_or_eyre(format!("File not found in memory: {}", path.as_ref().display()))?,
        };
        Ok(content)
    }

    pub fn write(&self, path: impl AsRef<Path>, content: impl Into<String>) -> Result<()> {
        match self {
            FileProvider::FileSystem => std::fs::write(path, content.into())?,
            FileProvider::Memory(mem) => {
                mem.write()
                    .map_err(|e| eyre!("{e}"))?
                    .insert(path.as_ref().to_path_buf(), content.into());
            }
        };
        Ok(())
    }
}
