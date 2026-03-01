use std::path::PathBuf;

use crate::{Tool, ToolError, ToolOutput};

/// Resolve `path` relative to `root` and verify the result stays within `root`.
///
/// For existing paths, canonicalization catches symlink escapes.
/// For non-existing paths, we normalize away `..` components manually so that
/// e.g. `../../etc/passwd` is rejected even before the file is created.
fn resolve_safe(root: &std::path::Path, path: &str) -> Result<PathBuf, ToolError> {
    let joined = root.join(path);

    if let Ok(canon_root) = root.canonicalize() {
        // For existing paths, canonicalize and compare.
        if joined.exists() {
            let canon = joined.canonicalize().map_err(ToolError::Io)?;
            if !canon.starts_with(&canon_root) {
                return Err(ToolError::Safety(format!(
                    "path escapes root: {}",
                    path
                )));
            }
            return Ok(canon);
        }

        // For non-existing paths, canonicalize the longest existing ancestor
        // and then re-attach the remaining components to check containment.
        let mut existing = joined.as_path();
        let mut tail_parts: Vec<&std::ffi::OsStr> = Vec::new();
        loop {
            if existing.exists() {
                break;
            }
            match existing.parent() {
                Some(parent) => {
                    if let Some(comp) = existing.file_name() {
                        tail_parts.push(comp);
                    }
                    existing = parent;
                }
                None => break,
            }
        }
        let canon_existing = existing.canonicalize().map_err(ToolError::Io)?;
        let mut rebuilt = canon_existing;
        for comp in tail_parts.into_iter().rev() {
            // Reject ".." in the non-existing tail
            if comp == ".." {
                return Err(ToolError::Safety(format!(
                    "path escapes root: {}",
                    path
                )));
            }
            rebuilt.push(comp);
        }
        if !rebuilt.starts_with(&canon_root) {
            return Err(ToolError::Safety(format!(
                "path escapes root: {}",
                path
            )));
        }
        return Ok(rebuilt);
    }

    Ok(joined)
}

// ── FileReadTool ─────────────────────────────────────────────────────

pub struct FileReadTool {
    root: PathBuf,
}

impl FileReadTool {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read file contents from the filesystem"
    }

    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("path".to_string()))?;

        let full = resolve_safe(&self.root, path)?;
        let content = std::fs::read_to_string(&full)?;
        Ok(ToolOutput::ok(serde_json::json!({ "content": content })))
    }
}

// ── FileWriteTool ────────────────────────────────────────────────────

pub struct FileWriteTool {
    root: PathBuf,
}

impl FileWriteTool {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Create or overwrite a file"
    }

    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("path".to_string()))?;
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("content".to_string()))?;

        let full = resolve_safe(&self.root, path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full, content)?;
        Ok(ToolOutput::ok(
            serde_json::json!({ "written": path }),
        ))
    }
}

// ── FileAppendTool ───────────────────────────────────────────────────

pub struct FileAppendTool {
    root: PathBuf,
}

impl FileAppendTool {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Tool for FileAppendTool {
    fn name(&self) -> &str {
        "file_append"
    }

