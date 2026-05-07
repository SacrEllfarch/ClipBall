const fs = require("node:fs");
const { execFileSync } = require("node:child_process");
const path = require("node:path");

const rootDir = path.resolve(__dirname, "..");
const releaseDir = path.join(rootDir, "release");
const outputDir = path.join(releaseDir, "ClipBall-win32-x64");
const appDir = path.join(outputDir, "resources", "app");
const electronVersion = require(path.join(rootDir, "package-lock.json")).packages["node_modules/electron"].version;
const electronCacheZip = path.join(rootDir, ".cache-electron-win32-x64.zip");
const electronCacheDir = path.join(rootDir, ".cache-electron-win32-x64");
const electronDownloadUrl = `https://github.com/electron/electron/releases/download/v${electronVersion}/electron-v${electronVersion}-win32-x64.zip`;

function assertInsideRoot(targetPath) {
  const resolved = path.resolve(targetPath);
  if (resolved !== rootDir && !resolved.startsWith(rootDir + path.sep)) {
    throw new Error(`Refusing to operate outside project root: ${resolved}`);
  }
}

function removeDir(targetPath) {
  assertInsideRoot(targetPath);
  fs.rmSync(targetPath, { recursive: true, force: true, maxRetries: 5, retryDelay: 500 });
}

function copyDir(source, target) {
  fs.cpSync(source, target, {
    recursive: true,
    filter: (sourcePath) => {
      const name = path.basename(sourcePath);
      return name !== ".git" && name !== "node_modules" && name !== "release";
    },
  });
}

function isUsableElectronDist(candidateDir) {
  return [
    "electron.exe",
    "chrome_100_percent.pak",
    "icudtl.dat",
    "resources",
    "locales",
    "v8_context_snapshot.bin",
  ].every((entry) => fs.existsSync(path.join(candidateDir, entry)));
}

function runPowerShell(command) {
  execFileSync("powershell.exe", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", command], {
    cwd: rootDir,
    stdio: "inherit",
  });
}

function ensureElectronRuntime() {
  const explicitDist = process.env.CLIPBALL_ELECTRON_DIST;
  if (explicitDist && isUsableElectronDist(explicitDist)) {
    return explicitDist;
  }

  if (isUsableElectronDist(electronCacheDir)) {
    return electronCacheDir;
  }

  const installedDist = path.join(rootDir, "node_modules", "electron", "dist");
  if (isUsableElectronDist(installedDist)) {
    return installedDist;
  }

  console.log(`Downloading Electron ${electronVersion} runtime for Windows x64...`);
  assertInsideRoot(electronCacheZip);
  assertInsideRoot(electronCacheDir);
  removeDir(electronCacheDir);

  const escapedUrl = electronDownloadUrl.replace(/'/g, "''");
  const escapedZip = electronCacheZip.replace(/'/g, "''");
  const escapedDir = electronCacheDir.replace(/'/g, "''");
  runPowerShell(`Invoke-WebRequest -Uri '${escapedUrl}' -OutFile '${escapedZip}' -UseBasicParsing`);
  runPowerShell(`Expand-Archive -LiteralPath '${escapedZip}' -DestinationPath '${escapedDir}' -Force`);

  if (!isUsableElectronDist(electronCacheDir)) {
    throw new Error("Downloaded Electron runtime is incomplete.");
  }

  return electronCacheDir;
}

function writeAppPackage() {
  const sourcePackage = JSON.parse(fs.readFileSync(path.join(rootDir, "package.json"), "utf8"));
  const appPackage = {
    name: sourcePackage.name,
    productName: "ClipBall",
    version: sourcePackage.version,
    description: sourcePackage.description,
    main: sourcePackage.main,
    type: sourcePackage.type,
    license: sourcePackage.license,
  };

  fs.writeFileSync(path.join(appDir, "package.json"), JSON.stringify(appPackage, null, 2));
}

function packageWindows() {
  const electronDistDir = ensureElectronRuntime();
  const sourceElectronExe = path.join(electronDistDir, "electron.exe");

  if (!fs.existsSync(sourceElectronExe)) {
    throw new Error("Electron runtime was not found. Run `npm install` before packaging.");
  }

  removeDir(outputDir);
  fs.mkdirSync(outputDir, { recursive: true });

  copyDir(electronDistDir, outputDir);

  const originalExe = path.join(outputDir, "electron.exe");
  const renamedExe = path.join(outputDir, "ClipBall.exe");
  if (fs.existsSync(renamedExe)) {
    fs.rmSync(renamedExe, { force: true });
  }
  fs.renameSync(originalExe, renamedExe);

  removeDir(appDir);
  fs.mkdirSync(appDir, { recursive: true });
  copyDir(path.join(rootDir, "src"), path.join(appDir, "src"));
  writeAppPackage();

  fs.writeFileSync(
    path.join(outputDir, "README.txt"),
    [
      "ClipBall portable build",
      "",
      "Run ClipBall.exe to start the app.",
      "Keep this folder intact; the exe depends on the bundled resources next to it.",
      "",
    ].join("\r\n"),
  );

  console.log(`Packaged ClipBall for Windows: ${renamedExe}`);
}

packageWindows();
