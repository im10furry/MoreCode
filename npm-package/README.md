# @morecode/agent

A multi-agent orchestrated AI coding assistant built with Rust + Ratatui.

## ⚠️ Alpha Version Notice

This is an early alpha version. Pre-built binaries are not available yet. You need to build from source.

## Installation

### From Source (Recommended for Alpha)

```bash
# 1. Clone the repository
git clone https://github.com/im10furry/MoreCode.git
cd MoreCode

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Build
cargo build -p cli --release

# 4. Run
./target/release/cli --help
```

### npm Installation (For Future Versions)

```bash
npm install -g @morecode/agent
morecode --help
```

## Quick Start

### Configure LLM Providers

```bash
# Create config directory
mkdir -p ~/.morecode

# Create providers config
cat > ~/.morecode/providers.toml << EOF
[providers.openai]
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"
EOF

# Set your API key
export OPENAI_API_KEY="your-api-key"
```

### Basic Usage

```bash
# Run a coding task
cargo run -p cli -- run "refactor the main function to be more readable"

# Check daemon status
cargo run -p cli -- daemon status

# Run diagnostics
cargo run -p cli -- doctor
```

## Features

- **Multi-agent orchestration**: 10 specialized agents collaborate by role
- **Recursive orchestration (Map-Filter-Reduce)**: Break down complex tasks
- **Intelligent routing**: Four-level routing complexity
- **Dual-layer security sandbox**: OS layer + WASM layer
- **Four-level progressive context compression**
- **Letta-style tiered memory system**
- **Multi-LLM provider support**
- **MCP protocol integration**
- **Five-layer prompt cache**
- **Ratatui terminal UI**
- **Daemon mode**

For full documentation, visit the [main repository](https://github.com/im10furry/MoreCode).

## License

MIT
