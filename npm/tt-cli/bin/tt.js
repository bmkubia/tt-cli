#!/usr/bin/env node
// Launcher that locates the platform-specific tt binary bundled via npm.

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PLATFORM_MAP = {
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-musl",
  "linux-arm64": "aarch64-unknown-linux-musl",
  "win32-x64": "x86_64-pc-windows-msvc",
  // Windows on ARM currently ships the x64 binary (runs via emulation).
  "win32-arm64": "x86_64-pc-windows-msvc",
};

const platformKey = `${process.platform}-${process.arch}`;
const targetTriple = PLATFORM_MAP[platformKey];

if (!targetTriple) {
  throw new Error(`Unsupported platform: ${platformKey}`);
}

const vendorRoot = path.join(__dirname, "..", "vendor");
const binaryDir = path.join(vendorRoot, targetTriple, "tt");
const binaryName = process.platform === "win32" ? "tt.exe" : "tt";
const binaryPath = path.join(binaryDir, binaryName);

if (!existsSync(binaryPath)) {
  throw new Error(`tt binary not found at ${binaryPath}`);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

const forwardSignal = (signal) => {
  if (child.killed) {
    return;
  }
  try {
    child.kill(signal);
  } catch {
    /* ignore */
  }
};

["SIGINT", "SIGTERM", "SIGHUP"].forEach((sig) => {
  process.on(sig, () => forwardSignal(sig));
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
