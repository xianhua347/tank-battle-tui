# Tank Battle TUI

一个由 Rust TUI、Vue Web 和 Tauri Desktop 组成的坦克大战项目。

> 说明：本项目由 OpenAI Codex agents 生成，包括 Rust TUI 游戏代码、项目结构和说明文档。

## 素材

项目中的 `assets/sprite-sheet-reference.png` 是使用 OpenAI imageGen 生成的像素风坦克大战素材参考图。终端运行时不能直接稳定渲染 PNG 精灵，所以游戏把这套素材映射为 TUI 字符和颜色：绿色玩家坦克、红色敌方坦克、黄色子弹、灰色砖墙、橙色爆炸、红心生命值和星星分数。

## 目标平台

- Web：通过 Vue 3 + Vite 构建，可以部署为静态站点。
- Desktop：通过 Tauri v2 支持 Windows、macOS 和 Linux。
- Terminal：保留纯 Rust + ratatui/crossterm 的 TUI 版本。

## 安装依赖

推荐安装 Vite+ 的 `vp` 命令，然后使用 Vite+ 工作流：

```powershell
irm https://vite.plus/ps1 | iex
vp install
```

本分支使用 Vite+ 原生命令面，`vp` 会管理 Node.js、包管理器和前端工具链。Vite+ 当前组合包含 Vite 8 与 Rolldown。

## 运行 Web 版本

```powershell
vp dev
```

如果需要通过 package script 调用：

```powershell
pnpm dev
```

## 运行 Tauri 桌面版本

```powershell
pnpm tauri:dev
```

构建桌面安装包：

```powershell
pnpm tauri:build
```

## 构建 Web 版本

```powershell
vp build
```

如果需要通过 package script 调用：

```powershell
pnpm build
```

## 运行 Rust TUI 版本

先安装 Rust 工具链，然后在项目目录执行：

```powershell
cd F:\workspace\tank-battle-tui
cargo run
```

## 操作

- `W/A/S/D` 或方向键：移动
- `Space`：射击
- `P`：暂停/继续
- `R`：游戏结束后重开
- `Q` 或 `Esc`：退出

## 玩法

消灭敌方坦克获得分数。敌人会自动移动和射击，玩家初始有 3 点生命值。被击中扣 1 点生命值，生命值归零后游戏结束。
