const appRoot = document.getElementById("app");
const floatingBall = document.getElementById("floating-ball");
const historyPanel = document.getElementById("history-panel");
const closeBtn = document.getElementById("close-btn");
const clearBtn = document.getElementById("clear-btn");
const settingsBtn = document.getElementById("settings-btn");
const historyList = document.getElementById("history-list");
const settingsView = document.getElementById("settings-view");
const panelTitle = document.getElementById("panel-title");
const toastEl = document.getElementById("toast");

const inputPaused = document.getElementById("setting-paused");
const inputAutostart = document.getElementById("setting-autostart");
const inputRemember = document.getElementById("setting-remember");
const inputMax = document.getElementById("setting-max");
const settingStatus = document.getElementById("setting-status");

const DRAG_THRESHOLD = 4;
let historyItems = [];
let currentSettings = null;
let view = "history"; // "history" | "settings"

const tauri = window.__TAURI__ || null;
const invoke = tauri?.core?.invoke || tauri?.tauri?.invoke || null;
const listen = tauri?.event?.listen || null;

const api = invoke
  ? {
      setMode: (mode) => invoke("set_window_mode", { mode }),
      togglePanel: () => invoke("toggle_panel"),
      startDragging: () => invoke("start_dragging"),
      quit: () => invoke("quit_app"),
      getHistory: () => invoke("get_history"),
      deleteItem: (id) => invoke("delete_history_item", { id }),
      clearAll: () => invoke("clear_history"),
      copyText: (body) => invoke("copy_to_clipboard", { body }),
      pasteText: (body) => invoke("paste_history_item", { body }),
      getSettings: () => invoke("get_settings"),
      updateSettings: (settings) => invoke("update_settings", { settings }),
    }
  : {
      setMode: async () => {},
      togglePanel: async () => {},
      startDragging: async () => {},
      quit: async () => {},
      getHistory: async () => [],
      deleteItem: async () => {},
      clearAll: async () => {},
      copyText: async () => {},
      pasteText: async () => {},
      getSettings: async () => ({
        maxHistory: 100,
        paused: false,
        autostart: false,
        rememberPosition: true,
      }),
      updateSettings: async (s) => s,
    };

function showToast(text, ms = 1400) {
  toastEl.textContent = text;
  toastEl.hidden = false;
  clearTimeout(showToast._t);
  showToast._t = setTimeout(() => {
    toastEl.hidden = true;
  }, ms);
}

async function loadHistory() {
  try {
    historyItems = await api.getHistory();
    renderHistory();
  } catch (e) {
    console.error("[clipball] Failed to load history:", e);
  }
}

async function loadSettings() {
  try {
    currentSettings = await api.getSettings();
    if (!currentSettings) return;
    inputPaused.checked = !!currentSettings.paused;
    inputAutostart.checked = !!currentSettings.autostart;
    inputRemember.checked = !!currentSettings.rememberPosition;
    inputMax.value = currentSettings.maxHistory ?? 100;
  } catch (e) {
    console.error("[clipball] Failed to load settings:", e);
  }
}

function setMode(mode) {
  appRoot.classList.toggle("app--ball", mode === "ball");
  appRoot.classList.toggle("app--panel", mode === "panel");
  api.setMode(mode);
  if (mode === "panel") {
    setView("history");
    loadHistory();
  }
}

function setView(next) {
  view = next;
  const isSettings = next === "settings";
  historyList.hidden = isSettings;
  settingsView.hidden = !isSettings;
  clearBtn.hidden = isSettings;
  panelTitle.textContent = isSettings ? "设置" : "剪贴板历史";
  settingsBtn.textContent = isSettings ? "←" : "⚙";
  settingsBtn.title = isSettings ? "返回" : "设置";
  if (isSettings) loadSettings();
}

function renderHistory() {
  historyList.textContent = "";

  if (historyItems.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = currentSettings?.paused
      ? "已暂停记录"
      : "暂无剪贴板历史";
    historyList.append(empty);
    return;
  }

  for (const item of historyItems) {
    const row = document.createElement("article");
    row.className = "history-item";
    row.tabIndex = 0;

    const meta = document.createElement("div");
    meta.className = "item-meta";
    const typeSpan = document.createElement("span");
    typeSpan.textContent = `${item.icon} ${item.typeLabel}`;
    const timeSpan = document.createElement("span");
    timeSpan.textContent = item.timeLabel;
    meta.append(typeSpan, timeSpan);

    const body = document.createElement("div");
    body.className = "item-body";
    body.textContent = item.body;

    const del = document.createElement("button");
    del.className = "delete-btn";
    del.type = "button";
    del.title = "删除";
    del.textContent = "✕";
    del.addEventListener("click", (event) => {
      event.stopPropagation();
      deleteItem(item.id);
    });

    row.append(meta, body, del);
    row.addEventListener("click", () => pasteItem(item));
    row.addEventListener("contextmenu", (event) => {
      event.preventDefault();
      copyItem(item);
    });
    row.addEventListener("keydown", (event) => {
      if (event.key === "Enter") pasteItem(item);
    });
    historyList.append(row);
  }
}

