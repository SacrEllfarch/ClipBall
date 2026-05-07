# 开发流程

本项目采用“小步开发、重大节点提交并推送”的节奏。每次进入新的主要能力阶段前，先确认当前工作区干净；完成阶段目标后，运行可用检查，提交并推送。

## 轻量化原则

- 前端不引入业务运行时依赖，`dependencies` 默认保持为空。
- 能用 Tauri/Rust 内置能力完成的，不新增第三方包。
- Rust 依赖通过 `Cargo.toml` 管理，优先使用轻量 crate。
- 不引入原生数据库模块；MVP 使用 JSON 文件存储历史和设置。
- 每个版本都必须保持 `npm run check` 和 `npm run tauri build` 可执行。

## 分支策略

- `main`：稳定主线，保持可运行或至少文档一致。
- `feature/*`：较大的功能可使用特性分支。
- `fix/*`：缺陷修复分支。

当前项目早期可以直接在 `main` 上推进；Tauri 迁移阶段建议创建 `migrate/tauri` 特性分支，完成后再合并回 `main`。

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
- 文本剪贴板监听完成。
- JSON 本地持久化完成。
- 快速粘贴完成。
- 设置面板完成。
- Tauri 工程初始化完成。
- 便携版 exe 发布流程完成。
- MSI / NSIS 安装包发布流程完成。

## 本地检查

项目初始化前以文档检查为主；进入应用开发后，应逐步补充：

- Rust 编译检查（`cargo check` / `cargo clippy`）。
- 前端单元测试（如后续引入）。
- Tauri 开发版启动检查（`npm run tauri dev`）。
- 手动验证悬浮球、拖拽、缩放、复制、粘贴、删除、清空。

## Windows 打包

使用 Tauri 内置打包命令：

```powershell
npm run tauri build
```

构建产物：

- 便携版 exe：
  ```text
  src-tauri/target/release/ClipBall.exe
  ```
- MSI 安装包：
  ```text
  src-tauri/target/release/bundle/msi/ClipBall_<version>_x64_en-US.msi
  ```
- NSIS 安装包：
  ```text
  src-tauri/target/release/bundle/nsis/ClipBall_<version>_x64-setup.exe
  ```

v0.x 优先发布便携版 `ClipBall.exe`；需要卸载入口、桌面快捷方式和开机启动时，直接使用 Tauri 产出的 MSI 或 NSIS 安装包，无需额外工具链。

## exe 版本发布步骤

1. 更新 `package.json` 和 `src-tauri/Cargo.toml` 中的版本号。
2. 更新 README 或 docs 中对应版本说明。
3. 运行 `npm run check`。
4. 运行 `npm run tauri build`。
5. 人工启动 `src-tauri/target/release/ClipBall.exe` 验证核心流程。
6. 将 `ClipBall.exe`（或 MSI/NSIS 安装包）作为发布产物。
7. 提交并推送代码，再创建对应版本标签。

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
