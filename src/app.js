const appRoot = document.getElementById("app");
const floatingBall = document.getElementById("floating-ball");
const historyPanel = document.getElementById("history-panel");
const closeBtn = document.getElementById("close-btn");
const clearBtn = document.getElementById("clear-btn");
const historyList = document.getElementById("history-list");

const DRAG_THRESHOLD = 4; // 移动超过该像素则视为拖动
let historyItems = [];

function getTauri() {
  return window.__TAURI__ || null;
}

const tauri = getTauri();
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
    };

async function loadHistory() {
  try {
    historyItems = await api.getHistory();
    renderHistory();
  } catch (e) {
    console.error("[clipball] Failed to load history:", e);
  }
}

function setMode(mode) {
  appRoot.classList.toggle("app--ball", mode === "ball");
  appRoot.classList.toggle("app--panel", mode === "panel");
  api.setMode(mode);
  if (mode === "panel") {
    loadHistory();
  }
}

function renderHistory() {
  historyList.textContent = "";

  if (historyItems.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = "暂无剪贴板历史";
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
    row.addEventListener("click", () => copyItem(item));
    row.addEventListener("keydown", (event) => {
      if (event.key === "Enter") copyItem(item);
    });
    historyList.append(row);
  }
}

async function copyItem(item) {
  try {
    await api.copyText(item.body);
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

// 悬浮球：短按展开，长按（超过阈值）则拖动窗口
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

// ESC 收起面板
historyPanel.addEventListener("keydown", (event) => {
  if (event.key === "Escape") setMode("ball");
});

// 监听后端事件
if (listen) {
  listen("history:updated", (event) => {
    const item = event.payload;
    if (item && !historyItems.some((i) => i.id === item.id)) {
      historyItems.unshift(item);
      if (historyItems.length > 100) historyItems.pop();
      if (appRoot.classList.contains("app--panel")) renderHistory();
    }
  });

  listen("history:deleted", (event) => {
    historyItems = historyItems.filter((i) => i.id !== event.payload);
    if (appRoot.classList.contains("app--panel")) renderHistory();
  });

  listen("history:cleared", () => {
    historyItems = [];
    if (appRoot.classList.contains("app--panel")) renderHistory();
  });

  // 全局快捷键触发的模式切换
  listen("window:mode", (event) => {
    const mode = event.payload;
    appRoot.classList.toggle("app--ball", mode === "ball");
    appRoot.classList.toggle("app--panel", mode === "panel");
    if (mode === "panel") loadHistory();
  });
}

loadHistory();
