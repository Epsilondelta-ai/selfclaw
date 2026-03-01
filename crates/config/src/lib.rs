use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ── Error types ──────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("config validation failed: {0}")]
    Validation(String),
}

// ── Config structs ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SelfClawConfig {
    #[serde(default)]
    pub agent: AgentConfig,

    #[serde(default)]
    pub llm: LlmConfig,

    #[serde(default)]
    pub safety: SafetyConfig,

    #[serde(default)]
    pub communication: CommsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    #[serde(default = "default_loop_interval")]
    pub loop_interval_secs: u64,

    #[serde(default = "default_consolidation_cycles")]
    pub consolidation_every_n_cycles: u64,

    #[serde(default = "default_max_actions")]
    pub max_actions_per_cycle: u64,

    /// Directories to load skills from (first-match-wins for duplicates).
    /// Supports tilde expansion (`~/` → home directory).
    #[serde(default = "default_skills_dirs")]
    pub skills_dirs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u64,

    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// Optional API key. If not set, the provider's env var is used
    /// (e.g. ANTHROPIC_API_KEY, OPENAI_API_KEY).
    #[serde(default)]
    pub api_key: Option<String>,

    /// Optional base URL override. Useful for proxies, self-hosted models,
    /// or any OpenAI-compatible endpoint.
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SafetyConfig {
    #[serde(default = "default_max_api_calls")]
    pub max_api_calls_per_hour: u64,

    #[serde(default = "default_max_file_writes")]
    pub max_file_writes_per_cycle: u64,

    #[serde(default = "default_sandbox_shell")]
    pub sandbox_shell: bool,

    #[serde(default = "default_allowed_directories")]
    pub allowed_directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommsConfig {
    #[serde(default = "default_cli_enabled")]
    pub cli_enabled: bool,

    #[serde(default)]
    pub web_ui_enabled: bool,

    #[serde(default = "default_web_ui_port")]
    pub web_ui_port: u16,

    #[serde(default)]
    pub discord: DiscordConfig,

    #[serde(default)]
    pub telegram: TelegramConfig,

    #[serde(default)]
    pub slack: SlackConfig,

    #[serde(default)]
    pub webchat: WebChatConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DiscordConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub bot_token: String,

    #[serde(default)]
    pub allowed_channel_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TelegramConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub bot_token: String,

    #[serde(default)]
    pub allowed_chat_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SlackConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub bot_token: String,

    #[serde(default)]
    pub app_token: String,

    #[serde(default)]
    pub allowed_channel_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebChatConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_webchat_port")]
    pub port: u16,
}

impl Default for WebChatConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_webchat_port(),
        }
    }
}

// ── Default value functions ──────────────────────────────────────────

fn default_loop_interval() -> u64 {
    60
}
fn default_consolidation_cycles() -> u64 {
    50
}
fn default_max_actions() -> u64 {
    5
}
fn default_skills_dirs() -> Vec<String> {
    vec![
        "~/.agents/skills".to_string(),
        "~/.selfclaw/skills".to_string(),
    ]
}
fn default_provider() -> String {
    "anthropic".to_string()
}
fn default_model() -> String {
    "claude-sonnet-4-6-20250217".to_string()
}
fn default_max_tokens() -> u64 {
    4096
}
fn default_temperature() -> f64 {
    0.7
}
fn default_max_api_calls() -> u64 {
    100
}
fn default_max_file_writes() -> u64 {
    10
}
fn default_sandbox_shell() -> bool {
    true
}
fn default_allowed_directories() -> Vec<String> {
    vec![
        "./memory".to_string(),
        "./skills".to_string(),
        "./output".to_string(),
    ]
}
fn default_cli_enabled() -> bool {
    true
}
fn default_web_ui_port() -> u16 {
    3000
}
fn default_webchat_port() -> u16 {
    3001
}

// ── Default trait impls ──────────────────────────────────────────────

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            loop_interval_secs: default_loop_interval(),
            consolidation_every_n_cycles: default_consolidation_cycles(),
            max_actions_per_cycle: default_max_actions(),
            skills_dirs: default_skills_dirs(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            api_key: None,
            base_url: None,
        }
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_api_calls_per_hour: default_max_api_calls(),
            max_file_writes_per_cycle: default_max_file_writes(),
            sandbox_shell: default_sandbox_shell(),
            allowed_directories: default_allowed_directories(),
        }
    }
}

