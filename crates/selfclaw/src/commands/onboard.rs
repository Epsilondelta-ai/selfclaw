//! `selfclaw onboard` — Interactive onboarding wizard.
//!
//! Two modes:
//! - **QuickStart**: Sensible defaults (Anthropic, env var for API key, skip channels, offer daemon).
//! - **Advanced**: Full wizard with provider, channels, daemon, and all options.

use crate::home;
use dialoguer::{Confirm, Input, Select};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Channel configuration collected during onboarding.
#[derive(Default)]
struct ChannelConfig {
    discord_token: String,
    discord_channels: String,
    telegram_token: String,
    telegram_chats: String,
    slack_bot_token: String,
    slack_app_token: String,
    slack_channels: String,
}

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

    // Choose setup mode.
    let modes = vec![
        "QuickStart (recommended) — sensible defaults, minimal prompts",
        "Advanced — full control over every setting",
    ];
    let mode = Select::new()
        .with_prompt("Setup mode")
        .items(&modes)
        .default(0)
        .interact()?;

    let is_quick = mode == 0;
    println!();

    // Step 1: Initialize directory structure
    let total_steps = if is_quick { 4 } else { 6 };
    println!("Step 1/{}: Initializing directory structure...", total_steps);
    super::init::execute(reset)?;
    println!();

    // Step 2: LLM provider selection
    println!("Step 2/{}: LLM Provider Configuration", total_steps);
    println!("─────────────────────────────────────");
    let (config_table, provider_name) = if is_quick {
        configure_llm_quick()?
    } else {
        configure_llm_advanced()?
    };
    println!();

    // Step 3 (Advanced only): Channel configuration
    let channels = if !is_quick {
        println!("Step 3/{}: Communication Channels", total_steps);
        println!("─────────────────────────────────");
        let ch = configure_channels()?;
        println!();
        ch
    } else {
        ChannelConfig::default()
    };

    // Build final config and write it.
    let step_write = if is_quick { 3 } else { 4 };
    println!("Step {}/{}: Writing configuration...", step_write, total_steps);
    let toml_str = build_config_toml(&config_table, &channels)?;
    let config_path = home::config_path();
    fs::write(&config_path, &toml_str)?;
    #[cfg(unix)]
    {
        fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))?;
    }
    println!("  Saved to {}", config_path.display());
    println!();

    // Daemon installation step
    let step_daemon = if is_quick { 4 } else { 5 };
    println!("Step {}/{}: Background Service", step_daemon, total_steps);
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

    // Health check (Advanced only, or when quick and space permits)
    if !is_quick {
        println!("Step {}/{}: Health Check", total_steps, total_steps);
        println!("──────────────────────");
        let _ = super::doctor::execute()?;
        println!();
    }

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

    if is_quick {
        println!("  Provider: {} ({})", provider_name, default_model_for(&provider_name));
        println!("  Run `selfclaw onboard --reset` for advanced setup anytime.");
        println!();
    }

    Ok(())
}

// ── QuickStart LLM config ────────────────────────────────────────────

/// QuickStart: auto-detect env vars, default to Anthropic, minimal prompts.
fn configure_llm_quick() -> anyhow::Result<(toml::map::Map<String, toml::Value>, String)> {
    // Try to detect an API key in the environment.
    let detected = detect_provider_from_env();

    let (provider, model) = if let Some((p, m)) = detected {
        println!("  Detected {} API key in environment.", p);
        let use_detected = Confirm::new()
            .with_prompt(format!("Use {} as your LLM provider?", p))
            .default(true)
            .interact()?;

        if use_detected {
            (p.to_string(), m.to_string())
        } else {
            ("anthropic".to_string(), default_model_for("anthropic").to_string())
        }
    } else {
        println!("  No API key detected in environment.");
        println!("  Defaulting to Anthropic (Claude). Set ANTHROPIC_API_KEY to authenticate.");
        ("anthropic".to_string(), default_model_for("anthropic").to_string())
    };

    let mut llm = toml::map::Map::new();
    llm.insert("provider".into(), toml::Value::String(provider.clone()));
    llm.insert("model".into(), toml::Value::String(model));
    llm.insert("max_tokens".into(), toml::Value::Integer(4096));
    llm.insert("temperature".into(), toml::Value::Float(0.7));

    Ok((llm, provider))
}

