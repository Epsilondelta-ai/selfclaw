use std::path::{Path, PathBuf};

use thiserror::Error;

// ── Error types ──────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("path is outside memory root: {0}")]
    OutsideRoot(String),
}

// ── Trait ─────────────────────────────────────────────────────────────

pub trait MemoryStore {
    fn read(&self, path: &str) -> Result<String, MemoryError>;
    fn write(&self, path: &str, content: &str) -> Result<(), MemoryError>;
    fn append(&self, path: &str, content: &str) -> Result<(), MemoryError>;
    fn list(&self, directory: &str) -> Result<Vec<String>, MemoryError>;
    fn exists(&self, path: &str) -> bool;
}

// ── FileMemoryStore ──────────────────────────────────────────────────

pub struct FileMemoryStore {
    root: PathBuf,
}

impl FileMemoryStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn resolve(&self, path: &str) -> Result<PathBuf, MemoryError> {
        let resolved = self.root.join(path);
        // Canonicalize the root (it must exist) and check the resolved path
        // starts with it. For new files the parent must exist.
        let canon_root = self
            .root
            .canonicalize()
            .map_err(|_| MemoryError::OutsideRoot(path.to_string()))?;

        // If the file already exists we can canonicalize it directly.
        if resolved.exists() {
            let canon = resolved.canonicalize()?;
            if !canon.starts_with(&canon_root) {
                return Err(MemoryError::OutsideRoot(path.to_string()));
            }
            return Ok(canon);
        }

        // For new files, canonicalize the parent.
        if let Some(parent) = resolved.parent() {
            if parent.exists() {
                let canon_parent = parent.canonicalize()?;
                if !canon_parent.starts_with(&canon_root) {
                    return Err(MemoryError::OutsideRoot(path.to_string()));
                }
                return Ok(resolved);
            }
        }

        // Parent doesn't exist yet — that's fine for write which creates dirs.
        Ok(resolved)
    }
}

impl MemoryStore for FileMemoryStore {
    fn read(&self, path: &str) -> Result<String, MemoryError> {
        let full = self.resolve(path)?;
        Ok(std::fs::read_to_string(full)?)
    }

    fn write(&self, path: &str, content: &str) -> Result<(), MemoryError> {
        let full = self.resolve(path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full, content)?;
        Ok(())
    }

    fn append(&self, path: &str, content: &str) -> Result<(), MemoryError> {
        let full = self.resolve(path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(full)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn list(&self, directory: &str) -> Result<Vec<String>, MemoryError> {
        let full = self.resolve(directory)?;
        if !full.is_dir() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(full)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            entries.push(name);
        }
        entries.sort();
        Ok(entries)
    }

    fn exists(&self, path: &str) -> bool {
        match self.resolve(path) {
            Ok(full) => full.exists(),
            Err(_) => false,
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, FileMemoryStore) {
        let dir = TempDir::new().unwrap();
        let store = FileMemoryStore::new(dir.path());
        (dir, store)
    }

    #[test]
    fn test_write_and_read() {
        let (_dir, store) = setup();
        store.write("test.md", "hello world").unwrap();
        let content = store.read("test.md").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_write_creates_subdirectories() {
        let (_dir, store) = setup();
        store
            .write("identity/purpose_journal.md", "# Purpose")
            .unwrap();
        let content = store.read("identity/purpose_journal.md").unwrap();
        assert_eq!(content, "# Purpose");
    }

    #[test]
    fn test_append() {
        let (_dir, store) = setup();
        store.write("log.md", "line1\n").unwrap();
        store.append("log.md", "line2\n").unwrap();
        let content = store.read("log.md").unwrap();
        assert_eq!(content, "line1\nline2\n");
    }

    #[test]
    fn test_append_creates_new_file() {
        let (_dir, store) = setup();
        store.append("new.md", "first\n").unwrap();
        let content = store.read("new.md").unwrap();
        assert_eq!(content, "first\n");
    }

    #[test]
    fn test_exists_true() {
        let (_dir, store) = setup();
        store.write("exists.md", "yes").unwrap();
        assert!(store.exists("exists.md"));
    }

    #[test]
    fn test_exists_false() {
        let (_dir, store) = setup();
        assert!(!store.exists("nope.md"));
    }

    #[test]
    fn test_list_directory() {
        let (_dir, store) = setup();
        store.write("sub/a.md", "a").unwrap();
        store.write("sub/b.md", "b").unwrap();
        store.write("sub/c.txt", "c").unwrap();
        let entries = store.list("sub").unwrap();
        assert_eq!(entries, vec!["a.md", "b.md", "c.txt"]);
    }

    #[test]
    fn test_list_empty_directory() {
        let (dir, store) = setup();
        std::fs::create_dir(dir.path().join("empty")).unwrap();
        let entries = store.list("empty").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_list_nonexistent_directory() {
        let (_dir, store) = setup();
        let entries = store.list("nonexistent").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_read_nonexistent_file_errors() {
        let (_dir, store) = setup();
        let result = store.read("missing.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_overwrite() {
        let (_dir, store) = setup();
        store.write("file.md", "original").unwrap();
        store.write("file.md", "updated").unwrap();
        assert_eq!(store.read("file.md").unwrap(), "updated");
    }
}
