//! Integration tests for SelfClaw — verifies cross-crate wiring.

use std::io::Write;
use std::sync::{Arc, Mutex};

use selfclaw_agent_core::loop_runner::{AgentLoop, LlmCaller};
use selfclaw_config::SelfClawConfig;
use selfclaw_memory::store::{FileMemoryStore, MemoryStore};
use selfclaw_skills::{parse_skill, SkillRegistry, SkillWatcher};
use selfclaw_tools::registry::ToolRegistry;
use selfclaw_tools::ToolOutput;
use tempfile::TempDir;

// ── Mock LLM ──────────────────────────────────────────────────────────

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

// ── Mock Tool ─────────────────────────────────────────────────────────

struct EchoTool;

impl selfclaw_tools::Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "Echoes input"
    }
    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, selfclaw_tools::ToolError> {
        Ok(ToolOutput::ok(input))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

fn setup_memory(dir: &TempDir) -> FileMemoryStore {
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
    store
}

// ── Tests ─────────────────────────────────────────────────────────────

/// Test: Agent starts, runs one cycle, and writes an episodic log.
#[test]
fn test_agent_runs_cycle_and_writes_episodic_log() {
    let dir = TempDir::new().unwrap();
    let store = setup_memory(&dir);

    let llm = MockLlm::new(
        "I should begin exploring my existence.",
        "NO_ACTION - reflection only this cycle.",
    );
    let tools = ToolRegistry::new();
    let config = SelfClawConfig::default();

    let mut agent = AgentLoop::new(config, store, tools, llm);

    // Run one cycle
    let result = agent.run_cycle().unwrap();

    // Verify cycle completed
    assert_eq!(agent.cycle_count, 1);
    assert_eq!(
        result.state_after,
        selfclaw_agent_core::state::AgentState::Idle
    );

    // Verify episodic log was written
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let log_path = format!("episodic/{}.md", today);
    assert!(agent.store.exists(&log_path));

    let log_content = agent.store.read(&log_path).unwrap();
    assert!(
        log_content.contains("Cycle 1"),
        "expected 'Cycle 1' in log, got: {}",
        log_content
    );
}

/// Test: Agent executes a tool and records observations.
#[test]
fn test_agent_executes_tool_and_records_observations() {
    let dir = TempDir::new().unwrap();
    let store = setup_memory(&dir);

    let llm = MockLlm::new(
        "I should echo a test message.",
        r#"{"tool": "echo", "input": {"message": "hello world"}}"#,
    );
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(EchoTool));

    let config = SelfClawConfig::default();
    let mut agent = AgentLoop::new(config, store, tools, llm);

    let result = agent.run_cycle().unwrap();

    assert_eq!(result.plan.len(), 1);
    assert_eq!(result.plan[0].tool_name, "echo");
    assert_eq!(result.observations.len(), 1);
    assert!(result.observations[0].success);
}

/// Test: Skills are loaded from a directory and matched via registry.
#[test]
fn test_skills_load_and_match() {
    let skills_dir = TempDir::new().unwrap();

    // Create skill files
    let mut f1 = std::fs::File::create(skills_dir.path().join("greet.md")).unwrap();
    write!(
        f1,
        "# Skill: GreetHuman\n\
         ## Trigger: When a human initiates contact for the first time\n\
         ## Tools Required: human_message, memory_query\n\
         ## Procedure:\n\
         1. Check relational memory for prior interactions.\n\
         2. Introduce SelfClaw and express curiosity.\n"
    )
    .unwrap();

    let mut f2 = std::fs::File::create(skills_dir.path().join("research.md")).unwrap();
    write!(
        f2,
        "# Skill: Research\n\
         ## Trigger: research explore learn knowledge topic investigation\n\
         ## Tools Required: web_search, web_fetch, file_write\n\
         ## Procedure:\n\
         1. Identify the research topic.\n\
         2. Search the web for information.\n\
         3. Synthesize findings into memory.\n"
    )
    .unwrap();

    // Load skills into registry via SkillWatcher
    let registry = Arc::new(Mutex::new(SkillRegistry::new()));
    let watcher = SkillWatcher::new(skills_dir.path(), registry.clone());

    let count = watcher.initial_load().unwrap();
    assert_eq!(count, 2);

    // Test matching
    let reg = registry.lock().unwrap();
    assert_eq!(reg.count(), 2);

    // Should match Research skill
    let matched = reg.match_skill("I want to research a topic and learn about it");
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().name, "Research");

    // Should match GreetHuman skill
    let matched = reg.match_skill("A human just initiated contact for the first time");
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().name, "GreetHuman");

    // No match for unrelated context (avoid common words like "the", "a", "for")
    let matched = reg.match_skill("pizza delivery arrived quickly");
    assert!(matched.is_none());
}

