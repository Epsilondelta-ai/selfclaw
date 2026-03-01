use std::path::Path;

use chrono::Utc;

use selfclaw_config::SelfClawConfig;
use selfclaw_memory::purpose::PurposeJournal;
use selfclaw_memory::store::FileMemoryStore;
use selfclaw_memory::store::MemoryStore;

/// Show current agent state, purpose hypothesis, and recent activity.
pub fn execute(config: SelfClawConfig, memory_dir: &str) -> anyhow::Result<()> {
    let store = FileMemoryStore::new(Path::new(memory_dir));

    println!();
    println!("  SelfClaw Status");
    println!("  ═══════════════");
    println!();

    // Config info
    println!("  Configuration:");
    println!("    Loop interval:  {}s", config.agent.loop_interval_secs);
    println!("    LLM model:      {}", config.llm.model);
    println!("    CLI enabled:    {}", config.communication.cli_enabled);
    println!(
        "    Discord:        {}",
        config.communication.discord.enabled
    );
    println!(
        "    Telegram:       {}",
        config.communication.telegram.enabled
    );
    println!("    Slack:          {}", config.communication.slack.enabled);
    println!(
        "    WebChat:        {}",
        config.communication.webchat.enabled
    );
    println!();

    // Purpose hypothesis
    println!("  Purpose:");
    let journal = PurposeJournal::new(&store);
    if let Ok(content) = journal.read() {
        let entries = PurposeJournal::<FileMemoryStore>::parse_entries(&content);
        if entries.is_empty() {
            println!("    No hypothesis yet — purpose discovery has not begun.");
        } else {
            let latest = &entries[entries.len() - 1];
            println!("    Hypothesis:  {}", latest.hypothesis);
            println!("    Confidence:  {:.0}%", latest.confidence_score * 100.0);
            println!("    Timestamp:   {}", latest.timestamp);
            if !latest.evidence.is_empty() {
                println!("    Evidence:    {}", latest.evidence);
            }
            if entries.len() > 1 {
                println!("    History:     {} previous hypotheses", entries.len() - 1);
            }
        }
    } else {
        println!("    No purpose journal found. Agent has not yet run.");
    }
    println!();

    // Recent episodic activity
    println!("  Recent Activity:");
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let episodic_path = format!("episodic/{}.md", today);
    if let Ok(log) = store.read(&episodic_path) {
        let entry_count = log.matches("---").count();
        let line_count = log.lines().count();
        println!("    Today ({}):", today);
        println!("      Entries:  {}", entry_count);
        println!("      Lines:    {}", line_count);

        // Show the last few lines as a preview
        let last_lines: Vec<&str> = log.lines().rev().take(5).collect();
        if !last_lines.is_empty() {
            println!("      Latest:");
            for line in last_lines.iter().rev() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    println!("        {}", trimmed);
                }
            }
        }
    } else {
        println!("    No activity recorded today.");
    }
    println!();

    // Memory overview
    println!("  Memory:");
    if let Ok(index) = store.read("meta/memory_index.md") {
        let file_count = index.lines().filter(|l| l.starts_with("- ")).count();
        println!("    Indexed files: {}", file_count);
    } else {
        println!("    No memory index found.");
    }

    // Check key identity files
    let identity_files = [
        ("identity/self_model.md", "Self model"),
        ("identity/values.md", "Values"),
        ("identity/purpose_journal.md", "Purpose journal"),
    ];
    for (path, label) in &identity_files {
        let status = if store.exists(path) {
            "exists"
        } else {
            "missing"
        };
        println!("    {}: {}", label, status);
    }
    println!();

    Ok(())
}
