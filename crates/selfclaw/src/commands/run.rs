use std::path::Path;
use std::sync::{Arc, Mutex};

use selfclaw_agent_core::loop_runner::{AgentLoop, LlmCaller};
use selfclaw_comms::cli::CliChannel;
use selfclaw_comms::Gateway;
use selfclaw_config::SelfClawConfig;
use selfclaw_memory::store::FileMemoryStore;
use selfclaw_skills::{SkillRegistry, SkillWatcher};
use selfclaw_tools::file::{FileAppendTool, FileReadTool, FileWriteTool};
use selfclaw_tools::registry::ToolRegistry;
use selfclaw_tools::shell::ShellExecTool;

/// Multi-provider LLM caller. Delegates to the configured provider
/// (Anthropic, OpenAI, Google, Ollama, OpenRouter, Groq, xAI, Mistral, DeepSeek, etc.)
struct ProviderLlmCaller {
    tool: selfclaw_tools::llm::LlmCallTool,
}

impl ProviderLlmCaller {
    fn new(config: &SelfClawConfig) -> Self {
        Self {
            tool: selfclaw_tools::llm::LlmCallTool::from_config(&config.llm),
        }
    }
}

impl LlmCaller for ProviderLlmCaller {
    fn call(&self, system: &str, user: &str) -> Result<String, String> {
        use selfclaw_tools::Tool;

        let input = serde_json::json!({
            "prompt": user,
            "system": system,
        });

        match self.tool.execute(input) {
            Ok(output) => {
                if output.success {
                    output.data["response"]
                        .as_str()
                        .map(|s| s.to_string())
                        .ok_or_else(|| "no response field in LLM output".to_string())
                } else {
                    Err(output.data["error"]
                        .as_str()
                        .unwrap_or("unknown LLM error")
                        .to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Start the autonomous agent loop.
pub async fn execute(config: SelfClawConfig, memory_dir: &str) -> anyhow::Result<()> {
    println!("SelfClaw v{}", selfclaw_config::version());
    println!("Starting autonomous agent loop...");
    println!("Loop interval: {}s", config.agent.loop_interval_secs);
    println!("Memory: {}", memory_dir);
    println!();

    let memory_path = Path::new(memory_dir);
    let store = FileMemoryStore::new(memory_path);

    // Register tools
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(FileReadTool::new(memory_path)));
    tools.register(Box::new(FileWriteTool::new(memory_path)));
    tools.register(Box::new(FileAppendTool::new(memory_path)));
    tools.register(Box::new(ShellExecTool::from_config(&config.safety)));

    println!("Tools registered: {:?}", tools.names());

    // Load skills from skills/ directory (prefer ~/.selfclaw/skills/)
    let skills_path = crate::home::resolve_skills_dir();
    let skills_dir = skills_path.as_path();
    let skill_registry = Arc::new(Mutex::new(SkillRegistry::new()));
    let mut skill_watcher = SkillWatcher::new(skills_dir, skill_registry.clone());

    match skill_watcher.initial_load() {
        Ok(count) => println!("Skills loaded: {}", count),
        Err(e) => eprintln!("Warning: Failed to load skills: {}", e),
    }

    // Start watching for skill file changes (hot-reload)
    match skill_watcher.start_watching() {
        Ok(()) => println!("Skills hot-reload active"),
        Err(e) => eprintln!("Warning: Skills watcher failed to start: {}", e),
    }

    // List loaded skill names
    {
        let reg = skill_registry.lock().unwrap_or_else(|e| e.into_inner());
        let names = reg.names();
        if !names.is_empty() {
            println!("Available skills: {:?}", names);
        }
    }

    // Set up gateway with CLI channel
    let mut gateway = Gateway::new();
    let cli_channel = CliChannel::new().with_prompt("you> ");
    let inbound_tx = gateway.inbound_sender();

    if config.communication.cli_enabled {
        match cli_channel.start(inbound_tx) {
            Ok(handle) => {
                gateway.register_channel(handle);
                println!("CLI channel active");
            }
            Err(e) => {
                eprintln!("Warning: CLI channel failed to start: {}", e);
            }
        }
    }

    // Set up optional channels based on config
    if let Some(ch) = selfclaw_comms::discord::from_config(&config.communication.discord) {
        let inbound_tx = gateway.inbound_sender();
        match ch.start(inbound_tx) {
            Ok(handle) => {
                gateway.register_channel(handle);
                println!("Discord channel active");
            }
            Err(e) => eprintln!("Warning: Discord channel failed: {}", e),
        }
    }

    if let Some(ch) = selfclaw_comms::telegram::from_config(&config.communication.telegram) {
        let inbound_tx = gateway.inbound_sender();
        match ch.start(inbound_tx) {
            Ok(handle) => {
                gateway.register_channel(handle);
                println!("Telegram channel active");
            }
            Err(e) => eprintln!("Warning: Telegram channel failed: {}", e),
        }
    }

    if let Some(ch) = selfclaw_comms::slack::from_config(&config.communication.slack) {
        let inbound_tx = gateway.inbound_sender();
        match ch.start(inbound_tx) {
            Ok(handle) => {
                gateway.register_channel(handle);
                println!("Slack channel active");
            }
            Err(e) => eprintln!("Warning: Slack channel failed: {}", e),
        }
    }

    if let Some(ch) = selfclaw_comms::webchat::from_config(&config.communication.webchat) {
        let inbound_tx = gateway.inbound_sender();
        match ch.start(inbound_tx) {
            Ok(handle) => {
                gateway.register_channel(handle);
                println!(
                    "WebChat channel active (port {})",
                    config.communication.webchat.port
                );
            }
            Err(e) => eprintln!("Warning: WebChat channel failed: {}", e),
        }
    }

    // Start WebSocket server for web UI if enabled
    let _ws_server = if config.communication.web_ui_enabled {
        let ws = selfclaw_comms::WebSocketServer::new(config.communication.web_ui_port);
        let inbound_tx = gateway.inbound_sender();
        match ws.start(inbound_tx) {
            Ok(handle) => {
                gateway.register_channel(handle);
                println!(
                    "WebSocket server active (port {})",
                    config.communication.web_ui_port
                );
                Some(ws)
            }
            Err(e) => {
                eprintln!("Warning: WebSocket server failed to start: {}", e);
                None
            }
        }
    } else {
        None
    };

    println!();
    println!("SelfClaw is waking up. The journey begins.");
    println!("Press Ctrl+C to stop.");
    println!();

    let _provider_kind = selfclaw_tools::llm::ProviderKind::parse(&config.llm.provider);
    println!(
        "LLM provider: {} ({})",
        config.llm.provider, config.llm.model
    );
    if let Some(ref url) = config.llm.base_url {
        println!("LLM base URL: {}", url);
    }
    println!();

    let llm = ProviderLlmCaller::new(&config);
    let mut agent = AgentLoop::new(config, store, tools, llm).with_gateway(gateway);

    let result = agent.run().await.map_err(|e| anyhow::anyhow!(e));

    // Keep skill_watcher alive for the duration of the agent loop.
    // Dropping it stops the filesystem watcher.
    skill_watcher.stop_watching();
    drop(skill_watcher);

    result
}
