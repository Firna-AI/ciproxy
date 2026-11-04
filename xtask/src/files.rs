//! Repository discovery with generated files excluded.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{IoResultExt, Result};

#[cfg_attr(test, unimock::unimock(api = [RepositoryPathsMock, RepositoryReadMock]))]
pub(crate) trait Repository {
    fn paths(&self) -> Result<Vec<String>>;
    fn read(&self, path: &str) -> Result<String>;
}

pub(crate) struct DiskRepository {
    pub(crate) root: PathBuf,
}

impl Repository for DiskRepository {
    fn paths(&self) -> Result<Vec<String>> {
        let mut paths = vec![String::new()];
        self.collect(Path::new(""), &mut paths)?;
        paths.sort();
        Ok(paths)
    }

    fn read(&self, path: &str) -> Result<String> {
        fs::read_to_string(self.root.join(path)).internal()
    }
}

impl DiskRepository {
    fn collect(&self, relative: &Path, paths: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(self.root.join(relative)).internal()? {
            let entry = entry.internal()?;
            let kind = entry.file_type().internal()?;
            let path = relative.join(entry.file_name());
            let name = path.to_string_lossy().replace('\\', "/");
            if ignored_path(&name) {
                continue;
            }
            paths.push(name);
            if kind.is_dir() {
                self.collect(&path, paths)?;
            }
        }
        Ok(())
    }
}

/// Shared exclusions for discovery and pure check inputs.
pub(crate) fn ignored_path(path: &str) -> bool {
    path.split('/').any(|part| {
        matches!(
            part,
            ".git"
                | ".context"
                | "node_modules"
                | "target"
                | "dist"
                | "build"
                | "web-build"
                | ".expo"
                | "expo-env.d.ts"
                | ".astro"
                | "mokly-generated"
                | ".mokly-export-reservations"
        )
    })
}
