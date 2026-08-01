const fs = require("fs");
const path = require("path");

const platform = process.platform;
const arch = process.arch;

// Map npm platform/arch to our binary naming
const platformMap = {
  "win32-x64": "oneinit.exe",
  "linux-x64": "oneinit",
  "darwin-x64": "oneinit",
  "darwin-arm64": "oneinit",
};

const key = `${platform}-${arch}`;
const binaryName = platformMap[key];

if (!binaryName) {
  console.log(`[oneinit] Unsupported platform: ${key}`);
  console.log(`[oneinit] Build from source: https://github.com/oneinitAI/oneinit`);
  process.exit(0);
}

const binDir = path.join(__dirname, "native");
fs.mkdirSync(binDir, { recursive: true });

const bundled = path.join(__dirname, binaryName);
const dest = path.join(binDir, binaryName);

if (fs.existsSync(bundled)) {
  fs.copyFileSync(bundled, dest);
  if (platform !== "win32") fs.chmodSync(dest, 0o755);
  console.log(`[oneinit] Binary installed for ${key}.`);
} else {
  console.log(`[oneinit] Binary for ${key} not bundled in this package.`);
  console.log(`[oneinit] Build from source: https://github.com/oneinitAI/oneinit`);
}
