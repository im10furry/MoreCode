# @morecode/agent v0.1.0-alpha.1 Release

## Overview

This is the first alpha release of the MoreCode Agent, available as an npm package.

## What's Included

### 📦 npm Package
- `@morecode/agent` npm package
- Wrapper around Rust binary
- CLI command `morecode`
- Post-install build script

### 📁 Package Structure
```
npm-package/
├── package.json       # npm config
├── tsconfig.json      # TypeScript config
├── README.md         # User guide
├── INSTALL.md       # Installation guide
├── RELEASE.md       # This file
├── src/             # TypeScript sources
│   ├── index.ts
│   ├── bin/morecode.ts
│   └── postinstall-simple.ts
└── dist/            # Compiled JS
    ├── index.js
    ├── bin/morecode.js
    └── postinstall-simple.js
```

## Installation

### For Testers

```bash
# Step 1: Install the npm package
npm install -g ./morecode-agent-0.1.0-alpha.1.tgz

# Step 2: Build the Rust binary
cd MoreCode
cargo build -p cli --bin morecode --release

# Step 3: Copy binary
# Linux/macOS:
cp target/release/morecode $(npm config get prefix)/lib/node_modules/@morecode/agent/dist/bin/morecode
chmod +x $(npm config get prefix)/lib/node_modules/@morecode/agent/dist/bin/morecode

# Windows:
copy target\release\morecode.exe $(npm config get prefix)\lib\node_modules\@morecode\agent\dist\bin\morecode.exe

# Step 4: Test
morecode --help
```

### For Developers (Alternative)

Just use cargo directly:

```bash
cd MoreCode
cargo build -p cli --bin morecode --release
./target/release/morecode --help
```

## Features

- **Multi-agent orchestration: Multiple specialized agents working together
- **Taskpile: Distributed task queue management
- **Cron support: Advanced scheduling capabilities
- **SQLite storage: Persistent encrypted storage
- **LLM providers: Support for OpenAI, DeepSeek, and more
- **Sandboxing: Secure execution environment

## Known Limitations (Alpha)

1. No pre-built binaries - need to build from source
2. npm package requires manual binary copy step
3. No Windows/macOS pre-compiled versions yet
4. Installation process is manual for alpha
5. Some features are still experimental

## Next Steps

For beta release, we plan:

- [ ] Pre-built binaries for major platforms
- [ ] Automatic binary downloading
- [ ] One-click installation
- [ ] Windows/macOS/Linux support
- [ ] Homebrew support
- [ ] APT/YUM/RPM packages
- [ ] Docker support

## Feedback

Please report issues and feedback to:
- GitHub Issues
- Project discussions

## Credits

- MoreCode Contributors
- Rust Community
- Open Source contributors

---
© 2026 MoreCode Project. All rights reserved.
