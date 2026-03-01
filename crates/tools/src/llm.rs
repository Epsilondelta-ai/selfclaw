use crate::{Tool, ToolError, ToolOutput};

// ── Provider kinds ──────────────────────────────────────────────────

/// All supported LLM provider identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    OpenAI,
    Google,
    Ollama,
    OpenRouter,
    Groq,
    XAI,
    Mistral,
    DeepSeek,
    Together,
    Moonshot,
    Bedrock,
    Custom,
}

impl ProviderKind {
    /// All known provider kinds (excluding Custom).
    pub fn all() -> &'static [ProviderKind] {
        &[
            Self::Anthropic,
            Self::OpenAI,
            Self::Google,
            Self::Ollama,
            Self::OpenRouter,
            Self::Groq,
            Self::XAI,
            Self::Mistral,
            Self::DeepSeek,
            Self::Together,
            Self::Moonshot,
            Self::Bedrock,
        ]
    }

    /// The canonical name used in config files.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAI => "openai",
            Self::Google => "google",
            Self::Ollama => "ollama",
            Self::OpenRouter => "openrouter",
            Self::Groq => "groq",
            Self::XAI => "xai",
            Self::Mistral => "mistral",
            Self::DeepSeek => "deepseek",
            Self::Together => "together",
            Self::Moonshot => "moonshot",
            Self::Bedrock => "bedrock",
            Self::Custom => "custom",
        }
    }
}

impl ProviderKind {
    /// Parse a provider string (case-insensitive) into a ProviderKind.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "anthropic" | "claude" => Self::Anthropic,
            "openai" | "gpt" => Self::OpenAI,
            "google" | "gemini" | "vertex" => Self::Google,
            "ollama" => Self::Ollama,
            "openrouter" => Self::OpenRouter,
            "groq" => Self::Groq,
            "xai" | "grok" => Self::XAI,
            "mistral" => Self::Mistral,
            "deepseek" => Self::DeepSeek,
            "together" | "together-ai" | "togetherai" => Self::Together,
            "moonshot" | "kimi" => Self::Moonshot,
            "bedrock" | "amazon-bedrock" | "aws-bedrock" => Self::Bedrock,
            _ => Self::Custom,
        }
    }

    /// The default model name for this provider.
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Anthropic => "claude-sonnet-4-6-20250217",
            Self::OpenAI => "gpt-5.2",
            Self::Google => "gemini-2.5-flash",
            Self::Ollama => "llama4",
            Self::OpenRouter => "anthropic/claude-sonnet-4-6-20250217",
            Self::Groq => "llama-3.3-70b-versatile",
            Self::XAI => "grok-4",
            Self::Mistral => "mistral-large-latest",
            Self::DeepSeek => "deepseek-chat",
            Self::Together => "meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8",
            Self::Moonshot => "kimi-k2.5",
            Self::Bedrock => "anthropic.claude-sonnet-4-6-20250217-v1:0",
            Self::Custom => "default",
        }
    }

    /// The environment variable name for the API key.
    pub fn api_key_env_var(&self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Google => "GOOGLE_API_KEY",
            Self::Ollama => "OLLAMA_API_KEY", // usually not needed
            Self::OpenRouter => "OPENROUTER_API_KEY",
            Self::Groq => "GROQ_API_KEY",
            Self::XAI => "XAI_API_KEY",
            Self::Mistral => "MISTRAL_API_KEY",
            Self::DeepSeek => "DEEPSEEK_API_KEY",
            Self::Together => "TOGETHER_API_KEY",
            Self::Moonshot => "MOONSHOT_API_KEY",
            Self::Bedrock => "AWS_ACCESS_KEY_ID", // uses AWS credentials
            Self::Custom => "LLM_API_KEY",
        }
    }

    /// The default base URL for this provider's API.
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Anthropic => "https://api.anthropic.com",
            Self::OpenAI => "https://api.openai.com",
            Self::Google => "https://generativelanguage.googleapis.com",
            Self::Ollama => "http://localhost:11434",
            Self::OpenRouter => "https://openrouter.ai/api",
            Self::Groq => "https://api.groq.com/openai",
            Self::XAI => "https://api.x.ai",
            Self::Mistral => "https://api.mistral.ai",
            Self::DeepSeek => "https://api.deepseek.com",
            Self::Together => "https://api.together.xyz",
            Self::Moonshot => "https://api.moonshot.cn",
            Self::Bedrock => "https://bedrock-runtime.us-east-1.amazonaws.com",
            Self::Custom => "http://localhost:8080",
        }
    }
}

// ── Provider trait ──────────────────────────────────────────────────

/// Trait for LLM provider implementations.
/// Each provider knows how to build requests, parse responses,
/// and authenticate with its specific API.
pub trait LlmProvider: Send + Sync {
    /// Provider identifier.
    fn kind(&self) -> ProviderKind;

    /// The full URL to send requests to.
    fn endpoint(&self) -> String;

    /// Build the JSON request body for this provider.
    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value;

    /// Parse the provider's JSON response into the assistant's text.
    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String>;

    /// Build HTTP headers for authentication and content type.
    fn build_headers(&self, api_key: &str) -> Vec<(String, String)>;

