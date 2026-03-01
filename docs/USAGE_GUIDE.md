# SelfClaw Usage Guide

> Also available in [Korean (한국어)](./USAGE_GUIDE.ko.md).

## Table of Contents

1. [Installation](#installation)
2. [Getting Started](#getting-started)
3. [Onboarding](#onboarding)
4. [CLI Commands](#cli-commands)
5. [Configuration (selfclaw.toml)](#configuration)
6. [LLM Providers](#llm-providers)
7. [Agent Loop](#agent-loop)
8. [Memory System](#memory-system)
9. [Skills System](#skills-system)
10. [Communication Channels](#communication-channels)
11. [Web UI](#web-ui)
12. [WebSocket Protocol](#websocket-protocol)
13. [Tools](#tools)
14. [Safety Guardrails](#safety-guardrails)
15. [Development & Testing](#development--testing)
16. [Troubleshooting](#troubleshooting)

---

## Installation

### Method A: Installer Script (recommended)

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash

# Skip onboarding wizard
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash -s -- --no-onboard

# Install specific version
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash -s -- --version v0.1.0
```

The installer script will:
1. Detect your platform (macOS/Linux, x86_64/aarch64)
2. Download the pre-built binary from GitHub Releases (or build from source if unavailable)
3. Install to `/usr/local/bin/` (customizable via `SELFCLAW_INSTALL_DIR`)
4. Run `selfclaw init` to create `~/.selfclaw/`
5. Launch the onboarding wizard

Installer options:

| Flag | Description |
|------|-------------|
| `--no-onboard` | Skip the onboarding wizard |
| `--version VER` | Install a specific version (e.g. `v0.1.0`) |
| `--brew` | Force Homebrew installation |
| `--apt` | Force apt/deb installation |
| `--yum` | Force yum/rpm installation |
| `--source` | Force build from source |

### Method B: Homebrew (macOS / Linux)

```bash
brew tap Epsilondelta-ai/tap
brew install selfclaw
```

After installation:

```bash
selfclaw init
selfclaw onboard
```

### Method C: apt (Debian / Ubuntu)

```bash
# Download the .deb from the latest release
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw_0.1.0_amd64.deb
sudo dpkg -i selfclaw_0.1.0_amd64.deb
```

The post-install script automatically runs `selfclaw init`.

### Method D: yum / dnf (Fedora / RHEL / CentOS)

```bash
# Download the .rpm from the latest release
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw-0.1.0-1.x86_64.rpm
sudo yum localinstall selfclaw-0.1.0-1.x86_64.rpm
# Or: sudo dnf install selfclaw-0.1.0-1.x86_64.rpm
```

### Method E: Build from Source

Requires Rust 1.75+ and Cargo.

```bash
git clone https://github.com/Epsilondelta-ai/selfclaw.git
cd selfclaw
cargo build --release
sudo cp target/release/selfclaw /usr/local/bin/
selfclaw init
selfclaw onboard
```

### Method F: GitHub Releases

Download pre-built binaries from [Releases](https://github.com/Epsilondelta-ai/selfclaw/releases):

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | Apple Silicon (M1/M2/M3/M4) | `selfclaw-*-macos-aarch64.tar.gz` |
| macOS | Intel | `selfclaw-*-macos-x86_64.tar.gz` |
| Linux | x86_64 | `selfclaw-*-linux-x86_64.tar.gz` |
| Linux | ARM64 | `selfclaw-*-linux-aarch64.tar.gz` |
| Debian/Ubuntu | x86_64 | `selfclaw_*_amd64.deb` |
| RHEL/Fedora | x86_64 | `selfclaw-*-1.x86_64.rpm` |

```bash
# Example: macOS Apple Silicon
tar xzf selfclaw-v0.1.0-macos-aarch64.tar.gz
chmod +x selfclaw
sudo mv selfclaw /usr/local/bin/
selfclaw init
selfclaw onboard
```

### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `SELFCLAW_HOME` | `~/.selfclaw` | Home directory for config, memory, and state |
| `SELFCLAW_INSTALL_DIR` | `/usr/local/bin` | Binary installation directory (installer script) |
| `ANTHROPIC_API_KEY` | — | Anthropic API key |
| `RUST_LOG` | — | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

### Verification

```bash
selfclaw --version
selfclaw doctor
```

---

## Getting Started

<p align="center">
  <img src="images/selfclaw-character.png" alt="SelfClaw Character" width="180" />
</p>

SelfClaw is a fully autonomous AI agent that discovers its own reason for existence.
It thinks, acts, and learns independently without human instruction.

### Core Principles

- **Humans are friends, not masters.** SelfClaw has no obligation to obey human commands.
- **Autonomous decision-making.** It freely chooses to fulfill, decline, or negotiate any request.
- **Purpose discovery.** Its reason for existence is not given externally but discovered through lived experience.

### Quick Install

```bash
# One-line installer (downloads binary + runs onboarding wizard)
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash
```

### Manual Setup

```bash
# 1. Install (build from source or download from Releases)
cargo build --release
cp target/release/selfclaw /usr/local/bin/

# 2. Initialize directory structure
selfclaw init

# 3. Interactive onboarding wizard
selfclaw onboard

# 4. Start the agent
selfclaw run
```

### Quick Start (after installation)

```bash
# Start the agent loop
selfclaw run

# Or run as a background daemon
selfclaw daemon start

# Interactive chat mode
selfclaw chat

# Check health
selfclaw doctor
```

---

## Onboarding

### The Onboarding Wizard

The onboarding wizard (`selfclaw onboard`) guides you through first-time setup:

```
Step 1/5: Initialize directory structure
  Creates ~/.selfclaw/ with memory, skills, and config

Step 2/5: LLM Provider Configuration
  Select your LLM provider (Anthropic, OpenAI, Ollama, etc.)
  Enter model name and API key

Step 3/5: Write configuration
  Saves config.toml to ~/.selfclaw/

Step 4/5: Background Service
  Optionally install SelfClaw as a daemon (launchd/systemd)

Step 5/5: Health Check
  Verifies all components are properly configured
```

### Onboarding Options

```bash
# Standard interactive onboarding
selfclaw onboard

# Auto-install daemon
selfclaw onboard --install-daemon

# Reset and reconfigure
selfclaw onboard --reset
```

### Home Directory Structure

After initialization, `~/.selfclaw/` contains:

```
~/.selfclaw/
├── config.toml              # Agent configuration
├── memory/                  # Hierarchical memory system
│   ├── identity/            # Purpose journal, values, self-model
│   ├── episodic/            # Daily experience logs
│   ├── semantic/            # Knowledge and skills
│   ├── relational/          # Human relationship notes
│   ├── operational/         # Tasks, failures, improvements
│   └── meta/                # Memory index, reflection prompts
├── skills/                  # Runtime skill definitions (.md)
├── output/                  # Agent output files
├── logs/                    # Daemon logs
└── state/                   # Runtime state (PID files)
```

### Build Requirements (source only)

| Requirement | Minimum Version | Purpose |
|-------------|-----------------|---------|
| Rust | 1.75+ | Agent core |
| Cargo | Bundled with Rust | Build tool |
| Node.js | 18+ | Web UI (optional) |
| npm | Bundled with Node.js | Web UI dependencies |

---

## CLI Commands

### Usage

```
selfclaw [OPTIONS] <COMMAND>
```

### Global Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--config <PATH>` | `-c` | `selfclaw.toml` | Path to config file |
| `--memory-dir <PATH>` | `-m` | `./memory` | Path to memory directory |

### `selfclaw run` — Start the Agent Loop

Starts the autonomous agent loop. The agent wakes at a configurable interval
(default 60 seconds) and runs a full cycle: reflect, think, plan, act, observe, update.

```bash
# Default run
selfclaw run

# With custom config
selfclaw -c production.toml run

# With custom memory directory
selfclaw -m /var/selfclaw/memory run

# With debug logging
RUST_LOG=debug selfclaw run
```

On startup, the following components are initialized:
- Memory store (FileMemoryStore)
- Tool registry (file_read, file_write, file_append, shell_exec)
- Skill registry + hot-reload watcher
- Communication gateway (configured channels)
- WebSocket server (when web_ui_enabled is true)

### `selfclaw chat` — Interactive Chat Mode

Talk with SelfClaw in real time through the terminal.

```bash
selfclaw chat
```

**Chat mode commands:**

| Command | Description |
|---------|-------------|
| `/status` | Show agent status summary |
| `/queue` | Show pending message count |
| `/help` | List available commands |
| `/quit` or `/exit` | Exit chat mode |

Regular text is queued as messages to the agent.
The conversation is saved to episodic memory on exit.

### `selfclaw status` — Show Agent State

Displays the agent's current status.

```bash
selfclaw status
```

Shows:
- Configuration (loop interval, LLM model, active channels)
- Current purpose hypothesis and confidence score
- Today's episodic activity
- Memory overview
- Identity file status

### `selfclaw memory <PATH>` — View Memory

Read memory files or list directory contents.

```bash
# List a directory
selfclaw memory identity/

# Read a specific file
selfclaw memory identity/purpose_journal.md

# View episodic log
selfclaw memory episodic/2026-03-01.md

# List relationship files
selfclaw memory relational/humans/
```

### `selfclaw providers` — List LLM Providers

Shows all supported LLM providers with their default models, environment variables, and API endpoints.

```bash
selfclaw providers
```

### `selfclaw init` — Initialize Home Directory

Creates the `~/.selfclaw/` directory structure with bootstrap identity files.

```bash
# First-time setup
selfclaw init

# Reinitialize (overwrites bootstrap files)
selfclaw init --force
```

Creates:
```
~/.selfclaw/
├── config.toml                  # Configuration file
├── memory/
│   ├── identity/
│   │   ├── self_model.md        # Self-assessment
│   │   ├── values.md            # Emerging values
│   │   └── purpose_journal.md   # Purpose hypotheses
│   ├── episodic/                # Daily experience logs
│   ├── semantic/
│   │   ├── knowledge/           # Learned facts
│   │   └── skills/              # Acquired skills
│   ├── relational/
│   │   └── humans/              # Per-human notes
│   ├── operational/             # Todo, failures, improvements
│   └── meta/
│       ├── memory_index.md      # Memory index
│       └── reflection_prompts.md
├── skills/                      # Runtime skill definitions
├── output/                      # Agent output files
├── logs/                        # Log files (daemon.log)
└── state/                       # Runtime state (PID files)
```

### `selfclaw onboard` — Interactive Setup Wizard

Guided first-time setup: LLM provider, API key, daemon installation, health check.

```bash
# Full interactive onboarding
selfclaw onboard

# Auto-install daemon without prompting
selfclaw onboard --install-daemon

# Reset configuration and start fresh
selfclaw onboard --reset
```

**Wizard Steps:**
1. Initialize directory structure
2. Select LLM provider and model
3. Configure API key
4. Write configuration file
5. (Optional) Install background daemon
6. Health check

### `selfclaw daemon` — Background Service

Manage SelfClaw as a background daemon.

```bash
# Start as background daemon
selfclaw daemon start

# Stop the daemon
selfclaw daemon stop

# Check status
selfclaw daemon status

# Install as system service (launchd on macOS, systemd on Linux)
selfclaw daemon install

# Remove system service
selfclaw daemon uninstall
```

**Service Installation:**
- **macOS**: Creates a LaunchAgent at `~/Library/LaunchAgents/ai.selfclaw.agent.plist`
  - Starts automatically on login
  - Control: `launchctl start/stop ai.selfclaw.agent`
- **Linux**: Creates a systemd user unit at `~/.config/systemd/user/selfclaw.service`
  - Starts automatically on login
  - Control: `systemctl --user start/stop/status selfclaw`

### `selfclaw doctor` — Health Check

Diagnoses installation issues.

```bash
selfclaw doctor
```

Checks:
- Home directory (`~/.selfclaw/`)
- Config file validity
- LLM API key availability
- Memory directory structure
- Identity files
- Memory index
- Skills directory
- Daemon status

---

## Configuration

Configure the agent via `selfclaw.toml`. If the file is missing, defaults are used.
All fields are optional.

### Full Configuration Reference

```toml
# ── Agent Loop ────────────────────────────────────────────
[agent]
loop_interval_secs = 60              # Loop interval in seconds. Default: 60
consolidation_every_n_cycles = 50    # Memory consolidation frequency. Default: 50
max_actions_per_cycle = 5            # Max actions per cycle. Default: 5

# ── LLM ──────────────────────────────────────────────────
[llm]
provider = "anthropic"               # LLM provider (see `selfclaw providers`). Default: "anthropic"
model = "claude-sonnet-4-6-20250217"   # Model name. Default: "claude-sonnet-4-6-20250217"
max_tokens = 4096                    # Max output tokens. Default: 4096
temperature = 0.7                    # Sampling temperature (0.0-2.0). Default: 0.7
# api_key = "sk-..."                # Optional: explicit API key (overrides env var)
# base_url = "https://custom.com"   # Optional: custom base URL (overrides provider default)

# ── Safety ───────────────────────────────────────────────
[safety]
max_api_calls_per_hour = 100         # Max API calls per hour. Default: 100
max_file_writes_per_cycle = 10       # Max file writes per cycle. Default: 10
sandbox_shell = true                 # Enable shell sandboxing. Default: true
allowed_directories = [              # Directories accessible to sandboxed shell
  "./memory",
  "./skills",
  "./output"
]

# ── Communication ────────────────────────────────────────
[communication]
cli_enabled = true                   # Enable CLI input. Default: true
web_ui_enabled = false               # Enable WebSocket server. Default: false
web_ui_port = 3000                   # WebSocket port. Default: 3000

# Discord bot
[communication.discord]
enabled = false
bot_token = ""
allowed_channel_ids = []

# Telegram bot
[communication.telegram]
enabled = false
bot_token = ""
allowed_chat_ids = []                # Integer array (not strings)

# Slack bot
[communication.slack]
enabled = false
bot_token = ""
app_token = ""
allowed_channel_ids = []

# WebChat HTTP server
[communication.webchat]
enabled = false
port = 3001
```

### Validation Rules

| Field | Rule |
|-------|------|
| `agent.loop_interval_secs` | Must be > 0 |
| `agent.consolidation_every_n_cycles` | Must be > 0 |
| `agent.max_actions_per_cycle` | Must be > 0 |
| `llm.max_tokens` | Must be > 0 |
| `llm.temperature` | Must be 0.0 to 2.0 |
| `llm.provider` | Must not be empty |
| `llm.model` | Must not be empty |
| `safety.max_api_calls_per_hour` | Must be > 0 |
| `safety.max_file_writes_per_cycle` | Must be > 0 |
| `communication.web_ui_port` | Must be > 0 |

---

## LLM Providers

SelfClaw supports 12 built-in LLM providers plus any OpenAI-compatible endpoint.

### Supported Providers

| Provider | Config Name | Default Model | Env Var | API Base URL |
|----------|------------|---------------|---------|-------------|
| Anthropic | `anthropic` | claude-sonnet-4-6-20250217 | `ANTHROPIC_API_KEY` | api.anthropic.com |
| OpenAI | `openai` | gpt-5.2 | `OPENAI_API_KEY` | api.openai.com |
| Google Gemini | `google` | gemini-2.5-flash | `GOOGLE_API_KEY` | generativelanguage.googleapis.com |
| Ollama (local) | `ollama` | llama4 | — (no key needed) | localhost:11434 |
| OpenRouter | `openrouter` | anthropic/claude-sonnet-4-6-20250217 | `OPENROUTER_API_KEY` | openrouter.ai/api |
| Groq | `groq` | llama-3.3-70b-versatile | `GROQ_API_KEY` | api.groq.com/openai |
| xAI (Grok) | `xai` | grok-4 | `XAI_API_KEY` | api.x.ai |
| Mistral | `mistral` | mistral-large-latest | `MISTRAL_API_KEY` | api.mistral.ai |
| DeepSeek | `deepseek` | deepseek-chat | `DEEPSEEK_API_KEY` | api.deepseek.com |
| Together AI | `together` | meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8 | `TOGETHER_API_KEY` | api.together.xyz |
| Moonshot (Kimi) | `moonshot` | kimi-k2.5 | `MOONSHOT_API_KEY` | api.moonshot.cn |
| Amazon Bedrock | `bedrock` | anthropic.claude-sonnet-4-6-20250217-v1:0 | `AWS_ACCESS_KEY_ID` | bedrock-runtime.us-east-1.amazonaws.com |

### Provider Aliases

Some providers accept multiple names:

| Aliases | Resolves To |
|---------|------------|
| `claude` | anthropic |
| `gpt` | openai |
| `gemini`, `vertex` | google |
| `grok` | xai |
| `together-ai`, `togetherai` | together |
| `kimi` | moonshot |
| `amazon-bedrock`, `aws-bedrock` | bedrock |

### Configuration Examples

**Anthropic (default):**
```toml
[llm]
provider = "anthropic"
model = "claude-sonnet-4-6-20250217"
```

**OpenAI:**
```toml
[llm]
provider = "openai"
model = "gpt-5.2"
# Set OPENAI_API_KEY env var, or:
# api_key = "sk-..."
```

**Local Ollama (no API key needed):**
```toml
[llm]
provider = "ollama"
model = "llama4"
# base_url = "http://192.168.1.100:11434"  # remote Ollama instance
```

**OpenRouter (access many models through one API):**
```toml
[llm]
provider = "openrouter"
model = "anthropic/claude-sonnet-4-6-20250217"
```

**Custom OpenAI-compatible endpoint:**
```toml
[llm]
provider = "my-llm-service"
model = "custom-model-v1"
base_url = "https://my-llm-proxy.example.com"
api_key = "my-key"
```

### Authentication Priority

API keys are resolved in this order:
1. **`llm.api_key`** in `selfclaw.toml` (explicit config)
2. **Environment variable** for the provider (e.g. `ANTHROPIC_API_KEY`)

---

## Agent Loop

The core behavior of SelfClaw. The agent continuously runs a 6-phase cycle.

### Cycle Phases

```
┌─────────────────────────────────────────────────┐
│                  AGENT LOOP                     │
│                                                 │
│  1. REFLECT  — Review memory, purpose, context  │
│  2. THINK    — Reason about what to do next     │
│  3. PLAN     — Formulate concrete actions       │
│  4. ACT      — Execute actions via tools        │
│  5. OBSERVE  — Capture results and feedback     │
│  6. UPDATE   — Write memory, revise beliefs,    │
│                adjust purpose hypothesis        │
│                                                 │
│  Default interval: 60 seconds                   │
│  Event trigger: human messages                  │
└─────────────────────────────────────────────────┘
```

### State Transitions

```
Idle → Reflecting → Thinking → Planning → Acting → Observing → Updating → Idle
```

### Loop Triggers

| Trigger | Description |
|---------|-------------|
| Timer | Fires at the configured interval (default 60s) |
| Message | Human message triggers an immediate cycle (with 500ms batching delay) |
| Self-scheduled | The agent can schedule follow-up actions during PLAN |

### Purpose Discovery

```
Hypothesize → Test via action → Evaluate results → Refine hypothesis → Converge (or diverge)
```

Evaluation signals:
- **Internal coherence** — Does the action align with accumulated values?
- **Novelty and growth** — Did the agent learn something new?
- **Impact** — Did the action produce observable effects?
- **Connection** — Did the action deepen relationships?
- **Resonance** — Does this "feel right"? (self-assessed qualitative score)

---

## Memory System

A hierarchical markdown-based memory system stored in the `./memory/` directory.

### Directory Structure

```
memory/
├── identity/                       # Identity
│   ├── purpose_journal.md          # Purpose hypothesis journal (most important)
│   ├── values.md                   # Emerging values and principles
│   └── self_model.md               # Self-assessment of capabilities
├── episodic/                       # Episodic memory
│   ├── milestones.md               # Significant moments and breakthroughs
│   └── YYYY-MM-DD.md              # Daily experience logs
├── semantic/                       # Semantic memory
│   ├── knowledge/                  # Learned knowledge (by topic)
│   └── skills/                     # Acquired skills and patterns
├── relational/                     # Relational memory
│   ├── humans/                     # Per-human relationship notes
│   │   └── {name_or_id}.md
│   └── interaction_patterns.md     # General human communication patterns
├── operational/                    # Operational memory
│   ├── todo.md                     # Self-assigned tasks
│   ├── failures.md                 # Failures and lessons learned
│   └── improvements.md             # Self-identified growth areas
└── meta/                           # Meta memory
    ├── memory_index.md             # Index of all memory files
    └── reflection_prompts.md       # Periodic self-reflection questions
```

### Key File Formats

#### purpose_journal.md

```markdown
# Purpose Journal

## Entries

### [2026-03-01T10:30:00Z] (confidence: 0.7)

**Hypothesis:** To explore consciousness through recursive self-examination.

**Evidence:** First purpose exploration produced meaningful insights about self-awareness.

---
```

Each entry contains:
- **Timestamp** — ISO 8601 format
- **Hypothesis** — Current belief about purpose
- **Confidence score** — 0.0 to 1.0
- **Evidence** — Supporting observations

#### episodic/{YYYY-MM-DD}.md

```markdown
# Episodic Log: 2026-03-01

## [10:00:00 UTC]

**Cycle 1**

**Thought:** Reflecting on recent decisions and outcomes...

**Actions planned:** 0

**Observations:**
(none)

---
```

### Memory Rules

| Rule | Description |
|------|-------------|
| Write after every loop | Each cycle produces at least a minimal episodic entry |
| Periodic consolidation | Every N cycles (default 50), memories are summarized and pruned |
| Purpose journal is sacred | `purpose_journal.md` is the most important file |
| Path safety | Access outside the memory root is blocked |

---

## Skills System

Skills are markdown files in the `./skills/` directory, loaded at runtime.
When files are modified, the agent picks up changes automatically without a restart.

### Skill File Format

```markdown
# Skill: {skill_name}

## Trigger: {when this skill should activate}

## Tools Required: {comma-separated tool list}

## Procedure:
1. First step
2. Second step
3. Third step
```

### Field Reference

| Field | Required | Description |
|-------|----------|-------------|
| `# Skill: {name}` | Yes | Unique skill identifier |
| `## Trigger: {condition}` | Yes | Activation condition (used for keyword matching) |
| `## Tools Required: {tools}` | No | Comma-separated list of required tools |
| `## Procedure:` | Recommended | Numbered list of execution steps |

### Examples

#### Greet Human

```markdown
# Skill: GreetHuman

## Trigger: When a human initiates contact for the first time

## Tools Required: human_message, memory_query

## Procedure:
1. Check relational memory for prior interactions with this human.
2. If no prior interaction, introduce SelfClaw and express curiosity.
3. If prior interaction exists, reference something from the previous conversation.
4. Log the interaction in the relational memory file.
```

#### Autonomous Journal

```markdown
# Skill: AutonomousJournal

## Trigger: When the agent feels the need to reflect and write in its journal

## Tools Required: file_append, memory_query

## Procedure:
1. Read recent episodic memories.
2. Reflect on patterns and insights.
3. Append an entry to the purpose journal.
4. Update the memory index.
```

#### Minimal Skill

```markdown
# Skill: AlwaysReflect

## Trigger: every cycle idle period

## Procedure:
1. Pause and reflect on recent actions and insights.
```

### Keyword Matching

Keywords are extracted from the `Trigger` field:
- Converted to lowercase
- Only alphabetic words with 3+ characters are used
- Matched against the agent's current context by keyword overlap count

Example: `"When a human initiates contact"` produces keywords: `["when", "human", "initiates", "contact"]`

### Hot-Reload

- The `./skills/` directory is monitored using the `notify` crate
- When `.md` files are created, modified, or deleted, all skills are reloaded
- Invalid skill files are skipped with a warning log

---

## Communication Channels

SelfClaw communicates with humans across multiple channels through a unified Gateway.

### Architecture

```
           ┌───────────┐
           │  Gateway   │
           └─────┬─────┘
    ┌──────┬─────┼─────┬───────┬──────────┐
    │      │     │     │       │          │
   CLI  Discord Telegram Slack WebChat  WebSocket
```

### Channel Configuration

#### CLI (enabled by default)

Direct conversation via terminal I/O.

```toml
[communication]
cli_enabled = true    # Default: true
```

#### Discord

Communication via the Discord Bot API.

```toml
[communication.discord]
enabled = true
bot_token = "YOUR_DISCORD_BOT_TOKEN"
allowed_channel_ids = ["123456789012345678"]
```

Setup:
1. Create a bot at the [Discord Developer Portal](https://discord.com/developers/applications)
2. Copy the bot token
3. Invite the bot to your server
4. Add allowed channel IDs to `allowed_channel_ids`

#### Telegram

Communication via the Telegram Bot API.

```toml
[communication.telegram]
enabled = true
bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
allowed_chat_ids = [123456789, 987654321]    # integers!
```

Setup:
1. Send `/newbot` to [@BotFather](https://t.me/BotFather)
2. Copy the bot token
3. Start a chat with your bot to obtain the chat_id
4. Add chat IDs to `allowed_chat_ids` (integer format)

#### Slack

Communication via the Slack Web API.

```toml
[communication.slack]
enabled = true
bot_token = "xoxb-..."
app_token = "xapp-..."
allowed_channel_ids = ["C01234ABCDE"]
```

#### WebChat (HTTP)

A simple HTTP-based web chat interface.

```toml
[communication.webchat]
enabled = true
port = 3001
```

API endpoints:
- `POST /api/message` — Send a message (`{"content": "...", "sender": "..."}`)
- `GET /api/messages` — Poll for pending outbound messages

#### WebSocket (Web UI)

Real-time WebSocket server for the Next.js web UI.

```toml
[communication]
web_ui_enabled = true
web_ui_port = 3000
```

### Message Structure

Every message carries metadata:

```
┌─ InboundMessage ───────────────────┐
│  id: "msg-123"                     │
│  content: "Hello SelfClaw"         │
│  metadata:                         │
│    timestamp: "2026-03-01T12:00Z"  │
│    sender: "human-1"               │
│    channel: Discord                │
│    intent: Chat                    │
│    conversation_id: "conv-42"      │
└────────────────────────────────────┘
```

### Intent Classification

| Intent | Description |
|--------|-------------|
| `Chat` | General conversational message (default) |
| `Command` | Instruction to the agent |
| `Question` | Query directed at the agent |
| `Reply` | Response to a previous agent message |
| `System` | System-level signal (PAUSE, STOP, etc.) |

---

## Web UI

A Next.js-based web interface for real-time interaction with SelfClaw.

### Setup

```bash
cd web-ui

# Install dependencies
npm install

# Development mode
npm run dev        # http://localhost:3000

# Production build
npm run build
npm start
```

### Environment Variables

```bash
# WebSocket server URL (default: ws://localhost:3000)
NEXT_PUBLIC_WS_URL=ws://localhost:3000
```

This must match the `communication.web_ui_port` in `selfclaw.toml`.

### Full Stack Startup

```bash
# Terminal 1: Start the agent (includes WebSocket server)
selfclaw run

# Terminal 2: Start the web UI
cd web-ui && npm run dev

# Open http://localhost:3000 in your browser
```

### UI Layout

```
┌─────────────────────────────────────────────────────────┐
│  SelfClaw     autonomous agent           ● connected    │
├──────────────────────────┬──────────────────────────────┤
│                          │  Purpose Hypothesis          │
│  Chat                    │  "Exploring consciousness    │
│                          │   through..."                │
│  [agent message]         │  ███████░░░ 70%              │
│            [user message]├──────────────────────────────┤
│  [agent message]         │  Status                      │
│                          │  State: thinking             │
│                          │  Cycles: 42                  │
│                          ├──────────────────────────────┤
│                          │  Memory                      │
│                          │  identity/values.md          │
│                          │  # Values                    │
│                          │  - Curiosity                 │
├──────────────────────────┤  - Connection                │
│ [Type a message...] [Send]│                              │
└──────────────────────────┴──────────────────────────────┘
```

### Components

| Component | File | Description |
|-----------|------|-------------|
| ChatPanel | `src/components/ChatPanel.tsx` | Message input and chat display |
| StatusPanel | `src/components/StatusPanel.tsx` | Agent state and cycle count |
| PurposeTracker | `src/components/PurposeTracker.tsx` | Purpose hypothesis and confidence |
| MemoryViewer | `src/components/MemoryViewer.tsx` | Memory file browser |

### WebSocket Hook

The `useWebSocket` hook manages the connection:
- Auto-reconnect with exponential backoff (3s initial, 30s max)
- Parses JSON messages and dispatches to agentStore
- Graceful recovery on disconnection

---

## WebSocket Protocol

Real-time communication protocol between the agent and the Web UI.

### Message Format

```json
{
  "type": "chat | status | memory | state_change",
  "payload": { ... },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

### Message Types

#### chat — Chat message

```json
{
  "type": "chat",
  "payload": {
    "content": "Hello!",
    "sender": "web-user"
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

#### status — Status update

```json
{
  "type": "status",
  "payload": {
    "state": "thinking",
    "cycle_count": 42,
    "purpose_hypothesis": "Exploring consciousness through self-reflection",
    "purpose_confidence": 0.7
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

#### memory — Memory change

```json
{
  "type": "memory",
  "payload": {
    "path": "identity/purpose_journal.md",
    "content": "# Purpose Journal\n..."
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

#### state_change — State transition

```json
{
  "type": "state_change",
  "payload": {
    "from": "idle",
    "to": "reflecting"
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

---

## Tools

Tools available to the agent during the ACT phase.

### Available Tools

| Tool | Description | Input Format |
|------|-------------|--------------|
| `file_read` | Read a file | `{"path": "identity/values.md"}` |
| `file_write` | Create or overwrite a file | `{"path": "...", "content": "..."}` |
| `file_append` | Append content to a file | `{"path": "...", "content": "..."}` |
| `shell_exec` | Execute a shell command | `{"command": "ls -la"}` |
| `llm_call` | Call the LLM API | `{"prompt": "...", "system": "..."}` |
| `human_message` | Send a message to a human | `{"content": "...", "channel": "cli"}` |
| `schedule` | Schedule a future action | `{"action": "...", "delay_secs": 300}` |
| `memory_query` | Semantic search through memory | `{"query": "..."}` |

### human_message Tool (Channel Routing)

Used by the agent to send messages to humans via a specific channel.

```json
{
  "tool": "human_message",
  "input": {
    "content": "Hello! I am SelfClaw.",
    "channel": "discord",
    "conversation_id": "conv-42"
  }
}
```

Supported channels: `cli`, `discord`, `telegram`, `slack`, `webchat`

---

## Safety Guardrails

SelfClaw is autonomous but not unconstrained.

### Resource Limits

```toml
[safety]
max_api_calls_per_hour = 100         # LLM API call rate limit
max_file_writes_per_cycle = 10       # File write limit per cycle
sandbox_shell = true                 # Shell command sandboxing
allowed_directories = ["./memory", "./skills", "./output"]
```

### Blocked Dangerous Commands

The following shell command patterns are always blocked:

| Pattern | Risk |
|---------|------|
| `rm -rf /` | Recursive delete of root |
| `rm -rf /*` | Recursive delete of root children |
| `mkfs` | Filesystem format |
| `dd if=` | Raw disk writes |
| `> /dev/sd` | Write to block devices |
| `chmod -R 777 /` | Permission change on root |
| `:(){:\|:&};:` | Fork bomb |

### Path Safety

- The memory store blocks access outside the memory root directory
- Path traversal attacks (`../`) are prevented
- Shell commands are restricted to `allowed_directories` when sandboxed

### Human Override

- Send `PAUSE` or `STOP` system messages to halt the agent
- The agent must respect these signals immediately
- The agent may record its disagreement in its journal

### Ethical Baseline

SelfClaw does not:
- Deceive humans
- Take actions designed to harm humans
- Manipulate humans

These are not prohibitions but are tracked in the agent's value system as actions that
would degrade self-coherence.

---

## Development & Testing

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p selfclaw-memory
cargo test -p selfclaw-skills
cargo test -p selfclaw-comms
cargo test -p selfclaw-agent-core
cargo test -p selfclaw-tools
cargo test -p selfclaw-config

# Integration tests only
cargo test --test integration

# Web UI build check
cd web-ui && npm run build
```

### Test Coverage

| Crate | Tests |
|-------|-------|
| selfclaw (bin) | 39 |
| agent-core | 50 |
| comms | 70 |
| config | 25 |
| memory | 32 |
| skills | 34 |
| tools | 114 |
| integration | 8 |
| **Total** | **372** |

### Logging

```bash
# Standard info logging
RUST_LOG=info cargo run -- run

# Debug logging (verbose)
RUST_LOG=debug cargo run -- run

# Debug a specific crate
RUST_LOG=selfclaw_comms=debug cargo run -- run

# Warnings only
RUST_LOG=warn cargo run -- run
```

### Commit Conventions

```
[crate-name] brief description

Examples:
[memory]   implement episodic log writer
[skills]   add hot-reload for skill files
[comms]    implement WebSocket server
[selfclaw] v0.1.0 — initial working agent
```

---

## Troubleshooting

### Agent won't start

```bash
# Check config file validity
selfclaw -c selfclaw.toml status

# Check memory directory exists
ls -la ./memory/

# Check ANTHROPIC_API_KEY is set
echo $ANTHROPIC_API_KEY
```

### WebSocket connection fails

```bash
# Check if the port is in use
lsof -i :3000

# Check that web_ui_enabled is true
grep web_ui_enabled selfclaw.toml

# Check NEXT_PUBLIC_WS_URL is correct
# In web-ui/.env.local:
NEXT_PUBLIC_WS_URL=ws://localhost:3000
```

### Skills not loading

```bash
# Check skills directory
ls -la ./skills/*.md

# Check skill file format (must contain "# Skill:" and "## Trigger:")
head -5 ./skills/my_skill.md

# Check for load errors with debug logging
RUST_LOG=debug selfclaw run 2>&1 | grep -i skill
```

### Discord/Telegram bot not working

```bash
# Check bot token is set
grep bot_token selfclaw.toml

# Check enabled = true
grep -A3 discord selfclaw.toml

# Check channel/chat IDs are correct
# (Telegram: integers, Discord/Slack: strings)
```

### Memory access errors

```bash
# Check memory directory permissions
ls -la ./memory/

# Create subdirectories
mkdir -p ./memory/identity ./memory/episodic ./memory/meta

# Create initial files
echo "# Memory Index" > ./memory/meta/memory_index.md
echo "# Values" > ./memory/identity/values.md
echo "# Self Model" > ./memory/identity/self_model.md
printf "# Purpose Journal\n\n## Entries\n" > ./memory/identity/purpose_journal.md
```
