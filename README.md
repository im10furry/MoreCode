# MoreCode Agent (WIP)

> An AI coding assistant for automating complex coding tasks via multi-agent orchestration, built with Rust + Ratatui, coordinated by specialized agents.

---

## Features

- **Multi-agent orchestration**: 10 specialized agents (Coordinator, Explorer, Impact Analyzer, Planner, Coder, Reviewer, Tester, Research, DocWriter, Debugger) collaborate, each with clear responsibilities.
- **Recursive orchestration (Map-Filter-Reduce)**: Any agent can act as a “sub-coordinator” to split tasks into subtasks for parallel processing. Supports dynamic token budgeting and recursion depth control.
- **Intelligent routing**: Four-level complexity routing (simple/medium/complex/research). 80% of routing can be accomplished without LLM, with memory-aware, project-knowledge-driven decisions.
- **Dual-layer sandbox**: OS layer (Landlock + Seccomp) as security foundation, WASM layer (Wasmtime + WASI) for function-level isolation and cross-platform capabilities.
- **Four-stage progressive context compression**: Micro → LLM summary → Memory compression → Reactive truncation, with Focus for adaptive innovation.
- **Letta-style layered memory system**: Core/Working/Recall/Archival memory supports cross-session reuse, incremental updates, and sleep-time compute.
- **Multi-LLM provider support**: Unified OpenAI-compatible layer supports DeepSeek/Moonshot/Baichuan/Ollama, optional Anthropic & Gemini, with built-in semantic cache.
- **MCP protocol integration**: Based on rmcp v1.4+ official SDK, acts as both MCP client and server, supports Stdio/HTTP/Unix Socket.
- **Five-level prompt cache**: Global/org/project/session/turn cache; expected to save 50-67% token cost per input.
- **Ratatui terminal UI**: Real-time agent execution/status/token usage/topology with progress/code/confirmation feedback modes.
- **Daemon mode**: 7×24 unattended run, checkpoint recovery, cost control, notification system.

---

## Tech Stack

| Category        | Tech                    | Description                                |
|-----------------|------------------------|--------------------------------------------|
| Main Language   | **Rust**               | High-performance, memory-safe, zero-cost abstraction |
| Concurrency     | **Actor + CSP Channel**| Each agent is an actor, async via channels |
| Async Runtime   | **Tokio**              | Reactor model, Rust ecosystem standard     |
| UI Framework    | **Ratatui**            | Terminal TUI, immediate mode rendering     |
| Sandboxing      | **Landlock + Seccomp + WASM** | Dual-layer sandbox                        |
| Tool Protocol   | **MCP (rmcp v1.4+)**   | AI tool interop protocol                   |
| Code Parsing    | **Tree-sitter**        | Incremental syntax parsing, repo mapping   |
| Observability   | **tracing + OpenTelemetry + Langfuse** | Agent tracing and self-hosted telemetry   |

---

## Architecture Overview

```text
┌─────────────────────────────────────────────┐
│              User Input (CLI/TUI)           │
│           Ratatui Terminal UI Layer         │
└───────────────┬─────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────┐
│              ┌─────────────┐                │
│              │ Coordinator │                │
│              └─────┬───────┘                │
│      ┌─────────────┼────────────┐            │
│  ┌───▼────┐ ┌──────▼────┐ ┌────▼───────┐    │
│  │Explorer│ │ Planner  │ │ImpactAnalyzer│    │
│  └────────┘ └──────────┘ └─────────────┘    │
│      ┌────────────┼────────────┐            │
│  ┌───▼────┐ ┌─────▼─────┐ ┌───▼─────┐      │
│  │ Coder  │ │ Reviewer  │ │ Tester  │      │
│  └────────┘ └───────────┘ └─────────┘      │
│ ─────────────────────────────────────────  │
│  ┌─────────┐ ┌───────────┐ ┌──────────┐     │
│  │Research │ │ DocWriter │ │ Debugger │     │
│  └─────────┘ └───────────┘ └──────────┘     │
└─────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────┐
│  Tools: FS ops | code search | Git | shell | LLM |
└─────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────┐
│ State Management: memory | config | checkpoint | audit |
└─────────────────────────────────────────────┘
```

### Design Principles

1. **Understand, analyze, then code** — Every coding task must go through Explorer → Impact Analyzer → Coder steps.
2. **Not every task needs multi-agent orchestration** — Simple tasks are routed directly to a single agent to avoid unnecessary overhead.
3. **Context-on-demand injection** — Each agent only receives the minimum context needed for its specific responsibility.
4. **Summary communication** — Agents communicate structured summaries (JSON), not raw context.

