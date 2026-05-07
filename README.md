# ClipBall

ClipBall 是一个以悬浮球形态常驻桌面的本地剪贴板历史工具。它在未展开时显示为轻量悬浮球，点击后展开为可拖拽、可调整大小的历史面板，方便用户查看复制历史并快速粘贴。

当前项目处于 Tauri 迁移阶段：已完成基于 Electron 的 v0.1 原型验证（悬浮球/历史面板 UI、窗口模式切换），正在将运行时迁移至 Tauri 以实质性降低包体积。迁移完成后将依次实现剪贴板监听、本地 JSON 持久化和快速粘贴。

## 功能目标

| 功能 | 说明 | 状态 |
| --- | --- | --- |
| 悬浮球 | 常驻桌面入口，点击展开历史面板 | 已完成基础 UI |
| 历史面板 | 展示复制历史，支持拖拽、缩放、关闭 | 已完成基础 UI |
| 搜索 | 按关键词过滤历史记录 | 已完成前端示例 |
| 删除/清空 | 删除单条或清空全部历史 | 已完成前端示例 |
| 剪贴板监听 | 记录系统剪贴板变化并去重 | 待开发 |
| 本地持久化 | 重启后恢复历史记录 | 待开发 |
| 快速粘贴 | 点击记录或快捷键粘贴到当前应用 | 待开发 |
| 设置面板 | 配置快捷键、最大记录数、隐私策略 | 待开发 |
| Windows exe | 生成便携版 Windows 可执行程序 | 已完成 |

## 技术栈

- Tauri（Rust 后端 + WebView2 前端）
- HTML / CSS / JavaScript
- Node.js（仅前端开发工具链）

项目从 Electron 迁移至 Tauri，核心目标是：

- 保持前端代码 100% 复用，不引入前端框架。
- 包体积从 ~120MB 降至 ~5-15MB。
- Rust 后端负责系统剪贴板、全局快捷键、窗口置顶和本地 JSON 存储。
- 不引入原生数据库，优先用内置能力完成可用闭环。

## 快速开始

### 环境准备

1. 安装 Node.js 依赖：

```powershell
npm install
```

2. 安装 Rust 工具链（如尚未安装）：

```powershell
winget install Rustlang.Rustup
rustup default stable
```

### 开发模式

启动 Tauri 开发版（前端热更新 + Rust 自动编译）：

```powershell
npm run tauri dev
```

运行基础检查：

```powershell
npm run check
```

### 打包 Windows exe

生成 Windows 便携版：

```powershell
npm run tauri build
```

构建产物：

```text
src-tauri/target/release/ClipBall.exe
```

Tauri 同时支持 MSI 安装包和 NSIS 安装包，产物位于：

```text
src-tauri/target/release/bundle/msi/ClipBall_<version>_x64_en-US.msi
src-tauri/target/release/bundle/nsis/ClipBall_<version>_x64-setup.exe
```

## 项目结构

```text
.
├─ src/                  # 前端 UI：悬浮球、历史面板、示例历史列表
│  ├─ index.html
│  ├─ styles.css
│  └─ app.js
├─ src-tauri/            # Tauri Rust 后端
│  ├─ src/
│  │  ├─ main.rs         # 应用入口、IPC 命令注册
│  │  ├─ window.rs       # 窗口管理、模式切换
│  │  ├─ clipboard.rs    # 剪贴板轮询、去重
│  │  ├─ history_store.rs # JSON 历史存储
│  │  └─ settings_store.rs # JSON 设置存储
│  ├─ Cargo.toml
│  └─ tauri.conf.json
├─ docs/
│  ├─ LIGHTWEIGHT_DESIGN.md  # 轻量化设计与 exe 版本方案
│  ├─ TAURI_MIGRATION.md     # Tauri 迁移计划
│  ├─ PRD.md                 # 产品需求说明
│  ├─ SDD.md                 # 软件设计说明
│  ├─ ROADMAP.md             # 迭代路线
│  └─ DEVELOPMENT.md         # 开发流程
├─ AGENTS.md             # 协作者与开发代理约定
└─ 剪贴板历史记录工具.html # 原始 UI 原型
```

## 文档

- [Tauri 迁移计划](docs/TAURI_MIGRATION.md)
- [轻量化设计与 exe 版本方案](docs/LIGHTWEIGHT_DESIGN.md)
- [产品需求说明](docs/PRD.md)
- [软件设计说明](docs/SDD.md)
- [迭代路线](docs/ROADMAP.md)
- [开发流程](docs/DEVELOPMENT.md)
- [开发代理约定](AGENTS.md)

## 开发节奏

项目采用“小步开发、重大节点提交并推送”的流程。每完成一个重大能力阶段，都应：

1. 更新代码和必要文档。
2. 运行 `npm run check`。
3. 提交 Git commit。
4. 推送到远端仓库。

## 隐私原则

ClipBall 默认定位为本地工具：不上传、不同步、不调用外部服务。后续实现剪贴板历史时，应提供暂停记录、删除单条、清空全部、最大记录数和自动过期等控制能力。

## 下一阶段

下一阶段计划：

1. 完成 Tauri 工程初始化与窗口能力移植。
2. Rust 后端监听系统剪贴板文本，哈希去重。
3. 将历史记录保存到本地 JSON 文件。
4. 渲染层展示真实历史记录。
5. 支持点击记录写回剪贴板。
