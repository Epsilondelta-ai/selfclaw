use crate::{Tool, ToolError, ToolOutput};

/// Builds the request body for the Anthropic Messages API.
pub fn build_request_body(
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
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    if let Some(sys) = system {
        body["system"] = serde_json::json!(sys);
    }

    body
}

pub struct LlmCallTool {
    model: String,
    max_tokens: u64,
    temperature: f64,
    api_key: Option<String>,
}

impl LlmCallTool {
    pub fn new(model: String, max_tokens: u64, temperature: f64) -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        Self {
            model,
            max_tokens,
            temperature,
            api_key,
        }
    }

    pub fn from_config(config: &selfclaw_config::LlmConfig) -> Self {
        Self::new(
            config.model.clone(),
            config.max_tokens,
            config.temperature,
        )
    }

    /// Build the request for inspection/testing without making an HTTP call.
    pub fn build_request(&self, prompt: &str, system: Option<&str>) -> serde_json::Value {
        build_request_body(&self.model, self.max_tokens, self.temperature, prompt, system)
    }
}

impl Tool for LlmCallTool {
    fn name(&self) -> &str {
        "llm_call"
    }

    fn description(&self) -> &str {
        "Call the Anthropic Claude API with a prompt"
    }

    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let prompt = input
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("prompt".to_string()))?;

        let system = input
            .get("system")
            .and_then(|v| v.as_str());

        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| {
                ToolError::InvalidInput("ANTHROPIC_API_KEY environment variable not set".to_string())
            })?;

        let body = self.build_request(prompt, system);

        // Synchronous HTTP call using reqwest::blocking is not available
        // since we depend on the async version. Use tokio runtime.
        let rt = tokio::runtime::Handle::try_current();
        let response_result = match rt {
            Ok(_handle) => {
                // We're inside a tokio runtime, use spawn_blocking + block_on
                // to avoid blocking the async runtime. Actually we need to
                // do an async HTTP call. Let's use a oneshot approach.
                std::thread::scope(|s| {
                    s.spawn(|| {
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| ToolError::Http(e.to_string()))?;
                        rt.block_on(Self::do_http_call(api_key, &body))
                    })
                    .join()
                    .unwrap()
                })
            }
            Err(_) => {
                // No tokio runtime, create one
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::Http(e.to_string()))?;
                rt.block_on(Self::do_http_call(api_key, &body))
            }
        };

        match response_result {
            Ok(response_body) => Ok(ToolOutput::ok(response_body)),
            Err(e) => Err(e),
        }
    }
}

impl LlmCallTool {
    async fn do_http_call(
        api_key: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
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

    #[test]
    fn test_build_request_body_basic() {
        let body = build_request_body("claude-sonnet-4-20250514", 4096, 0.7, "Hello", None);

        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["max_tokens"], 4096);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "Hello");
        assert!(body.get("system").is_none());
    }

    #[test]
    fn test_build_request_body_with_system() {
        let body =
            build_request_body("claude-sonnet-4-20250514", 4096, 0.7, "Hello", Some("Be helpful"));

        assert_eq!(body["system"], "Be helpful");
    }

    #[test]
    fn test_llm_tool_build_request() {
        let tool = LlmCallTool::new("claude-sonnet-4-20250514".to_string(), 4096, 0.7);
        let req = tool.build_request("What is 2+2?", None);

        assert_eq!(req["model"], "claude-sonnet-4-20250514");
        assert_eq!(req["messages"][0]["content"], "What is 2+2?");
    }

    #[test]
    fn test_llm_tool_build_request_with_system() {
        let tool = LlmCallTool::new("claude-sonnet-4-20250514".to_string(), 8192, 0.5);
        let req = tool.build_request("Hello", Some("You are SelfClaw"));

        assert_eq!(req["max_tokens"], 8192);
        assert_eq!(req["system"], "You are SelfClaw");
    }

    #[test]
    fn test_llm_tool_missing_prompt() {
        // Temporarily ensure no API key so we test field validation first
        let tool = LlmCallTool {
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
    fn test_llm_tool_missing_api_key() {
        let tool = LlmCallTool {
            model: "test".to_string(),
            max_tokens: 100,
            temperature: 0.5,
            api_key: None,
        };
        let result = tool.execute(serde_json::json!({ "prompt": "hello" }));
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("ANTHROPIC_API_KEY"),
        );
    }

    #[test]
    fn test_llm_tool_name_and_description() {
        let tool = LlmCallTool::new("test".to_string(), 100, 0.5);
        assert_eq!(tool.name(), "llm_call");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_llm_tool_from_config() {
        let config = selfclaw_config::LlmConfig {
            provider: "anthropic".to_string(),
            model: "claude-opus-4-20250514".to_string(),
            max_tokens: 8192,
            temperature: 0.3,
        };
        let tool = LlmCallTool::from_config(&config);
        let req = tool.build_request("test", None);
        assert_eq!(req["model"], "claude-opus-4-20250514");
        assert_eq!(req["max_tokens"], 8192);
    }

    #[test]
    fn test_build_request_temperature() {
        let body = build_request_body("model", 100, 1.5, "hi", None);
        let temp = body["temperature"].as_f64().unwrap();
        assert!((temp - 1.5).abs() < f64::EPSILON);
    }
}
