const appRoot = document.getElementById("app");
const floatingBall = document.getElementById("floating-ball");
const historyPanel = document.getElementById("history-panel");
const closeBtn = document.getElementById("close-btn");
const pinBtn = document.getElementById("pin-btn");
const clearBtn = document.getElementById("clear-btn");
const searchInput = document.getElementById("search-input");
const historyList = document.getElementById("history-list");
const historyCount = document.getElementById("history-count");
const resizeHandle = document.getElementById("resize-handle");

const historyItems = [
  {
    id: "text-1",
    type: "text",
    typeLabel: "文本",
    icon: "📝",
    timeLabel: "刚刚",
    body: "这是一个兼顾美观与实用性的剪贴板历史记录工具 UI/UX 方案设计。",
  },
  {
    id: "link-1",
    type: "link",
    typeLabel: "链接",
    icon: "🔗",
    timeLabel: "15 分钟前",
    body: "https://github.com/your-project/clipboard-ui",
  },
  {
    id: "file-1",
    type: "file",
    typeLabel: "文件",
    icon: "📁",
    timeLabel: "2 小时前",
    body: "产品设计规范V2.pdf - 2.4 MB",
  },
  {
    id: "text-2",
    type: "text",
    typeLabel: "文本",
    icon: "📝",
    timeLabel: "昨天",
    body: "console.log(\"Hello, Glassmorphism!\");",
  },
];

let isPinned = false;
let visibleItems = [...historyItems];

function getApi() {
  return window.clipboardBall ?? {
    setWindowMode: async () => {},
    resizeWindowBy: () => {},
  };
}

function setMode(mode) {
  appRoot.classList.toggle("app--ball", mode === "ball");
  appRoot.classList.toggle("app--panel", mode === "panel");
  getApi().setWindowMode(mode);

  if (mode === "panel") {
    window.requestAnimationFrame(() => searchInput.focus());
  }
}

function createActionButton(label, title, onClick) {
  const button = document.createElement("button");
  button.className = "action-btn";
  button.type = "button";
  button.title = title;
  button.setAttribute("aria-label", title);
  button.textContent = label;
  button.addEventListener("click", (event) => {
    event.stopPropagation();
    onClick();
  });
  return button;
}

function renderHistory(items) {
  historyList.textContent = "";

  if (items.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = "没有匹配的历史记录";
    historyList.append(empty);
    historyCount.textContent = "已记录 0 / 100 条";
    return;
  }

  for (const item of items) {
    const row = document.createElement("article");
    row.className = "history-item";
    row.tabIndex = 0;

    const header = document.createElement("div");
    header.className = "item-header";

    const type = document.createElement("span");
    type.className = "item-type";
    type.textContent = `${item.icon} ${item.typeLabel}`;

    const time = document.createElement("span");
    time.textContent = item.timeLabel;

    const body = document.createElement("div");
    body.className = "item-body";
    body.textContent = item.body;

    const actions = document.createElement("div");
    actions.className = "item-actions";
    actions.append(
      createActionButton("📋", "复制", () => markUsed(item)),
      createActionButton("⭐", "收藏", () => markUsed(item)),
      createActionButton("🗑", "删除", () => deleteItem(item.id)),
    );

    header.append(type, time);
    row.append(header, body, actions);
    row.addEventListener("click", () => markUsed(item));
    row.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        markUsed(item);
      }
    });
    historyList.append(row);
  }

  historyCount.textContent = `已记录 ${historyItems.length} / 100 条`;
}

function filterHistory() {
  const query = searchInput.value.trim().toLowerCase();
  visibleItems = historyItems.filter((item) => item.body.toLowerCase().includes(query));
  renderHistory(visibleItems);
}

function markUsed(item) {
  console.info("Selected clipboard history item:", item.id);
}

function deleteItem(id) {
  const itemIndex = historyItems.findIndex((item) => item.id === id);
  if (itemIndex >= 0) {
    historyItems.splice(itemIndex, 1);
    filterHistory();
  }
}

floatingBall.addEventListener("click", () => setMode("panel"));
closeBtn.addEventListener("click", () => setMode("ball"));
pinBtn.addEventListener("click", () => {
  isPinned = !isPinned;
  pinBtn.classList.toggle("is-active", isPinned);
});
clearBtn.addEventListener("click", () => {
  historyItems.splice(0, historyItems.length);
  filterHistory();
});
searchInput.addEventListener("input", filterHistory);
historyPanel.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !isPinned) {
    setMode("ball");
  }
});

let isResizing = false;
let resizeStart = { x: 0, y: 0 };

resizeHandle.addEventListener("mousedown", (event) => {
  isResizing = true;
  resizeStart = { x: event.screenX, y: event.screenY };
  document.body.style.userSelect = "none";
});

window.addEventListener("mousemove", (event) => {
  if (!isResizing) {
    return;
  }

  const deltaWidth = event.screenX - resizeStart.x;
  const deltaHeight = event.screenY - resizeStart.y;
  resizeStart = { x: event.screenX, y: event.screenY };
  getApi().resizeWindowBy(deltaWidth, deltaHeight);
});

window.addEventListener("mouseup", () => {
  isResizing = false;
  document.body.style.userSelect = "";
});

renderHistory(visibleItems);
