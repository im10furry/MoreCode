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
  // 否则提示用户手动构建
  throw new Error(`
MoreCode binary not found.

For this alpha version, you need to build it from source:
1. Clone: git clone https://github.com/im10furry/MoreCode
2. Install Rust: https://rustup.rs/
3. Build: cd MoreCode && cargo build -p cli --release
4. Copy the binary from target/release/${binName} to ${binPath}

Or use cargo directly: cd MoreCode && cargo run -p cli -- --help
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
