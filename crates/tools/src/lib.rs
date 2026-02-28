pub mod registry;
pub mod file;
pub mod shell;
pub mod llm;
pub mod scheduler;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ── Core types ───────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("safety violation: {0}")]
    Safety(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub success: bool,
    pub data: serde_json::Value,
}

impl ToolOutput {
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: serde_json::json!({ "error": message }),
        }
    }
}

/// The core Tool trait that all tools implement.
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError>;
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_tool_output_ok() {
        let out = ToolOutput::ok(serde_json::json!("hello"));
        assert!(out.success);
        assert_eq!(out.data, serde_json::json!("hello"));
    }

    #[test]
    fn test_tool_output_error() {
        let out = ToolOutput::error("bad");
        assert!(!out.success);
        assert_eq!(out.data["error"], "bad");
    }
}
