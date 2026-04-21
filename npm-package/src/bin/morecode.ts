#!/usr/bin/env node

import * as os from 'os';
import * as path from 'path';
import * as fs from 'fs';
import { spawn } from 'child_process';

const BINARY_NAME = 'morecode';

function getBinaryPath(): string {
  const platform = os.platform() as string;
  const binName = platform === 'win32' ? `${BINARY_NAME}.exe` : BINARY_NAME;

  // 先尝试 npm 包自带的二进制
  let binPath = path.join(__dirname, '..', 'bin', binName);
  if (fs.existsSync(binPath)) {
    return binPath;
  }

  // 尝试在 PATH 中查找
  const pathDirs = (process.env.PATH || '').split(path.delimiter);
  for (const dir of pathDirs) {
    const candidate = path.join(dir, binName);
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  // 尝试查找 cargo 构建的二进制
  const cargoHome = process.env.CARGO_HOME || path.join(os.homedir(), '.cargo');
  const cargoBinPath = path.join(cargoHome, 'bin', binName);
  if (fs.existsSync(cargoBinPath)) {
    return cargoBinPath;
  }

  // 否则提供友好的安装提示
  throw new Error(`
╔═══════════════════════════════════════════════════════════════╗
║                    MoreCode Binary Not Found                   ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  To use MoreCode, you need to build it from source first:     ║
║                                                               ║
║  Option 1: Quick install (recommended)                        ║
║  ──────────────────────────────────────────────────────────   ║
║  1. Install Rust: https://rustup.rs/                         ║
║  2. Run: cargo install morecode-agent --git https://github.com/im10furry/MoreCode.git ║
║                                                               ║
║  Option 2: Build from source                                  ║
║  ──────────────────────────────────────────────────────────   ║
║  1. git clone https://github.com/im10furry/MoreCode.git       ║
║  2. cd MoreCode                                               ║
║  3. cargo build -p cli --release                              ║
║  4. Add target/release/ to your PATH                          ║
║                                                               ║
║  Option 3: Use cargo directly                                 ║
║  ──────────────────────────────────────────────────────────   ║
║  cd MoreCode && cargo run -p cli -- --help                    ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
`);
}

function runBinary(): void {
  try {
    const binaryPath = getBinaryPath();

    // 设置可执行权限
    if ((os.platform() as string) !== 'win32') {
      fs.chmodSync(binaryPath, '755');
    }

    // 执行二进制
    const args = process.argv.slice(2);
    const child = spawn(binaryPath, args, {
      stdio: 'inherit',
      env: process.env,
    });

    child.on('exit', (code) => {
      process.exit(code || 0);
    });

    child.on('error', (err) => {
      console.error('Failed to run MoreCode:', err);
      process.exit(1);
    });
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
}

runBinary();
