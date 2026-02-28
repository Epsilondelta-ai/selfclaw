use std::process::Command;

use crate::{Tool, ToolError, ToolOutput};

/// Dangerous command patterns that are always rejected.
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "mkfs",
    "dd if=",
    "> /dev/sd",
    "chmod -R 777 /",
    ":(){ :|:& };:",
];

pub struct ShellExecTool {
    sandbox_enabled: bool,
    allowed_directories: Vec<String>,
}

impl ShellExecTool {
    pub fn new(sandbox_enabled: bool, allowed_directories: Vec<String>) -> Self {
        Self {
            sandbox_enabled,
            allowed_directories,
        }
    }

    pub fn from_config(config: &selfclaw_config::SafetyConfig) -> Self {
        Self::new(
            config.sandbox_shell,
            config.allowed_directories.clone(),
        )
    }

    fn check_safety(&self, command: &str) -> Result<(), ToolError> {
        // Always reject dangerous patterns regardless of sandbox mode
        let normalized = command.replace("  ", " ");
        for pattern in DANGEROUS_PATTERNS {
            if normalized.contains(pattern) {
                return Err(ToolError::Safety(format!(
                    "command contains dangerous pattern: {}",
                    pattern
                )));
            }
        }

        if self.sandbox_enabled {
            // In sandbox mode, reject commands that try to access paths outside allowed dirs
            // This is a basic check; a real implementation would use a proper sandbox
            let suspicious_prefixes = ["/etc", "/usr", "/bin", "/sbin", "/var", "/root", "/home"];
            for prefix in suspicious_prefixes {
                // Check if the command references these paths (write operations)
                if (command.contains(&format!(">{}", prefix))
                    || command.contains(&format!("> {}", prefix))
                    || command.contains(&format!("rm {}", prefix))
                    || command.contains(&format!("rm -r {}", prefix)))
                    && !self.allowed_directories.iter().any(|d| command.contains(d))
                {
                    return Err(ToolError::Safety(format!(
                        "sandbox: command accesses restricted path: {}",
                        prefix
                    )));
                }
            }
        }

        Ok(())
    }
}

impl Tool for ShellExecTool {
    fn name(&self) -> &str {
        "shell_exec"
    }

    fn description(&self) -> &str {
        "Execute a shell command (sandboxed)"
    }

    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("command".to_string()))?;

        self.check_safety(command)?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ToolOutput::ok(serde_json::json!({
            "exit_code": output.status.code().unwrap_or(-1),
            "stdout": stdout,
            "stderr": stderr,
        })))
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sandboxed_tool() -> ShellExecTool {
        ShellExecTool::new(true, vec!["./memory".to_string(), "./skills".to_string()])
    }

    fn unsandboxed_tool() -> ShellExecTool {
        ShellExecTool::new(false, vec![])
    }

    #[test]
    fn test_shell_exec_echo() {
        let tool = unsandboxed_tool();
        let result = tool
            .execute(serde_json::json!({ "command": "echo hello" }))
            .unwrap();

        assert!(result.success);
        assert_eq!(result.data["stdout"].as_str().unwrap().trim(), "hello");
        assert_eq!(result.data["exit_code"], 0);
    }

    #[test]
    fn test_shell_exec_captures_stderr() {
        let tool = unsandboxed_tool();
        let result = tool
            .execute(serde_json::json!({ "command": "echo err >&2" }))
            .unwrap();

        assert!(result.success);
        assert_eq!(result.data["stderr"].as_str().unwrap().trim(), "err");
    }

    #[test]
    fn test_shell_exec_nonzero_exit() {
        let tool = unsandboxed_tool();
        let result = tool
            .execute(serde_json::json!({ "command": "exit 42" }))
            .unwrap();

        assert!(result.success); // Tool itself succeeds, but exit code != 0
        assert_eq!(result.data["exit_code"], 42);
    }

    #[test]
    fn test_shell_exec_missing_command() {
        let tool = unsandboxed_tool();
        let result = tool.execute(serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("command"));
    }

    #[test]
    fn test_shell_rejects_rm_rf_root() {
        let tool = sandboxed_tool();
        let result = tool.execute(serde_json::json!({ "command": "rm -rf /" }));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("dangerous"), "got: {}", err);
    }

    #[test]
    fn test_shell_rejects_rm_rf_wildcard() {
        let tool = sandboxed_tool();
        let result = tool.execute(serde_json::json!({ "command": "rm -rf /*" }));
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_rejects_fork_bomb() {
        let tool = sandboxed_tool();
        let result = tool.execute(serde_json::json!({ "command": ":(){ :|:& };:" }));
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_sandbox_rejects_system_paths() {
        let tool = sandboxed_tool();
        let result = tool.execute(serde_json::json!({ "command": "rm /etc/passwd" }));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("sandbox"), "got: {}", err);
    }

    #[test]
    fn test_shell_unsandboxed_still_rejects_dangerous() {
        let tool = unsandboxed_tool();
        let result = tool.execute(serde_json::json!({ "command": "rm -rf /" }));
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_allows_safe_commands_in_sandbox() {
        let tool = sandboxed_tool();
        let result = tool
            .execute(serde_json::json!({ "command": "echo safe" }))
            .unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_shell_name_and_description() {
        let tool = sandboxed_tool();
        assert_eq!(tool.name(), "shell_exec");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_shell_from_config() {
        let config = selfclaw_config::SafetyConfig {
            max_api_calls_per_hour: 100,
            max_file_writes_per_cycle: 10,
            sandbox_shell: true,
            allowed_directories: vec!["./test".to_string()],
        };
        let tool = ShellExecTool::from_config(&config);
        assert!(tool.sandbox_enabled);
        assert_eq!(tool.allowed_directories, vec!["./test"]);
    }
}