/// Check common env vars to auto-detect the provider.
fn detect_provider_from_env() -> Option<(&'static str, &'static str)> {
    let checks = [
        ("ANTHROPIC_API_KEY", "anthropic", "claude-sonnet-4-6-20250217"),
        ("OPENAI_API_KEY", "openai", "gpt-5.2"),
        ("GOOGLE_API_KEY", "google", "gemini-2.5-flash"),
        ("OPENROUTER_API_KEY", "openrouter", "anthropic/claude-sonnet-4-6-20250217"),
        ("GROQ_API_KEY", "groq", "llama-3.3-70b-versatile"),
        ("XAI_API_KEY", "xai", "grok-4"),
        ("MISTRAL_API_KEY", "mistral", "mistral-large-latest"),
        ("DEEPSEEK_API_KEY", "deepseek", "deepseek-chat"),
        ("TOGETHER_API_KEY", "together", "meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8"),
        ("MOONSHOT_API_KEY", "moonshot", "kimi-k2.5"),
    ];

    for (env, provider, model) in &checks {
        if std::env::var(env).is_ok() {
            return Some((provider, model));
        }
    }
    None
}

// ── Advanced LLM config ──────────────────────────────────────────────

/// Advanced: full provider selection, model, API key, custom endpoint.
fn configure_llm_advanced() -> anyhow::Result<(toml::map::Map<String, toml::Value>, String)> {
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

    let mut llm = toml::map::Map::new();
    llm.insert("provider".into(), toml::Value::String(provider.to_string()));
    llm.insert("model".into(), toml::Value::String(model));
    llm.insert("max_tokens".into(), toml::Value::Integer(4096));
    llm.insert("temperature".into(), toml::Value::Float(0.7));
    if !api_key.is_empty() {
        llm.insert("api_key".into(), toml::Value::String(api_key));
    }
    if !base_url.is_empty() {
        llm.insert("base_url".into(), toml::Value::String(base_url));
    }

    Ok((llm, provider.to_string()))
}

// ── Channel configuration ────────────────────────────────────────────

fn configure_channels() -> anyhow::Result<ChannelConfig> {
    let mut cfg = ChannelConfig::default();

    println!("  SelfClaw can communicate through multiple channels.");
    println!("  You can configure these now or add them later in config.toml.");
    println!();

    // Discord
    let discord = Confirm::new()
        .with_prompt("Enable Discord?")
        .default(false)
        .interact()?;

    if discord {
        cfg.discord_token = Input::new()
            .with_prompt("  Discord bot token")
            .interact_text()?;
        cfg.discord_channels = Input::new()
            .with_prompt("  Allowed channel IDs (comma-separated, or empty for all)")
            .default(String::new())
            .show_default(false)
            .interact_text()?;
    }

    // Telegram
    let telegram = Confirm::new()
        .with_prompt("Enable Telegram?")
        .default(false)
        .interact()?;

    if telegram {
        cfg.telegram_token = Input::new()
            .with_prompt("  Telegram bot token")
            .interact_text()?;
        cfg.telegram_chats = Input::new()
            .with_prompt("  Allowed chat IDs (comma-separated, or empty for all)")
            .default(String::new())
            .show_default(false)
            .interact_text()?;
    }

    // Slack
    let slack = Confirm::new()
        .with_prompt("Enable Slack?")
        .default(false)
        .interact()?;

    if slack {
        cfg.slack_bot_token = Input::new()
            .with_prompt("  Slack bot token (xoxb-...)")
            .interact_text()?;
        cfg.slack_app_token = Input::new()
            .with_prompt("  Slack app token (xapp-...)")
            .interact_text()?;
        cfg.slack_channels = Input::new()
            .with_prompt("  Allowed channel IDs (comma-separated, or empty for all)")
            .default(String::new())
            .show_default(false)
            .interact_text()?;
    }

    if !discord && !telegram && !slack {
        println!("  No channels configured. CLI will be the primary interface.");
        println!("  Add channels later by editing ~/.selfclaw/config.toml.");
    }

    Ok(cfg)
}

