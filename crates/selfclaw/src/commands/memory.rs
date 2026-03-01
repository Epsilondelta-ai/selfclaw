use std::path::Path;

use selfclaw_memory::store::{FileMemoryStore, MemoryStore};

/// View a memory file.
pub fn execute(memory_dir: &str, path: &str) -> anyhow::Result<()> {
    // Sanitize path to prevent directory traversal.
    if path.contains("..") {
        eprintln!("  Error: path must not contain '..'");
        std::process::exit(1);
    }
    if Path::new(path).is_absolute() {
        eprintln!("  Error: path must be relative to the memory directory");
        std::process::exit(1);
    }

    let store = FileMemoryStore::new(Path::new(memory_dir));

    // If path looks like a directory (no extension, or ends with /), list files
    if path.ends_with('/') || (!path.contains('.') && !store.exists(path)) {
        let dir = path.strip_suffix('/').unwrap_or(path);

        match store.list(dir) {
            Ok(files) => {
                if files.is_empty() {
                    println!("  Directory '{}' is empty.", dir);
                } else {
                    println!("  Files in '{}':", dir);
                    for file in &files {
                        println!("    {}", file);
                    }
                }
            }
            Err(e) => {
                // Might be a file path without extension, try reading
                match store.read(path) {
                    Ok(content) => {
                        println!("{}", content);
                    }
                    Err(_) => {
                        eprintln!("  Error: could not read '{}': {}", path, e);
                        std::process::exit(1);
                    }
                }
            }
        }
    } else {
        match store.read(path) {
            Ok(content) => {
                println!("{}", content);
            }
            Err(e) => {
                eprintln!("  Error: could not read '{}': {}", path, e);
                eprintln!();
                eprintln!("  Available directories:");
                // Try to list common directories
                for dir in &[
                    "identity",
                    "episodic",
                    "semantic",
                    "relational",
                    "operational",
                    "meta",
                ] {
                    if let Ok(files) = store.list(dir) {
                        if !files.is_empty() {
                            eprintln!("    {}/", dir);
                            for file in &files {
                                eprintln!("      {}", file);
                            }
                        }
                    }
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
