# MoreCode Agent (In Development)

> A multi-agent orchestrated AI coding assistant built with Rust + Ratatui, automating complex coding tasks through collaboration among specialized agents.

---

## Features

- **Multi-agent orchestration**: 10 specialized agents (Coordinator, Explorer, Impact Analyzer, Planner, Coder, Reviewer, Tester, Research, DocWriter, Debugger) collaborate by role, with clear separation of responsibilities and flexible extension (see detailed architecture below).
- **Recursive orchestration (Map-Filter-Reduce)**: Any agent can serve as a "sub-coordinator" to break down complex tasks into subtasks to be executed in parallel, with dynamic token budget allocation and depth control.
- **Intelligent routing and decision-making**: Four-level routing complexity (Simple / Medium / Complex / Research); 80% of requests can be routed without an LLM; memory-aware routing accelerates with project knowledge.
- **Dual-layer security sandbox**: OS layer (Landlock + Seccomp) as the secure foundation, and WASM layer (Wasmtime + WASI) providing function-level isolation and cross-platform capabilities.
- **Four-level progressive context compression**: Micro-compression → LLM summarization → Memory compression → Reactive truncation, combined with Focus for innovative context compression.
- **Letta-style tiered memory system**: Four-layered memory (Core/Working/Recall/Archival) supporting cross-session reuse, incremental updates, and sleep-time compute.
- **Multi-LLM provider support**: Unified OpenAI-compatible layer covering DeepSeek, Zhipu, Tongyi, Moonshot, Ollama, with optional Anthropic and Google Gemini, plus built-in semantic cache middleware.
- **MCP protocol integration**: Based on rmcp v1.4+ official SDK, acts as both MCP client and server, supporting Stdio/HTTP/Unix Socket.
- **Five-layer prompt cache**: Global/Org/Project/Session/Round hierarchical caching, expected to save 50-67% of input token costs.
- **Ratatui terminal UI**: Real-time display of agent execution status, token consumption, communication topology, supporting progress/code/confirmation streams.
- **Daemon mode**: 24/7 autonomous operation, checkpoint recovery, cost control, notification system.

---

## Tech Stack

| Category         | Technology        | Description                                                   |
|------------------|------------------|---------------------------------------------------------------|
| Main Language    | **Rust**         | High-performance, memory safe, zero-cost abstractions, single binary build |
| Concurrency      | **Actor + CSP Channel** | Each agent as an independent actor, communicating async via channels  |
| Async Runtime    | **Tokio**        | Reactor-based scheduling, Rust de-facto standard              |
| UI Framework     | **Ratatui**      | Terminal TUI framework, immediate mode rendering              |
| Security Sandbox | **Landlock + Seccomp + WASM** | Dual-level sandbox design                                    |
| Tool Protocol    | **MCP (rmcp v1.4+)** | AI tool interoperability standard                            |
| Code Analysis    | **Tree-sitter**  | Incremental parsing, repo map generation                      |
| Observability    | **tracing + OpenTelemetry + Langfuse** | Agent-specific spans, open-source self-hosting          |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────[...]
│                        User Input (CLI / TUI)                    │
│                   Ratatui Terminal UI Layer                      │
└──────────────────────────┬───────────────────────────────────────[...]
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────[...]
│                     ┌───────────────────┐                        │
│                     │   Coordinator     │                        │
│                     │   (Main Agent)    │                        │
│                     └────────┬──────────┘                        │
│              ┌───────────────┼───────────────┐                   │
│     ┌────────▼──────┐ ┌─────▼──────┐ ┌──────▼────────┐          │
│     │   Explorer    │ │  Planner   │ │ ImpactAnalyzer│          │
│     └───────────────┘ └─────┬──────┘ └───────────────┘          │
│              ┌─────────────┼─────────────┐                      │
│     ┌────────▼──────┐ ┌───▼────────┐ ┌──▼──────────┐           │
│     │    Coder      │ │  Reviewer  │ │   Tester    │           │
│     └───────────────┘ └────────────┘ └─────────────┘           │
│  ─────────────────────────────────────────────────────────────  │
│     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │
│     │  Research   │ │  DocWriter  │ │  Debugger   │             │
│     └─────────────┘ └─────────────┘ └─────────────┘             │
└─────────────────────────────────────────────────────────────────[...]
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────[...]
│  Tool Layer: File I/O | Code Search | Git Ops | Terminal Exec | LLM Calls  │
└─────────────────────────────────────────────────────────────────[...]
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────[...]
│  State Layer: Memory Management | Configuration | Checkpoints | Audit Logs  │
└─────────────────────────────────────────────────────────────────[...]
```

### Design Principles

1. **Understand before analyzing or executing** — Coding tasks always pass through Explorer → Impact Analyzer → Coder.
2. **Not every task needs multiple agents** — Simple tasks are routed to a single agent to avoid unnecessary orchestration.
3. **Minimal context injection** — Each agent receives only the minimal context required for its responsibility.
4. **Summary-based communication** — Agents communicate via structured summaries (JSON), not raw context blobs.

---

## Project Structure

```
morecode-agent/
├── Cargo.toml                    # Workspace root config
├── crates/
│   ├── mc-core/                  # Core types and traits (no deps)
│   ├── mc-coordinator/           # Orchestrator: intent understanding, routing, monitoring
│   ├── mc-agent/                 # Agent trait + 10 agent implementations
│   ├── mc-communication/         # Four-level comms (control/status/data/broadcast)
│   ├── mc-llm/                   # LLM provider abstraction + multi-backend support
│   ├── mc-context/               # Context management & four-level compression
│   ├── mc-memory/                # Letta-style layered memory system
│   ├── mc-prompt/                # Prompt templates & five-layer caching
│   ├── mc-tool/                  # Tool registration & built-ins
│   ├── mc-config/                # Multi-level config (global/project/env)
│   ├── mc-sandbox/               # Guardian dual sandbox
│   ├── mc-recursive/             # Recursive orchestration engine (Map-Filter-Reduce)
│   ├── mc-daemon/                # Daemon mode & lifecycle management
│   ├── mc-tui/                   # Ratatui terminal UI
│   └── mc-cli/                   # CLI entry point (bin crate)
├── prompts/                      # Prompt templates (system/tools/org/project)
├── config/                       # Default configs
└── tests/                        # Integration & end-to-end tests
```

---

## Quick Start

### Requirements

- Rust stable (latest)
- <!-- TODO: Specify minimum Rust version -->

### Build

```bash
# Clone the project
git clone https://github.com/<!-- TODO: fill repo address -->/morecode-agent.git
cd morecode-agent