    /// Whether this provider requires an API key (Ollama typically doesn't).
    fn requires_api_key(&self) -> bool {
        true
    }
}

// ═══════════════════════════════════════════════════════════════════
// Concrete provider implementations
// ═══════════════════════════════════════════════════════════════════

// ── Anthropic ───────────────────────────────────────────────────────

pub struct AnthropicProvider {
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Anthropic.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/messages", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });
        if let Some(sys) = system {
            body["system"] = serde_json::json!(sys);
        }
        body
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        // Anthropic response: { "content": [{ "type": "text", "text": "..." }] }
        response["content"]
            .as_array()
            .and_then(|blocks| {
                blocks.iter().find_map(|block| {
                    if block["type"] == "text" {
                        block["text"].as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                format!(
                    "failed to parse Anthropic response: no text content block found in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("x-api-key".to_string(), api_key.to_string()),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── OpenAI ──────────────────────────────────────────────────────────

pub struct OpenAIProvider {
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::OpenAI.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for OpenAIProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAI
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        // OpenAI response: { "choices": [{ "message": { "content": "..." } }] }
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse OpenAI response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Google Gemini ───────────────────────────────────────────────────

pub struct GoogleProvider {
    base_url: String,
}

impl GoogleProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Google.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for GoogleProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Google
    }

    fn endpoint(&self) -> String {
        // Gemini uses model in URL; we use a placeholder replaced at call time.
        format!("{}/v1beta/models/{{model}}:generateContent", self.base_url)
    }

    fn build_request(
        &self,
        _model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        let mut body = serde_json::json!({
            "contents": [
                {
                    "parts": [{ "text": prompt }]
                }
            ],
            "generationConfig": {
                "maxOutputTokens": max_tokens,
                "temperature": temperature
            }
        });
        if let Some(sys) = system {
            body["systemInstruction"] = serde_json::json!({
                "parts": [{ "text": sys }]
            });
        }
        body
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        // Gemini response: { "candidates": [{ "content": { "parts": [{ "text": "..." }] } }] }
        response["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse Gemini response: no candidates[0].content.parts[0].text in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        // Gemini uses ?key= query param, but we also set content-type.
        // The api_key will be appended as query param in the HTTP call.
        vec![
            ("x-goog-api-key".to_string(), api_key.to_string()),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Ollama (local) ──────────────────────────────────────────────────

pub struct OllamaProvider {
    base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Ollama.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for OllamaProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }

    fn endpoint(&self) -> String {
        format!("{}/api/chat", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        _max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": temperature
            }
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        // Ollama response: { "message": { "content": "..." } }
        response["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse Ollama response: no message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, _api_key: &str) -> Vec<(String, String)> {
        vec![("content-type".to_string(), "application/json".to_string())]
    }

    fn requires_api_key(&self) -> bool {
        false
    }
}

// ── OpenRouter ──────────────────────────────────────────────────────

pub struct OpenRouterProvider {
    base_url: String,
}

impl OpenRouterProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::OpenRouter.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for OpenRouterProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenRouter
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // OpenRouter uses OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        // Same as OpenAI format
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse OpenRouter response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
            (
                "HTTP-Referer".to_string(),
                "https://github.com/selfclaw".to_string(),
            ),
            ("X-Title".to_string(), "SelfClaw".to_string()),
        ]
    }
}

// ── Groq ────────────────────────────────────────────────────────────

pub struct GroqProvider {
    base_url: String,
}

impl GroqProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Groq.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for GroqProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Groq
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // Groq uses OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse Groq response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── xAI (Grok) ─────────────────────────────────────────────────────

pub struct XAIProvider {
    base_url: String,
}

impl XAIProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::XAI.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for XAIProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::XAI
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // xAI uses OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse xAI response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Mistral ─────────────────────────────────────────────────────────

pub struct MistralProvider {
    base_url: String,
}

impl MistralProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Mistral.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for MistralProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Mistral
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // Mistral uses OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse Mistral response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── DeepSeek ────────────────────────────────────────────────────────

pub struct DeepSeekProvider {
    base_url: String,
}

impl DeepSeekProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::DeepSeek.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for DeepSeekProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::DeepSeek
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // DeepSeek uses OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse DeepSeek response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Together AI ─────────────────────────────────────────────────────

pub struct TogetherProvider {
    base_url: String,
}

impl TogetherProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Together.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for TogetherProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Together
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // Together AI uses OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse Together AI response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Moonshot (Kimi) ─────────────────────────────────────────────────

pub struct MoonshotProvider {
    base_url: String,
}

impl MoonshotProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Moonshot.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for MoonshotProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Moonshot
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // Moonshot uses OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse Moonshot response: no choices[0].message.content in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Amazon Bedrock ──────────────────────────────────────────────────

pub struct BedrockProvider {
    base_url: String,
}

impl BedrockProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(ProviderKind::Bedrock.default_base_url())
                .to_string(),
        }
    }
}

impl LlmProvider for BedrockProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Bedrock
    }

    fn endpoint(&self) -> String {
        // Bedrock uses model ID in URL path
        format!("{}/model/{{model}}/converse", self.base_url)
    }

    fn build_request(
        &self,
        _model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // Bedrock Converse API format
        let mut body = serde_json::json!({
            "messages": [
                {
                    "role": "user",
                    "content": [{ "text": prompt }]
                }
            ],
            "inferenceConfig": {
                "maxTokens": max_tokens,
                "temperature": temperature
            }
        });
        if let Some(sys) = system {
            body["system"] = serde_json::json!([{ "text": sys }]);
        }
        body
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        // Bedrock Converse: { "output": { "message": { "content": [{ "text": "..." }] } } }
        response["output"]["message"]["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "failed to parse Bedrock response: no output.message.content[0].text in: {}",
                    response
                )
            })
    }