    fn description(&self) -> &str {
        "Append content to an existing file"
    }

    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("path".to_string()))?;
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("content".to_string()))?;

        let full = resolve_safe(&self.root, path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full)?;
        file.write_all(content.as_bytes())?;

        Ok(ToolOutput::ok(
            serde_json::json!({ "appended": path }),
        ))
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_read_happy_path() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

        let tool = FileReadTool::new(dir.path());
        let result = tool
            .execute(serde_json::json!({ "path": "test.txt" }))
            .unwrap();

        assert!(result.success);
        assert_eq!(result.data["content"], "hello");
    }

    #[test]
    fn test_file_read_missing_path_field() {
        let dir = TempDir::new().unwrap();
        let tool = FileReadTool::new(dir.path());
        let result = tool.execute(serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path"));
    }

    #[test]
    fn test_file_read_nonexistent() {
        let dir = TempDir::new().unwrap();
        let tool = FileReadTool::new(dir.path());
        let result = tool.execute(serde_json::json!({ "path": "nope.txt" }));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_write_happy_path() {
        let dir = TempDir::new().unwrap();
        let tool = FileWriteTool::new(dir.path());

        let result = tool
            .execute(serde_json::json!({ "path": "out.txt", "content": "world" }))
            .unwrap();

        assert!(result.success);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("out.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    fn test_file_write_creates_directories() {
        let dir = TempDir::new().unwrap();
        let tool = FileWriteTool::new(dir.path());

        tool.execute(serde_json::json!({ "path": "sub/dir/file.txt", "content": "nested" }))
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("sub/dir/file.txt")).unwrap(),
            "nested"
        );
    }

    #[test]
    fn test_file_write_missing_content() {
        let dir = TempDir::new().unwrap();
        let tool = FileWriteTool::new(dir.path());
        let result = tool.execute(serde_json::json!({ "path": "f.txt" }));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_write_overwrite() {
        let dir = TempDir::new().unwrap();
        let tool = FileWriteTool::new(dir.path());

        tool.execute(serde_json::json!({ "path": "f.txt", "content": "v1" }))
            .unwrap();
        tool.execute(serde_json::json!({ "path": "f.txt", "content": "v2" }))
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "v2"
        );
    }

    #[test]
    fn test_file_append_happy_path() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("log.txt"), "a").unwrap();

        let tool = FileAppendTool::new(dir.path());
        tool.execute(serde_json::json!({ "path": "log.txt", "content": "b" }))
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("log.txt")).unwrap(),
            "ab"
        );
    }

    #[test]
    fn test_file_append_creates_new() {
        let dir = TempDir::new().unwrap();
        let tool = FileAppendTool::new(dir.path());

        tool.execute(serde_json::json!({ "path": "new.txt", "content": "first" }))
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("new.txt")).unwrap(),
            "first"
        );
    }

    #[test]
    fn test_file_append_missing_field() {
        let dir = TempDir::new().unwrap();
        let tool = FileAppendTool::new(dir.path());
        let result = tool.execute(serde_json::json!({ "path": "f.txt" }));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_read_rejects_path_traversal() {
        let dir = TempDir::new().unwrap();
        // Create a file outside the root to try reading it
        let outside = dir.path().join("../outside.txt");
        std::fs::write(&outside, "secret").ok();

        let tool = FileReadTool::new(dir.path());
        let result = tool.execute(serde_json::json!({ "path": "../outside.txt" }));
        // If the outside file exists, this should be a Safety error
        if outside.exists() {
            assert!(
                matches!(result, Err(ToolError::Safety(_))),
                "expected Safety error, got: {:?}",
                result
            );
            std::fs::remove_file(outside).ok();
        }
    }

    #[test]
    fn test_file_write_rejects_path_traversal() {
        let dir = TempDir::new().unwrap();
        let tool = FileWriteTool::new(dir.path());

        let result = tool.execute(serde_json::json!({
            "path": "../../escape.txt",
            "content": "pwned"
        }));
        assert!(
            matches!(result, Err(ToolError::Safety(_))),
            "expected Safety error, got: {:?}",
            result
        );
    }

    #[test]
    fn test_file_append_rejects_path_traversal() {
        let dir = TempDir::new().unwrap();
        let tool = FileAppendTool::new(dir.path());

        let result = tool.execute(serde_json::json!({
            "path": "../../escape.txt",
            "content": "pwned"
        }));
        assert!(
            matches!(result, Err(ToolError::Safety(_))),
            "expected Safety error, got: {:?}",
            result
        );
    }

    #[test]
    fn test_file_tool_names() {
        let dir = TempDir::new().unwrap();
        assert_eq!(FileReadTool::new(dir.path()).name(), "file_read");
        assert_eq!(FileWriteTool::new(dir.path()).name(), "file_write");
        assert_eq!(FileAppendTool::new(dir.path()).name(), "file_append");
    }

    #[test]
    fn test_file_tool_descriptions() {
        let dir = TempDir::new().unwrap();
        assert!(!FileReadTool::new(dir.path()).description().is_empty());
        assert!(!FileWriteTool::new(dir.path()).description().is_empty());
        assert!(!FileAppendTool::new(dir.path()).description().is_empty());
    }
}
