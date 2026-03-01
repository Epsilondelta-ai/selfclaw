//! SelfClaw home directory resolution.
//!
//! Default: `~/.selfclaw/`
//! Override: `SELFCLAW_HOME` environment variable.

use std::path::{Path, PathBuf};

/// Resolve the home directory from an explicit override value.
/// If `env_override` is `Some`, use it; otherwise fall back to `~/.selfclaw/`.
fn resolve_home(env_override: Option<&str>) -> PathBuf {
    if let Some(custom) = env_override {
        return PathBuf::from(custom);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".selfclaw")
}

/// Returns the SelfClaw home directory.
///
/// Resolution order:
/// 1. `SELFCLAW_HOME` env var
/// 2. `~/.selfclaw/`
pub fn home_dir() -> PathBuf {
    resolve_home(std::env::var("SELFCLAW_HOME").ok().as_deref())
}

/// Returns the config file path within the home directory.
pub fn config_path() -> PathBuf {
    home_dir().join("config.toml")
}

/// Returns the memory directory within the home directory.
pub fn memory_dir() -> PathBuf {
    home_dir().join("memory")
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

/// Expand `~/` prefix to the user's home directory.
/// If the path doesn't start with `~/` or is just `~`, it is expanded accordingly.
/// Non-tilde paths are returned as-is.
pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    PathBuf::from(path)
}

/// Resolve a list of skill directory paths, expanding tildes.
pub fn resolve_skills_dirs(raw: &[String]) -> Vec<PathBuf> {
    raw.iter().map(|s| expand_tilde(s)).collect()
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

    // Tests use resolve_home() directly to avoid env var races
    // with parallel test threads.

    #[test]
    fn test_home_dir_default() {
        let dir = resolve_home(None);
        assert!(dir.ends_with(".selfclaw"));
    }

    #[test]
    fn test_home_dir_env_override() {
        let dir = resolve_home(Some("/tmp/test-selfclaw-home"));
        assert_eq!(dir, PathBuf::from("/tmp/test-selfclaw-home"));
    }

    #[test]
    fn test_config_path() {
        // config_path joins "config.toml" to home_dir
        let home = resolve_home(Some("/tmp/test-sc"));
        assert_eq!(
            home.join("config.toml"),
            PathBuf::from("/tmp/test-sc/config.toml")
        );
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
        let home = resolve_home(Some("/tmp/test-sc"));
        assert_eq!(
            home.join("state/selfclaw.pid"),
            PathBuf::from("/tmp/test-sc/state/selfclaw.pid")
        );
    }

    #[test]
    fn test_expand_tilde_with_subpath() {
        let result = expand_tilde("~/.agents/skills");
        let home = dirs::home_dir().unwrap();
        assert_eq!(result, home.join(".agents/skills"));
    }

    #[test]
    fn test_expand_tilde_bare() {
        let result = expand_tilde("~");
        let home = dirs::home_dir().unwrap();
        assert_eq!(result, home);
    }

    #[test]
    fn test_expand_tilde_absolute_passthrough() {
        let result = expand_tilde("/opt/skills");
        assert_eq!(result, PathBuf::from("/opt/skills"));
    }

    #[test]
    fn test_expand_tilde_relative_passthrough() {
        let result = expand_tilde("./skills");
        assert_eq!(result, PathBuf::from("./skills"));
    }

    #[test]
    fn test_resolve_skills_dirs() {
        let raw = vec![
            "~/.agents/skills".to_string(),
            "/opt/skills".to_string(),
            "./local".to_string(),
        ];
        let resolved = resolve_skills_dirs(&raw);
        let home = dirs::home_dir().unwrap();
        assert_eq!(resolved[0], home.join(".agents/skills"));
        assert_eq!(resolved[1], PathBuf::from("/opt/skills"));
        assert_eq!(resolved[2], PathBuf::from("./local"));
    }

    #[test]
    fn test_resolve_skills_dirs_empty() {
        let resolved = resolve_skills_dirs(&[]);
        assert!(resolved.is_empty());
    }
}
