const path = require("node:path");
const { app, BrowserWindow, ipcMain, screen } = require("electron");

const BALL_BOUNDS = { width: 72, height: 72 };
const PANEL_BOUNDS = { width: 380, height: 540 };
const PANEL_MIN_BOUNDS = { width: 300, height: 380 };

let mainWindow;
let windowMode = "ball";

function getBottomRightBounds(size) {
  const { workArea } = screen.getPrimaryDisplay();
  return {
    x: Math.round(workArea.x + workArea.width - size.width - 32),
    y: Math.round(workArea.y + workArea.height - size.height - 32),
    width: size.width,
    height: size.height,
  };
}

function createMainWindow() {
  const initialBounds = getBottomRightBounds(BALL_BOUNDS);

  mainWindow = new BrowserWindow({
    ...initialBounds,
    minWidth: BALL_BOUNDS.width,
    minHeight: BALL_BOUNDS.height,
    frame: false,
    transparent: true,
    hasShadow: false,
    resizable: true,
    maximizable: false,
    fullscreenable: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    show: false,
    webPreferences: {
      preload: path.join(__dirname, "../preload/preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  mainWindow.setAlwaysOnTop(true, "floating");
  mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  mainWindow.once("ready-to-show", () => mainWindow.show());
}

function setWindowMode(mode) {
  if (!mainWindow || !["ball", "panel"].includes(mode)) {
    return windowMode;
  }

  windowMode = mode;
  const currentBounds = mainWindow.getBounds();

  if (mode === "panel") {
    const nextBounds = {
      x: currentBounds.x + currentBounds.width - PANEL_BOUNDS.width,
      y: currentBounds.y + currentBounds.height - PANEL_BOUNDS.height,
      ...PANEL_BOUNDS,
    };
    mainWindow.setMinimumSize(PANEL_MIN_BOUNDS.width, PANEL_MIN_BOUNDS.height);
    mainWindow.setBounds(nextBounds, true);
  } else {
    const nextBounds = {
      x: currentBounds.x + currentBounds.width - BALL_BOUNDS.width,
      y: currentBounds.y + currentBounds.height - BALL_BOUNDS.height,
      ...BALL_BOUNDS,
    };
    mainWindow.setMinimumSize(BALL_BOUNDS.width, BALL_BOUNDS.height);
    mainWindow.setBounds(nextBounds, true);
  }

  return windowMode;
}

function resizeWindowBy(deltaWidth, deltaHeight) {
  if (!mainWindow || windowMode !== "panel") {
    return;
  }

  const bounds = mainWindow.getBounds();
  const nextWidth = Math.max(PANEL_MIN_BOUNDS.width, bounds.width + deltaWidth);
  const nextHeight = Math.max(PANEL_MIN_BOUNDS.height, bounds.height + deltaHeight);
  const { workArea } = screen.getDisplayMatching(bounds);

  mainWindow.setBounds(
    {
      ...bounds,
      width: Math.min(nextWidth, workArea.width),
      height: Math.min(nextHeight, workArea.height),
    },
    false,
  );
}

app.whenReady().then(() => {
  createMainWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createMainWindow();
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

ipcMain.handle("window:set-mode", (_event, mode) => setWindowMode(mode));
ipcMain.handle("window:get-mode", () => windowMode);
ipcMain.on("window:resize-by", (_event, delta) => {
  resizeWindowBy(Number(delta?.width) || 0, Number(delta?.height) || 0);
});

