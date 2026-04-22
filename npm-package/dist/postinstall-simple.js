#!/usr/bin/env node
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const os = __importStar(require("os"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const child_process_1 = require("child_process");
const BINARY_NAME = 'morecode';
function getBinaryName() {
    return os.platform() === 'win32' ? `${BINARY_NAME}.exe` : BINARY_NAME;
}
function findWorkspaceRoot(startDir) {
    let dir = startDir;
    for (let i = 0; i < 8; i++) {
        const cargoToml = path.join(dir, 'Cargo.toml');
        const cliCargoToml = path.join(dir, 'cli', 'Cargo.toml');
        if (fs.existsSync(cargoToml) && fs.existsSync(cliCargoToml)) {
            return dir;
        }
        const parent = path.dirname(dir);
        if (parent === dir) {
            break;
        }
        dir = parent;
    }
    return null;
}
function getBinDir() {
    return path.join(__dirname, 'bin');
}
function getBinaryPath() {
    return path.join(getBinDir(), getBinaryName());
}
async function installBinary() {
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
    const packageDir = path.resolve(__dirname, '..');
    const workspaceRoot = findWorkspaceRoot(packageDir);
    if (workspaceRoot) {
        console.log('Building MoreCode from source...');
        const result = (0, child_process_1.spawnSync)('cargo', ['build', '-p', 'cli', '--bin', BINARY_NAME, '--release'], {
            stdio: 'inherit',
            cwd: workspaceRoot,
        });
        if (result.status !== 0) {
            console.error('Failed to build MoreCode');
            process.exit(1);
        }
        // 复制二进制文件
        const sourcePath = path.join(workspaceRoot, 'target', 'release', getBinaryName());
        const destPath = getBinaryPath();
        if (fs.existsSync(sourcePath)) {
            fs.copyFileSync(sourcePath, destPath);
            if (os.platform() !== 'win32') {
                fs.chmodSync(destPath, '755');
            }
            console.log('✅ MoreCode built successfully!');
        }
        else {
            console.error('Failed to find built binary at:', sourcePath);
            process.exit(1);
        }
    }
    else {
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
        console.log('   cargo build -p cli --bin morecode --release');
        console.log('');
        console.log('4. Run:');
        console.log('   ./target/release/morecode --help');
        console.log('');
        console.log('='.repeat(60));
    }
}
function which(cmd) {
    const envPath = process.env.PATH || '';
    const pathExt = process.env.PATHEXT || '';
    const directories = envPath.split(path.delimiter);
    const extensions = os.platform() === 'win32' ? pathExt.split(';') : [''];
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
