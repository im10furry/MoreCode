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
const { name: packageName, version } = require('../package.json');
const BINARY_NAME = 'morecode';
const PACKAGE_VERSION = version;
function getPlatform() {
    const platform = os.platform();
    if (platform === 'win32')
        return 'windows';
    return platform;
}
function getArch() {
    const arch = os.arch();
    if (arch === 'x64')
        return 'x86_64';
    if (arch === 'arm64')
        return 'aarch64';
    return arch;
}
function getBinaryUrl() {
    const platform = getPlatform();
    const arch = getArch();
    // 这里是一个占位 URL，需要根据实际构建情况更新
    // 实际使用时需要提供预编译的二进制下载地址
    return `https://github.com/im10furry/MoreCode/releases/download/v${PACKAGE_VERSION}/${BINARY_NAME}-${platform}-${arch}.tar.gz`;
}
function getBinaryPath() {
    const platform = getPlatform();
    const binDir = path.join(__dirname, 'bin');
    const binName = platform === 'windows' ? `${BINARY_NAME}.exe` : BINARY_NAME;
    return path.join(binDir, binName);
}
async function installBinary() {
    // 创建 bin 目录
    const binDir = path.join(__dirname, 'bin');
    if (!fs.existsSync(binDir)) {
        fs.mkdirSync(binDir, { recursive: true });
    }
    try {
        // 先尝试检查是否需要构建
        // 目前我们提供一个警告并提示用户手动构建
        console.warn('Warning: Pre-built binaries are not available for this alpha version.');
        console.warn('Please build MoreCode from source:');
        console.warn('1. Clone the repository from https://github.com/im10furry/MoreCode');
        console.warn('2. Install Rust: https://rustup.rs/');
        console.warn('3. Build: cd MoreCode && cargo build -p cli --release');
        console.warn('4. Copy the binary to: ' + getBinaryPath());
        console.warn('');
        console.warn('For this alpha version, you can also use cargo directly.');
        process.exit(0);
    }
    catch (error) {
        console.error('Installation failed:', error);
        process.exit(1);
    }
}
installBinary().catch(console.error);
