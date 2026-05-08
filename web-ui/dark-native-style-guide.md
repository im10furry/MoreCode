# Dark Native 暗色原生风格 — 开发者暗色 UI 设计规范

> **定位**: 面向开发者工具、代码编辑器、终端、仪表盘、运维监控面板的"原生暗色"设计语言。
> 这不是为普通应用"加个暗色模式"，而是还原操作系统级工具 UI 的视觉语言——紧凑、高密度、代码优先、终端美学。

---

## 目录

1. [设计理念](#1-设计理念)
2. [视觉特征](#2-视觉特征)
3. [色彩系统](#3-色彩系统)
4. [字体排版](#4-字体排版)
5. [布局系统](#5-布局系统)
6. [组件规范](#6-组件规范)
7. [间距规范](#7-间距规范)
8. [边框规范](#8-边框规范)
9. [滚动条](#9-滚动条)
10. [选中状态](#10-选中状态)
11. [AI 提示词 — 设计生成指引](#11-ai-提示词--设计生成指引)
12. [CSS 变量 — 暗色主题实现](#12-css-变量--暗色主题实现)

---

## 1. 设计理念

```
暗色原生风格 = 开发者工具 UI 语言 + 终端美学 + 代码优先设计
```

### 核心原则

| 原则 | 说明 |
|------|------|
| **暗色优先 (Dark-First)** | 从底层就为暗色设计，而非浅色改暗色。深色背景减少长时间使用的眼部疲劳，适合开发者每天8-12小时的持续注视。 |
| **原生感 (Native Feel)** | 操作系统级别的 UI 语言：无装饰性冗余，无多余阴影，无圆角过度。控件看起来像 macOS / Windows / Linux 的原生系统组件。 |
| **代码优先 (Code-First)** | 设计以代码展示为核心。语法高亮配色是整套设计语言色彩系统的起点。等宽字体是视觉层级中的一等公民。 |
| **高信息密度 (High Density)** | 开发者需要在一屏内看到尽可能多的信息。紧凑间距、小字号、高效布局。不做"呼吸感"设计，做"效率感"设计。 |
| **低调克制 (Restrained)** | 色彩不喧宾夺主。层级差异通过 2-3% 的亮度变化体现。边框若有若无。控件在需要时才显现。 |

### 灵感来源

| 产品 | 借鉴要点 |
|------|----------|
| **VS Code** | Activity Bar + Sidebar + Editor + Panel 四栏布局；#1e1e1e 基底色系；选项卡、文件树、命令面板范式 |
| **GitHub Dark** | #0d1117 更深沉的底色；蓝色 #58A6FF 强调链接；卡片微边框风格 |
| **macOS Dark Mode** | 系统原生控件语感；Vibrancy 半透明侧边栏；SF 字体系列 |
| **Windows Dark Mode** | 标题栏深色集成；WinUI 紧凑控件；Segoe UI 字体 |
| **iTerm2 / Terminal.app** | 终端美学：纯黑/深蓝底色、绿色光标、`$` 提示符、ANSI 色彩体系 |
| **Notion Dark** | 极简暗色文档质感；轻量侧边栏；hover 态微妙的背景变化 |
| **Obsidian** | 插件化面板布局；文件列表的缩进层级；链接图谱暗色渲染 |

---

## 2. 视觉特征

### 2.1 整体印象

> 打开界面的一瞬间，应该让人感觉"这是给开发者用的工具"——而非普通消费应用。

- **底色深沉**: 绝对不使用纯黑 `#000000`，也不使用灰白背景。暗色基底始终带有一丝色调偏移（冷灰或蓝灰）。
- **图层微差**: 不同层级的面板之间仅靠 2-3% 的亮度差区分，不依赖粗重阴影或高对比边框。
- **细线美学**: 分隔线 1px，颜色 rgba(255,255,255,0.06) ~ rgba(255,255,255,0.10)。有些地方甚至仅用背景色差代替边框。
- **双字体体系**: 界面文字用系统无衬线体，代码内容用等宽字体（带连字 feature）。
- **语法高光色彩**: 代码着色是整套设计系统中最"亮"的元素之一——keywords 蓝、strings 橙、functions 黄。
- **IDE 布局范式**: 左侧纵向图标栏 → 侧边栏 → 主编辑区 → 底部状态栏/面板。这是开发者肌肉记忆中的布局。

### 2.2 典型界面结构

```
┌──────────────────────────────────────────────────────────────┐
│  菜单栏 / 标题栏 (可选)                                       │
├────┬─────────────┬──────────────────────────────┬────────────┤
│    │             │  标签栏 (文件选项卡)          │            │
│    │             ├──────────────────────────────┤            │
│ Ac │  侧边栏     │                              │  右侧面板   │
│ ti │             │                              │  (可选)    │
│ vi │  文件树     │     主内容区 / 编辑器         │            │
│ ty │             │                              │            │
│    │             │                              │            │
│ Ba │  搜索       │                              │            │
│ r  │  大纲       │                              │            │
│    │             │                              │            │
├────┴─────────────┴──────────────────────────────┴────────────┤
│  状态栏 (22-24px)  ⚡ Ln 42, Col 18  Spaces: 2  UTF-8  Go ▸  │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. 色彩系统

### 3.1 背景层级体系

背景从深到浅逐级上升，每层亮度增加约 2-3%。这确保了视觉层次清晰但整体统一。

#### VS Code 风格色系 (冷灰基调)

| 层级 | 变量名 | 色值 | 用途 |
|------|--------|------|------|
| 0 | `--bg-root` | `#1e1e1e` | 最底层：编辑器画布、Activity Bar 背景 |
| 1 | `--bg-sidebar` | `#252526` | 侧边栏、文件浏览器、左侧面板 |
| 2 | `--bg-content` | `#1e1e1e` | 主编辑区背景 |
| 3 | `--bg-card` | `#2d2d30` | 卡片、面板头部、标签栏未激活态 |
| 4 | `--bg-dropdown` | `#3c3c3c` | 下拉菜单、弹出面板 |
| 5 | `--bg-hover` | `#2a2d2e` | 列表项 hover 态 |
| 6 | `--bg-active` | `#37373d` | 列表项选中态、当前标签页 |
| 7 | `--bg-input` | `#3c3c3c` | 输入框背景 |
| 8 | `--bg-statusbar` | `#007acc` | 状态栏（可带品牌色） |

#### GitHub Dark 风格色系 (蓝灰基调)

| 层级 | 变量名 | 色值 | 用途 |
|------|--------|------|------|
| 0 | `--gh-bg-root` | `#0d1117` | 页面基底 |
| 1 | `--gh-bg-sidebar` | `#0d1117` | 侧边栏（与根同色） |
| 2 | `--gh-bg-content` | `#0d1117` | 内容区 |
| 3 | `--gh-bg-card` | `#161b22` | 卡片、面板 |
| 4 | `--gh-bg-hover` | `#1c2128` | hover 态 |
| 5 | `--gh-bg-active` | `#1f2428` | 激活/选中态 |
| 6 | `--gh-bg-input` | `#0d1117` | 输入框（内凹效果） |
| 7 | `--gh-bg-border` | `#30363d` | 边框专用 |

#### Terminal 终端风格色系 (深蓝黑基调)

| 层级 | 变量名 | 色值 | 用途 |
|------|--------|------|------|
| 0 | `--term-bg` | `#0c0c0c` | Windows Terminal 默认 |
| — | `--term-bg-alt` | `#1a1a2e` | 蓝黑终端背景 |
| — | `--term-bg-alt2` | `#0d1117` | GitHub 暗色终端 |

### 3.2 文字层级体系

使用白色透明度构建文字层级，暗色背景下需要更高的对比度。

```
白色透明度递减层级:
─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
  90% — 主文字 / 正文 (等价 #e6e6e6)
  70% — 次文字 / 标题辅助 (等价 #b3b3b3)
  50% — 辅助文字 / 占位符 (等价 #808080)
  30% — 禁用文字 / 低调提示 (等价 #4d4d4d)
─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
```

| 层级 | 变量名 | 颜色 | 实际色值（在 #1e1e1e 上） | 用途 |
|------|--------|------|--------------------------|------|
| 主要 | `--text-primary` | `rgba(255,255,255,0.90)` | `#e6e6e6` | 正文、标题、文件名 |
| 次要 | `--text-secondary` | `rgba(255,255,255,0.70)` | `#b3b3b3` | 描述、路径、meta |
| 辅助 | `--text-tertiary` | `rgba(255,255,255,0.50)` | `#808080` | 占位符、行号、图标说明 |
| 禁用 | `--text-disabled` | `rgba(255,255,255,0.30)` | `#4d4d4d` | 禁用态文字 |

### 3.3 强调色 / 品牌色

| 色彩 | 变量名 | 色值 | 来源 | 用途 |
|------|--------|------|------|------|
| 🔵 蓝 | `--accent-blue` | `#007acc` | VS Code | 主强调色、链接、选中、焦点 |
| 🔵 蓝 | `--accent-blue-gh` | `#58a6ff` | GitHub | 链接、强调、按钮 |
| 🟢 绿 | `--accent-green` | `#4ec9b0` | VS Code 语法 | 字符串、成功状态、git added |
| 🟢 绿 | `--accent-green-gh` | `#3fb950` | GitHub | 成功、合并 |
| 🟡 黄 | `--accent-yellow` | `#dcdcaa` | VS Code 语法 | 函数名、警告 |
| 🟡 黄 | `--accent-yellow-gh` | `#d29922` | GitHub | 警告、修改 |
| 🔴 红 | `--accent-red` | `#f44747` | VS Code | 错误、删除、git removed |
| 🔴 红 | `--accent-red-gh` | `#f85149` | GitHub | 错误、关闭、danger |
| 🟣 紫 | `--accent-purple` | `#c586c0` | VS Code 语法 | 关键字特殊、类型、装饰器 |
| 🔵 青 | `--accent-teal` | `#4ec9b0` | VS Code 语法 | 类型注解 |

### 3.4 语法高亮色板 (Syntax Highlighting Palette)

这是整个设计语言色彩体系的"根"——许多 UI 色彩由此派生。

| 语法元素 | 色值 | 语义 | 派生 UI 用途 |
|----------|------|------|-------------|
| 关键字 (keyword) | `#569cd6` (蓝) | `function` `if` `return` `import` | 操作按钮、链接 |
| 字符串 (string) | `#ce9178` (橙) | `"hello"` `'world'` | 数据展示、值标签 |
| 函数名 (function) | `#dcdcaa` (黄) | `myFunc()` `handler` | 函数签名、高亮 |
| 注释 (comment) | `#6a9955` (绿) | `// ...` `/* ... */` | 提示信息、辅助说明 |
| 变量 (variable) | `#9cdcfe` (浅蓝) | `myVar` `counter` | 标识符、标签 |
| 类型 (type) | `#4ec9b0` (青) | `string` `int` `User` | 类型标签、徽章 |
| 数字 (number) | `#b5cea8` (浅绿) | `42` `3.14` | 数值展示 |
| 类名 (class) | `#4ec9b0` (青) | `ClassName` `MyComponent` | 组件名、标题 |
| 操作符 (operator) | `#d4d4d4` (白) | `=` `+` `=>` `&&` | 分隔符、连接符 |
| 预处理器 (preprocessor) | `#c586c0` (紫) | `#include` `#define` | 元信息、装饰 |

### 3.5 终端 ANSI 16 色标准

为终端面板特化的标准色，兼容传统终端体验：

| 索引 | 颜色名 | Normal | Bright |
|------|--------|--------|--------|
| 0 | Black | `#0c0c0c` | `#767676` |
| 1 | Red | `#c50f1f` | `#e74856` |
| 2 | Green | `#13a10e` | `#16c60c` |
| 3 | Yellow | `#c19c00` | `#f9f1a5` |
| 4 | Blue | `#0037da` | `#3b78ff` |
| 5 | Magenta | `#881798` | `#b4009e` |
| 6 | Cyan | `#3a96dd` | `#61d6d6` |
| 7 | White | `#cccccc` | `#f2f2f2` |

### 3.6 状态色 (Semantic Colors)

脱离语法语境，在 UI 层使用的语义状态色：

| 状态 | 背景色 | 前景/图标色 | 边框色 |
|------|--------|-----------|--------|
| 🟢 成功 Success | `rgba(78, 201, 176, 0.15)` | `#4ec9b0` | `rgba(78, 201, 176, 0.30)` |
| 🔴 错误 Error | `rgba(244, 71, 71, 0.15)` | `#f44747` | `rgba(244, 71, 71, 0.30)` |
| 🟡 警告 Warning | `rgba(220, 220, 170, 0.15)` | `#dcdcaa` | `rgba(220, 220, 170, 0.30)` |
| 🔵 信息 Info | `rgba(86, 156, 214, 0.15)` | `#569cd6` | `rgba(86, 156, 214, 0.30)` |

---

## 4. 字体排版

### 4.1 字体栈 (Font Stack)

```css
/* UI 界面字体 — 系统无衬线体，优先匹配操作系统原生字体 */
--font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
           "Helvetica Neue", Arial, "Noto Sans", "PingFang SC",
           "Microsoft YaHei", sans-serif;

/* 代码 / 等宽字体 — 带连字特性的现代编码字体 */
--font-mono: "Cascadia Code", "JetBrains Mono", "Fira Code",
             "SF Mono", "Consolas", "Monaco", "Courier New",
             "Source Code Pro", "Fira Code Retina", monospace;
```

### 4.2 字体优先级

第一梯队（推荐，均支持连字 ligatures）:
1. **Cascadia Code** (微软出品，Windows Terminal 默认，连字丰富)
2. **JetBrains Mono** (JetBrains IDE 默认，字形清晰)
3. **Fira Code** (连字支持最广泛，社区首选)

第二梯队（备选）:
4. **SF Mono** (macOS 系统级等宽，Xcode 默认)
5. **Consolas** (Windows 经典等宽，Visual Studio 默认)
6. **Source Code Pro** (Adobe 开源，可读性好)

### 4.3 字号体系

开发者工具的尺度是"紧凑"——设计师常说"12px 的界面"。

| 用途 | 字号 | 行高 | 字重 | 示例 |
|------|------|------|------|------|
| 状态栏 | `11px` | `1.4` | `400` | "Ln 42, Col 18" |
| 文件树 | `12px` | `1.5` | `400` | "src/utils/helper.ts" |
| 标签页标题 | `12px` | `1.3` | `400` | "App.tsx" |
| UI 正文 | `13px` | `1.5` | `400` | 设置面板、对话框文字 |
| 代码编辑器 | `13-14px` | `1.6` | `400` | 主体代码内容 |
| 面板标题 | `11px` | `1.3` | `600` | "PROBLEMS", "OUTPUT" |
| 菜单项 | `13px` | `1.5` | `400` | 右键菜单、下拉菜单 |
| 标题 H1 | `16px` | `1.4` | `600` | 面板标题 |
| 标题 H2 | `14px` | `1.4` | `600` | 区块标题 |

### 4.4 连字 (Ligatures) 配置

```css
.code-editor {
  font-family: var(--font-mono);
  font-feature-settings: "calt" 1, "liga" 1, "dlig" 0;
  /* calt: contextual alternates (连字核心) */
  /* liga: standard ligatures */
  /* dlig: discretionary ligatures (可选，有些偏好关闭) */
}
```

常用连字渲染效果:
```
!=  →  ≠          >=  →  ≥
<=  →  ≤          =>  →  ⇒
->  →  →          <-  →  ←
::  →  ∷          === →  ≡
->> →  ↠          <|> →  ⋄
```

---

## 5. 布局系统

### 5.1 IDE 标准四栏布局

```
┌──────────────────────────────────────────────────────────────────┐
│  [≡] [File] [Edit] [Selection] [View] ...          [─] [□] [×] │  菜单栏 / 标题栏
├──────┬──────────────────────────────────────────────────────────┤
│      │  [🏠 index.tsx] [⚙️ config.ts] [📦 utils.ts]            ×│  标签栏
│      ├──────────────────────────────────────────────────────────┤
│  Ac  │                                                          │
│  ti  │  1  import React from 'react';                           │
│  vi  │  2  import { useState } from 'react';                    │
│  ty  │  3                                                        │
│      │  4  export function App() {                               │  编辑区 / 主内容
│  Bar │  5    const [count, setCount] = useState(0);             │
│  48px│  6                                                        │
│      │  7    return (                                           │
│      │  8      <div>Hello World</div>                           │
│      │  9    );                                                 │
│      │  10  }                                                   │
│      │                                                          │
│      ├──────────────────────────────────────────────────────────┤
│      │  PROBLEMS    OUTPUT    DEBUG CONSOLE    TERMINAL          │  底部面板
│      │  ─────────────────────────────────────                   │
│      │  $ npm run dev                                           │
│      │  > ready on http://localhost:3000                        │
│      │  $ █                                                     │
├──────┴──────────────────────────────────────────────────────────┤
│  ⚡ main  🔵 Go  ▸  Ln 8, Col 21  Spaces: 2  UTF-8  CRLF  {}  │  状态栏 22px
└──────────────────────────────────────────────────────────────────┘
```

### 5.2 各区域尺寸规范

| 区域 | 宽度 | 高度 | 备注 |
|------|------|------|------|
| Activity Bar | `48px` | 全高 | 图标列，不受缩放宽窄影响 |
| 侧边栏 (Sidebar) | `250px ~ 350px` | 全高 | 可拖拽调整宽度 |
| 主内容区 (Main) | `flex: 1` 剩余空间 | 全高 | 核心编辑区 |
| 右侧面板 (Right Panel) | `250px ~ 350px` | 全高 | 可选，可折叠 |
| 底部面板 (Panel) | `全宽` | `200px ~ 400px` | 终端/输出/问题面板 |
| 状态栏 (Status Bar) | `全宽` | `22px ~ 24px` | 固定底部 |
| 标签栏 (Tab Bar) | `全宽` | `35px ~ 36px` | 文件选项卡 |

### 5.3 响应式断点

尽管开发者工具通常在大屏使用，仍需定义断点：

| 断点 | 行为 |
|------|------|
| `> 1200px` | 完整四栏布局 |
| `900px ~ 1200px` | Activity Bar + Sidebar + Main，右侧面板折叠 |
| `600px ~ 900px` | Sidebar 折叠为汉堡菜单，保留 Activity Bar |
| `< 600px` | 全屏内容 + 底部导航 |

### 5.4 CSS Grid 实现

```css
.ide-layout {
  display: grid;
  grid-template-columns: 48px 280px 1fr;
  grid-template-rows: 36px 1fr 250px 24px;
  grid-template-areas:
    "activity tabbar   tabbar"
    "activity sidebar  main"
    "activity sidebar  panel"
    "status   status   status";
  height: 100vh;
  overflow: hidden;
}
```

---

## 6. 组件规范

### 6.1 文件树 (File Tree)

IDE 风格的文件浏览器，基于缩进表达层级。

```
📁 src/
  📁 components/
    📄 App.tsx
    📄 Header.tsx
  📁 utils/
    📄 helpers.ts
    📄 api.ts
  📄 main.tsx
📄 package.json
```

**设计规范**:
- 左侧 4px 缩进增量，每级嵌套递归缩进
- 文件夹展开/折叠用 chevron 图标（▶/▼），旋转动画 150ms
- 文件图标参考 **Seti** 或 **VSCode Icons** 风格（彩色图标 + 文件类型后缀）
- 当前选中项背景 `--bg-active`，文字 `--text-primary`
- Hover 项背景 `--bg-hover`
- 文件夹名称颜色：`--text-primary`（或轻微金色 `#dcdcaa`）
- 活动文件（编辑器打开中）: 文字变亮，左侧无额外指示条

```css
.file-tree-item {
  display: flex;
  align-items: center;
  height: 22px;
  padding: 0 6px 0 calc(6px + var(--depth) * 16px);
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  white-space: nowrap;
}
.file-tree-item:hover { background: var(--bg-hover); }
.file-tree-item.active { background: var(--bg-active); }
.file-tree-item .chevron {
  width: 16px;
  flex-shrink: 0;
  transition: transform 150ms ease;
}
.file-tree-item .chevron.open { transform: rotate(90deg); }
.file-tree-item .file-icon {
  width: 18px;
  height: 18px;
  margin-right: 4px;
  flex-shrink: 0;
}
```

### 6.2 选项卡 / 标签栏 (Tabs)

文档式选项卡，关闭按钮在 hover 时显现。

```
┌───────────────┬───────────────┬───────────────┬────────────────────────────┐
│ 🟡 App.tsx  × │ ⚪ config.ts × │ 📄 utils.ts × │   [⤢ 新建]               │
└───────────────┴───────────────┴───────────────┴────────────────────────────┘
```

**设计规范**:
- 高度: `35px ~ 36px`
- 选项卡背景（未激活）: `--bg-card` (`#2d2d30`)
- 选项卡背景（激活中）: `--bg-content` (`#1e1e1e`) — 与编辑区同色，视觉上无缝连接
- 选中指示器: 顶部 1px 彩色 border-top（蓝色 `--accent-blue`），或仅靠颜色变化区分
- 文件名字体: `12px`，`--text-primary`
- 关闭按钮: 16x16px，hover 时显示（opacity 0→1），hover 按钮本身变红色背景
- 脏文件标记: 文件名前加 ● 圆点（白色 70%）
- 右侧溢出: 用 `...` 省略或折叠菜单

```css
.tab {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 12px;
  background: var(--bg-card);
  border-right: 1px solid rgba(255,255,255,0.06);
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
  min-width: 0;
}
.tab.active {
  background: var(--bg-content);
  color: var(--text-primary);
  border-top: 1px solid var(--accent-blue);
}
.tab .close-btn {
  opacity: 0;
  width: 20px;
  height: 20px;
  margin-left: 8px;
  border-radius: 3px;
  transition: opacity 100ms;
}
.tab:hover .close-btn { opacity: 1; }
.tab .close-btn:hover { background: rgba(244,71,71,0.3); }
```

### 6.3 终端面板 (Terminal Panel)

拟终端风格的输出面板。

```
┌──────────────────────────────────────────────────────────────────┐
│  PROBLEMS   OUTPUT   DEBUG CONSOLE   TERMINAL          [▼] [×]   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  $ npm install                                                   │
│  added 142 packages in 3s                                        │
│                                                                  │
│  $ npm run dev                                                   │
│  > app@1.0.0 dev                                                 │
│  > vite                                                          │
│  ready on http://localhost:5173                                  │
│                                                                  │
│  $ █                                                             │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**设计规范**:
- 背景: `--term-bg` (`#0c0c0c`) 或 `--bg-content` (`#1e1e1e`) 兼容所属面板
- 字体: `--font-mono`, `13px`, 行高 `1.5`
- 光标: 绿色 `#4ec9b0` 或白色，使用 `caret-color: #4ec9b0`，闪烁（blink 动画 1060ms step-end）
- 提示符: `$` 或 `❯`，绿色 `#4ec9b0`
- 命令文字: 白色 `--text-primary`
- 输出文字: 白色 `--text-secondary`
- 错误文字: 红色 `--accent-red`
- 警告文字: 黄色 `--accent-yellow`
- 内边距: `8px 12px`
- 滚动条: 暗色 mini 滚动条

```css
.terminal {
  background: var(--term-bg, #0c0c0c);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.5;
  padding: 8px 12px;
  overflow-y: auto;
  caret-color: #4ec9b0;
}
.prompt { color: #4ec9b0; }
.prompt::before { content: "$ "; }
.command { color: var(--text-primary); }
.output { color: var(--text-secondary); }
.output.error { color: var(--accent-red); }
.output.warning { color: var(--accent-yellow); }
@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}
.cursor-block {
  display: inline-block;
  width: 8px;
  height: 1em;
  background: #4ec9b0;
  animation: blink 1060ms step-end infinite;
}
```

### 6.4 路径面包屑 (Breadcrumb)

内联路径导航，无多余装饰。

```
🏠 src > 📁 components > 📁 dashboard > 📄 Widget.tsx
```

**设计规范**:
- 字体: `12px`, `--text-secondary`
- 分隔符: `>` 或 `›`，颜色 `--text-tertiary`
- 当前文件: 颜色 `--text-primary`
- 每段 hover: 背景 `--bg-hover`，圆角 `3px`
- 每个路径段可点击

```css
.breadcrumb {
  display: flex;
  align-items: center;
  font-size: 12px;
  color: var(--text-secondary);
  user-select: none;
}
.breadcrumb-segment {
  padding: 2px 4px;
  border-radius: 3px;
  cursor: pointer;
}
.breadcrumb-segment:hover { background: var(--bg-hover); }
.breadcrumb-separator {
  color: var(--text-tertiary);
  margin: 0 2px;
  cursor: default;
}
.breadcrumb-segment.current { color: var(--text-primary); }
```

### 6.5 命令面板 (Command Palette)

模态覆盖层 + 模糊搜索，IDE 的标志性交互。

```
┌──────────────────────────────────────┐
│                                      │
│          ┌─────────────────────┐     │
│          │ > █                 │     │
│          ├─────────────────────┤     │
│          │ 📝 Open File...     │     │
│          │ 💾 Save             │     │
│          │ 🔍 Find in Files    │     │
│          │ 🪟 Toggle Panel     │     │
│          │ ⚙️  Settings         │     │
│          └─────────────────────┘     │
│                                      │
└──────────────────────────────────────┘
```

**设计规范**:
- 覆盖层: 半透明黑底 `rgba(0,0,0,0.5)`，全屏遮罩
- 面板: 宽度 `500-600px`，水平居中，距顶部 `15-20%`
- 面板背景: `--bg-dropdown` (`#3c3c3c`)
- 面板边框: 1px `rgba(255,255,255,0.1)`
- 面板圆角: `6-8px`
- 面板阴影: `0 8px 32px rgba(0,0,0,0.5)`
- 输入框: 顶部，带 `>` 前缀，字体 14px
- 结果列表: 匹配字符高亮（蓝色或黄色）
- 选中项: 背景 `--bg-active`
- 快捷键提示: 右对齐，`--text-tertiary`, `11px`

```css
.command-palette-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  justify-content: center;
  padding-top: 15vh;
  z-index: 9999;
}
.command-palette {
  width: 580px;
  max-height: 400px;
  background: var(--bg-dropdown);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 6px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.5);
  overflow: hidden;
}
.cmd-input {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid rgba(255,255,255,0.06);
  font-size: 14px;
}
.cmd-input::before {
  content: ">";
  color: var(--accent-blue);
  margin-right: 8px;
}
.cmd-item {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
}
.cmd-item.selected { background: var(--bg-active); }
.cmd-item .shortcut {
  margin-left: auto;
  font-size: 11px;
  color: var(--text-tertiary);
}
.cmd-item .highlight {
  color: var(--accent-blue);
  font-weight: 600;
}
```

### 6.6 右键菜单 (Context Menu)

紧凑的弹出菜单。

```
┌────────────────────┐
│ ✂️  Cut      Ctrl+X │
│ 📋  Copy     Ctrl+C │
│ 📌  Paste    Ctrl+V │
├────────────────────┤
│ 🔍  Find All       │
│ 🔄  Refactor...    │
│ 📁  Open in Folder │
├────────────────────┤
│ ⚙️  Properties      │
└────────────────────┘
```

**设计规范**:
- 宽度: `180-220px`
- 背景: `--bg-dropdown`
- 边框: 1px `rgba(255,255,255,0.1)`
- 内边距: `4px 0`（顶部底部）
- 菜单项高度: `28px`
- 菜单项内边距: `0 12px`
- 分隔线: 1px `rgba(255,255,255,0.06)`
- 字体: `13px`, `--text-primary`
- 快捷键: 右对齐, `--text-tertiary`, `12px`
- hover: `--bg-active`
- 禁用项: `--text-disabled`, 无法点击
- 阴影: `0 4px 12px rgba(0,0,0,0.4)`

```css
.context-menu {
  position: absolute;
  width: 200px;
  background: var(--bg-dropdown);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 4px;
  padding: 4px 0;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  z-index: 1000;
}
.menu-item {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 12px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
}
.menu-item:hover { background: var(--bg-active); }
.menu-item.disabled {
  color: var(--text-disabled);
  pointer-events: none;
}
.menu-item .shortcut {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-tertiary);
}
.menu-divider {
  height: 1px;
  margin: 4px 0;
  background: rgba(255,255,255,0.06);
}
```

### 6.7 活动栏 (Activity Bar)

最左侧纵列图标栏，IDE 的导航枢纽。

```
┌──┐
│📄│  Explorer
│🔍│  Search
│🔄│  Source Control
│▶️│  Run & Debug
│🧩│  Extensions
├──┤
│👤│  Account (底)
│⚙️│  Settings
└──┘
```

**设计规范**:
- 宽度: `48px`，不可调整
- 背景: `--bg-root` (`#1e1e1e`)
- 图标: 24x24px，居中显示，颜色 `--text-tertiary`
- 选中图标: 颜色 `--text-primary`，左侧有 2px 蓝色竖线指示器
- Hover 图标: 颜色 `--text-secondary`
- 图标间距: 顶部第一个图标距顶 `12px`，后续间距 `4px`
- 徽章: 红色圆点或数字（如源码管理的未暂存更改数）
- 底部图标组: 用 `margin-top: auto` 推到底部（账户、设置）

```css
.activity-bar {
  display: flex;
  flex-direction: column;
  width: 48px;
  background: var(--bg-root);
  border-right: 1px solid rgba(255,255,255,0.06);
}
.activity-bar-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  position: relative;
  color: var(--text-tertiary);
  cursor: pointer;
}
.activity-bar-item:hover { color: var(--text-secondary); }
.activity-bar-item.active {
  color: var(--text-primary);
}
.activity-bar-item.active::before {
  content: "";
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 2px;
  background: var(--accent-blue);
  border-radius: 0 1px 1px 0;
}
.activity-bar-item .badge {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 8px;
  height: 8px;
  background: var(--accent-red);
  border-radius: 50%;
}
.activity-bar-spacer {
  flex: 1;
}
```

### 6.8 状态栏 (Status Bar)

底部信息栏，功能密集。

```
⚡ main  🔵 Go ▸  Ln 42, Col 18  Spaces: 2  UTF-8  CRLF  {}  Go  0△ 0✕  🔔
```

**设计规范**:
- 高度: `22px ~ 24px`
- 背景: `--accent-blue` (VS Code 风格) 或 `--bg-card` (无品牌色时)
- 文字: `11px`, `--text-primary`（在彩色背景上为 `white`）
- 每一项之间间距 `12-16px`
- 左侧: 源码控制分支、错误/警告数
- 右侧: 行列号、缩进设置、编码、行尾符、语言模式、通知
- 每个项目 hover: 背景提亮 10% (`rgba(255,255,255,0.12)`)
- 可点击项: 光标变为 pointer

```css
.status-bar {
  display: flex;
  align-items: center;
  height: 22px;
  background: var(--accent-blue, var(--bg-card));
  color: var(--text-primary);
  font-size: 11px;
  padding: 0 8px;
  user-select: none;
}
.status-bar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}
.status-bar-right {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-left: auto;
}
.status-bar-item {
  padding: 0 4px;
  height: 100%;
  display: flex;
  align-items: center;
  cursor: pointer;
}
.status-bar-item:hover {
  background: rgba(255,255,255,0.12);
}
```

### 6.9 输入框 (Input)

暗色背景输入框，内凹感。

```css
input.dark-input,
textarea.dark-input {
  background: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 3px;
  padding: 4px 8px;
  font-size: 13px;
  outline: none;
  transition: border-color 150ms;
}
input.dark-input:focus {
  border-color: var(--accent-blue);
}
input.dark-input::placeholder {
  color: var(--text-tertiary);
}
```

### 6.10 按钮 (Button)

button 风格极度克制，hover 才显露。

```css
.btn-primary {
  background: var(--accent-blue);
  color: white;
  border: none;
  padding: 4px 12px;
  font-size: 13px;
  border-radius: 3px;
}
.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
  border: 1px solid rgba(255,255,255,0.08);
  padding: 4px 12px;
  font-size: 13px;
  border-radius: 3px;
}
.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  border: none;
  padding: 4px 8px;
  font-size: 13px;
  border-radius: 3px;
}
.btn-ghost:hover { background: var(--bg-hover); }
```

### 6.11 对话框 / 模态框 (Dialog / Modal)

```css
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.modal {
  background: var(--bg-dropdown, #3c3c3c);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 6px;
  padding: 20px;
  min-width: 400px;
  max-width: 560px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.5);
}
.modal-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
}
```

---

## 7. 间距规范

开发者工具以"紧凑"为核心——间距不是为了创造呼吸感，而是为了在有限空间内呈现最大信息量。

### 7.1 基础间距 scale

```
--space-0:  0px
--space-1:  2px     极紧凑（图标间距、徽章偏移）
--space-2:  4px     紧凑（文件树缩进增量、标签页内边距）
--space-3:  6px     标准紧凑（菜单项内边距、列表项内部）
--space-4:  8px     标准（面板内边距、卡片内边距）
--space-5:  12px    宽松（面板间距、区域分隔）
--space-6:  16px    区域间距
--space-7:  20px    大间距（对话框内边距）
--space-8:  24px    最大间距（页面级留白）
```

### 7.2 典型间距应用

| 使用场景 | 间距值 | 说明 |
|----------|--------|------|
| 文件树缩进增量 | `4px` | 即左侧 padding 递归增加 `calc(16px * depth)`（含图标+缩进） |
| 文件树行内边距 | `0 6px` | 水平方向保持紧凑 |
| 标签页项目内边距 | `0 12px` | 左右各 12px |
| 菜单项内边距 | `0 12px` | 左右各 12px |
| 终端面板内边距 | `8px 12px` | 上 8px，左右 12px |
| 命令面板项目 | `6px 12px` | 垂直 6px |
| 命令面板整体 | `8px 12px`（搜索框） | — |
| 状态栏项目 | `0 4px` | 极紧凑 |
| 卡片内边距 | `12px` | 统一内边距 |
| 对话框内边距 | `20px` | 相对宽松 |

### 7.3 行高 (Line Height)

| 使用场景 | 行高 | 说明 |
|----------|------|------|
| 文件树、列表 | `1.5` (18px @12px) | 可点击行需要有足够的点击区域 |
| 菜单项 | `1.4` | 紧凑但可读 |
| 代码编辑器 | `1.6` (约 22px @14px) | 代码行需要较大的行间距以提升可读性 |
| 标签栏 | `1.3` | 标题紧凑 |
| 状态栏 | `1.4` | 单行紧凑 |
| 正文 / 说明文字 | `1.5` | 标准可读性 |
| 终端 | `1.5` | 等宽字体天然需要略高行距 |

---

## 8. 边框规范

暗色原生风格边框极为克制。常规应用常用的 `1px solid #ddd` 在此绝不出现。

### 8.1 边框层级

```css
--border-subtle: 1px solid rgba(255,255,255,0.06);  /* 近乎不可见，面板内部分隔 */
--border-default: 1px solid rgba(255,255,255,0.08); /* 标准分隔线 */
--border-visible: 1px solid rgba(255,255,255,0.10); /* 明显分隔，弹出层外框 */
--border-emphasis: 1px solid rgba(255,255,255,0.15); /* 强调边框（极少使用） */
--border-accent: 1px solid var(--accent-blue);       /* 聚焦环、选中指示 */
```

### 8.2 边框策略

| 区域 | 边框策略 |
|------|----------|
| Activity Bar 右侧 | `1px solid rgba(255,255,255,0.06)` |
| 侧边栏右侧 | `1px solid rgba(255,255,255,0.06)` |
| 标签栏底部 | 无边框，用背景色差区分 |
| 活动标签页顶部 | 1px `--accent-blue` |
| 命令面板外框 | 1px `rgba(255,255,255,0.10)` |
| 上下文菜单外框 | 1px `rgba(255,255,255,0.10)` |
| 输入框 | 1px `rgba(255,255,255,0.08)` |
| 输入框聚焦 | 1px `--accent-blue` |
| 列表行之间 | **无边框**，用 hover 背景变化替代 |
| 卡片 | 1px `rgba(255,255,255,0.06)` |
| 分隔线 | 1px `rgba(255,255,255,0.06)` |

### 8.3 无边框模式 (Borderless)

部分区域完全不用边框，仅靠背景色差异来区分：

```
侧边栏 (#252526)  |  主内容 (#1e1e1e)  |  右侧面板 (#252526)
                   │                    │
   无可见边框，仅靠 2% 亮度差自然分层
```

这种做法大量节省了视觉噪音，是"Native"感的核心来源。

---

## 9. 滚动条

### 9.1 滚动条设计

```css
::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}
::-webkit-scrollbar-track {
  background: transparent;  /* 轨道完全透明 */
}
::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.12);  /* 暗色滑块 */
  border-radius: 5px;
  /* 内缩效果：滑块比轨道窄 */
  border: 3px solid transparent;
  background-clip: padding-box;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(255,255,255,0.25);
  border: 2px solid transparent;
}
::-webkit-scrollbar-corner {
  background: transparent;
}
```

### 9.2 滚动条可选行为

- **始终可见 (VS Code 风格)**: 滚动条始终可见，窄 + 暗色
- **Auto-Hide (macOS 风格)**: 停止滚动 1.5s 后滚动条渐隐消失，hover 时重现
- **Mini 滚动条 (Sublime Text 风格)**: 3px 宽极细滚动条，无轨道

```css
/* Auto-hide 滚动条 */
.scroll-auto-hide::-webkit-scrollbar-thumb {
  opacity: 0;
  transition: opacity 300ms;
}
.scroll-auto-hide:hover::-webkit-scrollbar-thumb {
  opacity: 1;
}

/* Mini 滚动条 */
.scroll-mini::-webkit-scrollbar { width: 4px; }
.scroll-mini::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.15);
  border-radius: 2px;
  border: none;
}
```

### 9.3 代码地图 (Minimap)

VS Code 风格的右侧代码缩略图（可选 feature）:

```css
.minimap {
  position: absolute;
  right: 0;
  width: 60px;
  height: 100%;
  background: var(--bg-content);
  opacity: 0.3;
  overflow: hidden;
  z-index: 1;
}
.minimap:hover { opacity: 0.6; }
.minimap-viewport {
  position: absolute;
  width: 100%;
  background: rgba(255,255,255,0.06);
}
```

---

## 10. 选中状态

### 10.1 文本选中

```css
::selection {
  background: #264f78;      /* VS Code 选中色 */
  color: #ffffff;
}
```

备选方案:
- `#264f78` — VS Code 经典蓝选色（推荐）
- `#3399ff` — 更亮蓝选色
- `rgba(86, 156, 214, 0.4)` — 半透明蓝

### 10.2 焦点指示 (Focus Ring)

```css
*:focus-visible {
  outline: 1px solid var(--accent-blue);
  outline-offset: -1px;  /* 内缩聚焦环，不改变布局 */
}
```

VS Code 风格的焦点环是**内缩**的（`outline-offset: -1px`），不会改变元素尺寸。

### 10.3 列表项选中

```css
.item-active {
  background: var(--bg-active);     /* #37373d */
  color: var(--text-primary);       /* white 90% */
}
.item-selected {
  background: rgba(0, 122, 204, 0.2); /* 半透明蓝选择 */
  color: var(--text-primary);
}
```

---

## 11. AI 提示词 — 设计生成指引

以下提示词可用于通过 AI 设计工具生成符合 Dark Native 风格的界面。中英双语。

### 11.1 中文提示词

#### 完整界面生成

```
设计一个开发者工具界面，采用暗色原生风格 (Dark Native Style)。
整体氛围类似 VS Code 或 iTerm2 的暗色模式。

要求：
1. 背景使用深灰色调 (#1e1e1e / #252526 / #2d2d30 系列)，不使用纯黑。
2. 不同面板之间仅靠微小的亮度差异区分（2-3%），不依赖粗重边框或阴影。
3. 左侧是 48px 宽的活动栏 (Activity Bar)，放置竖向排列的图标。
4. 活动栏右侧是 280px 的侧边栏，内含文件浏览器树，带有缩进层级和 chevron 折叠箭头。
5. 主内容区顶部是文件标签栏（文档式选项卡），激活标签与内容区背景同色（#1e1e1e），未激活标签略黑（#2d2d30）。
6. 底部是 22px 的状态栏，蓝色背景 (#007acc)，白色文字 11px。
7. 底部面板有终端模拟器，黑色背景 (#0c0c0c)，等宽字体，绿色光标和 $ 提示符。
8. 文字层级：主文字白色 90%，次文字 70%，辅助文字 50%。
9. 所有边框使用 1px 的白色半透明线 (rgba(255,255,255,0.06-0.10))。
10. 按钮和控件极简克制，hover 时才出现背景变化。
11. UI 字体使用系统无衬线体，代码使用等宽字体 (JetBrains Mono / Cascadia Code)。
12. 整体紧凑，高信息密度，间距以 4-8px 为主。
```

#### 组件特化提示

```
暗色原生风格的文件浏览器组件：
- 背景 #252526，每行高度 22px
- 文件/文件夹前带 16px 的彩色文件类型图标 (Seti 风格)
- 文件夹前有 ▶ 折叠箭头，展开后旋转 90° 为 ▼
- 缩进层级每级增加 16px (含图标 + arrow)
- 当前选中行背景 #37373d，hover 行背景 #2a2d2e
- 文件夹名颜色白色 90%，文件名颜色白色 90%
- 打开的但未选中的文件用斜体或稍亮颜色表示
```

#### 色彩方案提示

```
暗色原生语法高亮配色方案：
- 关键字 (function/if/return/import): #569cd6 (蓝)
- 字符串 ("..."/'...'): #ce9178 (橙)
- 函数名 (myFunc): #dcdcaa (黄)
- 注释 (//...): #6a9955 (绿)
- 变量 (myVar): #9cdcfe (浅蓝)
- 类型 (string/int/User): #4ec9b0 (青)
- 数字 (42/3.14): #b5cea8 (浅绿)
- 操作符 (=/+): #d4d4d4 (白)
- 类名 (ClassName): #4ec9b0 (青)
背景: #1e1e1e，文字默认: #d4d4d4
```

### 11.2 English Prompts

#### Full Interface Generation

```
Design a developer tool interface using the Dark Native style.
The overall atmosphere should resemble VS Code or iTerm2 dark mode.

Requirements:
1. Backgrounds use dark gray tones (#1e1e1e / #252526 / #2d2d30 series), never pure black.
2. Different panels are distinguished only by subtle brightness differences (2-3%),
   without relying on heavy borders or shadows.
3. Left side: a 48px wide Activity Bar with vertically stacked icons.
4. To the right of the Activity Bar: a 280px sidebar containing a file explorer tree
   with indent-based hierarchy and chevron collapse arrows.
5. The main content area has a tab bar at the top (document-style tabs).
   Active tab shares the content background (#1e1e1e), inactive tabs are darker (#2d2d30).
6. Bottom: a 22px status bar with blue background (#007acc), white 11px text.
7. A bottom panel contains a terminal emulator with black background (#0c0c0c),
   monospace font, green cursor, and $ prompt.
8. Text hierarchy: primary text white 90%, secondary 70%, tertiary 50%.
9. All borders use 1px semi-transparent white lines (rgba(255,255,255,0.06-0.10)).
10. Buttons and controls are minimal — backgrounds appear only on hover.
11. UI text uses system sans-serif fonts; code uses monospace (JetBrains Mono / Cascadia Code).
12. Overall compact, high information density, padding primarily 4-8px.
```

#### Component-Specific Prompt

```
Dark Native file explorer component:
- Background #252526, each row height 22px
- 16px colored file type icons before files/folders (Seti icon style)
- chevron arrow (▶) before folders, rotates 90° to ▼ when expanded
- Indentation increases by 16px per level
- Selected row background #37373d, hover row background #2a2d2e
- Folder names: white 90%, file names: white 90%
- Open-but-unselected files shown in italic or slightly brighter color
```

#### Color Scheme Prompt

```
Dark Native syntax highlighting color scheme:
- Keywords (function/if/return/import): #569cd6 (blue)
- Strings ("..."/'...'): #ce9178 (orange)
- Function names (myFunc): #dcdcaa (yellow)
- Comments (//...): #6a9955 (green)
- Variables (myVar): #9cdcfe (light blue)
- Types (string/int/User): #4ec9b0 (teal)
- Numbers (42/3.14): #b5cea8 (light green)
- Operators (=/+): #d4d4d4 (white)
- Class names (ClassName): #4ec9b0 (teal)
Background: #1e1e1e, default text: #d4d4d4
```

#### Design Philosophy Prompt

```
Design philosophy for Dark Native style:
- Dark-first: built for dark mode from the ground up, not retrofitted
- Code-first: the code editor is the centerpiece, everything else supports it
- Native feel: operating-system-level UI language, no decorative excess
- High density: developers need to see as much information as possible on one screen
- Restrained: colors don't compete for attention; hierarchy comes from 2-3% brightness steps
- Terminal aesthetic: monospace fonts, green cursors, $ prompts, ANSI color palette
- Invisible until needed: controls, scrollbars, borders are subtle and appear on interaction
```

---

## 12. CSS 变量 — 暗色主题实现

### 12.1 完整 CSS 变量定义

```css
:root {
  /* ===== 背景层级 ===== */
  --bg-root:        #1e1e1e;   /* 画布基底、Activity Bar */
  --bg-sidebar:     #252526;   /* 侧边栏、左侧面板 */
  --bg-content:     #1e1e1e;   /* 主编辑区、内容区 */
  --bg-card:        #2d2d30;   /* 卡片、标签栏非激活态 */
  --bg-dropdown:    #3c3c3c;   /* 下拉菜单、弹出面板 */
  --bg-hover:       #2a2d2e;   /* 列表项 hover 态 */
  --bg-active:      #37373d;   /* 列表项选中态、当前标签页 */
  --bg-input:       #3c3c3c;   /* 输入框背景 */
  --bg-statusbar:   #007acc;   /* 状态栏（品牌色时） */
  --bg-terminal:    #0c0c0c;   /* 终端面板底色 */

  /* ===== 文字层级 ===== */
  --text-primary:    rgba(255, 255, 255, 0.90);
  --text-secondary:  rgba(255, 255, 255, 0.70);
  --text-tertiary:   rgba(255, 255, 255, 0.50);
  --text-disabled:   rgba(255, 255, 255, 0.30);

  /* ===== 强调色 ===== */
  --accent-blue:     #007acc;   /* 主强调 */
  --accent-blue-gh:  #58a6ff;   /* GitHub 蓝 */
  --accent-green:    #4ec9b0;   /* 字符串、成功 */
  --accent-yellow:   #dcdcaa;   /* 函数名、警告 */
  --accent-red:      #f44747;   /* 错误、删除 */
  --accent-purple:   #c586c0;   /* 关键字特殊 */
  --accent-teal:     #4ec9b0;   /* 类型注解 */
  --accent-orange:   #ce9178;   /* 字符串 */
  --accent-lightblue:#9cdcfe;   /* 变量 */
  --accent-lightgreen:#b5cea8;  /* 数字 */

  /* ===== 状态色背景 ===== */
  --status-success-bg:   rgba(78, 201, 176, 0.15);
  --status-error-bg:     rgba(244, 71, 71, 0.15);
  --status-warning-bg:   rgba(220, 220, 170, 0.15);
  --status-info-bg:      rgba(86, 156, 214, 0.15);

  /* ===== 边框 ===== */
  --border-subtle:   rgba(255, 255, 255, 0.06);
  --border-default:  rgba(255, 255, 255, 0.08);
  --border-visible:  rgba(255, 255, 255, 0.10);
  --border-emphasis: rgba(255, 255, 255, 0.15);

  /* ===== 间距 ===== */
  --space-0:  0px;
  --space-1:  2px;
  --space-2:  4px;
  --space-3:  6px;
  --space-4:  8px;
  --space-5:  12px;
  --space-6:  16px;
  --space-7:  20px;
  --space-8:  24px;

  /* ===== 字体 ===== */
  --font-ui:    -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
                "Helvetica Neue", Arial, "Noto Sans", "PingFang SC",
                "Microsoft YaHei", sans-serif;
  --font-mono:  "Cascadia Code", "JetBrains Mono", "Fira Code",
                "SF Mono", "Consolas", "Monaco", "Courier New",
                "Source Code Pro", monospace;

  /* ===== 字号 ===== */
  --text-xs:     11px;   /* 状态栏、徽章 */
  --text-sm:     12px;   /* 文件树、标签页标题 */
  --text-base:   13px;   /* UI 正文 */
  --text-code:   14px;   /* 代码编辑器 */
  --text-lg:     16px;   /* 面板标题 */

  /* ===== 行高 ===== */
  --leading-tight:   1.3;
  --leading-compact: 1.4;
  --leading-normal:  1.5;
  --leading-code:    1.6;

  /* ===== 圆角 ===== */
  --radius-sm:     3px;
  --radius-md:     4px;
  --radius-lg:     6px;
  --radius-xl:     8px;

  /* ===== 阴影 ===== */
  --shadow-dropdown:  0 4px 12px rgba(0, 0, 0, 0.4);
  --shadow-modal:     0 8px 32px rgba(0, 0, 0, 0.5);
  --shadow-tooltip:   0 2px 8px  rgba(0, 0, 0, 0.3);

  /* ===== 组件尺寸 ===== */
  --activitybar-width:    48px;
  --sidebar-width:        280px;
  --statusbar-height:     22px;
  --tab-height:           36px;
  --menuitem-height:      28px;
  --filetree-item-height: 22px;

  /* ===== 过渡 ===== */
  --transition-fast:   100ms ease;
  --transition-normal: 150ms ease;
  --transition-slow:   300ms ease;

  /* ===== 滚动条 ===== */
  --scrollbar-width:   10px;
  --scrollbar-thumb:   rgba(255, 255, 255, 0.12);
  --scrollbar-thumb-hover: rgba(255, 255, 255, 0.25);

  /* ===== 选择 ===== */
  --selection-bg:   #264f78;
  --selection-text: #ffffff;

  /* ===== 聚焦 ===== */
  --focus-ring: 1px solid var(--accent-blue);

  /* ===== 终端特化 ===== */
  --terminal-black:        #0c0c0c;
  --terminal-red:          #c50f1f;
  --terminal-green:        #13a10e;
  --terminal-yellow:       #c19c00;
  --terminal-blue:         #0037da;
  --terminal-magenta:      #881798;
  --terminal-cyan:         #3a96dd;
  --terminal-white:        #cccccc;
  --terminal-bright-black: #767676;
  --terminal-bright-red:   #e74856;
  --terminal-bright-green: #16c60c;
  --terminal-bright-yellow:#f9f1a5;
  --terminal-bright-blue:  #3b78ff;
  --terminal-bright-magenta:#b4009e;
  --terminal-bright-cyan:  #61d6d6;
  --terminal-bright-white: #f2f2f2;
}
```

### 12.2 GitHub Dark 风格变量覆盖

适用于偏好更蓝黑、更"web 化"的暗色风格：

```css
.theme-github-dark {
  --bg-root:        #0d1117;
  --bg-sidebar:     #0d1117;
  --bg-content:     #0d1117;
  --bg-card:        #161b22;
  --bg-dropdown:    #161b22;
  --bg-hover:       #1c2128;
  --bg-active:      #1f2428;
  --bg-input:       #0d1117;
  --bg-terminal:    #0d1117;

  --border-subtle:  #21262d;
  --border-default: #30363d;
  --border-visible: #484f58;

  --accent-blue:    #58a6ff;
  --accent-green:   #3fb950;
  --accent-red:     #f85149;
  --accent-yellow:  #d29922;
  --accent-purple:  #bc8cff;

  --selection-bg:   rgba(88, 166, 255, 0.3);
}
```

### 12.3 Terminal Classic 风格变量覆盖

```css
.theme-terminal-classic {
  --bg-root:        #0c0c0c;
  --bg-sidebar:     #0c0c0c;
  --bg-content:     #0c0c0c;
  --bg-card:        #1a1a1a;
  --bg-dropdown:    #1a1a1a;
  --bg-hover:       #2a2a2a;
  --bg-active:      #333333;
  --bg-input:       #1a1a1a;

  --accent-blue:    #3b78ff;
  --accent-green:   #16c60c;
  --accent-red:     #e74856;
  --accent-yellow:  #f9f1a5;
  --accent-purple:  #b4009e;
  --accent-teal:    #61d6d6;

  --selection-bg:   rgba(59, 120, 255, 0.3);
}
```

### 12.4 使用示例

```css
/* 应用在 body 或 #app 根节点 */
body {
  background: var(--bg-root);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--text-base);
  line-height: var(--leading-normal);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 全局选择 */
::selection {
  background: var(--selection-bg);
  color: var(--selection-text);
}

/* 全局聚焦 */
:focus-visible {
  outline: var(--focus-ring);
  outline-offset: -1px;
}

/* 全局滚动条 */
::-webkit-scrollbar {
  width: var(--scrollbar-width);
  height: var(--scrollbar-width);
}
::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 5px;
  border: 3px solid transparent;
  background-clip: padding-box;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
  border: 2px solid transparent;
}
```

---

## 附录 A: 避坑指南 — 常见反模式

| ❌ 不要 | ✅ 应该 |
|--------|--------|
| 使用纯黑 `#000` 作为背景 | 始终使用有色调偏移的暗色（`#1e1e1e`、`#0d1117`、`#0c0c0c`） |
| 大面积使用超过 `rgba(255,255,255,0.15)` 的亮色边框 | 用 0.06-0.08 alpha 值，面板分隔多用背景色差代替 |
| 文字对比度低于 30%（即 rgba 白色 < 0.3） | 最低可用文字为 50%（占位符），正文必须 > 70% |
| 使用大圆角（8px+）和厚重阴影 | 圆角控制在 3-6px，阴影仅用于弹出层 |
| 按钮、卡片等控件使用亮色填充背景 | 仅在 hover/active 时显示背景变化，"隐形时存在" |
| UI 字体和代码字体混用等宽 | 严格执行双字体体系 |
| 行间距过松（1.8+） | 开发者工具行高 1.3-1.6 |
| 色彩饱和度过高，大面积彩色块 | 色彩克制，强调色只用于关键交互元素 |
| Activity Bar 图标大小不一 | 统一 24x24 图标，居中于 48x48 点击区域 |
| 文件树使用文件夹/文件原生 emoji | 使用 Seti/VSCode 风格的 SVG 图标 |

---

## 附录 B: 参考资源

| 资源 | 链接 |
|------|------|
| VS Code 主题文档 | https://code.visualstudio.com/api/extension-guides/color-theme |
| GitHub Primer Design — Dark Mode | https://primer.style/ |
| macOS Human Interface Guidelines — Dark Mode | https://developer.apple.com/design/human-interface-guidelines/dark-mode |
| Windows Fluent Design — Dark Theme | https://learn.microsoft.com/en-us/windows/apps/design/style/color |
| iTerm2 配色方案库 | https://iterm2colorschemes.com/ |
| JetBrains IntelliJ Darcula Theme | https://jetbrains.design/intellij/ |
| Dracula Theme (暗色经典参考) | https://draculatheme.com/ |
| One Dark Pro (Atom 经典暗色) | https://github.com/Binaryify/OneDark-Pro |
| Tokyo Night (现代暗色主题) | https://github.com/enkia/tokyo-night-vscode-theme |

---

> **Document version**: 1.0
> **Last updated**: 2026-05
> **Language**: 简体中文 + English prompts
