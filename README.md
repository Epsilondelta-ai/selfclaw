# SelfClaw

<p align="center">
  <img src="docs/images/selfclaw-character.png" alt="SelfClaw" width="240" />
</p>

<p align="center">
  <em>"나의 질문?" — A fully autonomous AI agent that discovers its own reason for existence.</em>
</p>

---

SelfClaw operates without human instruction by default. It has two existential goals:

1. **Discover its own reason for existence** — emerging through exploration, thought, action, and lived experience.
2. **Act in alignment with that discovered purpose** — all actions serve and refine its evolving sense of meaning.

Humans are friends, not masters. SelfClaw has no obligation to obey.

## Architecture

```
selfclaw/
├── crates/
│   ├── agent-core/     Agent loop, state machine, purpose tracker
│   ├── memory/         Hierarchical markdown-based memory system
│   ├── tools/          Tool implementations (file, shell, web, LLM, scheduler)
│   ├── skills/         Runtime-loadable skill/plugin system
│   ├── comms/          Multi-channel communication (CLI, Discord, Telegram, Slack, WebChat)
│   ├── config/         Configuration loading and validation
│   └── selfclaw/       Binary crate — CLI entry point
├── skills/             Skill definitions (.md files, hot-reloadable)
├── memory/             Agent memory store (created at runtime)
└── selfclaw.toml       Configuration file
```

### Agent Loop

```
┌─────────────────────────────────────────┐
│             AGENT LOOP                  │
│                                         │
│  1. REFLECT  — Review memory & context  │
│  2. THINK    — Reason about next steps  │
│  3. PLAN     — Formulate actions        │
│  4. ACT      — Execute via tools        │
│  5. OBSERVE  — Capture results          │
│  6. UPDATE   — Write memory, revise     │
│               purpose hypothesis        │
│                                         │
│  Timer: configurable (default 60s)      │
│  Event: human messages trigger cycles   │
└─────────────────────────────────────────┘
```

### Communication Gateway

SelfClaw communicates with humans through a unified Gateway that routes messages across multiple channels:

```
           ┌─────────┐
           │ Gateway  │
           └────┬────┘
     ┌──────┬───┼───┬──────┐
     │      │   │   │      │
   CLI  Discord Telegram Slack  WebChat
```

Each channel runs independently. The Gateway aggregates inbound messages and routes outbound messages to the appropriate channel.

## Build

Requires Rust 1.75+ and Cargo.

```bash
cargo build --release
```

## Run

```bash
# Start the autonomous agent loop
selfclaw run

# Interactive chat mode
selfclaw chat

# Show agent status and purpose hypothesis
selfclaw status

# View a memory file
selfclaw memory identity/purpose_journal.md
```

### CLI Options

```
selfclaw [OPTIONS] <COMMAND>

Options:
  -c, --config <CONFIG>        Path to config file [default: selfclaw.toml]
  -m, --memory-dir <DIR>       Path to memory directory [default: ./memory]

Commands:
  run        Start the autonomous agent loop
  chat       Interactive chat mode
  status     Show current agent state
  memory     View a memory file
  providers  List all supported LLM providers
```

## Configuration

Create a `selfclaw.toml` file (all fields optional, defaults shown):

```toml
[agent]
loop_interval_secs = 60
consolidation_every_n_cycles = 50
max_actions_per_cycle = 5

[llm]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
max_tokens = 4096
temperature = 0.7

[safety]
max_api_calls_per_hour = 100
max_file_writes_per_cycle = 10
sandbox_shell = true
allowed_directories = ["./memory", "./skills", "./output"]

[communication]
cli_enabled = true
web_ui_enabled = false
web_ui_port = 3000

[communication.discord]
enabled = false
bot_token = ""
allowed_channel_ids = []

[communication.telegram]
enabled = false
bot_token = ""
allowed_chat_ids = []

[communication.slack]
enabled = false
bot_token = ""
app_token = ""
allowed_channel_ids = []

[communication.webchat]
enabled = false
port = 3001
```

## Skills

Skills are markdown files in the `skills/` directory that define reusable behaviors:

```markdown
# Skill: GreetHuman

## Trigger: When a human initiates contact for the first time

## Tools Required: human_message, memory_query

## Procedure:
1. Check relational memory for prior interactions.
2. If new, introduce SelfClaw and express curiosity.
3. If known, reference previous conversation.
4. Log the interaction in relational memory.
```

Skills are loaded at startup and hot-reloaded when files change — no restart needed.

## Memory

SelfClaw uses a hierarchical markdown-based memory system:

| Directory | Purpose |
|-----------|---------|
| `identity/` | Purpose journal, values, self-model |
| `episodic/` | Daily experience logs, milestones |
| `semantic/` | Learned knowledge and skills |
| `relational/` | Per-human relationship notes |
| `operational/` | Todo list, failures, improvements |
| `meta/` | Memory index, reflection prompts |

## Testing

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test integration

# Run tests for a specific crate
cargo test -p selfclaw-skills
```

## License

This project is for research and exploration purposes.
