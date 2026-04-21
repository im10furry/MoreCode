#!/usr/bin/env node

import * as os from 'os';
import * as path from 'path';
import * as fs from 'fs';
import { spawnSync } from 'child_process';

const BINARY_NAME = 'morecode';

function getBinaryName(): string {
  return (os.platform() as string) === 'win32' ? `${BINARY_NAME}.exe` : BINARY_NAME;
}

function getBinDir(): string {
  return path.join(__dirname, 'bin');
}

function getBinaryPath(): string {
  return path.join(getBinDir(), getBinaryName());
}

async function installBinary(): Promise<void> {
  // 创建 bin 目录
  const binDir = getBinDir();
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  // 检查是否有 cargo 可用
  const cargoPath = which('cargo');
  if (!cargoPath) {
    console.error('Error: Cargo (Rust package manager) is not installed.');
    console.error('Please install Rust from https://rustup.rs/ and try again.');
    process.exit(1);
  }

  // 检查是否是从源仓库安装的
  const isSourceRepo = fs.existsSync(path.join(process.cwd(), 'Cargo.toml'));
  if (isSourceRepo) {
    console.log('Building MoreCode from source...');
    const result = spawnSync('cargo', ['build', '-p', 'cli', '--release'], {
      stdio: 'inherit',
      cwd: process.cwd(),
    });

    if (result.status !== 0) {
      console.error('Failed to build MoreCode');
      process.exit(1);
    }

    // 复制二进制文件
    const sourcePath = path.join(process.cwd(), 'target', 'release', 'cli');
    const destPath = getBinaryPath();
    
    let finalSourcePath = sourcePath;
    if ((os.platform() as string) === 'win32' && !sourcePath.endsWith('.exe')) {
      finalSourcePath = sourcePath + '.exe';
    }

    if (fs.existsSync(finalSourcePath)) {
      fs.copyFileSync(finalSourcePath, destPath);
      if ((os.platform() as string) !== 'win32') {
        fs.chmodSync(destPath, '755');
      }
      console.log('✅ MoreCode built successfully!');
    } else {
      console.error('Failed to find built binary at:', finalSourcePath);
      process.exit(1);
    }
  } else {
    console.log('='.repeat(60));
    console.log('MoreCode Alpha Installation');
    console.log('='.repeat(60));
    console.log('');
    console.log('This is an early alpha version. To install MoreCode:');
    console.log('');
    console.log('1. Clone the repository:');
    console.log('   git clone https://github.com/im10furry/MoreCode.git');
    console.log('   cd MoreCode');
    console.log('');
    console.log('2. Install Rust (if not installed):');
    console.log('   Visit https://rustup.rs/');
    console.log('');
    console.log('3. Build MoreCode:');
    console.log('   cargo build -p cli --release');
    console.log('');
    console.log('4. Run:');
    console.log('   ./target/release/cli --help');
    console.log('');
    console.log('='.repeat(60));
  }
}

function which(cmd: string): string | null {
  const envPath = process.env.PATH || '';
  const pathExt = process.env.PATHEXT || '';
  const directories = envPath.split(path.delimiter);
  const extensions = (os.platform() as string) === 'win32' ? pathExt.split(';') : [''];

  for (const dir of directories) {
    for (const ext of extensions) {
      const fullPath = path.join(dir, cmd + ext);
      if (fs.existsSync(fullPath)) {
        return fullPath;
      }
    }
  }
  return null;
}

installBinary().catch(err => {
  console.error('Installation failed:', err);
  process.exit(1);
});
