use chrono::Utc;
use tracing::{info, warn};

use selfclaw_config::SelfClawConfig;
use selfclaw_memory::episodic::EpisodicLogger;
use selfclaw_memory::purpose::{PurposeEntry, PurposeJournal};
use selfclaw_memory::store::MemoryStore;
use selfclaw_tools::registry::ToolRegistry;
use selfclaw_tools::ToolOutput;

use crate::prompt;
use crate::purpose::{ActionSignal, PurposeTracker};
use crate::state::AgentState;

/// An LLM caller abstraction for testing.
pub trait LlmCaller: Send + Sync {
    fn call(&self, system: &str, user: &str) -> Result<String, String>;
}

/// A planned action parsed from LLM output.
#[derive(Debug, Clone)]
pub struct PlannedAction {
    pub tool_name: String,
    pub input: serde_json::Value,
}

/// The result of executing one full agent loop cycle.
#[derive(Debug)]
pub struct CycleResult {
    pub reflection: String,
    pub thought: String,
    pub plan: Vec<PlannedAction>,
    pub observations: Vec<ToolOutput>,
    pub state_after: AgentState,
}

/// The main agent loop controller.
pub struct AgentLoop<S: MemoryStore, L: LlmCaller> {
    pub config: SelfClawConfig,
    pub store: S,
    pub tools: ToolRegistry,
    pub llm: L,
    pub state: AgentState,
    pub purpose: PurposeTracker,
    pub cycle_count: u64,
}

impl<S: MemoryStore, L: LlmCaller> AgentLoop<S, L> {
    pub fn new(config: SelfClawConfig, store: S, tools: ToolRegistry, llm: L) -> Self {
        Self {
            config,
            store,
            tools,
            llm,
            state: AgentState::Idle,
            purpose: PurposeTracker::new(),
            cycle_count: 0,
        }
    }

