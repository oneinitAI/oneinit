#!/usr/bin/env node
/**
 * OneInit npm postinstall — download pre-built binary.
 *
 * Sources (tried in order):
 *   1. ONEINIT_CDN env var (e.g. https://cdn.ogmua.cn/oneinit)
 *   2. picui CDN fallback
 *   3. GitHub Releases (if available)
 *
 * If no binary is found, prints build-from-source instructions.
 */

"use strict";

const os = require("os");
const path = require("path");
const fs = require("fs");
const https = require("https");
const { execSync } = require("child_process");

// ── Config ──────────────────────────────────────────────────────────
const CDN_BASE =
  process.env.ONEINIT_CDN || "https://picui.ogmua.cn/oneinit";
const UA = "oneinit-npm-installer/1.0";

// ── Platform detection ──────────────────────────────────────────────
const PLATFORM_MAP = { linux: "linux", darwin: "macos", win32: "windows" };
const ARCH_MAP = { x64: "x86_64", arm64: "aarch64" };

const platform = PLATFORM_MAP[os.platform()];
const arch = ARCH_MAP[os.arch()];

if (!platform || !arch) {
  console.error(`[ERROR] Unsupported platform: ${os.platform()}/${os.arch()}`);
  process.exit(1);
}

const isWin = platform === "windows";
const ext = isWin ? ".exe" : "";
const archiveExt = isWin ? "zip" : "tar.gz";
const binaryName = `oneinit${ext}`;
const installDir = path.dirname(process.argv[1] || __dirname);

console.log(`[INSTALL] OneInit for ${platform}/${arch}`);

// ── HTTP helper ─────────────────────────────────────────────────────
function get(url) {
  return new Promise((resolve, reject) => {
    const opts = { headers: { "User-Agent": UA } };
    const req = https.get(url, opts, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        return https.get(res.headers.location, opts, (r2) => {
          const chunks = [];
          r2.on("data", (c) => chunks.push(c));
          r2.on("end", () => resolve(Buffer.concat(chunks)));
          r2.on("error", reject);
        }).on("error", reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`HTTP ${res.statusCode}`));
      }
      const chunks = [];
      res.on("data", (c) => chunks.push(c));
      res.on("end", () => resolve(Buffer.concat(chunks)));
    });
    req.on("error", reject);
    req.setTimeout(15000, () => { req.destroy(); reject(new Error("timeout")); });
  });
}

// ── Download & extract ──────────────────────────────────────────────
async function tryDownload(name, url) {
  console.log(`[INSTALL] Trying: ${url}`);
  const buf = await get(url);

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "oneinit-"));
  const archivePath = path.join(tmpDir, name);

  fs.writeFileSync(archivePath, buf);

  if (isWin) {
    try {
      execSync(
        `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}' -Force"`,
        { stdio: "pipe", timeout: 30000 }
      );
    } catch {
      // ignore — may extract via another method
    }
  } else {
    try {
      execSync(`tar -xzf "${archivePath}" -C "${tmpDir}"`, { stdio: "pipe" });
    } catch {
      // ignore
    }
  }

  // Find binary
  function find(dir, depth = 0) {
    if (depth > 3) return null;
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const e of entries) {
      const full = path.join(dir, e.name);
      if (e.isFile() && (e.name === binaryName || e.name === "oneinit")) return full;
      if (e.isDirectory()) {
        const r = find(full, depth + 1);
        if (r) return r;
      }
    }
    return null;
  }

  const found = find(tmpDir);
  if (found) {
    const dest = path.join(installDir, binaryName);
    fs.copyFileSync(found, dest);
    if (!isWin) fs.chmodSync(dest, 0o755);
    fs.rmSync(tmpDir, { recursive: true, force: true });
    return dest;
  }

  fs.rmSync(tmpDir, { recursive: true, force: true });
  return null;
}

// ── Main ────────────────────────────────────────────────────────────
async function main() {
  const tag = process.env.ONEINIT_VERSION || "latest";
  const archive = `oneinit-${tag}-${platform}-${arch}.${archiveExt}`;

  // 1. Try CDN
  try {
    const dest = await tryDownload(archive, `${CDN_BASE}/${archive}`);
    if (dest) {
      console.log(`[OK] Installed to: ${dest}`);
      printNextSteps();
      return;
    }
  } catch (e) {
    console.error(`[WARN] CDN: ${e.message}`);
  }

  // 2. Try GitHub (may be unavailable)
  const ghUrl = `https://github.com/BG4JTS/oneinit/releases/download/${tag}/${archive}`;
  try {
    const dest = await tryDownload(archive, ghUrl);
    if (dest) {
      console.log(`[OK] Installed to: ${dest}`);
      printNextSteps();
      return;
    }
  } catch {
    console.error("[WARN] GitHub: unavailable");
  }

  // 3. No binary found — print build instructions
  console.error("");
  console.error("╔══════════════════════════════════════════════════════════╗");
  console.error("║  No pre-built binary found for your platform.           ║");
  console.error("║                                                        ║");
  console.error("║  Build from source:                                    ║");
  console.error("║    git clone https://github.com/BG4JTS/oneinit.git      ║");
  console.error("║    cd oneinit && cargo build --release                  ║");
  console.error("║                                                        ║");
  console.error("║  Or set custom CDN:                                    ║");
  console.error("║    ONEINIT_CDN=https://your-cdn.com/oneinit npm i -g     ║");
  console.error("╚══════════════════════════════════════════════════════════╝");
  console.error("");

  // Still write a placeholder so the npm bin shim works
  const dest = path.join(installDir, binaryName);
  if (!fs.existsSync(dest)) {
    const placeholder = isWin
      ? `@echo off\r\necho oneinit binary not installed. Run: cargo install oneinit\r\n`
      : `#!/bin/sh\necho "oneinit binary not installed. Run: cargo install oneinit"\n`;
    fs.writeFileSync(dest, placeholder, { mode: isWin ? 0o644 : 0o755 });
  }
}

function printNextSteps() {
  console.log("");
  console.log("Next steps:");
  console.log("  oneinit --version");
  console.log("  oneinit doctor");
  console.log("  oneinit install python3.11");
  console.log("");
}

main().catch((err) => {
  console.error(`[ERROR] ${err.message}`);
  process.exit(1);
});