impl Default for CommsConfig {
    fn default() -> Self {
        Self {
            cli_enabled: default_cli_enabled(),
            web_ui_enabled: false,
            web_ui_port: default_web_ui_port(),
            discord: DiscordConfig::default(),
            telegram: TelegramConfig::default(),
            slack: SlackConfig::default(),
            webchat: WebChatConfig::default(),
        }
    }
}

// ── Loading ──────────────────────────────────────────────────────────

impl SelfClawConfig {
    /// Load config from a TOML file. Missing fields use defaults.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let config: SelfClawConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Load config from a TOML string. Missing fields use defaults.
    pub fn parse_toml(s: &str) -> Result<Self, ConfigError> {
        let config: SelfClawConfig = toml::from_str(s)?;
        config.validate()?;
        Ok(config)
    }

    /// Load from file, falling back to defaults if file doesn't exist.
    pub fn load_or_default(path: &Path) -> Result<Self, ConfigError> {
        if path.exists() {
            Self::from_file(path)
        } else {
            Ok(Self::default())
        }
    }

    /// Validate config values.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.agent.loop_interval_secs == 0 {
            return Err(ConfigError::Validation(
                "agent.loop_interval_secs must be > 0".to_string(),
            ));
        }
        if self.agent.consolidation_every_n_cycles == 0 {
            return Err(ConfigError::Validation(
                "agent.consolidation_every_n_cycles must be > 0".to_string(),
            ));
        }
        if self.agent.max_actions_per_cycle == 0 {
            return Err(ConfigError::Validation(
                "agent.max_actions_per_cycle must be > 0".to_string(),
            ));
        }
        if self.llm.max_tokens == 0 {
            return Err(ConfigError::Validation(
                "llm.max_tokens must be > 0".to_string(),
            ));
        }
        if !(0.0..=2.0).contains(&self.llm.temperature) {
            return Err(ConfigError::Validation(
                "llm.temperature must be between 0.0 and 2.0".to_string(),
            ));
        }
        if self.llm.provider.is_empty() {
            return Err(ConfigError::Validation(
                "llm.provider must not be empty".to_string(),
            ));
        }
        if self.llm.model.is_empty() {
            return Err(ConfigError::Validation(
                "llm.model must not be empty".to_string(),
            ));
        }
        if self.safety.max_api_calls_per_hour == 0 {
            return Err(ConfigError::Validation(
                "safety.max_api_calls_per_hour must be > 0".to_string(),
            ));
        }
        if self.safety.max_file_writes_per_cycle == 0 {
            return Err(ConfigError::Validation(
                "safety.max_file_writes_per_cycle must be > 0".to_string(),
            ));
        }
        if self.communication.web_ui_port == 0 {
            return Err(ConfigError::Validation(
                "communication.web_ui_port must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    // ── Loading valid config ─────────────────────────────────────

    #[test]
    fn test_load_valid_config_from_string() {
        let toml = r#"
[agent]
loop_interval_secs = 30
consolidation_every_n_cycles = 25
max_actions_per_cycle = 10

[llm]
provider = "anthropic"
model = "claude-sonnet-4-6-20250217"
max_tokens = 8192
temperature = 0.5

[safety]
max_api_calls_per_hour = 200
max_file_writes_per_cycle = 20
sandbox_shell = false
allowed_directories = ["./memory"]

[communication]
cli_enabled = false
web_ui_enabled = true
web_ui_port = 8080
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert_eq!(config.agent.loop_interval_secs, 30);
        assert_eq!(config.agent.consolidation_every_n_cycles, 25);
        assert_eq!(config.agent.max_actions_per_cycle, 10);
        assert_eq!(config.llm.provider, "anthropic");
        assert_eq!(config.llm.model, "claude-sonnet-4-6-20250217");
        assert_eq!(config.llm.max_tokens, 8192);
        assert!((config.llm.temperature - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.safety.max_api_calls_per_hour, 200);
        assert_eq!(config.safety.max_file_writes_per_cycle, 20);
        assert!(!config.safety.sandbox_shell);
        assert_eq!(config.safety.allowed_directories, vec!["./memory"]);
        assert!(!config.communication.cli_enabled);
        assert!(config.communication.web_ui_enabled);
        assert_eq!(config.communication.web_ui_port, 8080);
    }

    #[test]
    fn test_load_valid_config_from_file() {
        let toml = r#"
[agent]
loop_interval_secs = 120

[llm]
provider = "anthropic"
model = "claude-sonnet-4-6-20250217"
max_tokens = 4096
temperature = 0.7
"#;
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", toml).unwrap();
        let config = SelfClawConfig::from_file(f.path()).unwrap();
        assert_eq!(config.agent.loop_interval_secs, 120);
    }

    // ── Defaults when file missing ───────────────────────────────

    #[test]
    fn test_defaults_when_file_missing() {
        let config =
            SelfClawConfig::load_or_default(Path::new("/nonexistent/selfclaw.toml")).unwrap();
        assert_eq!(config.agent.loop_interval_secs, 60);
        assert_eq!(config.agent.consolidation_every_n_cycles, 50);
        assert_eq!(config.agent.max_actions_per_cycle, 5);
        assert_eq!(config.llm.provider, "anthropic");
        assert_eq!(config.llm.model, "claude-sonnet-4-6-20250217");
        assert_eq!(config.llm.max_tokens, 4096);
        assert!((config.llm.temperature - 0.7).abs() < f64::EPSILON);
        assert_eq!(config.safety.max_api_calls_per_hour, 100);
        assert_eq!(config.safety.max_file_writes_per_cycle, 10);
        assert!(config.safety.sandbox_shell);
        assert_eq!(
            config.safety.allowed_directories,
            vec!["./memory", "./skills", "./output"]
        );
        assert!(config.communication.cli_enabled);
        assert!(!config.communication.web_ui_enabled);
        assert_eq!(config.communication.web_ui_port, 3000);
    }

    #[test]
    fn test_default_matches_expected_values() {
        let config = SelfClawConfig::default();
        assert_eq!(config.agent.loop_interval_secs, 60);
        assert_eq!(config.llm.provider, "anthropic");
        assert!(config.safety.sandbox_shell);
        assert!(config.communication.cli_enabled);
    }

    // ── Validation rejects invalid values ────────────────────────

    #[test]
    fn test_validation_rejects_zero_loop_interval() {
        let toml = r#"
[agent]
loop_interval_secs = 0
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("loop_interval_secs"), "got: {}", err);
    }

    #[test]
    fn test_validation_rejects_zero_consolidation_cycles() {
        let toml = r#"
[agent]
consolidation_every_n_cycles = 0
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("consolidation_every_n_cycles"), "got: {}", err);
    }

    #[test]
    fn test_validation_rejects_zero_max_actions() {
        let toml = r#"
[agent]
max_actions_per_cycle = 0
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_rejects_zero_max_tokens() {
        let toml = r#"
[llm]
max_tokens = 0
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_rejects_negative_temperature() {
        let toml = r#"
[llm]
temperature = -0.1
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("temperature"), "got: {}", err);
    }

    #[test]
    fn test_validation_rejects_high_temperature() {
        let toml = r#"
[llm]
temperature = 2.5
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_rejects_empty_provider() {
        let toml = r#"
[llm]
provider = ""
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_rejects_empty_model() {
        let toml = r#"
[llm]
model = ""
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_rejects_zero_api_calls() {
        let toml = r#"
[safety]
max_api_calls_per_hour = 0
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_rejects_zero_file_writes() {
        let toml = r#"
[safety]
max_file_writes_per_cycle = 0
"#;
        let result = SelfClawConfig::parse_toml(toml);
        assert!(result.is_err());
    }

    // ── Partial config (missing fields filled by defaults) ───────

    #[test]
    fn test_partial_config_only_agent() {
        let toml = r#"
[agent]
loop_interval_secs = 120
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        // Specified field
        assert_eq!(config.agent.loop_interval_secs, 120);
        // Defaults for rest of agent
        assert_eq!(config.agent.consolidation_every_n_cycles, 50);
        assert_eq!(config.agent.max_actions_per_cycle, 5);
        // Defaults for other sections
        assert_eq!(config.llm.provider, "anthropic");
        assert_eq!(config.safety.max_api_calls_per_hour, 100);
        assert!(config.communication.cli_enabled);
    }

    #[test]
    fn test_partial_config_only_llm() {
        let toml = r#"
[llm]
model = "claude-opus-4-20250514"
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert_eq!(config.llm.model, "claude-opus-4-20250514");
        // Defaults for rest of llm
        assert_eq!(config.llm.provider, "anthropic");
        assert_eq!(config.llm.max_tokens, 4096);
        // Defaults for other sections
        assert_eq!(config.agent.loop_interval_secs, 60);
    }

    #[test]
    fn test_empty_toml_uses_all_defaults() {
        let config = SelfClawConfig::parse_toml("").unwrap();
        assert_eq!(config, SelfClawConfig::default());
    }

    // ── Error type tests ─────────────────────────────────────────

    #[test]
    fn test_io_error_on_unreadable_file() {
        let result = SelfClawConfig::from_file(Path::new("/nonexistent/file.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("read config file"), "got: {}", err);
    }

    #[test]
    fn test_parse_error_on_invalid_toml() {
        let result = SelfClawConfig::parse_toml("this is not valid toml {{{{");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("parse config"), "got: {}", err);
    }

    // ── Multi-provider config fields ─────────────────────────────

    #[test]
    fn test_llm_api_key_from_config() {
        let toml = r#"
[llm]
provider = "openai"
model = "gpt-4o"
api_key = "sk-my-secret-key"
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4o");
        assert_eq!(config.llm.api_key, Some("sk-my-secret-key".to_string()));
    }

    #[test]
    fn test_llm_base_url_from_config() {
        let toml = r#"
[llm]
provider = "ollama"
model = "llama3.1"
base_url = "http://192.168.1.100:11434"
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert_eq!(config.llm.provider, "ollama");
        assert_eq!(
            config.llm.base_url,
            Some("http://192.168.1.100:11434".to_string())
        );
    }

    #[test]
    fn test_llm_defaults_no_api_key_or_base_url() {
        let config = SelfClawConfig::default();
        assert!(config.llm.api_key.is_none());
        assert!(config.llm.base_url.is_none());
    }

    #[test]
    fn test_llm_custom_provider_with_base_url() {
        let toml = r#"
[llm]
provider = "my-custom-provider"
model = "custom-model-v1"
base_url = "https://my-llm-proxy.example.com"
api_key = "custom-key-123"
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert_eq!(config.llm.provider, "my-custom-provider");
        assert_eq!(config.llm.model, "custom-model-v1");
        assert_eq!(
            config.llm.base_url,
            Some("https://my-llm-proxy.example.com".to_string())
        );
        assert_eq!(config.llm.api_key, Some("custom-key-123".to_string()));
    }

    // ── skills_dirs field ────────────────────────────────────────

    #[test]
    fn test_skills_dirs_default() {
        let config = SelfClawConfig::default();
        assert_eq!(
            config.agent.skills_dirs,
            vec!["~/.agents/skills", "~/.selfclaw/skills"]
        );
    }

    #[test]
    fn test_skills_dirs_custom() {
        let toml = r#"
[agent]
skills_dirs = ["/opt/skills", "~/my-skills"]
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert_eq!(config.agent.skills_dirs, vec!["/opt/skills", "~/my-skills"]);
    }

    #[test]
    fn test_skills_dirs_empty_allowed() {
        let toml = r#"
[agent]
skills_dirs = []
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert!(config.agent.skills_dirs.is_empty());
    }

    #[test]
    fn test_skills_dirs_unspecified_uses_default() {
        let toml = r#"
[agent]
loop_interval_secs = 30
"#;
        let config = SelfClawConfig::parse_toml(toml).unwrap();
        assert_eq!(
            config.agent.skills_dirs,
            vec!["~/.agents/skills", "~/.selfclaw/skills"]
        );
    }

    #[test]
    fn test_all_known_providers_parse() {
        let providers = [
            "anthropic",
            "openai",
            "google",
            "ollama",
            "openrouter",
            "groq",
            "xai",
            "mistral",
            "deepseek",
        ];
        for provider in &providers {
            let toml = format!(
                r#"
[llm]
provider = "{}"
"#,
                provider
            );
            let config = SelfClawConfig::parse_toml(&toml).unwrap();
            assert_eq!(config.llm.provider, *provider);
        }
    }
}
