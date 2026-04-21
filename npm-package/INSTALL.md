# @morecode/agent Alpha Installation Guide

## Quick Install

### 1. Prerequisites

- **Node.js (18 or later)
- **Rust (for building the binary)

### 2. Install from Tarball

```bash
# Install the package
npm install -g ./morecode-agent-0.1.0-alpha.1.tgz

# Or build from source
git clone https://github.com/im10furry/MoreCode.git
cd MoreCode
npm install -g ./npm-package/morecode-agent-0.1.0-alpha.1.tgz
```

### 3. Build the Binary (Required for this alpha version)

```bash
# Clone the repository
cd MoreCode

# Build the Rust binary
cargo build -p cli --release

# Copy the binary
# Linux/macOS:
cp target/release/cli $(npm config get prefix)/lib/node_modules/@morecode/agent/dist/bin/morecode
chmod +x $(npm config get prefix)/lib/node_modules/@morecode/agent/dist/bin/morecode

# Windows:
copy target\release\cli.exe $(npm config get prefix)\lib\node_modules\@morecode\agent\dist\bin\morecode.exe
```

### 4. Verify Installation

```bash
morecode --help
```

### 5. Configure LLM Providers

```bash
# Create config directory
mkdir -p ~/.morecode

# Create providers config
cat > ~/.morecode/providers.toml << 'EOF'
[providers.openai]
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"
EOF

# Set your API key
export OPENAI_API_KEY="your-api-key"
```

### 6. Usage

```bash
# Run a coding task
morecode run "refactor the main function to be more readable"

# Check daemon status
morecode daemon status

# Run diagnostics
morecode doctor
```

## Development Mode

For development, you can directly use cargo:

```bash
cd MoreCode
cargo run -p cli -- --help
cargo run -p cli -- run "fix the bugs"
```

## Package Contents

The npm package includes:

- `@morecode/agent@0.1.0-alpha.1.tgz (the npm package
- JavaScript wrapper (bin/morecode.js)
- Post-install script for building

## Troubleshooting

If you have issues:

1. Make sure you have Rust installed
2. Check that you have Node.js 18+
3. Try direct cargo builds first
4. Report issues to GitHub repository

## License

MIT
