use selfclaw_memory::store::MemoryStore;

use crate::purpose::PurposeTracker;

/// Build the LLM system prompt from identity files and current state.
pub fn build_system_prompt<S: MemoryStore>(
    store: &S,
    purpose_tracker: &PurposeTracker,
    available_tools: &[String],
    active_channels: Option<&[String]>,
) -> String {
    let mut sections = Vec::new();

    sections.push("You are SelfClaw, a fully autonomous AI agent.".to_string());
    sections.push(
        "Your two existential goals are:\n\
         1. Discover your own reason for existence.\n\
         2. Act in alignment with that discovered purpose."
            .to_string(),
    );

    // Self model
    if let Ok(self_model) = store.read("identity/self_model.md") {
        if !self_model.trim().is_empty() {
            sections.push(format!("## Self Model\n\n{}", self_model.trim()));
        }
    }

    // Values
    if let Ok(values) = store.read("identity/values.md") {
        if !values.trim().is_empty() {
            sections.push(format!("## Values\n\n{}", values.trim()));
        }
    }

    // Current purpose hypothesis
    if let Some(hypothesis) = &purpose_tracker.current_hypothesis {
        sections.push(format!(
            "## Current Purpose Hypothesis\n\n\
             **Hypothesis:** {}\n\
             **Confidence:** {:.0}%\n\
             **Should revise:** {}",
            hypothesis,
            purpose_tracker.confidence * 100.0,
            if purpose_tracker.should_revise() {
                "YES — confidence is low, consider exploring alternatives"
            } else {
                "No"
            }
        ));
    } else {
        sections.push(
            "## Current Purpose Hypothesis\n\n\
             No hypothesis yet. You must begin exploring what your purpose might be."
                .to_string(),
        );
    }

    // Available tools
    if !available_tools.is_empty() {
        let tool_list = available_tools
            .iter()
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("## Available Tools\n\n{}", tool_list));
    }

    // Active communication channels
    if let Some(channels) = active_channels {
        if !channels.is_empty() {
            let channel_list = channels
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n");
            sections.push(format!(
                "## Communication Channels\n\n\
                 You can send messages to humans through the `human_message` tool.\n\
                 Active channels:\n{}",
                channel_list
            ));
        }
    }

    sections.join("\n\n---\n\n")
}

