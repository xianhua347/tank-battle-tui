# Tank Battle TUI

一个使用 Rust、ratatui 和 crossterm 编写的终端版坦克大战。

> 说明：本项目由 OpenAI Codex agents 生成，包括 Rust TUI 游戏代码、项目结构和说明文档。

## 素材

项目中的 `assets/sprite-sheet-reference.png` 是使用 OpenAI imageGen 生成的像素风坦克大战素材参考图。终端运行时不能直接稳定渲染 PNG 精灵，所以游戏把这套素材映射为 TUI 字符和颜色：绿色玩家坦克、红色敌方坦克、黄色子弹、灰色砖墙、橙色爆炸、红心生命值和星星分数。

## 运行

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
