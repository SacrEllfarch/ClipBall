const { spawnSync } = require("node:child_process");
const path = require("node:path");
const fs = require("node:fs");

const rootDir = path.resolve(__dirname, "..");

const jsFiles = [
  "src/app.js",
  "scripts/check.js",
];

const requiredFiles = [
  "src/index.html",
  "src/styles.css",
  "src/app.js",
  "src-tauri/Cargo.toml",
  "src-tauri/tauri.conf.json",
  "src-tauri/src/main.rs",
  "src-tauri/src/window.rs",
  "scripts/check.js",
];

let hasError = false;

// Check required files exist
for (const file of requiredFiles) {
  const fullPath = path.join(rootDir, file);
  if (!fs.existsSync(fullPath)) {
    console.error(`Missing required file: ${file}`);
    hasError = true;
  }
}

// Check JS syntax
for (const file of jsFiles) {
  const result = spawnSync(process.execPath, ["--check", path.join(rootDir, file)], {
    cwd: rootDir,
    stdio: "inherit",
  });

  if (result.status !== 0) {
    hasError = true;
  }
}

if (hasError) {
  process.exit(1);
}

console.log("All checks passed.");