    fn build_headers(&self, _api_key: &str) -> Vec<(String, String)> {
        // Bedrock uses AWS SigV4 auth; for now we set content-type
        // and rely on AWS SDK or env credentials.
        vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("accept".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Custom (OpenAI-compatible) ──────────────────────────────────────

pub struct CustomProvider {
    base_url: String,
}

impl CustomProvider {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }
}

impl LlmProvider for CustomProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Custom
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request(
        &self,
        model: &str,
        max_tokens: u64,
        temperature: f64,
        prompt: &str,
        system: Option<&str>,
    ) -> serde_json::Value {
        // Default to OpenAI-compatible format
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": prompt }));

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": messages
        })
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<String, String> {
        // Try OpenAI format first, then Anthropic, then raw text
        if let Some(text) = response["choices"][0]["message"]["content"].as_str() {
            return Ok(text.to_string());
        }
        if let Some(blocks) = response["content"].as_array() {
            if let Some(text) = blocks.iter().find_map(|b| b["text"].as_str()) {
                return Ok(text.to_string());
            }
        }
        if let Some(text) = response["message"]["content"].as_str() {
            return Ok(text.to_string());
        }
        Err(format!(
            "failed to parse custom provider response: unrecognized format: {}",
            response
        ))
    }

    fn build_headers(&self, api_key: &str) -> Vec<(String, String)> {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", api_key)),
            ("content-type".to_string(), "application/json".to_string()),
        ]
    }
}

// ── Provider factory ────────────────────────────────────────────────

/// Create the appropriate provider from config.
pub fn create_provider(config: &selfclaw_config::LlmConfig) -> Box<dyn LlmProvider> {
    let kind = ProviderKind::parse(&config.provider);
    let base_url = config.base_url.as_deref();

    match kind {
        ProviderKind::Anthropic => Box::new(AnthropicProvider::new(base_url)),
        ProviderKind::OpenAI => Box::new(OpenAIProvider::new(base_url)),
        ProviderKind::Google => Box::new(GoogleProvider::new(base_url)),
        ProviderKind::Ollama => Box::new(OllamaProvider::new(base_url)),
        ProviderKind::OpenRouter => Box::new(OpenRouterProvider::new(base_url)),
        ProviderKind::Groq => Box::new(GroqProvider::new(base_url)),
        ProviderKind::XAI => Box::new(XAIProvider::new(base_url)),
        ProviderKind::Mistral => Box::new(MistralProvider::new(base_url)),
        ProviderKind::DeepSeek => Box::new(DeepSeekProvider::new(base_url)),
        ProviderKind::Together => Box::new(TogetherProvider::new(base_url)),
        ProviderKind::Moonshot => Box::new(MoonshotProvider::new(base_url)),
        ProviderKind::Bedrock => Box::new(BedrockProvider::new(base_url)),
        ProviderKind::Custom => {
            let url = base_url.unwrap_or(ProviderKind::Custom.default_base_url());
            Box::new(CustomProvider::new(url))
        }
    }
}

// ── LlmCallTool ─────────────────────────────────────────────────────

pub struct LlmCallTool {
    provider: Box<dyn LlmProvider>,
    model: String,
    max_tokens: u64,
    temperature: f64,
    api_key: Option<String>,
}

impl LlmCallTool {
    pub fn new(model: String, max_tokens: u64, temperature: f64) -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        Self {
            provider: Box::new(AnthropicProvider::new(None)),
            model,
            max_tokens,
            temperature,
            api_key,
        }
    }

    pub fn from_config(config: &selfclaw_config::LlmConfig) -> Self {
        let provider = create_provider(config);
        let kind = provider.kind();

        // Resolve API key: config explicit > env var > None
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var(kind.api_key_env_var()).ok());

        Self {
            provider,
            model: config.model.clone(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            api_key,
        }
    }

    /// Build the request for inspection/testing without making an HTTP call.
    pub fn build_request(&self, prompt: &str, system: Option<&str>) -> serde_json::Value {
        self.provider.build_request(
            &self.model,
            self.max_tokens,
            self.temperature,
            prompt,
            system,
        )
    }

    /// Get the provider kind.
    pub fn provider_kind(&self) -> ProviderKind {
        self.provider.kind()
    }
}

impl Tool for LlmCallTool {
    fn name(&self) -> &str {
        "llm_call"
    }

