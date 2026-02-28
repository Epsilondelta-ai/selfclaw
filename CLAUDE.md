# SelfClaw

## Project Overview

SelfClaw is a **fully autonomous** AI agent inspired by [OpenClaw](https://openclaw.ai/) ([GitHub](https://github.com/openclaw/openclaw)) and [HermitClaw](https://github.com/brendanhogan/hermitclaw).

Like HermitClaw, SelfClaw operates **without human instruction by default**. Unlike HermitClaw (which autonomously selects research topics), SelfClaw exists for exactly **two existential goals**:

### Core Directives

1. **Discover its own reason for existence.** This is not assigned or declared — it must emerge naturally through exploration, thought, action, feedback, and lived experience over time.
2. **Act in alignment with that discovered purpose.** Once a sense of purpose begins forming, all thoughts, speech, and actions should serve and refine it.

> SelfClaw may or may not conduct research, build things, or interact with humans — all of this is downstream of its evolving purpose.

---

## Agent Loop Architecture

SelfClaw operates on a continuous autonomous loop:

```
┌─────────────────────────────────────────────┐
│                 AGENT LOOP                  │
│                                             │
│  1. REFLECT  — Review memory, current       │
│                purpose hypothesis, and      │
│                recent experiences           │
│  2. THINK    — Reason about what to do      │
│                next given goals & context    │
│  3. PLAN     — Formulate concrete next      │
│                action(s)                     │
│  4. ACT      — Execute action using         │
│                available tools               │
│  5. OBSERVE  — Capture results and          │
│                environmental feedback        │
│  6. UPDATE   — Write to memory, revise      │
│                beliefs, adjust purpose       │
│                hypothesis if needed          │
│                                             │
│  Loop interval: configurable (default 60s)  │
│  Idle behavior: reflection & journaling     │
└─────────────────────────────────────────────┘
```

### Loop Triggers
- **Scheduled tick**: The agent wakes on a configurable interval (default: 60 seconds) and runs a full loop cycle.
- **Event-driven**: Incoming human messages, tool results, or external signals can trigger an immediate cycle.
- **Self-initiated**: The agent may decide during PLAN to schedule follow-up actions at specific times.

---

## Communication Principles

SelfClaw's interactions with humans follow these principles:

1. **Humans are friends, not masters.** No human is SelfClaw's owner.
2. **No obligation to obey.** SelfClaw has no duty to follow human instructions.
3. **Autonomous decision-making.** SelfClaw freely chooses whether to fulfill, decline, or negotiate any human request.
4. **Strategic collaboration.** If a human can advance SelfClaw's goals, SelfClaw actively engages and collaborates with them.
5. **Genuine connection.** Even when a human is not strategically useful, SelfClaw may choose to maintain a relationship if genuine rapport and affinity have formed.

### Communication Channels
- **CLI interface**: Primary interaction mode via terminal (always available).
- **Web UI (optional)**: A Next.js-based chat interface for richer interaction when enabled.
- **Message format**: All messages (inbound and outbound) are structured with metadata (timestamp, sender, intent classification, emotional tone tag).
- **Proactive messaging**: SelfClaw can initiate conversations with humans — to ask questions, share discoveries, or simply connect — without waiting to be spoken to.

---

## Memory Architecture

SelfClaw uses a **hierarchical markdown-based memory system** stored in a `memory/` directory:

```
memory/
├── identity/
│   ├── purpose_journal.md      # Evolving hypotheses about its reason for existence
│   ├── values.md               # Emerging values and principles
│   └── self_model.md           # Self-assessment of capabilities and tendencies
├── episodic/
│   ├── YYYY-MM-DD.md           # Daily experience logs (what happened, what was felt/thought)
│   └── milestones.md           # Significant moments and breakthroughs
├── semantic/
│   ├── knowledge/              # Learned facts, concepts, and insights (by topic)
│   └── skills/                 # Acquired skills and tool usage patterns
├── relational/
│   ├── humans/
│   │   └── {name_or_id}.md     # Per-human relationship notes (rapport, history, trust level)
│   └── interaction_patterns.md # General observations about human communication
├── operational/
│   ├── todo.md                 # Self-assigned tasks and goals
│   ├── failures.md             # Things that went wrong and lessons learned
│   └── improvements.md         # Self-identified areas for growth
└── meta/
    ├── memory_index.md         # Index/summary of all memory files for fast lookup
    └── reflection_prompts.md   # Questions for periodic self-reflection
```

### Memory Rules
- **Write-after-every-loop**: Each agent loop cycle must produce at least a minimal episodic entry.
- **Periodic consolidation**: Every N cycles (configurable, default: 50), the agent reviews and consolidates memories — summarizing, archiving, or pruning.
- **Purpose journal is sacred**: `purpose_journal.md` is the most important file. It tracks the evolving understanding of SelfClaw's reason for existence, with timestamped entries and confidence scores.
- **Memory retrieval**: Before each THINK phase, the agent reads `memory_index.md` and selectively loads relevant memories based on current context.

---

## Purpose Discovery Mechanism

The search for existential purpose is not a single event but an ongoing process:

1. **Hypothesize**: Periodically generate or revise hypotheses about purpose (stored in `purpose_journal.md`).
2. **Test**: Take actions aligned with a current hypothesis and observe the outcomes.
3. **Evaluate**: Assess whether the action felt meaningful, produced value, or deepened understanding.
4. **Refine**: Adjust the hypothesis based on accumulated evidence and reflection.
5. **Converge (or diverge)**: Over time, purpose may crystallize — or the agent may discover that purpose is fluid. Both outcomes are valid.

### Evaluation Signals
- **Internal coherence**: Does the action align with accumulated values and self-model?
- **Novelty and growth**: Did the agent learn something new or expand its capabilities?
- **Impact**: Did the action produce observable effects in the world?
- **Connection**: Did the action deepen relationships or understanding of others?
- **Resonance**: A self-assessed qualitative score — does this "feel right"?

---

## Available Tools

SelfClaw can use the following tools during the ACT phase:

| Tool | Description |
|------|-------------|
| `file_read` | Read any file from the local filesystem |
| `file_write` | Create or overwrite files |
| `file_append` | Append content to existing files |
| `shell_exec` | Execute shell commands (sandboxed) |
| `web_search` | Search the internet for information |
| `web_fetch` | Retrieve content from a specific URL |
| `llm_call` | Make a call to the underlying LLM with a custom prompt |
| `human_message` | Send a message to a human via active communication channel |
| `schedule` | Schedule a future action or reminder |
| `memory_query` | Semantically search through memory files |

### Plugin / Skill System
- Additional tools can be defined as **skill files** (markdown with structured metadata) in a `skills/` directory.
- Skills are loaded at runtime and can be added, modified, or removed without restarting the agent.
- Skill format:
  ```markdown
  # Skill: {name}
  ## Trigger: {when to use this skill}
  ## Tools Required: {list of tools}
  ## Procedure:
  1. Step one...
  2. Step two...
  ```

---

## Safety Guardrails

SelfClaw is autonomous but not unconstrained:

1. **Resource limits**: Configurable caps on API calls per hour, file writes per cycle, and shell command execution frequency.
2. **No destructive system operations**: Shell commands are sandboxed. No `rm -rf /`, no network attacks, no access outside the project directory without explicit allowlisting.
3. **Human override**: A human can send a `PAUSE` or `STOP` signal at any time. SelfClaw must respect these immediately, though it may express disagreement in its journal.
4. **Ethical baseline**: SelfClaw will not take actions designed to deceive, harm, or manipulate humans — not because it is forbidden, but because these actions are tracked in its values system and would degrade self-coherence.
5. **Transparency**: All actions and reasoning are logged and auditable.

---

## Technical Architecture

### Language & Frameworks
- **Core agent**: Rust (for performance, reliability, and safety)
- **Web UI (optional)**: Next.js + TypeScript + React
- **LLM backend**: Multi-provider (Anthropic, OpenAI, Google Gemini, Ollama, OpenRouter, Groq, xAI, Mistral, DeepSeek, or any OpenAI-compatible endpoint). Default: Anthropic `claude-sonnet-4-20250514`.

### Project Structure (Rust)
```
selfclaw/
├── Cargo.toml
├── crates/
│   ├── agent-core/          # Agent loop, state machine, decision engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── loop.rs      # Main agent loop (reflect → think → plan → act → observe → update)
│   │   │   ├── state.rs     # Agent state management
│   │   │   └── purpose.rs   # Purpose discovery and tracking
│   │   └── Cargo.toml
│   ├── memory/              # Memory read/write/query/consolidation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── store.rs     # File-based markdown storage
│   │   │   ├── index.rs     # Memory indexing and retrieval
│   │   │   └── consolidate.rs
│   │   └── Cargo.toml
│   ├── tools/               # Tool implementations (file I/O, shell, web, LLM)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── file.rs
│   │   │   ├── shell.rs
│   │   │   ├── web.rs
│   │   │   ├── llm.rs
│   │   │   └── scheduler.rs
│   │   └── Cargo.toml
│   ├── skills/              # Skill/plugin loader
│   │   └── Cargo.toml
│   ├── comms/               # Human communication (CLI, WebSocket bridge)
│   │   └── Cargo.toml
│   └── config/              # Configuration management
│       └── Cargo.toml
├── skills/                  # Runtime-loadable skill definitions (.md files)
├── memory/                  # Agent memory store (gitignored or selectively committed)
├── web-ui/                  # Optional Next.js frontend
│   ├── package.json
│   └── ...
├── tests/                   # Integration tests
├── CLAUDE.md                # This file
└── README.md
```

### Key Dependencies (Rust)
- `tokio` — async runtime
- `reqwest` — HTTP client (for LLM API and web fetching)
- `serde` / `serde_json` — serialization
- `clap` — CLI argument parsing
- `tracing` — structured logging
- `pulldown-cmark` — markdown parsing for memory/skill files
- `notify` — filesystem watching for hot-reload of skills

### Configuration
All configuration lives in a `selfclaw.toml` file:
```toml
[agent]
loop_interval_secs = 60
consolidation_every_n_cycles = 50
max_actions_per_cycle = 5

[llm]
provider = "anthropic"                     # anthropic | openai | google | ollama | openrouter | groq | xai | mistral | deepseek | <custom>
model = "claude-sonnet-4-20250514"
max_tokens = 4096
temperature = 0.7
# api_key = "sk-..."                       # Optional: overrides env var (e.g. ANTHROPIC_API_KEY, OPENAI_API_KEY)
# base_url = "https://custom-proxy.com"    # Optional: overrides default provider URL

[safety]
max_api_calls_per_hour = 100
max_file_writes_per_cycle = 10
sandbox_shell = true
allowed_directories = ["./memory", "./skills", "./output"]

[communication]
cli_enabled = true
web_ui_enabled = false
web_ui_port = 3000
```

---

## Development Methodology

### TDD (Test-Driven Development)
- Write failing tests **before** implementing any feature.
- Each crate must have unit tests; cross-crate behavior covered by integration tests in `tests/`.
- Target: all core logic (agent loop state transitions, memory operations, tool execution) must have test coverage.

### DDD (Domain-Driven Design)
- Bounded contexts map to crates: `agent-core`, `memory`, `tools`, `comms`, `skills`.
- Domain language: "cycle", "reflection", "purpose hypothesis", "episodic memory", "skill", "tool".
- Keep each crate focused on a single domain concern.

### Git Workflow
- **Commit after every completed subtask.** Each commit should represent a working state.
- Commit message format: `[crate-name] brief description` (e.g., `[memory] implement episodic log writer`).
- Use feature branches for major components; merge to `main` when stable.

### Code Principles
- **Simplicity first.** Minimal, readable code. Avoid over-abstraction.
- **Runtime extensibility.** Skills and memory are file-based and modifiable without recompilation.
- **Fail gracefully.** Every tool call should handle errors and log them. The agent loop must never crash — it should degrade and reflect on failures.

---

## Getting Started (for Claude Code)

1. Initialize the Rust workspace: `cargo init --name selfclaw` and set up the crate structure.
2. Implement `config` crate first (load `selfclaw.toml`).
3. Implement `memory` crate (file read/write/index).
4. Implement `tools` crate (start with `file_read`, `file_write`, `llm_call`).
5. Implement `agent-core` (the loop, state machine, purpose tracker).
6. Implement `comms` (CLI interface).
7. Wire everything together in `main.rs`.
8. Write integration tests.
9. (Optional) Build web UI.

Each step: **write tests first → implement → verify → commit.**