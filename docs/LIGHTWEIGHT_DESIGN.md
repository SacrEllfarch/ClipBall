# 轻量化设计与 exe 版本方案

## 1. 设计目标

ClipBall 的首要目标不是做一个功能很重的剪贴板平台，而是做一个可以长期常驻、启动快、稳定可用、便于分发的桌面小工具。

核心判断：

- 优先保证文本历史闭环可用。
- 优先保证 Windows exe 可以稳定产出。
- 优先使用 Electron 和 Node.js 内置能力，避免早期引入原生数据库、复杂打包器或前端框架。
- 保留现有 HTML 原型的视觉方向，但工程实现应服务于轻量和可维护。

## 2. 产品边界

### 2.1 首版必须做好

- 悬浮球常驻桌面。
- 点击展开历史面板。
- 文本和链接剪贴板历史记录。
- 本地持久化。
- 搜索、删除单条、清空全部。
- 点击记录写回剪贴板。
- 可生成 Windows 便携版 exe。

### 2.2 首版暂不进入

- 云同步、账号、多设备。
- OCR、AI 摘要、复杂规则引擎。
- 图片和文件完整内容存储。
- 多平台安装包。
- 重型前端框架迁移。
- 原生数据库依赖。

## 3. 技术取舍

| 领域 | 首版选择 | 原因 |
| --- | --- | --- |
| 桌面运行时 | Tauri | 前端 100% 复用，包体积从 ~120MB 降至 ~5-15MB，支持 exe/msi/自动更新 |
| UI 技术 | 原生 HTML/CSS/JS | 依赖少，启动快，从 Electron 迁移零成本 |
| 数据存储 | JSON 文件 | 无原生依赖，便携版 exe 更容易稳定交付 |
| 打包方式 | `tauri build` | 内置打包，支持 exe / msi / nsis，无需自定义脚本 |
| 快速粘贴 | Rust 平台适配层 | 前端不能执行系统命令 |
| 剪贴板监听 | Rust 后端低频轮询 | Tauri 无内置剪贴板事件，轮询最稳 |

## 4. 轻量化工程规则

- `dependencies` 默认保持为空；能用 Tauri/Rust 内置能力完成的，不引入第三方包。
- `devDependencies` 仅保留前端开发工具链（如需要），Rust 依赖通过 `Cargo.toml` 管理。
- 不引入 React/Vue/Vite/TypeScript，除非 UI 复杂度已经明显超过原生维护成本。
- 不引入 SQLite 原生模块；当历史规模、查询复杂度或迁移需求真正出现后再评估。
- 打包不依赖 electron-builder；先维护便携版目录，再在 v1.0 前评估安装包。
- 每个功能模块保持小文件、小 API：主进程负责系统能力，渲染层只负责显示和用户操作。

## 5. 推荐代码结构

```text
src/                     # 前端代码
├─ index.html
├─ styles.css
└─ app.js

src-tauri/
├─ src/
│  ├─ main.rs           # 应用入口、Tauri 命令注册
│  ├─ window.rs         # 窗口管理、模式切换
│  ├─ clipboard.rs      # 剪贴板轮询、类型识别、去重
│  ├─ history_store.rs  # JSON 本地存储、最大数量清理
│  ├─ settings_store.rs # 设置读写
│  └─ paste_adapter.rs  # 快速粘贴平台适配
└─ tauri.conf.json      # Tauri 配置
```

首版可以先按功能拆 `src/main`，渲染层在复杂度上升前继续保持一个 `app.js`。

## 6. 数据策略

MVP 使用 Tauri 应用数据目录下的 JSON 文件：

```text
<app_data_dir>/
├─ history.json
├─ settings.json
└─ logs/
```

`history.json` 保存最近记录数组，写入时采用临时文件加重命名的方式降低损坏概率。

建议默认限制：

- `maxItems`: 100
- `pollIntervalMs`: 800
- `maxTextLength`: 20000
- `previewLength`: 300
- `recordImages`: false
- `recordFiles`: false

历史项结构：

```js
{
  id: "text_...",
  type: "text",
  preview: "显示预览",
  contentText: "完整文本",
  hash: "sha256...",
  isFavorite: false,
  createdAt: "2026-05-07T00:00:00.000Z",
  lastUsedAt: null,
  useCount: 0
}
```

## 7. exe 版本策略

### 7.1 v0.x：便携版与安装包并行

Tauri 原生支持多种 Windows 产物：

```text
src-tauri/target/release/
├── ClipBall.exe                  # 便携版主程序
└── bundle/
    ├── msi/                      # MSI 安装包
    └── nsis/                     # NSIS 安装包
```

- 早期快速迭代和手动分发优先使用便携版 `ClipBall.exe`。
- 需要桌面快捷方式、卸载入口、开机启动时直接使用 Tauri 产出的 MSI 或 NSIS 安装包，无需额外工具链。

### 7.2 v1.0：自动更新评估

Tauri 内置自动更新插件（`updater`），当核心功能稳定后可配置：

- 版本检测与下载提示。
- 增量更新或全量替换。
- 签名验证。

## 8. 版本路线

| 版本 | 目标 | exe 交付 |
| --- | --- | --- |
| v0.1 | Electron 原型验证（已完成） | 便携版目录 |
| v0.2 | Tauri 工程初始化、窗口移植、文本剪贴板监听 | 便携版 exe |
| v0.3 | 本地 JSON 持久化、删除、清空、搜索、点击复制 | 便携版 exe |
| v0.4 | 快捷键打开面板、快速粘贴降级策略 | 便携版 exe |
| v0.5 | 设置面板、最大数量、暂停记录 | 便携版 exe / MSI |
| v1.0 | 图标、托盘、日志、自动更新、安装包 | MSI / NSIS |

## 9. 发布检查

每次准备 exe 版本时执行：

```powershell
npm run check
npm run tauri build
```

人工验证：

- 双击 `ClipBall.exe` 可以启动。
- 悬浮球显示在屏幕右下角。
- 面板可展开、关闭、拖拽、缩放。
- 复制文本后历史更新。
- 重启后历史仍存在。
- 删除和清空不会留下 UI 脏状态。
- 点击记录后系统剪贴板变为该记录内容。
