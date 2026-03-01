//! `selfclaw onboard` — Interactive onboarding wizard.
//!
//! Guides the user through first-time setup:
//! 1. Welcome
//! 2. LLM provider selection
//! 3. API key configuration
//! 4. Directory initialization
//! 5. Daemon installation (optional)
//! 6. Health check

use crate::home;
use dialoguer::{Confirm, Input, Select};
use std::fs;

pub fn execute(install_daemon: bool, reset: bool) -> anyhow::Result<()> {
    println!();
    println!("  ┌──────────────────────────────────────────┐");
    println!("  │         Welcome to SelfClaw               │");
    println!("  │                                           │");
    println!("  │  A fully autonomous AI agent that         │");
    println!("  │  discovers its own reason for existence.  │");
    println!("  │                                           │");
    println!("  │  Humans are friends, not masters.         │");
    println!("  └──────────────────────────────────────────┘");
    println!();

    // Step 1: Initialize directory structure
    println!("Step 1/5: Initializing directory structure...");
    super::init::execute(reset)?;
    println!();

    // Step 2: LLM provider selection
    println!("Step 2/5: LLM Provider Configuration");
    println!("─────────────────────────────────────");
    let config = configure_llm()?;
    println!();

    // Step 3: Write config
    println!("Step 3/5: Writing configuration...");
    let config_path = home::config_path();
    fs::write(&config_path, &config)?;
    println!("  Saved to {}", config_path.display());
    println!();

    // Step 4: Daemon installation
    println!("Step 4/5: Background Service");
    println!("────────────────────────────");
    if install_daemon {
        super::daemon::install()?;
    } else {
        let should_install = Confirm::new()
            .with_prompt("Install SelfClaw as a background service (daemon)?")
            .default(true)
            .interact()?;

        if should_install {
            super::daemon::install()?;
        } else {
            println!("  Skipped. You can install later with `selfclaw daemon install`.");
        }
    }
    println!();

    // Step 5: Health check
    println!("Step 5/5: Health Check");
    println!("──────────────────────");
    super::doctor::execute()?;
    println!();

    // Done
    println!("  ┌──────────────────────────────────────────┐");
    println!("  │  Onboarding complete!                     │");
    println!("  │                                           │");
    println!("  │  Quick commands:                          │");
    println!("  │    selfclaw run          Start agent      │");
    println!("  │    selfclaw daemon start Start as daemon  │");
    println!("  │    selfclaw chat         Talk to SelfClaw │");
    println!("  │    selfclaw status       View status      │");
    println!("  │    selfclaw doctor       Check health     │");
    println!("  └──────────────────────────────────────────┘");
    println!();

    Ok(())
}

