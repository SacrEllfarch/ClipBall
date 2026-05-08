const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const rootDir = path.resolve(__dirname, "..");
const iconsDir = path.join(rootDir, "src-tauri", "icons");
const svgPath = path.join(iconsDir, "icon.svg");

function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

function runNodeScript(script) {
  const tmpFile = path.join(rootDir, ".tmp-icon-gen.js");
  fs.writeFileSync(tmpFile, script, "utf8");
  try {
    execFileSync(process.execPath, [tmpFile], { cwd: rootDir, stdio: "inherit" });
  } finally {
    fs.rmSync(tmpFile, { force: true });
  }
}

function main() {
  if (!fs.existsSync(svgPath)) {
    console.error(`SVG not found: ${svgPath}`);
    process.exit(1);
  }

  ensureDir(iconsDir);

  // Use sharp to convert SVG to various sizes
  const sharpScript = `
const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');
const svgPath = path.join(iconsDir, 'icon.svg');

async function generate() {
  const svgBuffer = fs.readFileSync(svgPath);

  // Generate PNGs
  await sharp(svgBuffer).resize(32, 32).png().toFile(path.join(iconsDir, '32x32.png'));
  console.log('Generated 32x32.png');

  await sharp(svgBuffer).resize(128, 128).png().toFile(path.join(iconsDir, '128x128.png'));
  console.log('Generated 128x128.png');

  await sharp(svgBuffer).resize(256, 256).png().toFile(path.join(iconsDir, '128x128@2x.png'));
  console.log('Generated 128x128@2x.png');

  // Generate ICO (Windows needs 256x256 max)
  await sharp(svgBuffer).resize(256, 256).toFile(path.join(iconsDir, 'icon.ico'));
  console.log('Generated icon.ico');

  console.log('All icons generated!');
}

generate().catch(err => {
  console.error(err);
  process.exit(1);
});
`;

  console.log("Installing sharp (one-time)...");
  try {
    execFileSync(process.execPath, ["-e", "require('sharp')"], { cwd: rootDir, stdio: "pipe" });
  } catch {
    execFileSync("npm", ["install", "sharp", "--no-save"], { cwd: rootDir, stdio: "inherit" });
  }

  console.log("Generating icons...");
  runNodeScript(sharpScript);
}

main();
