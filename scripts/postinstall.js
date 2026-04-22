#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const rootDir = path.resolve(__dirname, "..");
const exeSuffix = process.platform === "win32" ? ".exe" : "";
const binaryName = `morecode${exeSuffix}`;
const cargoBinary = process.env.CARGO || "cargo";
const nativeDir = path.join(rootDir, "bin", "native");
const releaseBinary = path.join(rootDir, "target", "release", binaryName);
const installedBinary = path.join(nativeDir, binaryName);

if (process.env.MORECODE_SKIP_BUILD === "1") {
  console.log("Skipping MoreCode native build because MORECODE_SKIP_BUILD=1.");
  process.exit(0);
}

const build = spawnSync(
  cargoBinary,
  ["build", "--release", "-p", "cli", "--bin", "morecode"],
  {
    cwd: rootDir,
    stdio: "inherit",
  }
);

if (build.error) {
  console.error(`Failed to start Cargo: ${build.error.message}`);
  console.error("Install Rust and Cargo first, then rerun the package install.");
  process.exit(1);
}

if (build.status !== 0) {
  process.exit(build.status || 1);
}

fs.mkdirSync(nativeDir, { recursive: true });
fs.copyFileSync(releaseBinary, installedBinary);

if (process.platform !== "win32") {
  fs.chmodSync(installedBinary, 0o755);
}

console.log(`Installed MoreCode native binary: ${installedBinary}`);
