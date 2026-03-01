//! `selfclaw doctor` — Diagnose installation health.
//!
//! Checks:
//! - Home directory exists and is initialized
//! - Config file is valid
//! - LLM API key is configured
//! - Memory directory structure is intact
//! - Identity files exist
//! - Daemon status
//! - Skills directory

use crate::home;
use std::path::Path;

struct Check {
    name: &'static str,
    passed: bool,
    detail: String,
}

pub fn execute() -> anyhow::Result<()> {
    let mut checks: Vec<Check> = Vec::new();

    // 1. Home directory
    let home = home::home_dir();
    checks.push(Check {
        name: "Home directory",
        passed: home.exists(),
        detail: if home.exists() {
            format!("{}", home.display())
        } else {
            format!("{} (not found — run `selfclaw init`)", home.display())
        },
    });

    // 2. Config file
    let config_path = home::config_path();
    let config_ok = config_path.exists();
    let config_valid = if config_ok {
        selfclaw_config::SelfClawConfig::load_or_default(&config_path).is_ok()
    } else {
        false
    };
    checks.push(Check {
        name: "Config file",
        passed: config_ok && config_valid,
        detail: if config_ok && config_valid {
            format!("{} (valid)", config_path.display())
        } else if config_ok {
            format!("{} (exists but invalid)", config_path.display())
        } else {
            format!("{} (not found — run `selfclaw onboard`)", config_path.display())
        },
    });

    // 3. LLM API key
    let (api_key_ok, api_key_detail) = check_api_key(&config_path);
    checks.push(Check {
        name: "LLM API key",
        passed: api_key_ok,
        detail: api_key_detail,
    });

    // 4. Memory directory
    let mem_dir = home::memory_dir();
    checks.push(Check {
        name: "Memory directory",
        passed: mem_dir.exists(),
        detail: if mem_dir.exists() {
            format!("{}", mem_dir.display())
        } else {
            format!("{} (not found)", mem_dir.display())
        },
    });

    // 5. Identity files
    let identity_files = [
        "memory/identity/self_model.md",
        "memory/identity/values.md",
        "memory/identity/purpose_journal.md",
    ];
    let identity_count = identity_files
        .iter()
        .filter(|f| home.join(f).exists())
        .count();
    checks.push(Check {
        name: "Identity files",
        passed: identity_count == identity_files.len(),
        detail: format!("{}/{} present", identity_count, identity_files.len()),
    });

    // 6. Meta files
    let meta_ok = home.join("memory/meta/memory_index.md").exists();
    checks.push(Check {
        name: "Memory index",
        passed: meta_ok,
        detail: if meta_ok {
            "present".to_string()
        } else {
            "missing (run `selfclaw init`)".to_string()
        },
    });

    // 7. Skills directory
    let skills_dir = home::skills_dir();
    checks.push(Check {
        name: "Skills directory",
        passed: skills_dir.exists(),
        detail: if skills_dir.exists() {
            let count = std::fs::read_dir(&skills_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "md")
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0);
            format!("{} ({} skills loaded)", skills_dir.display(), count)
        } else {
            format!("{} (not found)", skills_dir.display())
        },
    });

    // 8. Daemon status
    let pid_file = home::pid_file();
    let daemon_running = if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(_pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    std::process::Command::new("kill")
                        .args(["-0", pid_str.trim()])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                }
                #[cfg(not(unix))]
                {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    checks.push(Check {
        name: "Daemon",
        passed: daemon_running,
        detail: if daemon_running {
            let pid = std::fs::read_to_string(&pid_file)
                .unwrap_or_default()
                .trim()
                .to_string();
            format!("running (PID: {})", pid)
        } else {
            "not running".to_string()
        },
    });

    // Print results
    println!();
    let mut all_passed = true;
    for check in &checks {
        let color_icon = if check.passed { "  [OK]" } else { "  [!!]" };
        println!("{}  {}: {}", color_icon, check.name, check.detail);
        if !check.passed {
            all_passed = false;
        }
    }
    let _ = all_passed; // suppress unused warning; used for future exit code

    println!();
    let passed = checks.iter().filter(|c| c.passed).count();
    let total = checks.len();
    println!("  {}/{} checks passed.", passed, total);

    if !all_passed {
        println!("\n  Run `selfclaw init` to create missing directories.");
        println!("  Run `selfclaw onboard` for guided setup.");
    }

    Ok(())
}

fn check_api_key(config_path: &Path) -> (bool, String) {
    // Try loading config to get provider.
    let config = match selfclaw_config::SelfClawConfig::load_or_default(config_path) {
        Ok(c) => c,
        Err(_) => return (false, "cannot read config".to_string()),
    };

    let provider = &config.llm.provider;

    // Check config-level api_key.
    if let Some(ref key) = config.llm.api_key {
        if !key.is_empty() {
            return (true, format!("configured in config.toml ({})", provider));
        }
    }

    // Ollama doesn't need a key.
    if provider == "ollama" {
        return (true, "not required (ollama)".to_string());
    }

    // Check environment variable.
    let env_var = match provider.as_str() {
        "anthropic" => "ANTHROPIC_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "google" => "GOOGLE_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "groq" => "GROQ_API_KEY",
        "xai" => "XAI_API_KEY",
        "mistral" => "MISTRAL_API_KEY",
        "deepseek" => "DEEPSEEK_API_KEY",
        "together" => "TOGETHER_API_KEY",
        "moonshot" => "MOONSHOT_API_KEY",
        "bedrock" => "AWS_ACCESS_KEY_ID",
        _ => "API_KEY",
    };

    if std::env::var(env_var).is_ok() {
        return (true, format!("found in ${}", env_var));
    }

    (
        false,
        format!(
            "not found — set ${} or add api_key to config.toml",
            env_var
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_doctor_runs_without_panic() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SELFCLAW_HOME", tmp.path().join(".selfclaw"));
        // Should not panic even when nothing is initialized.
        let _ = execute();
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_doctor_after_init() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SELFCLAW_HOME", tmp.path().join(".selfclaw"));
        super::super::init::execute(false).unwrap();
        let result = execute();
        assert!(result.is_ok());
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_check_api_key_ollama() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(
            &config_path,
            "[llm]\nprovider = \"ollama\"\nmodel = \"llama4\"\n",
        )
        .unwrap();
        let (ok, detail) = check_api_key(&config_path);
        assert!(ok);
        assert!(detail.contains("ollama"));
    }
}
