# MoreCode Agent - NPM Package Usage

## Overview

This document explains how to use the MoreCode Agent npm package. While we recommend using `cargo install` for the best experience, the npm package provides a convenient way to integrate MoreCode into Node.js projects.

## Installation

### Option 1: Global Installation

```bash
npm install -g @morecode/agent
```

### Option 2: Local Project Installation

```bash
npm install --save-dev @morecode/agent
```

## Prerequisites

The MoreCode npm package requires the MoreCode binary to be installed. You can install it using Cargo:

```bash
# Install Rust (if not already installed)
# Visit https://rustup.rs/

# Install MoreCode binary
cargo install morecode-agent --git https://github.com/im10furry/MoreCode.git
```

## Usage

### Global Installation

If you installed the package globally, you can run MoreCode directly:

```bash
morecode --help

# Start the TUI
morecode tui

# Run a task
morecode run "Your task description"

# Check environment
morecode doctor
```

### Local Project Installation

If you installed the package locally, you can run it using npx:

```bash
npx morecode --help

# Or add scripts to package.json
# "scripts": {
#   "morecode": "morecode"
# }
# Then run: npm run morecode -- --help
```

## How It Works

The npm package provides a wrapper around the MoreCode binary. When you run `morecode`:

1. It looks for the MoreCode binary in these locations (in order):
   - Inside the npm package's `dist/bin/` directory
   - In your system's PATH
   - In your Cargo home directory (`~/.cargo/bin/`)

2. If the binary is found, it executes it with the provided arguments
3. If the binary is not found, it provides clear instructions on how to install it

## Troubleshooting

### Binary Not Found

If you see a "MoreCode binary not found" error:

1. Make sure Rust is installed: https://rustup.rs/
2. Run: `cargo install morecode-agent --git https://github.com/im10furry/MoreCode.git`
3. Verify the installation: `morecode --version`

### Permission Issues

If you encounter permission errors:

- On Linux/macOS: Try running with `sudo` or adjust your permissions
- On Windows: Run your terminal as Administrator

### Environment Variables

You can set these environment variables to customize the behavior:

- `CARGO_HOME`: Path to your Cargo home directory (default: `~/.cargo`)
- `PATH`: Ensure your Cargo bin directory is in your PATH

## Integrating with Node.js Projects

You can use the npm package to integrate MoreCode into your Node.js projects:

```javascript
const { spawn } = require('child_process');

function runMoreCode(args) {
  return new Promise((resolve, reject) => {
    const child = spawn('morecode', args, {
      stdio: 'inherit',
      env: process.env
    });
    
    child.on('exit', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`MoreCode exited with code ${code}`));
      }
    });
  });
}

// Example usage
runMoreCode(['run', 'Create a new React component'])
  .then(() => console.log('Task completed!'))
  .catch(err => console.error('Error:', err));
```

## License

MIT