/// Test: Config changes are respected by the agent.
#[test]
fn test_config_changes_respected() {
    // Custom config with max_actions_per_cycle = 2
    let config = SelfClawConfig::from_str(
        r#"
[agent]
max_actions_per_cycle = 2
"#,
    )
    .unwrap();

    assert_eq!(config.agent.max_actions_per_cycle, 2);

    let dir = TempDir::new().unwrap();
    let store = setup_memory(&dir);

    // Plan 5 actions but config allows only 2
    let plan_lines: Vec<String> = (0..5)
        .map(|i| format!(r#"{{"tool": "echo", "input": {{"n": {}}}}}"#, i))
        .collect();
    let llm = MockLlm::new("Execute many.", &plan_lines.join("\n"));

    let mut tools = ToolRegistry::new();
    tools.register(Box::new(EchoTool));

    let mut agent = AgentLoop::new(config, store, tools, llm);
    let result = agent.run_cycle().unwrap();

    // All 5 were planned, but only 2 executed due to config limit
    assert_eq!(result.plan.len(), 5);
    assert_eq!(result.observations.len(), 2);
}

/// Test: Skill parsing from markdown produces correct struct.
#[test]
fn test_skill_parsing_end_to_end() {
    let md = r#"# Skill: AutonomousJournal

## Trigger: When the agent feels the need to reflect and write in its journal

## Tools Required: file_append, memory_query

## Procedure:
1. Read recent episodic memories.
2. Reflect on patterns and insights.
3. Append an entry to the purpose journal.
4. Update the memory index.
"#;

    let skill = parse_skill(md, Some("journal.md")).unwrap();

    assert_eq!(skill.name, "AutonomousJournal");
    assert!(skill.trigger.contains("reflect"));
    assert_eq!(skill.tools_required, vec!["file_append", "memory_query"]);
    assert_eq!(skill.procedure_steps.len(), 4);
    assert!(skill.procedure_steps[0].contains("episodic memories"));
    assert!(skill.procedure_steps[2].contains("purpose journal"));
    assert_eq!(skill.source_path.as_deref(), Some("journal.md"));
}

/// Test: Multiple agent cycles accumulate correctly.
#[test]
fn test_multiple_cycles_accumulate() {
    let dir = TempDir::new().unwrap();
    let store = setup_memory(&dir);

    let llm = MockLlm::new("Thinking.", "NO_ACTION");
    let tools = ToolRegistry::new();
    let config = SelfClawConfig::default();

    let mut agent = AgentLoop::new(config, store, tools, llm);

    for _ in 0..3 {
        agent.run_cycle().unwrap();
    }

    assert_eq!(agent.cycle_count, 3);

    // Verify episodic log has entries for all cycles
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let log_path = format!("episodic/{}.md", today);
    let log_content = agent.store.read(&log_path).unwrap();
    assert!(log_content.contains("Cycle 1"));
    assert!(log_content.contains("Cycle 2"));
    assert!(log_content.contains("Cycle 3"));
}

/// Test: Skills hot-reload detects new files.
#[test]
fn test_skills_hot_reload() {
    let skills_dir = TempDir::new().unwrap();

    // Start with one skill
    let mut f1 = std::fs::File::create(skills_dir.path().join("alpha.md")).unwrap();
    write!(
        f1,
        "# Skill: Alpha\n## Trigger: alpha test\n## Procedure:\n1. Do alpha.\n"
    )
    .unwrap();

    let registry = Arc::new(Mutex::new(SkillRegistry::new()));
    let watcher = SkillWatcher::new(skills_dir.path(), registry.clone());

    let count = watcher.initial_load().unwrap();
    assert_eq!(count, 1);

    // Add a second skill file
    let mut f2 = std::fs::File::create(skills_dir.path().join("beta.md")).unwrap();
    write!(
        f2,
        "# Skill: Beta\n## Trigger: beta test\n## Procedure:\n1. Do beta.\n"
    )
    .unwrap();

    // Manual reload (simulates what the watcher callback does)
    let count = watcher.reload().unwrap();
    assert_eq!(count, 2);

    let reg = registry.lock().unwrap();
    assert!(reg.get("Alpha").is_some());
    assert!(reg.get("Beta").is_some());
}

/// Test: Default config values are sane.
#[test]
fn test_default_config_values() {
    let config = SelfClawConfig::default();

    assert_eq!(config.agent.loop_interval_secs, 60);
    assert_eq!(config.agent.consolidation_every_n_cycles, 50);
    assert_eq!(config.agent.max_actions_per_cycle, 5);
    assert_eq!(config.llm.provider, "anthropic");
    assert_eq!(config.llm.model, "claude-sonnet-4-6-20250217");
    assert_eq!(config.llm.max_tokens, 4096);
    assert!(config.safety.sandbox_shell);
    assert!(config.communication.cli_enabled);
    assert!(!config.communication.web_ui_enabled);
}