    /// Execute one full loop cycle: Reflect -> Think -> Plan -> Act -> Observe -> Update.
    pub fn run_cycle(&mut self) -> Result<CycleResult, String> {
        let now = Utc::now();
        let date = now.format("%Y-%m-%d").to_string();
        let time = now.format("%H:%M:%S UTC").to_string();

        // ── REFLECT ──
        self.state = self.state.next(); // Idle -> Reflecting
        info!(state = %self.state, "Starting cycle {}", self.cycle_count + 1);
        let reflection_context = prompt::build_reflection_context(&self.store, &date);
        let system_prompt =
            prompt::build_system_prompt(&self.store, &self.purpose, &self.tools.names());

        // ── THINK ──
        self.state = self.state.next(); // Reflecting -> Thinking
        let think_prompt = format!(
            "## Current Context\n\n{}\n\n---\n\n\
             Based on the above context, what should you do next and why? \
             Consider your purpose hypothesis and recent experiences. \
             Respond with your reasoning.",
            reflection_context
        );
        let thought = self
            .llm
            .call(&system_prompt, &think_prompt)
            .unwrap_or_else(|e| format!("(LLM error during THINK: {})", e));

        // ── PLAN ──
        self.state = self.state.next(); // Thinking -> Planning
        let plan_prompt = format!(
            "## Your Reasoning\n\n{}\n\n---\n\n\
             Based on your reasoning above, produce a concrete action plan.\n\
             Format each action as a JSON object on its own line:\n\
             ```json\n{{\"tool\": \"tool_name\", \"input\": {{...}}}}\n```\n\
             If no action is needed, respond with: NO_ACTION\n\
             Available tools: {:?}",
            thought,
            self.tools.names()
        );
        let plan_response = self
            .llm
            .call(&system_prompt, &plan_prompt)
            .unwrap_or_else(|e| format!("(LLM error during PLAN: {})", e));

        let plan = parse_plan(&plan_response);

        // ── ACT ──
        self.state = self.state.next(); // Planning -> Acting
        let mut observations = Vec::new();
        let max_actions = self.config.agent.max_actions_per_cycle as usize;

        for (i, action) in plan.iter().enumerate() {
            if i >= max_actions {
                warn!("Hit max_actions_per_cycle limit ({})", max_actions);
                break;
            }

            if let Some(tool) = self.tools.get(&action.tool_name) {
                match tool.execute(action.input.clone()) {
                    Ok(output) => {
                        info!(tool = %action.tool_name, "Tool executed successfully");
                        observations.push(output);
                    }
                    Err(e) => {
                        warn!(tool = %action.tool_name, error = %e, "Tool execution failed");
                        observations.push(ToolOutput::error(&e.to_string()));
                    }
                }
            } else {
                warn!(tool = %action.tool_name, "Tool not found in registry");
                observations.push(ToolOutput::error(&format!(
                    "tool not found: {}",
                    action.tool_name
                )));
            }
        }

        // ── OBSERVE ──
        self.state = self.state.next(); // Acting -> Observing
        let observation_summary = observations
            .iter()
            .enumerate()
            .map(|(i, o)| {
                format!(
                    "Action {}: success={}, data={}",
                    i + 1,
                    o.success,
                    serde_json::to_string(&o.data).unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Evaluate actions for purpose tracking
        for obs in &observations {
            let signal = if obs.success {
                ActionSignal::Reinforcing
            } else {
                ActionSignal::Neutral
            };
            self.purpose.evaluate_action(signal);
        }

        // ── UPDATE ──
        self.state = self.state.next(); // Observing -> Updating
        let episodic_logger = EpisodicLogger::new(&self.store);
        let log_content = format!(
            "**Cycle {}**\n\n\
             **Thought:** {}\n\n\
             **Actions planned:** {}\n\n\
             **Observations:**\n{}",
            self.cycle_count + 1,
            thought.chars().take(500).collect::<String>(),
            plan.len(),
            observation_summary
        );
        let _ = episodic_logger.log_for_date(&date, &log_content, &time);

        // Optionally update purpose journal if confidence warrants
        if self.purpose.should_revise() && self.purpose.has_hypothesis() {
            let journal = PurposeJournal::new(&self.store);
            let entry = PurposeEntry {
                timestamp: now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                hypothesis: self
                    .purpose
                    .current_hypothesis
                    .clone()
                    .unwrap_or_default(),
                confidence_score: self.purpose.confidence as f64,
                evidence: format!("Confidence dropped below threshold after cycle {}", self.cycle_count + 1),
            };
            let _ = journal.append_entry(&entry);
        }

        // Return to idle
        self.state = self.state.next(); // Updating -> Idle
        self.cycle_count += 1;

        Ok(CycleResult {
            reflection: reflection_context,
            thought,
            plan,
            observations,
            state_after: self.state,
        })
    }

    /// Run the agent loop continuously with the configured interval.
    pub async fn run(&mut self) -> Result<(), String> {
        let interval = std::time::Duration::from_secs(self.config.agent.loop_interval_secs);

        loop {
            match self.run_cycle() {
                Ok(result) => {
                    info!(
                        cycle = self.cycle_count,
                        actions = result.plan.len(),
                        "Cycle complete"
                    );
                }
                Err(e) => {
                    warn!(error = %e, "Cycle failed");
                }
            }
            tokio::time::sleep(interval).await;
        }
    }
}

/// Parse LLM plan output into structured actions.
pub fn parse_plan(response: &str) -> Vec<PlannedAction> {
    if response.contains("NO_ACTION") {
        return Vec::new();
    }

    let mut actions = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();
        // Try to parse lines that look like JSON objects
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(tool) = val.get("tool").and_then(|v| v.as_str()) {
                    let input = val.get("input").cloned().unwrap_or(serde_json::Value::Null);
                    actions.push(PlannedAction {
                        tool_name: tool.to_string(),
                        input,
                    });
                }
            }
        }
    }

