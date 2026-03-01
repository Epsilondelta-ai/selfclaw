//! SelfClaw home directory resolution.
//!
//! Default: `~/.selfclaw/`
//! Override: `SELFCLAW_HOME` environment variable.

use std::path::{Path, PathBuf};

/// Returns the SelfClaw home directory.
///
/// Resolution order:
/// 1. `SELFCLAW_HOME` env var
/// 2. `~/.selfclaw/`
pub fn home_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("SELFCLAW_HOME") {
        return PathBuf::from(custom);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".selfclaw")
}

/// Returns the config file path within the home directory.
pub fn config_path() -> PathBuf {
    home_dir().join("config.toml")
}

/// Returns the memory directory within the home directory.
pub fn memory_dir() -> PathBuf {
    home_dir().join("memory")
}

/// Returns the skills directory within the home directory.
pub fn skills_dir() -> PathBuf {
    home_dir().join("skills")
}

/// Returns the output directory within the home directory.
pub fn output_dir() -> PathBuf {
    home_dir().join("output")
}

/// Returns the logs directory within the home directory.
pub fn logs_dir() -> PathBuf {
    home_dir().join("logs")
}

/// Returns the state directory (PID files, runtime state).
pub fn state_dir() -> PathBuf {
    home_dir().join("state")
}

/// Returns the PID file path for the daemon.
pub fn pid_file() -> PathBuf {
    state_dir().join("selfclaw.pid")
}

/// Returns the log file path for the daemon.
pub fn daemon_log_file() -> PathBuf {
    logs_dir().join("daemon.log")
}

/// Resolve config path: prefer explicit flag, then home dir, then project-local.
pub fn resolve_config(flag_value: &str) -> PathBuf {
    let flag_path = Path::new(flag_value);

    // If user explicitly passed a non-default path, use it.
    if flag_value != "selfclaw.toml" {
        return flag_path.to_path_buf();
    }

    // Try home dir config first.
    let home_config = config_path();
    if home_config.exists() {
        return home_config;
    }

    // Fall back to project-local.
    flag_path.to_path_buf()
}

/// Resolve memory dir: prefer explicit flag, then home dir, then project-local.
pub fn resolve_memory_dir(flag_value: &str) -> PathBuf {
    let flag_path = Path::new(flag_value);

    // If user explicitly passed a non-default path, use it.
    if flag_value != "./memory" {
        return flag_path.to_path_buf();
    }

    // Try home dir memory first.
    let home_mem = memory_dir();
    if home_mem.exists() {
        return home_mem;
    }

    // Fall back to project-local.
    flag_path.to_path_buf()
}

/// Check if SelfClaw has been initialized (home dir exists with config).
pub fn is_initialized() -> bool {
    home_dir().exists() && config_path().exists()
}

/// All subdirectories to create during init.
pub fn all_dirs() -> Vec<PathBuf> {
    let home = home_dir();
    vec![
        home.clone(),
        home.join("memory"),
        home.join("memory/identity"),
        home.join("memory/episodic"),
        home.join("memory/semantic"),
        home.join("memory/semantic/knowledge"),
        home.join("memory/semantic/skills"),
        home.join("memory/relational"),
        home.join("memory/relational/humans"),
        home.join("memory/operational"),
        home.join("memory/meta"),
        home.join("skills"),
        home.join("output"),
        home.join("logs"),
        home.join("state"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_dir_default() {
        // When SELFCLAW_HOME is not set, should end with .selfclaw
        std::env::remove_var("SELFCLAW_HOME");
        let dir = home_dir();
        assert!(dir.ends_with(".selfclaw"));
    }

    #[test]
    fn test_home_dir_env_override() {
        std::env::set_var("SELFCLAW_HOME", "/tmp/test-selfclaw-home");
        let dir = home_dir();
        assert_eq!(dir, PathBuf::from("/tmp/test-selfclaw-home"));
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_config_path() {
        std::env::set_var("SELFCLAW_HOME", "/tmp/test-sc");
        assert_eq!(config_path(), PathBuf::from("/tmp/test-sc/config.toml"));
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_all_dirs_count() {
        let dirs = all_dirs();
        assert!(dirs.len() >= 15);
    }

    #[test]
    fn test_resolve_config_explicit() {
        let p = resolve_config("/custom/path.toml");
        assert_eq!(p, PathBuf::from("/custom/path.toml"));
    }

    #[test]
    fn test_resolve_memory_dir_explicit() {
        let p = resolve_memory_dir("/custom/mem");
        assert_eq!(p, PathBuf::from("/custom/mem"));
    }

    #[test]
    fn test_pid_file_path() {
        std::env::set_var("SELFCLAW_HOME", "/tmp/test-sc");
        assert_eq!(pid_file(), PathBuf::from("/tmp/test-sc/state/selfclaw.pid"));
        std::env::remove_var("SELFCLAW_HOME");
    }
}
