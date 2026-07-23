#!/usr/bin/env node
// oneinit — npm wrapper that launches the native binary
// The binary is installed via postinstall script to bin/native/

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const ext = process.platform === "win32" ? ".exe" : "";
const binaryName = `oneinit${ext}`;
const binaryPath = path.join(__dirname, "native", binaryName);

// If native binary exists, run it directly
if (fs.existsSync(binaryPath)) {
    const child = spawn(binaryPath, process.argv.slice(2), {
        stdio: "inherit",
        windowsHide: false,
    });
    child.on("close", (code) => process.exit(code || 0));
    child.on("error", (err) => {
        console.error("Failed to launch oneinit:", err.message);
        process.exit(1);
    });
} else {
    console.error("oneinit binary not found at:", binaryPath);
    console.error("");
    console.error("This may happen if the postinstall script was skipped.");
    console.error("Try reinstalling: npm install -g oneinit");
    process.exit(1);
}
