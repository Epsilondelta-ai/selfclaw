# SelfClaw

> Also available in [Korean (한국어)](./README.ko.md).

![SelfClaw](docs/images/selfclaw-character.png)

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

## Installation

### Quick Install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash
```

This will download the binary (or build from source), initialize `~/.selfclaw/`, and launch the onboarding wizard.

Options:
- `--no-onboard` — Skip the onboarding wizard
- `--version v0.1.0` — Install a specific version
- `--brew` — Force Homebrew installation
- `--apt` — Force apt/deb installation
- `--yum` — Force yum/rpm installation
- `--source` — Force build from source

### Homebrew (macOS / Linux)

```bash
brew tap Epsilondelta-ai/tap
brew install selfclaw
```

### apt (Debian / Ubuntu)

```bash
# Download the .deb from the latest release
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw_0.1.0_amd64.deb
sudo dpkg -i selfclaw_0.1.0_amd64.deb
```

### yum / dnf (Fedora / RHEL / CentOS)

```bash
# Download the .rpm from the latest release
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw-0.1.0-1.x86_64.rpm
sudo yum localinstall selfclaw-0.1.0-1.x86_64.rpm
# Or: sudo dnf install selfclaw-0.1.0-1.x86_64.rpm
```

### From Source

```bash
git clone https://github.com/Epsilondelta-ai/selfclaw.git
cd selfclaw
cargo build --release
sudo cp target/release/selfclaw /usr/local/bin/
selfclaw init
selfclaw onboard
```

### From GitHub Releases

Download pre-built binaries from [Releases](https://github.com/Epsilondelta-ai/selfclaw/releases):

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | Apple Silicon (M1+) | `selfclaw-*-macos-aarch64.tar.gz` |
| macOS | Intel | `selfclaw-*-macos-x86_64.tar.gz` |
| Linux | x86_64 | `selfclaw-*-linux-x86_64.tar.gz` |
| Linux | ARM64 | `selfclaw-*-linux-aarch64.tar.gz` |
| Debian/Ubuntu | x86_64 | `selfclaw_*_amd64.deb` |
| RHEL/Fedora | x86_64 | `selfclaw-*-1.x86_64.rpm` |

## Getting Started

```bash
# 1. Initialize directory structure (~/.selfclaw/)
selfclaw init

# 2. Interactive setup wizard (LLM provider, API key, daemon)
selfclaw onboard

# 3. Start the agent (foreground)
selfclaw run

# Or start as a background daemon
selfclaw daemon start
```

### CLI Commands

```
selfclaw [OPTIONS] <COMMAND>

Setup:
  init         Initialize ~/.selfclaw/ directory structure
  onboard      Interactive onboarding wizard
  doctor       Diagnose installation health

Agent:
  run          Start the autonomous agent loop
  chat         Interactive chat mode
  status       Show current agent state
  memory       View a memory file
  providers    List all supported LLM providers

Daemon:
  daemon start     Start as background daemon
  daemon stop      Stop the daemon
  daemon restart   Restart the daemon (stop + start)
  daemon status    Check daemon status
  daemon install   Install as system service (launchd/systemd)
  daemon uninstall Remove system service

Options:
  -c, --config <CONFIG>   Path to config file [default: ~/.selfclaw/config.toml]
  -m, --memory-dir <DIR>  Path to memory directory [default: ~/.selfclaw/memory]
```

## Configuration

Create a `selfclaw.toml` file (all fields optional, defaults shown):

```toml
[agent]
loop_interval_secs = 60
consolidation_every_n_cycles = 50
max_actions_per_cycle = 5
skills_dirs = ["~/.agents/skills", "~/.selfclaw/skills"]

[llm]
provider = "anthropic"
model = "claude-sonnet-4-6-20250217"
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

Skills are markdown files that define reusable behaviors. SelfClaw loads skills from multiple directories (configurable via `skills_dirs`):

| Directory | Purpose |
|-----------|---------|
| `~/.agents/skills/` | Shared across AI agents (AntiGravity, Cursor, etc.) |
| `~/.selfclaw/skills/` | SelfClaw-specific skills |

When the same skill name exists in multiple directories, the first directory in the list wins.

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
