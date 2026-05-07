const { spawnSync } = require("node:child_process");
const path = require("node:path");

const rootDir = path.resolve(__dirname, "..");
const files = [
  "src/main/main.js",
  "src/preload/preload.js",
  "src/renderer/app.js",
  "scripts/check.js",
  "scripts/package-win.js",
  "scripts/start.js",
];

for (const file of files) {
  const result = spawnSync(process.execPath, ["--check", path.join(rootDir, file)], {
    cwd: rootDir,
    stdio: "inherit",
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
