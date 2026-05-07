# 开发流程

本项目采用“小步开发、重大节点提交并推送”的节奏。每次进入新的主要能力阶段前，先确认当前工作区干净；完成阶段目标后，运行可用检查，提交并推送。

## 分支策略

- `main`：稳定主线，保持可运行或至少文档一致。
- `feature/*`：较大的功能可使用特性分支。
- `fix/*`：缺陷修复分支。

当前项目早期可以直接在 `main` 上推进；当进入打包发布、跨平台适配或复杂功能时，再拆分特性分支。

## 提交策略

建议提交信息格式：

```text
type(scope): summary
```

常用类型：

- `docs`：文档。
- `feat`：新功能。
- `fix`：修复缺陷。
- `refactor`：不改变行为的结构调整。
- `test`：测试。
- `chore`：工程配置。

示例：

```text
docs(project): add product and software design docs
feat(app): add floating ball shell
feat(history): persist text clipboard items
fix(paste): fall back to copy when quick paste fails
```

## 版本推进规则

每一次重大版本改进都应完成以下动作：

1. 明确本阶段目标。
2. 完成代码和必要文档更新。
3. 运行可用的检查或测试。
4. 执行一次 Git 提交。
5. 如果已配置远端仓库，立即 `git push`。

重大版本改进包括：

- 工程初始化完成。
- 悬浮球和历史面板可运行。
- 剪贴板监听完成。
- 本地持久化完成。
- 快速粘贴完成。
- 设置面板完成。
- 图片或文件历史支持完成。
- 打包发布流程完成。

## 本地检查

项目初始化前以文档检查为主；进入应用开发后，应逐步补充：

- TypeScript 类型检查。
- 单元测试。
- UI 构建。
- Electron 启动检查。
- 手动验证悬浮球、拖拽、缩放、复制、粘贴、删除、清空。

## Windows 打包

当前项目提供便携版 exe 打包脚本：

```powershell
npm run package:win
```

构建产物位于：

```text
release/ClipBall-win32-x64/ClipBall.exe
```

这个 exe 依赖同目录下的 Electron 运行时资源，因此交付时应保留整个 `ClipBall-win32-x64` 文件夹。需要单文件安装包时，再接入 `electron-builder` 或 NSIS。

## 推送约定

远端仓库配置后，默认使用：

```powershell
git push -u origin main
```

后续已建立 upstream 后使用：

```powershell
git push
```

如果远端不存在，需要先创建 GitHub 仓库或提供远端地址，再执行：

```powershell
git remote add origin <remote-url>
git push -u origin main
```
