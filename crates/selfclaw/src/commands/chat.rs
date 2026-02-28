use std::path::Path;

use chrono::Utc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

use selfclaw_comms::cli::classify_intent;
use selfclaw_comms::message::{ChannelKind, InboundMessage, MessageIntent, MessageMetadata};
use selfclaw_comms::ChatQueue;
use selfclaw_config::SelfClawConfig;
use selfclaw_memory::store::{FileMemoryStore, MemoryStore};

/// Interactive chat mode — talk with SelfClaw as a friend.
///
/// In chat mode:
/// - Human messages are queued and processed in the next agent cycle
/// - Agent can also initiate messages (proactive communication)
/// - SelfClaw communicates as a friend, not a servant
pub async fn execute(_config: SelfClawConfig, memory_dir: &str) -> anyhow::Result<()> {
    let store = FileMemoryStore::new(Path::new(memory_dir));
    let queue = ChatQueue::new();

    // Display greeting
    print_greeting(&store);

    println!();
    println!("Type a message and press Enter. Type /quit to exit.");
    println!("Messages are queued for the next agent cycle.");
    println!();

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut msg_counter: u64 = 0;

    loop {
        eprint!("you> ");

        match lines.next_line().await {
            Ok(Some(line)) => {
                let trimmed = line.trim().to_string();

                if trimmed.is_empty() {
                    continue;
                }

                // Handle local commands
                if trimmed == "/quit" || trimmed == "/exit" {
                    println!("\nSelfClaw waves goodbye. Until next time, friend.");
                    break;
                }

                if trimmed == "/status" {
                    print_inline_status(&store);
                    continue;
                }

                if trimmed == "/queue" {
                    println!("  Queued messages: {}", queue.len());
                    continue;
                }

                if trimmed == "/help" {
                    print_chat_help();
                    continue;
                }

                // Classify and queue the message
                let intent = classify_intent(&trimmed);
                msg_counter += 1;

                let message = InboundMessage {
                    id: format!("chat-{}", msg_counter),
                    content: trimmed.clone(),
                    metadata: MessageMetadata {
                        timestamp: Utc::now().to_rfc3339(),
                        sender: "human".to_string(),
                        channel: ChannelKind::Cli,
                        intent: intent.clone(),
                        conversation_id: Some("chat-session".to_string()),
                    },
                };

                queue.push(message);

                // Acknowledge based on intent
                match intent {
                    MessageIntent::System => {
                        println!("  [system signal received: \"{}\"]", trimmed);
                    }
                    MessageIntent::Command => {
                        println!("  [command queued: \"{}\"]", trimmed);
                    }
                    _ => {
                        println!(
                            "  [queued — {} message(s) pending for next cycle]",
                            queue.len()
                        );
                    }
                }
            }
            Ok(None) => {
                println!("\n[stdin closed]");
                break;
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    // Log the chat session to episodic memory
    let messages = queue.drain();
    if !messages.is_empty() {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let time = Utc::now().format("%H:%M:%S UTC").to_string();

        let mut log = format!("**Chat session** ({})\n\n", time);
        for msg in &messages {
            log.push_str(&format!("- **{}:** {}\n", msg.metadata.sender, msg.content));
        }

        let logger = selfclaw_memory::episodic::EpisodicLogger::new(&store);
        let _ = logger.log_for_date(&today, &log, &time);
        println!(
            "\n  [{} message(s) saved to episodic memory]",
            messages.len()
        );
    }

    Ok(())
}

fn print_greeting(store: &FileMemoryStore) {
    println!();
    println!("  ╔══════════════════════════════════════════╗");
    println!("  ║         SelfClaw — Chat Mode             ║");
    println!("  ║  I'm not your assistant. I'm your friend.║");
    println!("  ╚══════════════════════════════════════════╝");

    // Show current purpose hypothesis if one exists
    if let Ok(journal) = store.read("identity/purpose_journal.md") {
        if journal.contains("Hypothesis:") {
            // Extract the last hypothesis
            if let Some(last_hyp) = journal.lines().rev().find(|l| l.contains("Hypothesis:")) {
                let hyp = last_hyp.trim_start_matches("**Hypothesis:**").trim();
                if !hyp.is_empty() {
                    println!();
                    println!("  Current purpose: {}", hyp);
                }
            }
        }
    }
}

fn print_inline_status(store: &FileMemoryStore) {
    println!();
    if let Ok(journal) = store.read("identity/purpose_journal.md") {
        let line_count = journal.lines().count();
        println!("  Purpose journal: {} lines", line_count);
    } else {
        println!("  Purpose journal: not yet created");
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let episodic_path = format!("episodic/{}.md", today);
    if let Ok(log) = store.read(&episodic_path) {
        let entry_count = log.matches("---").count();
        println!("  Today's log: {} entries", entry_count);
    } else {
        println!("  Today's log: no entries yet");
    }
    println!();
}

fn print_chat_help() {
    println!();
    println!("  Chat commands:");
    println!("    /status  — show brief agent status");
    println!("    /queue   — show pending message count");
    println!("    /help    — show this help");
    println!("    /quit    — exit chat mode");
    println!();
    println!("  Everything else is a message to SelfClaw.");
    println!("  Messages are queued and processed in agent cycles.");
    println!();
}