async function pasteItem(item) {
  try {
    await api.pasteText(item.body);
  } catch (e) {
    console.error("[clipball] Paste failed:", e);
    showToast("粘贴失败，已复制到剪贴板");
  }
}

async function copyItem(item) {
  try {
    await api.copyText(item.body);
    showToast("已复制");
  } catch (e) {
    console.error("[clipball] Copy failed:", e);
  }
}

async function deleteItem(id) {
  try {
    await api.deleteItem(id);
    historyItems = historyItems.filter((i) => i.id !== id);
    renderHistory();
  } catch (e) {
    console.error("[clipball] Delete failed:", e);
  }
}

async function clearAll() {
  try {
    await api.clearAll();
    historyItems = [];
    renderHistory();
  } catch (e) {
    console.error("[clipball] Clear failed:", e);
  }
}

async function persistSettings(partial) {
  if (!currentSettings) {
    currentSettings = await api.getSettings();
  }
  const next = { ...currentSettings, ...partial };
  try {
    currentSettings = await api.updateSettings(next);
    settingStatus.textContent = "已保存";
    setTimeout(() => {
      if (settingStatus.textContent === "已保存") settingStatus.textContent = "";
    }, 1200);
  } catch (e) {
    console.error("[clipball] Save settings failed:", e);
    settingStatus.textContent = "保存失败";
  }
}

// 悬浮球：短按展开，长按拖动
let ballDownPos = null;
let ballDragging = false;

floatingBall.addEventListener("mousedown", (event) => {
  if (event.button !== 0) return;
  ballDownPos = { x: event.clientX, y: event.clientY };
  ballDragging = false;
});

floatingBall.addEventListener("mousemove", (event) => {
  if (!ballDownPos || ballDragging) return;
  const dx = event.clientX - ballDownPos.x;
  const dy = event.clientY - ballDownPos.y;
  if (Math.hypot(dx, dy) > DRAG_THRESHOLD) {
    ballDragging = true;
    api.startDragging();
  }
});

floatingBall.addEventListener("mouseup", (event) => {
  if (!ballDownPos) return;
  const wasDragging = ballDragging;
  ballDownPos = null;
  ballDragging = false;
  if (!wasDragging && event.button === 0) {
    setMode("panel");
  }
});

floatingBall.addEventListener("mouseleave", () => {
  ballDownPos = null;
});

// 面板按钮
closeBtn.addEventListener("click", () => setMode("ball"));
clearBtn.addEventListener("click", () => {
  if (confirm("确定清空所有剪贴板历史？")) clearAll();
});
settingsBtn.addEventListener("click", () => {
  setView(view === "settings" ? "history" : "settings");
});

// 设置项变更
inputPaused.addEventListener("change", () =>
  persistSettings({ paused: inputPaused.checked })
);
inputAutostart.addEventListener("change", () =>
  persistSettings({ autostart: inputAutostart.checked })
);
inputRemember.addEventListener("change", () =>
  persistSettings({ rememberPosition: inputRemember.checked })
);
inputMax.addEventListener("change", () => {
  let v = parseInt(inputMax.value, 10);
  if (isNaN(v)) v = 100;
  v = Math.max(10, Math.min(500, v));
  inputMax.value = v;
  persistSettings({ maxHistory: v });
});

// ESC：先退出设置，再收起
historyPanel.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    if (view === "settings") setView("history");
    else setMode("ball");
  }
});

// 监听后端事件
if (listen) {
  listen("history:updated", (event) => {
    const item = event.payload;
    if (item && !historyItems.some((i) => i.id === item.id)) {
      historyItems.unshift(item);
      const max = currentSettings?.maxHistory ?? 100;
      if (historyItems.length > max) historyItems.length = max;
      if (appRoot.classList.contains("app--panel") && view === "history") {
        renderHistory();
      }
    }
  });

  listen("history:deleted", (event) => {
    historyItems = historyItems.filter((i) => i.id !== event.payload);
    if (appRoot.classList.contains("app--panel") && view === "history") {
      renderHistory();
    }
  });

  listen("history:cleared", () => {
    historyItems = [];
    if (appRoot.classList.contains("app--panel") && view === "history") {
      renderHistory();
    }
  });

  listen("window:mode", (event) => {
    const mode = event.payload;
    appRoot.classList.toggle("app--ball", mode === "ball");
    appRoot.classList.toggle("app--panel", mode === "panel");
    if (mode === "panel") {
      setView("history");
      loadHistory();
    }
  });

  listen("settings:updated", (event) => {
    currentSettings = event.payload;
  });

  listen("paste:fallback", () => {
    showToast("粘贴失败，已复制到剪贴板");
  });
}

(async () => {
  await loadSettings();
  await loadHistory();
})();
