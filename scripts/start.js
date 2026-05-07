const path = require("node:path");

const rootDir = path.resolve(__dirname, "..");
process.chdir(rootDir);
process.argv = [
  process.argv[0],
  require.resolve("electron/cli"),
  ".",
  ...process.argv.slice(2),
];
require("electron/cli");