---

## Project Structure

```text
morecode-agent/
├── Cargo.toml                    # Workspace root config
├── crates/
│   ├── mc-core/                  # Core types and traits
│   ├── mc-coordinator/           # Coordinator (intent, routing, monitoring)
│   ├── mc-agent/                 # Agent trait + 10 agent implementations
│   ├── mc-communication/         # Four-level messaging system
│   ├── mc-llm/                   # LLM provider abstraction + multiple backends
│   ├── mc-context/               # Context management & compression policies
│   ├── mc-memory/                # Letta-style layered memory
│   ├── mc-prompt/                # Prompt templates & cache
│   ├── mc-tool/                  # Tool registration & built-ins
│   ├── mc-config/                # Multi-level config
│   ├── mc-sandbox/               # Dual-layer sandbox
│   ├── mc-recursive/             # Recursive orchestration engine
│   ├── mc-daemon/                # Daemon mode & lifecycle
│   ├── mc-tui/                   # Ratatui UI
│   └── mc-cli/                   # CLI entry
├── prompts/                      # Prompt templates
├── config/                       # Default config templates
└── tests/                        # Integration and E2E tests
```

---

## Getting Started

### Prerequisites

- Rust stable (latest)
- <!-- TODO: Specify minimum required Rust version -->

### Build

```bash
# Clone the repo
git clone https://github.com/<!-- TODO: Add correct repo address here -->/morecode-agent.git
cd morecode-agent

# Minimal build (core only)
cargo build -p mc-cli --no-default-features

# Full build (all providers and sandboxes)
cargo build --all-features
```

### Basic Usage

```bash
# Run interactively
morecode

# Daemon mode
morecode daemon start

# Diagnose environment
morecode doctor
```

### Feature Flags

| Feature     | Description                                     |
|-------------|-------------------------------------------------|
| `tui`       | Ratatui terminal UI (default enabled)           |
| `daemon`    | Daemon mode (default enabled)                   |
| `anthropic` | Anthropic Claude provider                       |
| `google`    | Google Gemini provider                          |
| `landlock`  | Linux Landlock FS sandbox                       |
| `seccomp`   | Seccomp syscall filtering                       |
| `wasm`      | WASM sandbox (Wasmtime + WASI)                  |
| `mock`      | Mock LLM provider (for testing)                 |

---

## Configuration

Configuration is multi-level merged: **project > global > code defaults**

```bash
# Global config directory
~/.morecode/
├── config.toml          # Global defaults
├── routing.toml         # Routing rules
├── daemon.toml          # Daemon config
├── providers.toml       # LLM providers config
└── cost.toml            # Cost budget

# Project config dir (overrides global)
.morecode/
├── config.toml
└── routing.toml
```

### Example LLM Provider Config

```toml
# ~/.morecode/providers.toml
[providers.openai]
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[providers.deepseek]
model = "deepseek-chat"
base_url = "https://api.deepseek.com/v1"
api_key_env = "DEEPSEEK_API_KEY"
```

---

## Extension Development

MoreCode Agent is zero-intrusive and highly extensible — adding new components requires no changes to existing code:

| Extension   | How to add                                                                                  |
|-------------|--------------------------------------------------------------------------------------------|
| **New Agent**        | ① Implement in `mc-agent/src/{name}/mod.rs` → ② Register in `registry.rs` → ③ Add prompt in `prompts/system/{name}.md` |
| **New LLM Provider** | ① Implement in `mc-llm/src/{name}/` → ② Add feature flag in `Cargo.toml`          |
| **New Tool**         | ① Implement in `mc-tool/src/builtin/{name}.rs` → ② Assign in `catalog/` → ③ Register in `registry.rs`                |
| **New Sandbox**      | ① Implement in `mc-sandbox/src/os_layer/{name}.rs` → ② Add feature flag in `lib.rs`                                    |

---

## Roadmap

```text
Phase 1 (MVP):
  LLM Provider → Token counting → Context compression (L1+L4) → Permission mgmt → Sandbox → Communication → Basic AST

Phase 2 (Enhance):
  Prompt cache (5-level) → Streaming output → Interrupt/cancel → Letta memory → Hooks → MCP → Prompt templates → Tool registry

Phase 3 (Innovate):
  Context compression (Focus) → AST (Probe+LSP-MCP) → Telemetry → Config → FS watching → Checkpoints

Phase 4 (Frontier):
  Context compression (ACON distillation) → Semantic retrieval (Mem0-G) → WASM load → MCP Server → IDE Integration
```

---

## License

[GNU General Public License v3.0](LICENSE)