    actions
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use selfclaw_memory::store::FileMemoryStore;
    use selfclaw_tools::{Tool, ToolError};
    use tempfile::TempDir;

    // ── Mock LLM ──

    struct MockLlm {
        think_response: String,
        plan_response: String,
    }

    impl MockLlm {
        fn new(think: &str, plan: &str) -> Self {
            Self {
                think_response: think.to_string(),
                plan_response: plan.to_string(),
            }
        }
    }

    impl LlmCaller for MockLlm {
        fn call(&self, _system: &str, user: &str) -> Result<String, String> {
            if user.contains("what should you do next") {
                Ok(self.think_response.clone())
            } else if user.contains("action plan") {
                Ok(self.plan_response.clone())
            } else {
                Ok("(unrecognized prompt)".to_string())
            }
        }
    }

    struct FailingLlm;

    impl LlmCaller for FailingLlm {
        fn call(&self, _system: &str, _user: &str) -> Result<String, String> {
            Err("LLM unavailable".to_string())
        }
    }

    // ── Mock Tool ──

    struct EchoTool;

    impl Tool for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Echoes input"
        }
        fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::ok(input))
        }
    }

    // ── Helpers ──

    fn setup() -> (TempDir, FileMemoryStore) {
        let dir = TempDir::new().unwrap();
        let store = FileMemoryStore::new(dir.path());
        store
            .write("identity/self_model.md", "# Self Model\n\nI am SelfClaw.")
            .unwrap();
        store
            .write("identity/values.md", "# Values\n\n- Curiosity")
            .unwrap();
        store
            .write(
                "identity/purpose_journal.md",
                "# Purpose Journal\n\n## Entries\n",
            )
            .unwrap();
        store
            .write("meta/memory_index.md", "# Memory Index")
            .unwrap();
        (dir, store)
    }

    fn default_config() -> SelfClawConfig {
        SelfClawConfig::default()
    }

    // ── parse_plan tests ──

    #[test]
    fn test_parse_plan_with_actions() {
        let response = r#"Here's what I'll do:
{"tool": "file_read", "input": {"path": "test.md"}}
{"tool": "echo", "input": {"message": "hello"}}
"#;
        let actions = parse_plan(response);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].tool_name, "file_read");
        assert_eq!(actions[1].tool_name, "echo");
    }

    #[test]
    fn test_parse_plan_no_action() {
        let actions = parse_plan("NO_ACTION - nothing to do right now.");
        assert!(actions.is_empty());
    }

    #[test]
    fn test_parse_plan_invalid_json() {
        let actions = parse_plan("not json at all");
        assert!(actions.is_empty());
    }

    #[test]
    fn test_parse_plan_missing_tool_field() {
        let actions = parse_plan(r#"{"input": {"x": 1}}"#);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_parse_plan_missing_input() {
        let actions = parse_plan(r#"{"tool": "echo"}"#);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].input.is_null());
    }

    // ── AgentLoop tests ──

    #[test]
    fn test_run_cycle_no_action() {
        let (_dir, store) = setup();
        let llm = MockLlm::new(
            "I should reflect on my existence.",
            "NO_ACTION - let me just think for now.",
        );
        let tools = ToolRegistry::new();

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        let result = agent.run_cycle().unwrap();

        assert_eq!(result.state_after, AgentState::Idle);
        assert!(result.plan.is_empty());
        assert!(result.observations.is_empty());
        assert!(result.thought.contains("reflect"));
        assert_eq!(agent.cycle_count, 1);
    }

    #[test]
    fn test_run_cycle_with_tool_execution() {
        let (_dir, store) = setup();
        let llm = MockLlm::new(
            "I want to echo a message.",
            r#"{"tool": "echo", "input": {"msg": "hello"}}"#,
        );
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(EchoTool));

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        let result = agent.run_cycle().unwrap();

        assert_eq!(result.plan.len(), 1);
        assert_eq!(result.plan[0].tool_name, "echo");
        assert_eq!(result.observations.len(), 1);
        assert!(result.observations[0].success);
    }

    #[test]
    fn test_run_cycle_missing_tool() {
        let (_dir, store) = setup();
        let llm = MockLlm::new(
            "I need a tool that doesn't exist.",
            r#"{"tool": "nonexistent", "input": {}}"#,
        );
        let tools = ToolRegistry::new();

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        let result = agent.run_cycle().unwrap();

        assert_eq!(result.observations.len(), 1);
        assert!(!result.observations[0].success);
    }

    #[test]
    fn test_run_cycle_llm_failure_graceful() {
        let (_dir, store) = setup();
        let llm = FailingLlm;
        let tools = ToolRegistry::new();

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        let result = agent.run_cycle().unwrap();

        // Even with LLM failure, the cycle completes gracefully
        assert_eq!(result.state_after, AgentState::Idle);
        assert!(result.thought.contains("LLM error"));
    }

    #[test]
    fn test_run_cycle_writes_episodic_log() {
        let (_dir, store) = setup();
        let llm = MockLlm::new("Thinking...", "NO_ACTION");
        let tools = ToolRegistry::new();

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        agent.run_cycle().unwrap();

        // Check that an episodic log was written for today
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let log_path = format!("episodic/{}.md", today);
        assert!(agent.store.exists(&log_path));
        let log = agent.store.read(&log_path).unwrap();
        assert!(log.contains("Cycle 1"), "log: {}", log);
    }

    #[test]
    fn test_run_cycle_respects_max_actions() {
        let (_dir, store) = setup();
        // Plan 10 actions but config allows only 5 (default)
        let plan_lines: Vec<String> = (0..10)
            .map(|i| format!(r#"{{"tool": "echo", "input": {{"n": {}}}}}"#, i))
            .collect();
        let llm = MockLlm::new("Execute many.", &plan_lines.join("\n"));
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(EchoTool));

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        let result = agent.run_cycle().unwrap();

        assert_eq!(result.plan.len(), 10); // All planned
        assert_eq!(result.observations.len(), 5); // Only 5 executed
    }

    #[test]
    fn test_state_progresses_through_cycle() {
        let (_dir, store) = setup();
        let llm = MockLlm::new("Thinking.", "NO_ACTION");
        let tools = ToolRegistry::new();

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        assert_eq!(agent.state, AgentState::Idle);

        agent.run_cycle().unwrap();
        assert_eq!(agent.state, AgentState::Idle); // Back to idle after cycle
    }

    #[test]
    fn test_cycle_count_increments() {
        let (_dir, store) = setup();
        let llm = MockLlm::new("Thinking.", "NO_ACTION");
        let tools = ToolRegistry::new();

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        assert_eq!(agent.cycle_count, 0);

        agent.run_cycle().unwrap();
        assert_eq!(agent.cycle_count, 1);

        agent.run_cycle().unwrap();
        assert_eq!(agent.cycle_count, 2);
    }

    #[test]
    fn test_purpose_confidence_updates_on_success() {
        let (_dir, store) = setup();
        let llm = MockLlm::new(
            "Echo test.",
            r#"{"tool": "echo", "input": {"x": 1}}"#,
        );
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(EchoTool));

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);
        agent.purpose.set_hypothesis("Test purpose".to_string(), 0.5);

        let initial = agent.purpose.confidence;
        agent.run_cycle().unwrap();
        assert!(agent.purpose.confidence > initial); // Reinforcing
    }

    #[test]
    fn test_multiple_cycles() {
        let (_dir, store) = setup();
        let llm = MockLlm::new("Cycle.", "NO_ACTION");
        let tools = ToolRegistry::new();

        let mut agent = AgentLoop::new(default_config(), store, tools, llm);

        for _ in 0..3 {
            agent.run_cycle().unwrap();
        }
        assert_eq!(agent.cycle_count, 3);
        assert_eq!(agent.state, AgentState::Idle);
    }
}
