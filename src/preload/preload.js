const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("clipboardBall", {
  setWindowMode: (mode) => ipcRenderer.invoke("window:set-mode", mode),
  getWindowMode: () => ipcRenderer.invoke("window:get-mode"),
  resizeWindowBy: (width, height) => ipcRenderer.send("window:resize-by", { width, height }),
});