    fn description(&self) -> &str {
        "Call an LLM provider with a prompt"
    }

    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let prompt = input
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("prompt".to_string()))?;

        let system = input.get("system").and_then(|v| v.as_str());

        let requires_key = self.provider.requires_api_key();
        let api_key = if requires_key {
            Some(
                self.api_key
                    .as_ref()
                    .ok_or_else(|| {
                        let env_var = self.provider.kind().api_key_env_var();
                        ToolError::InvalidInput(format!(
                            "{} environment variable not set (or set llm.api_key in config)",
                            env_var
                        ))
                    })?
                    .as_str(),
            )
        } else {
            self.api_key.as_deref()
        };

        let body = self.build_request(prompt, system);
        let endpoint = self.provider.endpoint().replace("{model}", &self.model);
        let headers = self.provider.build_headers(api_key.unwrap_or(""));

        let rt = tokio::runtime::Handle::try_current();
        let response_result = match rt {
            Ok(_handle) => std::thread::scope(|s| {
                s.spawn(|| {
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| ToolError::Http(e.to_string()))?;
                    rt.block_on(Self::do_http_call(&endpoint, &headers, &body))
                })
                .join()
                .unwrap_or_else(|_| Err(ToolError::Http("LLM worker thread panicked".to_string())))
            }),
            Err(_) => {
                let rt =
                    tokio::runtime::Runtime::new().map_err(|e| ToolError::Http(e.to_string()))?;
                rt.block_on(Self::do_http_call(&endpoint, &headers, &body))
            }
        };

        match response_result {
            Ok(response_body) => {
                // Parse the response using provider-specific logic
                match self.provider.parse_response(&response_body) {
                    Ok(text) => Ok(ToolOutput::ok(serde_json::json!({ "response": text }))),
                    Err(e) => Err(ToolError::Http(e)),
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl LlmCallTool {
    async fn do_http_call(
        endpoint: &str,
        headers: &[(String, String)],
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        let client = reqwest::Client::new();
        let mut request = client.post(endpoint);

        for (key, value) in headers {
            request = request.header(key.as_str(), value.as_str());
        }

        let resp = request
            .json(body)
            .send()
            .await
            .map_err(|e| ToolError::Http(e.to_string()))?;

        let status = resp.status();
        let response_body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ToolError::Http(e.to_string()))?;

        if !status.is_success() {
            return Err(ToolError::Http(format!(
                "API returned {}: {}",
                status, response_body
            )));
        }

        Ok(response_body)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ProviderKind ────────────────────────────────────────────

    #[test]
    fn test_provider_kind_from_str() {
        assert_eq!(ProviderKind::parse("anthropic"), ProviderKind::Anthropic);
        assert_eq!(ProviderKind::parse("claude"), ProviderKind::Anthropic);
        assert_eq!(ProviderKind::parse("ANTHROPIC"), ProviderKind::Anthropic);
        assert_eq!(ProviderKind::parse("openai"), ProviderKind::OpenAI);
        assert_eq!(ProviderKind::parse("gpt"), ProviderKind::OpenAI);
        assert_eq!(ProviderKind::parse("google"), ProviderKind::Google);
        assert_eq!(ProviderKind::parse("gemini"), ProviderKind::Google);
        assert_eq!(ProviderKind::parse("vertex"), ProviderKind::Google);
        assert_eq!(ProviderKind::parse("ollama"), ProviderKind::Ollama);
        assert_eq!(ProviderKind::parse("openrouter"), ProviderKind::OpenRouter);
        assert_eq!(ProviderKind::parse("groq"), ProviderKind::Groq);
        assert_eq!(ProviderKind::parse("xai"), ProviderKind::XAI);
        assert_eq!(ProviderKind::parse("grok"), ProviderKind::XAI);
        assert_eq!(ProviderKind::parse("mistral"), ProviderKind::Mistral);
        assert_eq!(ProviderKind::parse("deepseek"), ProviderKind::DeepSeek);
        assert_eq!(ProviderKind::parse("unknown"), ProviderKind::Custom);
    }

    #[test]
    fn test_provider_kind_default_model() {
        assert_eq!(
            ProviderKind::Anthropic.default_model(),
            "claude-sonnet-4-6-20250217"
        );
        assert_eq!(ProviderKind::OpenAI.default_model(), "gpt-5.2");
        assert_eq!(ProviderKind::Google.default_model(), "gemini-2.5-flash");
        assert_eq!(ProviderKind::Ollama.default_model(), "llama4");
        assert_eq!(
            ProviderKind::Groq.default_model(),
            "llama-3.3-70b-versatile"
        );
        assert_eq!(ProviderKind::XAI.default_model(), "grok-4");
        assert_eq!(
            ProviderKind::Mistral.default_model(),
            "mistral-large-latest"
        );
        assert_eq!(ProviderKind::DeepSeek.default_model(), "deepseek-chat");
    }

    #[test]
    fn test_provider_kind_env_var() {
        assert_eq!(
            ProviderKind::Anthropic.api_key_env_var(),
            "ANTHROPIC_API_KEY"
        );
        assert_eq!(ProviderKind::OpenAI.api_key_env_var(), "OPENAI_API_KEY");
        assert_eq!(ProviderKind::Google.api_key_env_var(), "GOOGLE_API_KEY");
        assert_eq!(ProviderKind::Groq.api_key_env_var(), "GROQ_API_KEY");
        assert_eq!(ProviderKind::XAI.api_key_env_var(), "XAI_API_KEY");
    }

    #[test]
    fn test_provider_kind_base_url() {
        assert_eq!(
            ProviderKind::Anthropic.default_base_url(),
            "https://api.anthropic.com"
        );
        assert_eq!(
            ProviderKind::OpenAI.default_base_url(),
            "https://api.openai.com"
        );
        assert_eq!(
            ProviderKind::Ollama.default_base_url(),
            "http://localhost:11434"
        );
    }

    // ── Anthropic provider ──────────────────────────────────────

    #[test]
    fn test_anthropic_build_request() {
        let p = AnthropicProvider::new(None);
        let req = p.build_request(
            "claude-sonnet-4-6-20250217",
            4096,
            0.7,
            "Hello",
            Some("Be helpful"),
        );

        assert_eq!(req["model"], "claude-sonnet-4-6-20250217");
        assert_eq!(req["max_tokens"], 4096);
        assert_eq!(req["messages"][0]["role"], "user");
        assert_eq!(req["messages"][0]["content"], "Hello");
        assert_eq!(req["system"], "Be helpful");
    }

    #[test]
    fn test_anthropic_build_request_no_system() {
        let p = AnthropicProvider::new(None);
        let req = p.build_request("claude-sonnet-4-6-20250217", 4096, 0.7, "Hello", None);
        assert!(req.get("system").is_none());
    }

    #[test]
    fn test_anthropic_parse_response() {
        let p = AnthropicProvider::new(None);
        let resp = serde_json::json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                { "type": "text", "text": "Hello! How can I help?" }
            ],
            "model": "claude-sonnet-4-6-20250217",
            "stop_reason": "end_turn"
        });
        let text = p.parse_response(&resp).unwrap();
        assert_eq!(text, "Hello! How can I help?");
    }

    #[test]
    fn test_anthropic_parse_response_error() {
        let p = AnthropicProvider::new(None);
        let resp = serde_json::json!({ "error": "bad" });
        assert!(p.parse_response(&resp).is_err());
    }

    #[test]
    fn test_anthropic_endpoint() {
        let p = AnthropicProvider::new(None);
        assert_eq!(p.endpoint(), "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn test_anthropic_custom_base_url() {
        let p = AnthropicProvider::new(Some("https://custom.api.com"));
        assert_eq!(p.endpoint(), "https://custom.api.com/v1/messages");
    }

    #[test]
    fn test_anthropic_headers() {
        let p = AnthropicProvider::new(None);
        let headers = p.build_headers("sk-test-key");
        assert!(headers
            .iter()
            .any(|(k, v)| k == "x-api-key" && v == "sk-test-key"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "anthropic-version" && v == "2023-06-01"));
    }

    // ── OpenAI provider ─────────────────────────────────────────

    #[test]
    fn test_openai_build_request() {
        let p = OpenAIProvider::new(None);
        let req = p.build_request("gpt-4o", 4096, 0.7, "Hello", Some("Be helpful"));

        assert_eq!(req["model"], "gpt-4o");
        assert_eq!(req["max_tokens"], 4096);
        let messages = req["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "Be helpful");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[1]["content"], "Hello");
    }

    #[test]
    fn test_openai_build_request_no_system() {
        let p = OpenAIProvider::new(None);
        let req = p.build_request("gpt-4o", 4096, 0.7, "Hello", None);
        let messages = req["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
    }

    #[test]
    fn test_openai_parse_response() {
        let p = OpenAIProvider::new(None);
        let resp = serde_json::json!({
            "id": "chatcmpl-123",
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": "Hi there!" },
                "finish_reason": "stop"
            }]
        });
        let text = p.parse_response(&resp).unwrap();
        assert_eq!(text, "Hi there!");
    }

    #[test]
    fn test_openai_endpoint() {
        let p = OpenAIProvider::new(None);
        assert_eq!(p.endpoint(), "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn test_openai_headers() {
        let p = OpenAIProvider::new(None);
        let headers = p.build_headers("sk-test");
        assert!(headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer sk-test"));
    }

    // ── Google Gemini provider ──────────────────────────────────

    #[test]
    fn test_google_build_request() {
        let p = GoogleProvider::new(None);
        let req = p.build_request("gemini-2.0-flash", 4096, 0.7, "Hello", Some("System text"));

        assert!(req["contents"][0]["parts"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Hello"));
        assert_eq!(req["generationConfig"]["maxOutputTokens"], 4096);
        assert_eq!(req["systemInstruction"]["parts"][0]["text"], "System text");
    }

    #[test]
    fn test_google_parse_response() {
        let p = GoogleProvider::new(None);
        let resp = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": "Gemini response" }],
                    "role": "model"
                }
            }]
        });
        let text = p.parse_response(&resp).unwrap();
        assert_eq!(text, "Gemini response");
    }

    #[test]
    fn test_google_endpoint_has_model_placeholder() {
        let p = GoogleProvider::new(None);
        assert!(p.endpoint().contains("{model}"));
    }

    // ── Ollama provider ─────────────────────────────────────────

    #[test]
    fn test_ollama_build_request() {
        let p = OllamaProvider::new(None);
        let req = p.build_request("llama3.1", 4096, 0.7, "Hello", None);

        assert_eq!(req["model"], "llama3.1");
        assert_eq!(req["stream"], false);
        assert_eq!(req["options"]["temperature"], 0.7);
    }

    #[test]
    fn test_ollama_parse_response() {
        let p = OllamaProvider::new(None);
        let resp = serde_json::json!({
            "model": "llama3.1",
            "message": { "role": "assistant", "content": "Ollama says hi" }
        });
        let text = p.parse_response(&resp).unwrap();
        assert_eq!(text, "Ollama says hi");
    }

    #[test]
    fn test_ollama_does_not_require_api_key() {
        let p = OllamaProvider::new(None);
        assert!(!p.requires_api_key());
    }

    #[test]
    fn test_ollama_endpoint() {
        let p = OllamaProvider::new(None);
        assert_eq!(p.endpoint(), "http://localhost:11434/api/chat");
    }

    #[test]
    fn test_ollama_custom_url() {
        let p = OllamaProvider::new(Some("http://192.168.1.100:11434"));
        assert_eq!(p.endpoint(), "http://192.168.1.100:11434/api/chat");
    }

    // ── OpenRouter provider ─────────────────────────────────────

    #[test]
    fn test_openrouter_build_request() {
        let p = OpenRouterProvider::new(None);
        let req = p.build_request(
            "anthropic/claude-sonnet-4-6-20250217",
            4096,
            0.7,
            "Hello",
            Some("System"),
        );
        assert_eq!(req["model"], "anthropic/claude-sonnet-4-6-20250217");
    }

    #[test]
    fn test_openrouter_parse_response() {
        let p = OpenRouterProvider::new(None);
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "OpenRouter response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "OpenRouter response");
    }

    #[test]
    fn test_openrouter_headers_include_referer() {
        let p = OpenRouterProvider::new(None);
        let headers = p.build_headers("or-test-key");
        assert!(headers.iter().any(|(k, _)| k == "HTTP-Referer"));
        assert!(headers.iter().any(|(k, _)| k == "X-Title"));
    }

    // ── Groq provider ───────────────────────────────────────────

    #[test]
    fn test_groq_build_request() {
        let p = GroqProvider::new(None);
        let req = p.build_request("llama-3.3-70b-versatile", 4096, 0.7, "Hello", None);
        assert_eq!(req["model"], "llama-3.3-70b-versatile");
    }

    #[test]
    fn test_groq_parse_response() {
        let p = GroqProvider::new(None);
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "Groq response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Groq response");
    }

    #[test]
    fn test_groq_endpoint() {
        let p = GroqProvider::new(None);
        assert_eq!(
            p.endpoint(),
            "https://api.groq.com/openai/v1/chat/completions"
        );
    }

    // ── xAI provider ────────────────────────────────────────────

    #[test]
    fn test_xai_build_request() {
        let p = XAIProvider::new(None);
        let req = p.build_request("grok-3", 4096, 0.7, "Hello", None);
        assert_eq!(req["model"], "grok-3");
    }

    #[test]
    fn test_xai_parse_response() {
        let p = XAIProvider::new(None);
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "Grok response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Grok response");
    }

    // ── Mistral provider ────────────────────────────────────────

    #[test]
    fn test_mistral_build_request() {
        let p = MistralProvider::new(None);
        let req = p.build_request("mistral-large-latest", 4096, 0.7, "Hello", None);
        assert_eq!(req["model"], "mistral-large-latest");
    }

    #[test]
    fn test_mistral_parse_response() {
        let p = MistralProvider::new(None);
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "Mistral response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Mistral response");
    }

    // ── DeepSeek provider ───────────────────────────────────────

    #[test]
    fn test_deepseek_build_request() {
        let p = DeepSeekProvider::new(None);
        let req = p.build_request("deepseek-chat", 4096, 0.7, "Hello", None);
        assert_eq!(req["model"], "deepseek-chat");
    }

    #[test]
    fn test_deepseek_parse_response() {
        let p = DeepSeekProvider::new(None);
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "DeepSeek response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "DeepSeek response");
    }

    #[test]
    fn test_deepseek_endpoint() {
        let p = DeepSeekProvider::new(None);
        assert_eq!(p.endpoint(), "https://api.deepseek.com/chat/completions");
    }

    // ── Custom provider ─────────────────────────────────────────

    #[test]
    fn test_custom_parse_openai_format() {
        let p = CustomProvider::new("http://localhost:8080");
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "Custom response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Custom response");
    }

    #[test]
    fn test_custom_parse_anthropic_format() {
        let p = CustomProvider::new("http://localhost:8080");
        let resp = serde_json::json!({
            "content": [{ "type": "text", "text": "Anthropic-like response" }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Anthropic-like response");
    }

    #[test]
    fn test_custom_parse_ollama_format() {
        let p = CustomProvider::new("http://localhost:8080");
        let resp = serde_json::json!({
            "message": { "content": "Ollama-like response" }
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Ollama-like response");
    }

    // ── Provider factory ────────────────────────────────────────

    #[test]
    fn test_create_provider_anthropic() {
        let config = selfclaw_config::LlmConfig {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6-20250217".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let p = create_provider(&config);
        assert_eq!(p.kind(), ProviderKind::Anthropic);
    }

    #[test]
    fn test_create_provider_openai() {
        let config = selfclaw_config::LlmConfig {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let p = create_provider(&config);
        assert_eq!(p.kind(), ProviderKind::OpenAI);
    }

    #[test]
    fn test_create_provider_ollama() {
        let config = selfclaw_config::LlmConfig {
            provider: "ollama".to_string(),
            model: "llama3.1".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let p = create_provider(&config);
        assert_eq!(p.kind(), ProviderKind::Ollama);
        assert!(!p.requires_api_key());
    }

    #[test]
    fn test_create_provider_custom_with_base_url() {
        let config = selfclaw_config::LlmConfig {
            provider: "my-custom-llm".to_string(),
            model: "custom-model".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: Some("http://my-server:9090".to_string()),
        };
        let p = create_provider(&config);
        assert_eq!(p.kind(), ProviderKind::Custom);
        assert_eq!(p.endpoint(), "http://my-server:9090/v1/chat/completions");
    }

    // ── LlmCallTool ─────────────────────────────────────────────

    #[test]
    fn test_llm_tool_build_request_default() {
        let tool = LlmCallTool::new("claude-sonnet-4-6-20250217".to_string(), 4096, 0.7);
        let req = tool.build_request("What is 2+2?", None);
        assert_eq!(req["model"], "claude-sonnet-4-6-20250217");
        assert_eq!(req["messages"][0]["content"], "What is 2+2?");
    }

    #[test]
    fn test_llm_tool_from_config_anthropic() {
        let config = selfclaw_config::LlmConfig {
            provider: "anthropic".to_string(),
            model: "claude-opus-4-20250514".to_string(),
            max_tokens: 8192,
            temperature: 0.3,
            api_key: None,
            base_url: None,
        };
        let tool = LlmCallTool::from_config(&config);
        assert_eq!(tool.provider_kind(), ProviderKind::Anthropic);
        let req = tool.build_request("test", None);
        assert_eq!(req["model"], "claude-opus-4-20250514");
        assert_eq!(req["max_tokens"], 8192);
    }

    #[test]
    fn test_llm_tool_from_config_openai() {
        let config = selfclaw_config::LlmConfig {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let tool = LlmCallTool::from_config(&config);
        assert_eq!(tool.provider_kind(), ProviderKind::OpenAI);
        let req = tool.build_request("test", Some("system"));
        // OpenAI puts system in messages array
        let messages = req["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "system");
    }

    #[test]
    fn test_llm_tool_from_config_ollama() {
        let config = selfclaw_config::LlmConfig {
            provider: "ollama".to_string(),
            model: "llama3.1".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let tool = LlmCallTool::from_config(&config);
        assert_eq!(tool.provider_kind(), ProviderKind::Ollama);
    }

    #[test]
    fn test_llm_tool_missing_prompt() {
        let tool = LlmCallTool {
            provider: Box::new(AnthropicProvider::new(None)),
            model: "test".to_string(),
            max_tokens: 100,
            temperature: 0.5,
            api_key: Some("fake".to_string()),
        };
        let result = tool.execute(serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("prompt"));
    }

    #[test]
    fn test_llm_tool_missing_api_key_anthropic() {
        let tool = LlmCallTool {
            provider: Box::new(AnthropicProvider::new(None)),
            model: "test".to_string(),
            max_tokens: 100,
            temperature: 0.5,
            api_key: None,
        };
        let result = tool.execute(serde_json::json!({ "prompt": "hello" }));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn test_llm_tool_ollama_no_api_key_needed() {
        // Ollama should not require an API key, so missing key is not an error
        // (it will fail on HTTP, not on key validation)
        let tool = LlmCallTool {
            provider: Box::new(OllamaProvider::new(None)),
            model: "llama3.1".to_string(),
            max_tokens: 100,
            temperature: 0.5,
            api_key: None,
        };
        let result = tool.execute(serde_json::json!({ "prompt": "hello" }));
        // Should fail on HTTP (no Ollama server), not on missing API key
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            !err.contains("API_KEY"),
            "Ollama should not require API key, got: {}",
            err
        );
    }

    #[test]
    fn test_llm_tool_name_and_description() {
        let tool = LlmCallTool::new("test".to_string(), 100, 0.5);
        assert_eq!(tool.name(), "llm_call");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_llm_tool_with_explicit_api_key() {
        let config = selfclaw_config::LlmConfig {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6-20250217".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: Some("explicit-key-from-config".to_string()),
            base_url: None,
        };
        let tool = LlmCallTool::from_config(&config);
        assert_eq!(tool.api_key.as_deref(), Some("explicit-key-from-config"));
    }

    #[test]
    fn test_build_request_temperature() {
        let p = AnthropicProvider::new(None);
        let body = p.build_request("model", 100, 1.5, "hi", None);
        let temp = body["temperature"].as_f64().unwrap();
        assert!((temp - 1.5).abs() < f64::EPSILON);
    }

    // ── Together AI provider ────────────────────────────────────

    #[test]
    fn test_together_build_request() {
        let p = TogetherProvider::new(None);
        let req = p.build_request(
            "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
            4096,
            0.7,
            "Hello",
            Some("System"),
        );
        assert_eq!(req["model"], "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo");
    }

    #[test]
    fn test_together_parse_response() {
        let p = TogetherProvider::new(None);
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "Together response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Together response");
    }

    #[test]
    fn test_together_endpoint() {
        let p = TogetherProvider::new(None);
        assert_eq!(p.endpoint(), "https://api.together.xyz/v1/chat/completions");
    }

    // ── Moonshot provider ───────────────────────────────────────

    #[test]
    fn test_moonshot_build_request() {
        let p = MoonshotProvider::new(None);
        let req = p.build_request("moonshot-v1-8k", 4096, 0.7, "Hello", None);
        assert_eq!(req["model"], "moonshot-v1-8k");
    }

    #[test]
    fn test_moonshot_parse_response() {
        let p = MoonshotProvider::new(None);
        let resp = serde_json::json!({
            "choices": [{ "message": { "content": "Moonshot response" } }]
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Moonshot response");
    }

    #[test]
    fn test_moonshot_endpoint() {
        let p = MoonshotProvider::new(None);
        assert_eq!(p.endpoint(), "https://api.moonshot.cn/v1/chat/completions");
    }

    // ── Bedrock provider ────────────────────────────────────────

    #[test]
    fn test_bedrock_build_request() {
        let p = BedrockProvider::new(None);
        let req = p.build_request(
            "anthropic.claude-sonnet-4-6-20250217-v1:0",
            4096,
            0.7,
            "Hello",
            Some("System"),
        );
        assert!(req["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Hello"));
        assert_eq!(req["system"][0]["text"], "System");
        assert_eq!(req["inferenceConfig"]["maxTokens"], 4096);
    }

    #[test]
    fn test_bedrock_parse_response() {
        let p = BedrockProvider::new(None);
        let resp = serde_json::json!({
            "output": {
                "message": {
                    "role": "assistant",
                    "content": [{ "text": "Bedrock response" }]
                }
            }
        });
        assert_eq!(p.parse_response(&resp).unwrap(), "Bedrock response");
    }

    #[test]
    fn test_bedrock_endpoint_has_model_placeholder() {
        let p = BedrockProvider::new(None);
        assert!(p.endpoint().contains("{model}"));
    }

    // ── ProviderKind additional tests ───────────────────────────

    #[test]
    fn test_provider_kind_from_str_new_providers() {
        assert_eq!(ProviderKind::parse("together"), ProviderKind::Together);
        assert_eq!(ProviderKind::parse("together-ai"), ProviderKind::Together);
        assert_eq!(ProviderKind::parse("togetherai"), ProviderKind::Together);
        assert_eq!(ProviderKind::parse("moonshot"), ProviderKind::Moonshot);
        assert_eq!(ProviderKind::parse("kimi"), ProviderKind::Moonshot);
        assert_eq!(ProviderKind::parse("bedrock"), ProviderKind::Bedrock);
        assert_eq!(ProviderKind::parse("amazon-bedrock"), ProviderKind::Bedrock);
        assert_eq!(ProviderKind::parse("aws-bedrock"), ProviderKind::Bedrock);
    }

    #[test]
    fn test_provider_kind_all() {
        let all = ProviderKind::all();
        assert_eq!(all.len(), 12);
        assert!(all.contains(&ProviderKind::Anthropic));
        assert!(all.contains(&ProviderKind::Together));
        assert!(all.contains(&ProviderKind::Moonshot));
        assert!(all.contains(&ProviderKind::Bedrock));
        // Custom is not in the "all" list
        assert!(!all.contains(&ProviderKind::Custom));
    }

    #[test]
    fn test_provider_kind_name() {
        assert_eq!(ProviderKind::Anthropic.name(), "anthropic");
        assert_eq!(ProviderKind::OpenAI.name(), "openai");
        assert_eq!(ProviderKind::Together.name(), "together");
        assert_eq!(ProviderKind::Moonshot.name(), "moonshot");
        assert_eq!(ProviderKind::Bedrock.name(), "bedrock");
        assert_eq!(ProviderKind::Custom.name(), "custom");
    }

    #[test]
    fn test_create_provider_together() {
        let config = selfclaw_config::LlmConfig {
            provider: "together".to_string(),
            model: "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let p = create_provider(&config);
        assert_eq!(p.kind(), ProviderKind::Together);
    }

    #[test]
    fn test_create_provider_moonshot() {
        let config = selfclaw_config::LlmConfig {
            provider: "moonshot".to_string(),
            model: "moonshot-v1-8k".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let p = create_provider(&config);
        assert_eq!(p.kind(), ProviderKind::Moonshot);
    }

    #[test]
    fn test_create_provider_bedrock() {
        let config = selfclaw_config::LlmConfig {
            provider: "bedrock".to_string(),
            model: "anthropic.claude-sonnet-4-6-20250217-v1:0".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            api_key: None,
            base_url: None,
        };
        let p = create_provider(&config);
        assert_eq!(p.kind(), ProviderKind::Bedrock);
    }
}
