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
function getBinaryPath() {
    const platform = os.platform();
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
function runBinary() {
    try {
        const binaryPath = getBinaryPath();
        // 设置可执行权限
        if (os.platform() !== 'win32') {
            fs.chmodSync(binaryPath, '755');
        }
        // 执行二进制
        const args = process.argv.slice(2);
        const child = (0, child_process_1.spawn)(binaryPath, args, {
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
    }
    catch (err) {
        console.error(err);
        process.exit(1);
    }
}
runBinary();
