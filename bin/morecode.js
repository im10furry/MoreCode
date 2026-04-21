#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const rootDir = path.resolve(__dirname, "..");
const exeSuffix = process.platform === "win32" ? ".exe" : "";
const binaryName = `morecode${exeSuffix}`;
const candidates = [
  path.join(rootDir, "bin", "native", binaryName),
  path.join(rootDir, "target", "release", binaryName),
  path.join(rootDir, "target", "debug", binaryName),
];

const binaryPath = candidates.find((candidate) => fs.existsSync(candidate));

if (!binaryPath) {
  console.error("The MoreCode native binary was not found.");
  console.error("Try reinstalling the package, or run `npm rebuild morecode`.");
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.error) {
  console.error(`Failed to launch ${binaryName}: ${result.error.message}`);
  process.exit(1);
}

if (typeof result.status === "number") {
  process.exit(result.status);
}

process.exit(1);