// ── Config builder ───────────────────────────────────────────────────

fn build_config_toml(
    llm: &toml::map::Map<String, toml::Value>,
    channels: &ChannelConfig,
) -> anyhow::Result<String> {
    let mut config = toml::map::Map::new();

    // [agent]
    let mut agent = toml::map::Map::new();
    agent.insert("loop_interval_secs".into(), toml::Value::Integer(60));
    agent.insert("consolidation_every_n_cycles".into(), toml::Value::Integer(50));
    agent.insert("max_actions_per_cycle".into(), toml::Value::Integer(5));
    config.insert("agent".into(), toml::Value::Table(agent));

    // [llm]
    config.insert("llm".into(), toml::Value::Table(llm.clone()));

    // [safety]
    let mut safety = toml::map::Map::new();
    safety.insert("max_api_calls_per_hour".into(), toml::Value::Integer(100));
    safety.insert("max_file_writes_per_cycle".into(), toml::Value::Integer(10));
    safety.insert("sandbox_shell".into(), toml::Value::Boolean(true));
    safety.insert("allowed_directories".into(), toml::Value::Array(vec![
        toml::Value::String("./memory".into()),
        toml::Value::String("./skills".into()),
        toml::Value::String("./output".into()),
    ]));
    config.insert("safety".into(), toml::Value::Table(safety));

    // [communication]
    let mut comm = toml::map::Map::new();
    comm.insert("cli_enabled".into(), toml::Value::Boolean(true));
    comm.insert("web_ui_enabled".into(), toml::Value::Boolean(false));
    comm.insert("web_ui_port".into(), toml::Value::Integer(3000));

    // [communication.discord]
    let mut discord = toml::map::Map::new();
    discord.insert("enabled".into(), toml::Value::Boolean(!channels.discord_token.is_empty()));
    discord.insert("bot_token".into(), toml::Value::String(channels.discord_token.clone()));
    discord.insert("allowed_channel_ids".into(), parse_id_list(&channels.discord_channels));
    comm.insert("discord".into(), toml::Value::Table(discord));

    // [communication.telegram]
    let mut telegram = toml::map::Map::new();
    telegram.insert("enabled".into(), toml::Value::Boolean(!channels.telegram_token.is_empty()));
    telegram.insert("bot_token".into(), toml::Value::String(channels.telegram_token.clone()));
    telegram.insert("allowed_chat_ids".into(), parse_id_list(&channels.telegram_chats));
    comm.insert("telegram".into(), toml::Value::Table(telegram));

    // [communication.slack]
    let mut slack = toml::map::Map::new();
    slack.insert("enabled".into(), toml::Value::Boolean(!channels.slack_bot_token.is_empty()));
    slack.insert("bot_token".into(), toml::Value::String(channels.slack_bot_token.clone()));
    slack.insert("app_token".into(), toml::Value::String(channels.slack_app_token.clone()));
    slack.insert("allowed_channel_ids".into(), parse_id_list(&channels.slack_channels));
    comm.insert("slack".into(), toml::Value::Table(slack));

    // [communication.webchat]
    let mut webchat = toml::map::Map::new();
    webchat.insert("enabled".into(), toml::Value::Boolean(false));
    webchat.insert("port".into(), toml::Value::Integer(3001));
    comm.insert("webchat".into(), toml::Value::Table(webchat));

    config.insert("communication".into(), toml::Value::Table(comm));

    let toml_str = format!(
        "# SelfClaw Configuration\n# Generated by `selfclaw onboard`\n\n{}",
        toml::to_string_pretty(&toml::Value::Table(config))?
    );

    Ok(toml_str)
}

/// Parse a comma-separated ID list into a TOML array.
fn parse_id_list(input: &str) -> toml::Value {
    let ids: Vec<toml::Value> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(toml::Value::String)
        .collect();
    toml::Value::Array(ids)
}

