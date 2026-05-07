# ClipBall

ClipBall 是一个以悬浮球形态常驻桌面的本地剪贴板历史工具。它在未展开时显示为轻量悬浮球，点击后展开为可拖拽、可调整大小的历史面板，方便用户查看复制历史并快速粘贴。

当前项目处于早期开发阶段：已完成 Electron 桌面应用骨架、悬浮球/历史面板 UI、窗口模式切换和 Windows 便携版 exe 打包流程；真实剪贴板监听、本地持久化和快速粘贴还在后续阶段。

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

- Electron
- HTML / CSS / JavaScript
- Node.js

首版选择 Electron 是为了复用现有 HTML 原型，并方便后续接入系统剪贴板、全局快捷键、托盘、置顶窗口和本地存储。

## 快速开始

安装依赖：

```powershell
npm install
```

启动开发版桌面应用：

```powershell
npm start
```

运行基础检查：

```powershell
npm run check
```

## 打包 Windows exe

生成 Windows 便携版：

```powershell
npm run package:win
```

构建产物：

```text
release/ClipBall-win32-x64/ClipBall.exe
```

当前产物是便携版应用目录，不是单文件安装包。运行时需要保留整个 `release/ClipBall-win32-x64` 文件夹，因为 `ClipBall.exe` 依赖同目录下的 Electron 运行时资源。

## 项目结构

```text
.
├─ src/
│  ├─ main/              # Electron 主进程：窗口、模式切换、后续系统能力
│  ├─ preload/           # 安全桥：向渲染层暴露受控 API
│  └─ renderer/          # UI：悬浮球、历史面板、示例历史列表
├─ scripts/
│  └─ package-win.js     # Windows 便携版 exe 打包脚本
├─ docs/
│  ├─ PRD.md             # 产品需求说明
│  ├─ SDD.md             # 软件设计说明
│  ├─ ROADMAP.md         # 迭代路线
│  └─ DEVELOPMENT.md     # 开发流程
├─ AGENTS.md             # 协作者与开发代理约定
└─ 剪贴板历史记录工具.html # 原始 UI 原型
```

## 文档

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

下一步建议进入文本历史 MVP：

1. 主进程监听系统剪贴板文本。
2. 对复制内容做哈希去重。
3. 将历史记录保存到本地。
4. 渲染层展示真实历史记录。
5. 支持点击记录写回剪贴板。

