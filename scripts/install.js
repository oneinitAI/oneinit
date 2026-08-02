#!/usr/bin/env node
/**
 * OneInit network installer — download pre-built binary with SHA256 verification.
 *
 * Sources (tried in order):
 *   1. GitHub Releases (primary; requires SHA256SUMS.txt asset)
 *   2. ONEINIT_CDN env var / picui CDN fallback (also requires SHA256SUMS.txt)
 *
 * Every binary is verified against its release SHA256SUMS.txt before install.
 * If no verifiable binary can be obtained, prints build-from-source
 * instructions and exits non-zero — an unverified binary is never installed.
 */

"use strict";

const os = require("os");
const path = require("path");
const fs = require("fs");
const https = require("https");
const crypto = require("crypto");
const { execSync } = require("child_process");

// ── Config ──────────────────────────────────────────────────────────
const REPO = "oneinitAI/oneinit";
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

// ── SHA256 verification ─────────────────────────────────────────────
function sha256(buf) {
  return crypto.createHash("sha256").update(buf).digest("hex");
}

// Parse SHA256SUMS.txt content -> { filename: lowercase-hex }
function parseSums(text) {
  const map = {};
  for (const line of text.split(/\r?\n/)) {
    const m = line.match(/^([0-9a-fA-F]{64})\s+[* ]?(.+)$/);
    if (m) map[m[2].trim()] = m[1].toLowerCase();
  }
  return map;
}

// Throws on missing entry or hash mismatch (tamper -> refuse to install)
function verifyArchive(buf, archiveName, sumsText, source) {
  const expected = parseSums(sumsText)[archiveName];
  if (!expected) {
    throw new Error(
      `SHA256SUMS.txt from ${source} has no entry for ${archiveName}`
    );
  }
  const actual = sha256(buf);
  if (actual !== expected) {
    throw new Error(
      `SHA256 mismatch for ${archiveName} (from ${source})\n` +
        `  expected: ${expected}\n` +
        `  actual:   ${actual}\n` +
        `Refusing to install a possibly tampered binary.`
    );
  }
  console.log(`[OK] SHA256 verified (${source}): ${archiveName}`);
}

// ── Download, verify, extract ───────────────────────────────────────
async function tryInstall(archiveName, baseUrl) {
  const archiveUrl = `${baseUrl}/${archiveName}`;
  const sumsUrl = `${baseUrl}/SHA256SUMS.txt`;

  console.log(`[INSTALL] Trying: ${archiveUrl}`);

  // 必须先拿到 SHA256SUMS.txt，否则拒绝安装未校验的二进制
  let sumsText;
  try {
    sumsText = (await get(sumsUrl)).toString("utf8");
  } catch (e) {
    console.error(`[WARN] No SHA256SUMS.txt at ${sumsUrl} (${e.message}). Refusing unverified binary.`);
    return null;
  }

  let buf;
  try {
    buf = await get(archiveUrl);
  } catch (e) {
    console.error(`[WARN] Download failed: ${archiveUrl} (${e.message})`);
    return null;
  }

  try {
    verifyArchive(buf, archiveName, sumsText, baseUrl);
  } catch (e) {
    console.error(`[ERROR] ${e.message}`);
    return null;
  }

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "oneinit-"));
  const archivePath = path.join(tmpDir, archiveName);

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
  console.error(`[WARN] No binary found inside ${archiveName}`);
  return null;
}

// ── Main ────────────────────────────────────────────────────────────
async function main() {
  let tag = process.env.ONEINIT_VERSION || "latest";

  // Resolve latest tag from GitHub API
  if (tag === "latest") {
    try {
      const api = `https://api.github.com/repos/${REPO}/releases/latest`;
      const body = JSON.parse((await get(api)).toString("utf8"));
      tag = body.tag_name;
      console.log(`[INSTALL] Latest release: ${tag}`);
    } catch (e) {
      console.warn(`[WARN] Could not resolve latest tag (${e.message}). Trying 'latest' literally.`);
    }
  }

  const archive = `oneinit-${tag}-${platform}-${arch}.${archiveExt}`;

  // 1. GitHub Releases (primary)
  const ghBase = `https://github.com/${REPO}/releases/download/${tag}`;
  const ghDest = await tryInstall(archive, ghBase);
  if (ghDest) {
    console.log(`[OK] Installed to: ${ghDest}`);
    printNextSteps();
    return;
  }

  // 2. CDN fallback — 同样要求 SHA256SUMS.txt
  const cdnDest = await tryInstall(archive, CDN_BASE);
  if (cdnDest) {
    console.log(`[OK] Installed to: ${cdnDest}`);
    printNextSteps();
    return;
  }

  // 3. No verifiable binary found — refuse, print build instructions
  console.error("");
  console.error("╔══════════════════════════════════════════════════════════╗");
  console.error("║  No VERIFIED pre-built binary found for your platform.  ║");
  console.error("║  (Download skipped — SHA256 could not be confirmed.)    ║");
  console.error("║                                                        ║");
  console.error("║  Build from source:                                    ║");
  console.error(`║    git clone https://github.com/${REPO}.git               ║`);
  console.error("║    cd oneinit && cargo build --release                  ║");
  console.error("║                                                        ║");
  console.error("║  Or set custom CDN:                                    ║");
  console.error("║    ONEINIT_CDN=https://your-cdn.com/oneinit npm i -g     ║");
  console.error("╚══════════════════════════════════════════════════════════╝");
  console.error("");
  process.exitCode = 1;
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