// ── Helpers ──────────────────────────────────────────────────────────

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

    #[test]
    fn test_parse_id_list_empty() {
        let result = parse_id_list("");
        assert_eq!(result, toml::Value::Array(vec![]));
    }

    #[test]
    fn test_parse_id_list_single() {
        let result = parse_id_list("123456");
        assert_eq!(result, toml::Value::Array(vec![
            toml::Value::String("123456".into()),
        ]));
    }

    #[test]
    fn test_parse_id_list_multiple() {
        let result = parse_id_list("111, 222, 333");
        assert_eq!(result, toml::Value::Array(vec![
            toml::Value::String("111".into()),
            toml::Value::String("222".into()),
            toml::Value::String("333".into()),
        ]));
    }

    #[test]
    fn test_build_config_toml_valid() {
        let mut llm = toml::map::Map::new();
        llm.insert("provider".into(), toml::Value::String("anthropic".into()));
        llm.insert("model".into(), toml::Value::String("claude-sonnet-4-6-20250217".into()));
        llm.insert("max_tokens".into(), toml::Value::Integer(4096));
        llm.insert("temperature".into(), toml::Value::Float(0.7));

        let channels = ChannelConfig::default();
        let result = build_config_toml(&llm, &channels).unwrap();

        // Verify it's valid TOML by parsing it.
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        let table = parsed.as_table().unwrap();
        assert!(table.contains_key("agent"));
        assert!(table.contains_key("llm"));
        assert!(table.contains_key("safety"));
        assert!(table.contains_key("communication"));

        // Verify communication has channel subsections.
        let comm = table["communication"].as_table().unwrap();
        assert!(comm.contains_key("discord"));
        assert!(comm.contains_key("telegram"));
        assert!(comm.contains_key("slack"));
        assert!(comm.contains_key("webchat"));
    }

    #[test]
    fn test_build_config_toml_with_special_chars_in_api_key() {
        let mut llm = toml::map::Map::new();
        llm.insert("provider".into(), toml::Value::String("openai".into()));
        llm.insert("model".into(), toml::Value::String("gpt-5.2".into()));
        // API key with special characters that would break naive string interpolation.
        llm.insert("api_key".into(), toml::Value::String("sk-abc\"def\\ghi\nnewline".into()));
        llm.insert("max_tokens".into(), toml::Value::Integer(4096));
        llm.insert("temperature".into(), toml::Value::Float(0.7));

        let channels = ChannelConfig::default();
        let result = build_config_toml(&llm, &channels).unwrap();

        // Must be valid TOML despite special chars.
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        let api_key = parsed["llm"]["api_key"].as_str().unwrap();
        assert!(api_key.contains("abc"));
        assert!(api_key.contains("def"));
    }

    #[test]
    fn test_detect_provider_from_env_none() {
        // Ensure no provider env vars are set for this test.
        let result = detect_provider_from_env();
        // May or may not be None depending on test environment, so just check it doesn't panic.
        let _ = result;
    }

    #[test]
    fn test_build_config_with_channels() {
        let mut llm = toml::map::Map::new();
        llm.insert("provider".into(), toml::Value::String("anthropic".into()));
        llm.insert("model".into(), toml::Value::String("claude-sonnet-4-6-20250217".into()));
        llm.insert("max_tokens".into(), toml::Value::Integer(4096));
        llm.insert("temperature".into(), toml::Value::Float(0.7));

        let channels = ChannelConfig {
            discord_token: "bot-token-123".into(),
            discord_channels: "111, 222".into(),
            telegram_token: "".into(),
            telegram_chats: "".into(),
            slack_bot_token: "xoxb-test".into(),
            slack_app_token: "xapp-test".into(),
            slack_channels: "C001".into(),
        };

        let result = build_config_toml(&llm, &channels).unwrap();
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        let comm = parsed["communication"].as_table().unwrap();

        // Discord enabled
        assert_eq!(comm["discord"]["enabled"].as_bool(), Some(true));
        assert_eq!(comm["discord"]["bot_token"].as_str(), Some("bot-token-123"));

        // Telegram disabled (empty token)
        assert_eq!(comm["telegram"]["enabled"].as_bool(), Some(false));

        // Slack enabled
        assert_eq!(comm["slack"]["enabled"].as_bool(), Some(true));
        assert_eq!(comm["slack"]["bot_token"].as_str(), Some("xoxb-test"));
    }
}