# Minimal build (no optional features)
cargo build -p mc-cli --no-default-features

# Full build (all providers and sandbox backends)
cargo build --all-features
```

### Basic Usage

```bash
# Interactive run
morecode

# Daemon mode
morecode daemon start

# Environment diagnostics
morecode doctor
```

### Feature Flags

| Feature     | Description                 |
|-------------|----------------------------|
| `tui`       | Ratatui terminal UI (on by default) |
| `daemon`    | Daemon mode (on by default)         |
| `anthropic` | Anthropic Claude provider           |
| `google`    | Google Gemini provider              |
| `landlock`  | Linux Landlock file system sandbox  |
| `seccomp`   | Seccomp syscall filtering           |
| `wasm`      | WASM sandbox (Wasmtime + WASI)      |
| `mock`      | Mock LLM provider (for testing)     |

---

## Configuration

Configuration uses a multi-level merge strategy: **Project > Global > Code Defaults**

```bash
# Global config dir
~/.morecode/
├── config.toml          # Global defaults
├── routing.toml         # Routing rules
├── daemon.toml          # Daemon config
├── providers.toml       # LLM provider configs
└── cost.toml            # Cost budget config

# Project config dir (overrides global)
.morecode/
├── config.toml
└── routing.toml
```

### LLM Provider Config Example

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

MoreCode Agent is designed for zero-intrusion extensibility—add new components without touching existing code:

| Extension Scenario   | Steps                                                         |
|---------------------|---------------------------------------------------------------|
| **Add Agent**           | ① Implement `Agent` trait in `mc-agent/src/{name}/mod.rs` → ② Register in `registry.rs` → ③ Add system prompt to `prompts/system/{name}.md`    |
| **Add LLM Provider**    | ① Implement `LlmProvider` trait in `mc-llm/src/{name}/` → ② Add feature flag in Cargo.toml                             |
| **Add Tool**            | ① Implement `Tool` trait in `mc-tool/src/builtin/{name}.rs` → ② Assign visibility in `catalog/` → ③ Register in `registry.rs`                  |
| **Add Sandbox Backend** | ① Implement in `mc-sandbox/src/os_layer/{name}.rs` → ② Add feature gate in `lib.rs`                                     |

---

## Roadmap

```
Phase 1 (MVP):
  LLM Provider → Token counting → Context compression (L1+L4) → Permissions → Sandbox → Comms → AST (basic)

Phase 2 (Enhance):
  Prompt cache (5 layers) → Streaming output → Interrupt/cancel → Letta memory → Hook → MCP → Prompt templates → Tool registration

Phase 3 (Innovate):
  Context compression (Focus) → AST (Probe + LSP-MCP) → Telemetry → Config → File watching → Checkpoint

Phase 4 (Frontier):
  Context compression (ACON distillation) → Semantic retrieval (Mem0-G) → WASM loading → MCP Server → IDE integration
```

---

## License

[GNU General Public License v3.0](LICENSE)