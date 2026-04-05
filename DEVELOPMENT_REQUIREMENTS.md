# Keyboard Player 开发需求文档

## 1. 技术栈

- 语言：Rust（edition 2024）
- 游戏引擎：Bevy 0.18.1
- 平台：桌面应用（macOS / Windows / Linux）
- 视觉风格：卡通像素风格

## 2. 项目结构

```
kb-player-bevy/
├── assets/
│   ├── texts/                  # 范文库（JSON 文件平铺存放）
│   ├── fonts/                  # Ark Pixel 像素字体（OFL-1.1）
│   └── audio/                  # Kenney CC0 音效文件
├── src/
│   ├── main.rs                 # 入口，插件注册，相机/资源初始化
│   ├── states/                 # 游戏状态机
│   │   ├── mod.rs              # GameState 枚举 + StatesPlugin
│   │   ├── menu.rs             # 主菜单（键盘操作）
│   │   ├── selection.rs        # 语言/年级/难度选择（键盘+鼠标）
│   │   ├── playing.rs          # 打字练习（输入/光标/暂停/计分）
│   │   └── result.rs           # 结算页面
│   ├── systems/                # 纯逻辑 Systems
│   │   ├── mod.rs
│   │   ├── scoring.rs          # （预留）
│   │   └── difficulty.rs       # 困难模式字符隐藏算法
│   ├── resources/              # Bevy Resources
│   │   ├── mod.rs
│   │   ├── font_assets.rs      # FontAssets（FromWorld 初始化）
│   │   ├── game_config.rs      # 用户选择配置
│   │   └── game_data.rs        # TextLibrary / CurrentPassage
│   ├── data/                   # 数据模型与加载
│   │   ├── mod.rs
│   │   ├── text_model.rs       # TextPassage / Language / Grade / Difficulty
│   │   └── text_loader.rs      # 范文目录扫描加载
│   ├── audio/                  # 音效管理
│   │   ├── mod.rs
│   │   └── sfx.rs              # SfxHandles（FromWorld）+ 音效播放
│   ├── storage/                # 本地持久化
│   │   ├── mod.rs
│   │   └── records.rs          # JSON 记录读写
│   ├── components/             # （预留）
│   │   └── mod.rs
│   └── ui/                     # （预留）
│       └── mod.rs
├── Cargo.toml
└── README.md
```

## 3. 窗口与 UI 设计

### 3.1 窗口设置

- 支持全屏切换
- 默认比例 16:9（如 1280x720）
- 窗口标题："Keyboard Player"

### 3.2 配色方案（护眼深色主题）

