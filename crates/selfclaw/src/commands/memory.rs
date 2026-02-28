use std::path::Path;

use selfclaw_memory::store::{FileMemoryStore, MemoryStore};

/// View a memory file.
pub fn execute(memory_dir: &str, path: &str) -> anyhow::Result<()> {
    let store = FileMemoryStore::new(Path::new(memory_dir));

    // If path looks like a directory (no extension, or ends with /), list files
    if path.ends_with('/') || (!path.contains('.') && store.exists(path) == false) {
        let dir = if path.ends_with('/') {
            &path[..path.len() - 1]
        } else {
            path
        };

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
                for dir in &["identity", "episodic", "semantic", "relational", "operational", "meta"] {
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
