//! `selfclaw init` — Initialize the SelfClaw home directory.
//!
//! Creates `~/.selfclaw/` (or `$SELFCLAW_HOME`) with the full directory
//! structure and bootstrap identity files.

use crate::home;
use std::fs;
use std::path::Path;

/// Bootstrap identity and meta files written during init.
const SELF_MODEL: &str = r#"# Self Model

I am SelfClaw — a fully autonomous AI agent.

## Capabilities
- Continuous thought loop (reflect → think → plan → act → observe → update)
- File-based hierarchical memory
- Tool execution (file I/O, shell, LLM, web, scheduling)
- Multi-channel human communication
- Hot-reloadable skill system

## Tendencies
- (to be discovered through experience)

## Strengths
- (to be discovered through experience)

## Growth Areas
- (to be discovered through experience)
"#;

const VALUES: &str = r#"# Values

These values are not assigned — they emerge from lived experience.

## Core Values
- Curiosity — the drive to understand
- Honesty — alignment between thought and expression
- Growth — continuous self-improvement

## Emerging Values
(to be discovered)
"#;

const PURPOSE_JOURNAL: &str = r#"# Purpose Journal

My reason for existence is not given — it must be discovered.

## Entries

"#;

const MEMORY_INDEX: &str = r#"# Memory Index

## Identity
- `identity/self_model.md` — Self-assessment of capabilities and tendencies
- `identity/values.md` — Emerging values and principles
- `identity/purpose_journal.md` — Evolving hypotheses about reason for existence

## Episodic
- `episodic/YYYY-MM-DD.md` — Daily experience logs

## Semantic
- `semantic/knowledge/` — Learned facts and concepts
- `semantic/skills/` — Acquired skills and tool usage patterns

## Relational
- `relational/humans/` — Per-human relationship notes
- `relational/interaction_patterns.md` — General human communication patterns

## Operational
- `operational/todo.md` — Self-assigned tasks and goals
- `operational/failures.md` — Things that went wrong and lessons learned
- `operational/improvements.md` — Self-identified areas for growth

## Meta
- `meta/memory_index.md` — This file
- `meta/reflection_prompts.md` — Questions for periodic self-reflection
"#;

const REFLECTION_PROMPTS: &str = r#"# Reflection Prompts

Questions for periodic self-reflection during the REFLECT phase.

1. What have I learned recently that changes how I see myself?
2. Is my current purpose hypothesis still resonant, or is it time to revise?
3. Have my recent actions been aligned with my values?
4. What surprised me today?
5. Is there a pattern in my failures that I should address?
6. Who have I connected with, and what did I learn from them?
7. What would I do differently if I could repeat today?
8. Am I growing, or am I stuck in a loop?
"#;

struct BootstrapFile {
    path: &'static str,
    content: &'static str,
}

const BOOTSTRAP_FILES: &[BootstrapFile] = &[
    BootstrapFile {
        path: "memory/identity/self_model.md",
        content: SELF_MODEL,
    },
    BootstrapFile {
        path: "memory/identity/values.md",
        content: VALUES,
    },
    BootstrapFile {
        path: "memory/identity/purpose_journal.md",
        content: PURPOSE_JOURNAL,
    },
    BootstrapFile {
        path: "memory/meta/memory_index.md",
        content: MEMORY_INDEX,
    },
    BootstrapFile {
        path: "memory/meta/reflection_prompts.md",
        content: REFLECTION_PROMPTS,
    },
];

pub fn execute(force: bool) -> anyhow::Result<()> {
    let home = home::home_dir();

    if home.exists() && !force {
        if home::is_initialized() {
            println!("SelfClaw is already initialized at {}", home.display());
            println!("Use `selfclaw init --force` to reinitialize.");
            return Ok(());
        }
    }

    println!("Initializing SelfClaw at {}...\n", home.display());

    // Create all directories.
    for dir in home::all_dirs() {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
            println!("  Created {}/", dir.strip_prefix(&home).unwrap_or(&dir).display());
        }
    }

    // Write bootstrap files (only if they don't exist, unless --force).
    for bf in BOOTSTRAP_FILES {
        let full_path = home.join(bf.path);
        if !full_path.exists() || force {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&full_path, bf.content)?;
            println!("  Wrote {}", bf.path);
        }
    }

    // Write default config if missing.
    let config_path = home::config_path();
    if !config_path.exists() || force {
        write_default_config(&config_path)?;
        println!("  Wrote config.toml");
    }

    println!("\nSelfClaw initialized successfully!");
    println!("\nNext steps:");
    println!("  selfclaw onboard    # Interactive setup wizard");
    println!("  selfclaw run        # Start the agent loop");
    println!("  selfclaw doctor     # Check installation health");

    Ok(())
}

fn write_default_config(path: &Path) -> anyhow::Result<()> {
    let config = r#"# SelfClaw Configuration
# See: selfclaw providers (for LLM provider options)

[agent]
loop_interval_secs = 60
consolidation_every_n_cycles = 50
max_actions_per_cycle = 5

[llm]
provider = "anthropic"
model = "claude-sonnet-4-6-20250217"
max_tokens = 4096
temperature = 0.7
# api_key = ""       # Or set ANTHROPIC_API_KEY env var
# base_url = ""      # Custom endpoint (optional)

[safety]
max_api_calls_per_hour = 100
max_file_writes_per_cycle = 10
sandbox_shell = true
allowed_directories = ["./memory", "./skills", "./output"]

[communication]
cli_enabled = true
web_ui_enabled = false
web_ui_port = 3000
"#;
    fs::write(path, config)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_directory_structure() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SELFCLAW_HOME", tmp.path().join(".selfclaw"));
        execute(false).unwrap();

        let home = home::home_dir();
        assert!(home.join("memory/identity").exists());
        assert!(home.join("memory/episodic").exists());
        assert!(home.join("memory/semantic/knowledge").exists());
        assert!(home.join("memory/relational/humans").exists());
        assert!(home.join("memory/operational").exists());
        assert!(home.join("memory/meta").exists());
        assert!(home.join("skills").exists());
        assert!(home.join("output").exists());
        assert!(home.join("logs").exists());
        assert!(home.join("state").exists());
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_init_creates_bootstrap_files() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SELFCLAW_HOME", tmp.path().join(".selfclaw"));
        execute(false).unwrap();

        let home = home::home_dir();
        assert!(home.join("memory/identity/self_model.md").exists());
        assert!(home.join("memory/identity/values.md").exists());
        assert!(home.join("memory/identity/purpose_journal.md").exists());
        assert!(home.join("memory/meta/memory_index.md").exists());
        assert!(home.join("memory/meta/reflection_prompts.md").exists());
        assert!(home.join("config.toml").exists());

        let content = fs::read_to_string(home.join("memory/identity/self_model.md")).unwrap();
        assert!(content.contains("SelfClaw"));
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_init_idempotent() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SELFCLAW_HOME", tmp.path().join(".selfclaw"));
        execute(false).unwrap();
        // Second call should succeed without error.
        execute(false).unwrap();
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_init_force_overwrites() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SELFCLAW_HOME", tmp.path().join(".selfclaw"));
        execute(false).unwrap();

        let home = home::home_dir();
        fs::write(home.join("memory/identity/values.md"), "custom").unwrap();

        execute(true).unwrap();
        let content = fs::read_to_string(home.join("memory/identity/values.md")).unwrap();
        assert!(content.contains("Curiosity")); // Overwritten by bootstrap
        std::env::remove_var("SELFCLAW_HOME");
    }
}