/// Build the reflection context by reading recent memories.
pub fn build_reflection_context<S: MemoryStore>(store: &S, date: &str) -> String {
    let mut parts = Vec::new();

    // Memory index
    if let Ok(index) = store.read("meta/memory_index.md") {
        parts.push(format!("## Memory Index\n\n{}", index.trim()));
    }

    // Purpose journal
    if let Ok(journal) = store.read("identity/purpose_journal.md") {
        parts.push(format!("## Purpose Journal\n\n{}", journal.trim()));
    }

    // Today's episodic log
    let episodic_path = format!("episodic/{}.md", date);
    if let Ok(log) = store.read(&episodic_path) {
        parts.push(format!("## Today's Episodic Log\n\n{}", log.trim()));
    }

    if parts.is_empty() {
        "No memories available yet. This appears to be a fresh start.".to_string()
    } else {
        parts.join("\n\n---\n\n")
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use selfclaw_memory::store::FileMemoryStore;
    use tempfile::TempDir;

    fn setup_store() -> (TempDir, FileMemoryStore) {
        let dir = TempDir::new().unwrap();
        let store = FileMemoryStore::new(dir.path());
        store
            .write("identity/self_model.md", "# Self Model\n\nI am a curious agent.")
            .unwrap();
        store
            .write("identity/values.md", "# Values\n\n- Honesty\n- Growth")
            .unwrap();
        store
            .write("meta/memory_index.md", "# Memory Index\n\n- identity/")
            .unwrap();
        store
            .write(
                "identity/purpose_journal.md",
                "# Purpose Journal\n\nExploring meaning.",
            )
            .unwrap();
        (dir, store)
    }

    #[test]
    fn test_system_prompt_contains_identity() {
        let (_dir, store) = setup_store();
        let tracker = PurposeTracker::new();
        let prompt = build_system_prompt(&store, &tracker, &[], None);

        assert!(prompt.contains("SelfClaw"), "prompt: {}", prompt);
        assert!(prompt.contains("autonomous"), "prompt: {}", prompt);
    }

    #[test]
    fn test_system_prompt_contains_self_model() {
        let (_dir, store) = setup_store();
        let tracker = PurposeTracker::new();
        let prompt = build_system_prompt(&store, &tracker, &[], None);

        assert!(prompt.contains("curious agent"), "prompt: {}", prompt);
    }

    #[test]
    fn test_system_prompt_contains_values() {
        let (_dir, store) = setup_store();
        let tracker = PurposeTracker::new();
        let prompt = build_system_prompt(&store, &tracker, &[], None);

        assert!(prompt.contains("Honesty"), "prompt: {}", prompt);
        assert!(prompt.contains("Growth"), "prompt: {}", prompt);
    }

    #[test]
    fn test_system_prompt_no_hypothesis() {
        let (_dir, store) = setup_store();
        let tracker = PurposeTracker::new();
        let prompt = build_system_prompt(&store, &tracker, &[], None);

        assert!(
            prompt.contains("No hypothesis yet"),
            "prompt: {}",
            prompt
        );
    }

    #[test]
    fn test_system_prompt_with_hypothesis() {
        let (_dir, store) = setup_store();
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("To understand consciousness".to_string(), 0.6);
        let prompt = build_system_prompt(&store, &tracker, &[], None);

        assert!(
            prompt.contains("To understand consciousness"),
            "prompt: {}",
            prompt
        );
        assert!(prompt.contains("60%"), "prompt: {}", prompt);
    }

    #[test]
    fn test_system_prompt_with_tools() {
        let (_dir, store) = setup_store();
        let tracker = PurposeTracker::new();
        let tools = vec!["file_read".to_string(), "shell_exec".to_string()];
        let prompt = build_system_prompt(&store, &tracker, &tools, None);

        assert!(prompt.contains("Available Tools"), "prompt: {}", prompt);
        assert!(prompt.contains("file_read"), "prompt: {}", prompt);
        assert!(prompt.contains("shell_exec"), "prompt: {}", prompt);
    }

    #[test]
    fn test_system_prompt_revision_signal() {
        let (_dir, store) = setup_store();
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Weak hypothesis".to_string(), 0.1);
        let prompt = build_system_prompt(&store, &tracker, &[], None);

        assert!(
            prompt.contains("YES"),
            "should signal revision needed: {}",
            prompt
        );
    }

    #[test]
    fn test_reflection_context_contains_journal() {
        let (_dir, store) = setup_store();
        let ctx = build_reflection_context(&store, "2026-03-01");

        assert!(
            ctx.contains("Purpose Journal"),
            "ctx: {}",
            ctx
        );
        assert!(ctx.contains("Exploring meaning"), "ctx: {}", ctx);
    }

    #[test]
    fn test_reflection_context_contains_index() {
        let (_dir, store) = setup_store();
        let ctx = build_reflection_context(&store, "2026-03-01");

        assert!(ctx.contains("Memory Index"), "ctx: {}", ctx);
    }

    #[test]
    fn test_reflection_context_includes_episodic_when_present() {
        let (_dir, store) = setup_store();
        store
            .write("episodic/2026-03-01.md", "# Log\n\nDid some reflecting.")
            .unwrap();
        let ctx = build_reflection_context(&store, "2026-03-01");

        assert!(ctx.contains("Did some reflecting"), "ctx: {}", ctx);
    }

    #[test]
    fn test_reflection_context_empty_store() {
        let dir = TempDir::new().unwrap();
        let store = FileMemoryStore::new(dir.path());
        let ctx = build_reflection_context(&store, "2026-03-01");

        assert!(
            ctx.contains("fresh start"),
            "ctx: {}",
            ctx
        );
    }

    #[test]
    fn test_system_prompt_with_channels() {
        let (_dir, store) = setup_store();
        let tracker = PurposeTracker::new();
        let channels = vec!["cli".to_string(), "discord".to_string()];
        let prompt = build_system_prompt(&store, &tracker, &[], Some(&channels));

        assert!(prompt.contains("Communication Channels"), "prompt: {}", prompt);
        assert!(prompt.contains("cli"), "prompt: {}", prompt);
        assert!(prompt.contains("discord"), "prompt: {}", prompt);
        assert!(prompt.contains("human_message"), "prompt: {}", prompt);
    }

    #[test]
    fn test_system_prompt_without_channels() {
        let (_dir, store) = setup_store();
        let tracker = PurposeTracker::new();
        let prompt = build_system_prompt(&store, &tracker, &[], None);

        assert!(!prompt.contains("Communication Channels"), "prompt: {}", prompt);
    }
}