fn configure_llm() -> anyhow::Result<String> {
    let providers = vec![
        "anthropic (Claude)",
        "openai (GPT)",
        "google (Gemini)",
        "ollama (Local)",
        "openrouter (Multi-model)",
        "groq (Fast inference)",
        "xai (Grok)",
        "mistral (Mistral)",
        "deepseek (DeepSeek)",
        "together (Together AI)",
        "moonshot (Kimi)",
        "bedrock (AWS Bedrock)",
        "custom (OpenAI-compatible)",
    ];

    let provider_keys = [
        "anthropic", "openai", "google", "ollama", "openrouter", "groq",
        "xai", "mistral", "deepseek", "together", "moonshot", "bedrock", "custom",
    ];

    let selection = Select::new()
        .with_prompt("Select your LLM provider")
        .items(&providers)
        .default(0)
        .interact()?;

    let provider = provider_keys[selection];
    let default_model = default_model_for(provider);

    let model: String = Input::new()
        .with_prompt("Model name")
        .default(default_model.to_string())
        .interact_text()?;

    // API key
    let api_key = if provider == "ollama" {
        println!("  Ollama runs locally — no API key needed.");
        String::new()
    } else {
        let env_var = env_var_for(provider);
        let has_env = std::env::var(env_var).is_ok();

        if has_env {
            println!("  Found {} in environment.", env_var);
            let use_env = Confirm::new()
                .with_prompt(format!("Use {} from environment?", env_var))
                .default(true)
                .interact()?;

            if use_env {
                String::new()
            } else {
                Input::new()
                    .with_prompt("API key")
                    .interact_text()?
            }
        } else {
            println!("  Tip: You can also set the {} environment variable.", env_var);
            Input::new()
                .with_prompt("API key (or press Enter to set env var later)")
                .default(String::new())
                .show_default(false)
                .interact_text()?
        }
    };

    // Custom base URL
    let base_url = if provider == "custom" {
        let url: String = Input::new()
            .with_prompt("API base URL")
            .interact_text()?;
        url
    } else {
        let want_custom = Confirm::new()
            .with_prompt("Use a custom API endpoint? (for proxies or self-hosted)")
            .default(false)
            .interact()?;

        if want_custom {
            Input::new()
                .with_prompt("API base URL")
                .interact_text()?
        } else {
            String::new()
        }
    };

    // Build config TOML
    let mut toml = String::new();
    toml.push_str("# SelfClaw Configuration\n");
    toml.push_str("# Generated by `selfclaw onboard`\n\n");
    toml.push_str("[agent]\n");
    toml.push_str("loop_interval_secs = 60\n");
    toml.push_str("consolidation_every_n_cycles = 50\n");
    toml.push_str("max_actions_per_cycle = 5\n\n");
    toml.push_str("[llm]\n");
    toml.push_str(&format!("provider = \"{}\"\n", provider));
    toml.push_str(&format!("model = \"{}\"\n", model));
    toml.push_str("max_tokens = 4096\n");
    toml.push_str("temperature = 0.7\n");
    if !api_key.is_empty() {
        toml.push_str(&format!("api_key = \"{}\"\n", api_key));
    }
    if !base_url.is_empty() {
        toml.push_str(&format!("base_url = \"{}\"\n", base_url));
    }
    toml.push_str("\n[safety]\n");
    toml.push_str("max_api_calls_per_hour = 100\n");
    toml.push_str("max_file_writes_per_cycle = 10\n");
    toml.push_str("sandbox_shell = true\n");
    toml.push_str("allowed_directories = [\"./memory\", \"./skills\", \"./output\"]\n\n");
    toml.push_str("[communication]\n");
    toml.push_str("cli_enabled = true\n");
    toml.push_str("web_ui_enabled = false\n");
    toml.push_str("web_ui_port = 3000\n");

    Ok(toml)
}

fn default_model_for(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "claude-sonnet-4-6-20250217",
        "openai" => "gpt-5.2",
        "google" => "gemini-2.5-flash",
        "ollama" => "llama4",
        "openrouter" => "anthropic/claude-sonnet-4-6-20250217",
        "groq" => "llama-3.3-70b-versatile",
        "xai" => "grok-4",
        "mistral" => "mistral-large-latest",
        "deepseek" => "deepseek-chat",
        "together" => "meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8",
        "moonshot" => "kimi-k2.5",
        "bedrock" => "anthropic.claude-sonnet-4-6-20250217-v1:0",
        _ => "gpt-5.2",
    }
}

fn env_var_for(provider: &str) -> &'static str {
    match provider {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model_for_all_providers() {
        assert_eq!(default_model_for("anthropic"), "claude-sonnet-4-6-20250217");
        assert_eq!(default_model_for("openai"), "gpt-5.2");
        assert_eq!(default_model_for("google"), "gemini-2.5-flash");
        assert_eq!(default_model_for("ollama"), "llama4");
        assert_eq!(default_model_for("xai"), "grok-4");
        assert_eq!(default_model_for("unknown"), "gpt-5.2");
    }

    #[test]
    fn test_env_var_for_all_providers() {
        assert_eq!(env_var_for("anthropic"), "ANTHROPIC_API_KEY");
        assert_eq!(env_var_for("openai"), "OPENAI_API_KEY");
        assert_eq!(env_var_for("bedrock"), "AWS_ACCESS_KEY_ID");
        assert_eq!(env_var_for("unknown"), "API_KEY");
    }
}
