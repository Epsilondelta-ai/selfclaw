use std::path::Path;

use crate::store::{FileMemoryStore, MemoryError, MemoryStore};

const INDEX_PATH: &str = "meta/memory_index.md";

pub struct MemoryIndex<'a> {
    store: &'a FileMemoryStore,
}

impl<'a> MemoryIndex<'a> {
    pub fn new(store: &'a FileMemoryStore) -> Self {
        Self { store }
    }

    /// Scan all .md files under the memory root and regenerate the index file.
    pub fn rebuild_index(&self) -> Result<String, MemoryError> {
        let mut sections: Vec<(String, Vec<String>)> = Vec::new();

        let top_dirs = self.store.list("")?;
        for dir_name in &top_dirs {
            // Skip files at root level, only process directories
            let dir_path = Path::new(dir_name);
            if !self.store.root().join(dir_path).is_dir() {
                continue;
            }

            let mut files = Vec::new();
            self.collect_md_files(dir_name, &mut files)?;

            if !files.is_empty() {
                sections.push((dir_name.clone(), files));
            }
        }

        let mut content = String::from("# Memory Index\n\nAuto-generated index of all memory files.\n");

        for (section, files) in &sections {
            content.push_str(&format!(
                "\n## {}\n",
                capitalize_first(section)
            ));
            for file in files {
                content.push_str(&format!("- `{}`\n", file));
            }
        }

        self.store.write(INDEX_PATH, &content)?;
        Ok(content)
    }

    /// Read the current index content.
    pub fn read_index(&self) -> Result<String, MemoryError> {
        self.store.read(INDEX_PATH)
    }

    fn collect_md_files(&self, dir: &str, out: &mut Vec<String>) -> Result<(), MemoryError> {
        let entries = self.store.list(dir)?;
        for entry in entries {
            let path = format!("{}/{}", dir, entry);
            let full = self.store.root().join(&path);
            if full.is_dir() {
                self.collect_md_files(&path, out)?;
            } else if entry.ends_with(".md") {
                out.push(path);
            }
        }
        Ok(())
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_with_files() -> (TempDir, FileMemoryStore) {
        let dir = TempDir::new().unwrap();
        let store = FileMemoryStore::new(dir.path());

        // Create the standard memory structure
        store
            .write("identity/purpose_journal.md", "# Purpose Journal")
            .unwrap();
        store
            .write("identity/values.md", "# Values")
            .unwrap();
        store
            .write("episodic/2026-03-01.md", "# Day log")
            .unwrap();
        store
            .write("meta/reflection_prompts.md", "# Prompts")
            .unwrap();

        (dir, store)
    }

    #[test]
    fn test_rebuild_index_creates_file() {
        let (_dir, store) = setup_with_files();
        let index = MemoryIndex::new(&store);

        let content = index.rebuild_index().unwrap();

        assert!(content.contains("# Memory Index"));
        assert!(store.exists("meta/memory_index.md"));
    }

    #[test]
    fn test_rebuild_index_lists_all_md_files() {
        let (_dir, store) = setup_with_files();
        let index = MemoryIndex::new(&store);

        let content = index.rebuild_index().unwrap();

        assert!(content.contains("identity/purpose_journal.md"), "content: {}", content);
        assert!(content.contains("identity/values.md"), "content: {}", content);
        assert!(content.contains("episodic/2026-03-01.md"), "content: {}", content);
        assert!(content.contains("meta/reflection_prompts.md"), "content: {}", content);
    }

    #[test]
    fn test_rebuild_index_has_section_headers() {
        let (_dir, store) = setup_with_files();
        let index = MemoryIndex::new(&store);

        let content = index.rebuild_index().unwrap();

        assert!(content.contains("## Identity"), "content: {}", content);
        assert!(content.contains("## Episodic"), "content: {}", content);
        assert!(content.contains("## Meta"), "content: {}", content);
    }

    #[test]
    fn test_read_index_after_rebuild() {
        let (_dir, store) = setup_with_files();
        let index = MemoryIndex::new(&store);

        let written = index.rebuild_index().unwrap();
        let read = index.read_index().unwrap();

        assert_eq!(written, read);
    }

    #[test]
    fn test_rebuild_index_empty_store() {
        let dir = TempDir::new().unwrap();
        // Create just the meta directory so the index can be written
        std::fs::create_dir(dir.path().join("meta")).unwrap();
        let store = FileMemoryStore::new(dir.path());
        let index = MemoryIndex::new(&store);

        let content = index.rebuild_index().unwrap();

        assert!(content.contains("# Memory Index"));
        // Only meta section (with the newly created index itself)
    }

    #[test]
    fn test_rebuild_index_ignores_non_md_files() {
        let dir = TempDir::new().unwrap();
        let store = FileMemoryStore::new(dir.path());
        store.write("data/notes.md", "notes").unwrap();
        store.write("data/image.png", "binary").unwrap();
        std::fs::create_dir_all(dir.path().join("meta")).unwrap();

        let index = MemoryIndex::new(&store);
        let content = index.rebuild_index().unwrap();

        assert!(content.contains("data/notes.md"));
        assert!(!content.contains("image.png"));
    }
}
