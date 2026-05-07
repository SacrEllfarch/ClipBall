# Tauri 迁移计划

## 背景

项目首版基于 Electron 完成悬浮球外壳、历史面板 UI 和 Windows 便携版打包流程。为了实质性降低包体积（从 ~120MB 降至 ~5-15MB）和内存占用，决定将运行时从 Electron 迁移至 Tauri。

## 迁移原则

1. **前端零重写**：`src/renderer/` 中的 HTML / CSS / JS 全部保留，仅替换与主进程通信的 API 调用方式。
2. **功能不丢失**：迁移后的产物必须覆盖 v0.1 已有的全部能力（悬浮球、面板切换、拖拽、缩放、示例列表）。
3. **逐步替换**：先在文档层面完成规划，再分步实施代码迁移，每步都可验证。
4. **exe 优先**：迁移完成后的首个可运行产物必须是 Windows exe，确保后续迭代基础稳定。

## 技术映射

| Electron 概念 | Tauri 对应 | 说明 |
|--------------|-----------|------|
| 主进程 (Node.js) | Rust 后端 (`src-tauri/src/main.rs` 及模块) | 系统能力由 Rust 提供 |
| `BrowserWindow` | `tauri::WindowBuilder` | 窗口创建与控制 |
| `preload.js` | Tauri `invoke` + `event` 机制 | 无需显式 preload 脚本 |
| `ipcMain` / `ipcRenderer` | `tauri::command` + `invoke` | 前端调用后端命令 |
| `app.getPath("userData")` | `tauri::api::path::app_data_dir()` | 本地存储路径 |
| `clipboard` 模块 | `arboard` crate (Rust) | 剪贴板读写 |
| `globalShortcut` | `tauri::GlobalShortcutManager` | 全局快捷键 |
| 自定义打包脚本 | `tauri build` | 内置打包，产物含 exe |

## 代码结构变化

### 迁移前 (Electron)

```text
.
├─ src/
│  ├─ main/
│  │  └─ main.js           # 主进程入口
│  ├─ preload/
│  │  └─ preload.js        # 安全桥
│  └─ renderer/
│     ├─ index.html
│     ├─ styles.css
│     └─ app.js
├─ scripts/
│  └─ package-win.js       # 自定义打包
└─ package.json
```

### 迁移后 (Tauri)

```text
.
├─ src/                    # 前端代码（原 renderer，位置不变）
│  ├─ index.html
│  ├─ styles.css
│  └─ app.js
├─ src-tauri/              # Tauri Rust 后端
│  ├─ src/
│  │  ├─ main.rs           # 应用入口
│  │  ├─ clipboard.rs      # 剪贴板轮询与去重
│  │  ├─ history_store.rs  # JSON 历史存储
│  │  ├─ settings_store.rs # JSON 设置存储
│  │  └─ window.rs         # 窗口管理（模式切换、拖拽、置顶）
│  ├─ Cargo.toml
│  └─ tauri.conf.json      # Tauri 配置
├─ docs/
└─ package.json            # 仅保留前端构建/开发依赖
```

## 前端适配点

### 1. IPC 调用替换

将 `window.clipboardBall.*` 替换为 Tauri 的 `invoke`：

```js
// 迁移前 (Electron)
const api = window.clipboardBall;
api.setWindowMode('panel');

// 迁移后 (Tauri)
import { invoke } from '@tauri-apps/api/tauri';
invoke('set_window_mode', { mode: 'panel' });
```

### 2. 拖拽区域标记

将 CSS 中的 `-webkit-app-region: drag` 替换为 Tauri 的数据属性：

```html
<!-- 迁移前 -->
<div class="panel-header" style="-webkit-app-region: drag">

<!-- 迁移后 -->
<div class="panel-header" data-tauri-drag-region>
```

### 3. 事件监听

后端向前端推送更新时，从 `ipcRenderer.on` 改为 Tauri 的 `listen`：

```js
// 迁移前
window.clipboardBall.onItemsChanged(callback);

// 迁移后
import { listen } from '@tauri-apps/api/event';
listen('history:changed', (event) => { ... });
```

## Rust 后端职责

对应原 `src/main/main.js` 的能力：

| 能力 | Rust 模块 | 关键 crate |
|------|----------|-----------|
| 窗口创建与模式切换 | `window.rs` | `tauri::WindowBuilder` |
| 剪贴板轮询 | `clipboard.rs` | `arboard` |
| 历史 JSON 存储 | `history_store.rs` | `serde_json`, `std::fs` |
| 设置 JSON 存储 | `settings_store.rs` | `serde_json`, `std::fs` |
| 全局快捷键 | `main.rs` | `tauri::GlobalShortcutManager` |
| 快速粘贴 | `paste_adapter.rs` | `enigo` 或 `windows` crate |

## 打包产物

```text
src-tauri/target/release/
├── ClipBall.exe          # 主程序（约 5-15MB）
├── ClipBall.exe.sig      # 签名（如启用）
└── ...系统 DLL
```

使用 `tauri build --target x86_64-pc-windows-msvc` 产出。

Tauri 同时原生支持：
- `.msi` 安装包
- `.exe` 单文件（便携版）
- 自动更新（`updater` 插件）

## 迁移步骤

1. **工程初始化**：安装 Rust，初始化 Tauri 项目，保留现有前端代码。
2. **窗口移植**：在 Rust 中复现透明无边框窗口、悬浮球/面板模式切换、置顶。
3. **IPC 打通**：建立前端 `invoke` 与 Rust `command` 的基础通信。
4. **功能回填**：逐步移植剪贴板监听、JSON 存储、搜索、删除等能力。
5. **打包验证**：运行 `tauri build`，验证 Windows exe 启动和核心流程。
6. **清理**：移除 Electron 相关代码、脚本和依赖。

## 风险与应对

| 风险 | 应对 |
|------|------|
| Rust 学习成本 | 业务逻辑简单（轮询 + 文件 IO），不需要高级 Rust 特性 |
| WebView2 未安装 | Windows 10/11 绝大多数已预装；Tauri 会自动检测并引导安装 |
| 剪贴板 crate 兼容性 | `arboard` 跨平台成熟，先验证 Windows 文本读写 |
| 快捷键冲突 | 与 Electron 版保持相同快捷键，观察用户反馈 |

## 验收标准

迁移完成后：

- `npm run tauri build` 可产出 Windows exe。
- 双击 exe 启动后悬浮球显示在右下角。
- 点击悬浮球展开历史面板，再次关闭回到悬浮球。
- 面板可拖拽移动并调整大小。
- 示例历史列表、搜索、删除、清空 UI 正常。
- 包体积相比 Electron 版缩小 80% 以上。