| 元素         | 颜色                  |
|-------------|----------------------|
| 背景色       | 深灰蓝 (#1a1a2e)     |
| 范文文字     | 柔和白 (#e0e0e0)      |
| 正确输入     | 绿色 (#4ade80)        |
| 错误输入     | 红色 (#f87171)        |
| 隐藏字符占位  | 像素风问号方块图标      |
| UI 元素      | 与像素风格一致的按钮、边框 |

### 3.3 字体

- 使用像素风格字体（需支持中英文）
- 需寻找开源可用的像素字体

## 4. 游戏状态机

```
MainMenu -> Selection -> Playing -> Result
   ^            ^           |          |
   |            └── Pause ──┘          |
   └───────────────────────────────────┘
```

- `MainMenu`：主菜单，Enter/Space 开始游戏
- `Selection`：依次选择语言 → 年级 → 难度，支持键盘方向键 + Enter 和鼠标点击
- `Playing`：打字练习核心玩法，Esc 暂停
- `Result`：结算页面，显示成绩，是否破纪录

### 4.1 选择界面键盘交互

- `SelectionState` 资源追踪当前步骤（Language/Grade/Difficulty）和焦点索引
- 每个选项按钮携带 `ButtonIndex(usize)` 组件
- 焦点按钮高亮显示（绿色边框 + 悬浮背景色）
- 键盘与鼠标操作兼容共存，互不干扰
- 切换步骤时焦点自动重置到第一个选项

## 5. 输入处理

### 5.1 英文输入

- 捕获键盘事件（KeyboardInput / ReceivedCharacter）
- 逐字符与范文比对
- 实时反馈颜色变化

### 5.2 中文输入（优先级较低）

- 支持 IME 输入法事件
- 通过 Bevy 的 IME 相关事件获取已确认的输入文本
- 逐字与范文比对

### 5.3 自动分行

- 英文按单词边界分行，每行不超过 60 字符（不拆词）
- 中文在句末标点（。！？；）处分行，每行不超过 25 字符

## 5.5 暂停系统

### 5.5.1 暂停状态管理

- `GamePaused` 资源追踪：暂停标志、暂停开始时间、累计暂停时长
- 暂停时通过 `not_paused` 运行条件阻止所有游戏逻辑系统执行
- `handle_pause_input` 始终运行，不受暂停条件影响

### 5.5.2 暂停交互

| 按键 | 效果 |
|-----|------|
| Esc（游戏中） | 暂停，显示遮罩 |
| Esc / Enter / Space（暂停中） | 恢复游戏 |
| Q（暂停中） | 退出到主菜单，不保存成绩 |

### 5.5.3 计时处理

- 暂停期间 `total_paused` 累加暂停时长
- HUD 计时和最终成绩均减去 `total_paused`，确保暂停时间不计入
- 暂停期间 HUD 显示冻结（系统不执行）

### 5.5.4 暂停 UI

- 全屏半透明暗色遮罩（`GlobalZIndex(10)` 置顶）
- 显示 "PAUSED" 标题和操作提示
- 遮罩使用 `DespawnOnExit(GameState::Playing)` 确保状态切换时自动清理

## 5.6 输入光标

- 当前输入行末尾显示闪烁光标 `▏`
- 光标以 0.53 秒间隔在可见/透明之间切换（`CursorTimer` 资源）
- 空输入时光标显示在行首
- 光标跟随输入内容实时移动
- 光标闪烁通过设置 `input_dirty` 触发输入显示重建

## 6. 范文数据

### 6.1 数据格式

- JSON 格式
- 存放于 `assets/texts/` 目录
- 命名规则：`{language}_{grade}_{number}.json`

### 6.2 数据结构（Rust）

```rust
#[derive(Deserialize)]
struct TextPassage {
    id: String,           // 与文件名一致
    language: Language,   // En / Zh
    grade: Grade,         // Elementary / Middle / High
    title: String,
    author: Option<String>,
    content: String,      // 完整文本，程序自动分行
}

enum Language {
    En,
    Zh,
}

enum Grade {
    Elementary,
    Middle,
    High,
}
```

### 6.3 加载逻辑

- 启动时扫描 `assets/texts/` 目录下所有 JSON 文件
- 解析并按 language + grade 分类索引
- 根据用户选择随机抽取一篇

## 7. 困难模式实现

### 7.1 英文隐藏规则

- 统计范文总字母数，随机选择不超过 1/3 隐藏
- 约束：每个i单词最多隐藏 1 个字符
- 不隐藏空格和标点

### 7.2 中文隐藏规则

- 按短句（以逗号、句号、问号、感叹号分割）处理
- 每句若大于2个字，则随机隐藏 1 个汉字
- 不隐藏标点符号

### 7.3 显示方式

- 被隐藏的字符位置显示为像素风图标（如像素问号方块）
- 图标大小与文字大小一致，保持排版不变

### 7.4 可配置性

- 隐藏比例、每单词/每句上限等参数通过配置文件或常量定义
- 支持后续调整

## 8. 计分与记录系统

### 8.1 实时数据

- 计时器：从第一次按键开始计时
- KPM：实时计算 = 已输入字符数 / 已用时间（分钟）
- 正确率：正确字符数 / 总输入字符数

### 8.2 结算数据

- 总完成时间
- 平均 KPM
- 正确率
- 是否破纪录

### 8.3 本地持久化

- 存储格式：JSON
- 存储位置：用户数据目录（通过 `dirs` crate 获取）
- 记录内容：每次练习的范文 ID、完成时间、KPM、正确率、日期
- 支持查询历史最佳记录

## 9. 音效系统

### 9.1 音效列表

| 场景     | 描述              | 格式     |
|---------|------------------|---------|
| 按键正确  | 短促清脆的正反馈音  | OGG/WAV |
| 按键错误  | 轻微的错误提示音   | OGG/WAV |
| 完成一行  | 行完成的提示音     | OGG/WAV |
| 破纪录   | 庆祝/胜利音效      | OGG/WAV |

### 9.2 音效来源

- 从免费可商用音效网站获取（freesound.org、opengameart.org 等）
- 优先选择 8-bit / 像素风格音效以匹配整体视觉风格

## 10. 依赖项

| Crate      | 版本    | 用途                 |
|-----------|--------|---------------------|
| bevy       | 0.18.1 | 游戏引擎             |
| serde      | 1      | 序列化/反序列化       |
| serde_json | 1      | JSON 处理            |
| rand       | 0.8    | 随机选择范文和隐藏字符  |
| dirs       | 6      | 获取用户数据目录       |

## 10.5 Bevy 0.18 API 要点

与旧版本/网上教程的主要差异，开发时须注意：

| 概念 | Bevy 0.18 用法 |
|-----|---------------|
| 状态实体清理 | `DespawnOnExit(state)`（非 `StateScoped`） |
| 事件定义 | `#[derive(Message)]`（非 `#[derive(Event)]`） |
| 事件读写 | `MessageReader` / `MessageWriter`，`.write()` 发送 |
| 子实体构建 | `ChildSpawnerCommands`（非 `ChildBuilder`） |
| 删除子实体 | `despawn_related::<Children>()`（非 `despawn_descendants()`） |
| 窗口分辨率 | `WindowResolution` 接受 `(u32, u32)` |
| 单实体查询 | `Single<&mut Window>` |
| 键盘输入文本 | `KeyboardInput` 的 `.text` 字段 |
| IME 事件 | `MessageReader<Ime>` |
| 资源初始化 | 需要 `AssetServer` 的资源用 `FromWorld` + `init_resource`，确保在 `OnEnter` 之前可用 |

## 11. 开发阶段规划

| 阶段     | 内容                         | 优先级 |
|---------|------------------------------|-------|
| Phase 1 | 项目脚手架 + 状态机 + 窗口配置  | 高    |
| Phase 2 | 选择界面 + 范文数据加载        | 高    |
| Phase 3 | 核心打字玩法（英文）           | 高    |
| Phase 4 | 困难模式                      | 中    |
| Phase 5 | 计分系统 + 本地存储            | 中    |
| Phase 6 | 音效与正反馈                   | 中    |
| Phase 7 | UI 打磨与像素风格适配          | 中    |
| Phase 8 | 中文 IME 支持                 | 低    |

## 12. 测试策略

### 12.1 单元测试

项目内嵌 `#[cfg(test)]` 模块，覆盖以下纯逻辑：

| 模块 | 测试内容 |
|------|---------|
| `systems/difficulty.rs` | 英文/中文隐藏位置生成、边界条件、每词/每句约束 |
| `states/playing.rs` | 英文/中文分行算法、色彩段构建、空输入处理 |
| `data/text_loader.rs` | 范文加载、语言年级键匹配、不存在目录容错 |
| `storage/records.rs` | 空存储、最高 KPM、分类筛选、JSON 序列化往返 |

运行方式：`cargo test`

### 12.2 运行时验证

- `cargo run` 验证完整游戏流程：主菜单 → 选择 → 打字 → 暂停/恢复 → 结算
- 验证键盘全流程可操作（无需鼠标）
- 验证中途退出不产生成绩记录

## 13. 外部资源

| 资源 | 来源 | 许可证 |
|------|------|--------|
| Ark Pixel Font 12px | [ark-pixel-font](https://github.com/TakWolf/ark-pixel-font) | OFL-1.1 |
| Interface Sounds | [Kenney](https://kenney.nl/) | CC0 |
