# MoreCode Agent

> A multi-agent orchestrated AI coding assistant built with Rust + Ratatui

---

## Quick Start

### Option 1: Cargo Install (Recommended)

The simplest way to install MoreCode is using Cargo:

```bash
# 1. Install Rust (if not already installed)
# Visit https://rustup.rs/

# 2. Install MoreCode
cargo install morecode-agent --git https://github.com/im10furry/MoreCode.git

# 3. Run
morecode --help
```

### Option 2: Build from Source

```bash
# 1. Clone the repository
git clone https://github.com/im10furry/MoreCode.git
cd MoreCode

# 2. Build
cargo build -p cli --release

# 3. Run
./target/release/cli --help
```

### Option 3: Use npm Package

```bash
# Install the npm package
npm install -g @morecode/agent

# Then install MoreCode via Cargo (recommended)
cargo install morecode-agent --git https://github.com/im10furry/MoreCode.git

# Now you can run
morecode --help
```

---

## Features

- **Multi-agent orchestration**: 10 specialized agents collaborate on coding tasks
- **Recursive orchestration**: Break down complex tasks into parallel subtasks
- **Intelligent routing**: Memory-aware routing, 80% of requests without LLM
- **Dual-layer security sandbox**: OS layer (Landlock + Seccomp) + WASM layer
- **Four-level progressive context compression**: Micro-compression → LLM summarization → Memory compression → Reactive truncation
- **Letta-style tiered memory system**: Core/Working/Recall/Archival four-layer memory
- **Multi-LLM provider support**: OpenAI, DeepSeek, Zhipu, Tongyi, Moonshot, Ollama, Anthropic, Google
- **MCP protocol integration**: Stdio/HTTP/Unix Socket support
- **Five-layer prompt cache**: Global/Org/Project/Session/Round caching
- **Ratatui terminal UI**: Real-time agent status, token consumption, communication topology
- **Daemon mode**: 24/7 autonomous operation with checkpoint recovery

---

## Usage

```bash
# Show help
morecode --help

# Start the TUI
morecode tui

# Run a task
morecode run "Your task description"

# Check environment
morecode doctor
```

---

## License

MIT
