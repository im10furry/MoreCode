# 项目概览对比：Claude Code vs Codex CLI

## Claude Code 实现

### 基本信息

| 维度 | Claude Code |
|------|-------------|
| **开发商** | Anthropic |
| **仓库地址** | [github.com/anthropics/claude-code](https://github.com/anthropics/claude-code) |
| **许可证** | 闭源 |
| **主要语言** | TypeScript（严格模式） |
| **运行时** | Bun |
| **代码规模** | ~50 万行，1884 个 TS 文件 |
| **内置工具** | 40+ |
| **斜杠命令** | 87+ |
| **React Hooks** | 70+ |
| **后台服务** | 13 个子系统 |
| **API 后端** | Anthropic / Bedrock / Vertex / Foundry |

### 技术栈

| 组件 | Claude Code |
|------|-------------|
| **主语言** | TypeScript（Bun 运行时） |
| **UI 框架** | React + Ink（终端渲染） |
| **CLI 解析** | Commander.js |
| **异步运行时** | Bun 内置 |
| **HTTP 客户端** | 内置 fetch |
| **Schema 验证** | Zod v4 |
| **遥测** | OpenTelemetry + GrowthBook + Statsig + Sentry |
| **构建** | Bun bundle（特性标志死代码消除） |
| **代码高亮** | 内置 |
| **MCP 协议** | 自研实现（4 种传输） |
| **终端样式** | Chalk |

---

## Codex CLI 实现

### 基本信息

| 维度 | Codex CLI |
|------|-----------|
| **开发商** | OpenAI |
| **仓库地址** | [github.com/openai/codex](https://github.com/openai/codex) |
| **许可证** | Apache 2.0 开源 |
| **主要语言** | Rust（从 TypeScript 迁移） |
| **运行时** | 原生二进制 |
| **代码规模** | ~8 万行 Rust，60+ Crate |
| **内置工具** | 25+ |
| **斜杠命令** | N/A |
| **React Hooks** | N/A |
| **后台服务** | N/A |
| **API 后端** | OpenAI（支持 Ollama/LM Studio 本地模型） |
| **Stars** | 75,000+ |
| **贡献者** | 421+ |

### 技术栈

| 组件 | Codex CLI |
|------|-----------|
| **主语言** | Rust（Edition 2024） |
| **UI 框架** | Ratatui 0.29 + crossterm 0.28 |
| **CLI 解析** | clap 4 |
| **异步运行时** | Tokio 1 |
| **HTTP 客户端** | reqwest 0.12 |
| **Schema 验证** | serde + serde_json |
| **遥测** | 内置 OpenTelemetry SDK |
| **构建** | Bazel 9（CI）+ Cargo（开发） |
| **代码高亮** | tree-sitter |
| **MCP 协议** | rmcp 0.12 |
| **终端样式** | crossterm |
| **可复现构建** | Nix（flake.nix） |

---

## 对比分析

### 基本信息对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **开发商** | Anthropic | OpenAI |
| **开源状态** | 闭源 | Apache 2.0 开源 |
| **主要语言** | TypeScript | Rust |
| **运行时** | Bun | 原生二进制 |
| **代码规模** | ~50 万行 | ~8 万行 |
| **模块化** | 1884 个 TS 文件 | 60+ Crate |
| **内置工具** | 40+ | 25+ |
| **API 后端** | Anthropic / Bedrock / Vertex / Foundry | OpenAI + 本地模型 |
| **社区规模** | 未知（闭源） | 75K+ Stars, 421+ 贡献者 |

### 技术栈对比

| 组件 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **UI 框架** | React + Ink | Ratatui + crossterm |
| **CLI 解析** | Commander.js | clap 4 |
| **异步运行时** | Bun 内置 | Tokio 1 |
| **Schema 验证** | Zod v4 | serde + serde_json |
| **构建系统** | Bun bundle | Bazel 9 + Cargo |
| **MCP 协议** | 自研实现（4 种传输） | rmcp 0.12 |
| **可复现构建** | 无 | Nix flake |

### 计费模式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **按量计费** | 按 token 计费 | 按 token 计费（API 模式） |
| **Claude Sonnet** | $3/1M input, $15/1M output | N/A |
| **Claude Opus** | $15/1M input, $75/1M output | N/A |
| **订阅模式** | N/A | ChatGPT Plus $20/月 |
| **本地模型** | 不支持 | 支持 Ollama/LM Studio（免费） |
| **企业方案** | Bedrock/Vertex 按云厂商计费 | ChatGPT Enterprise |
| **缓存优惠** | Prompt caching 减少输入成本 | 前缀缓存优化 |

### 安装方式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **npm 安装** | `npm install -g @anthropic-ai/claude-code` | `npm install -g @openai/codex` |
| **Cargo 安装** | 不支持 | `cargo install codex-cli` |
| **预编译二进制** | 不支持 | 支持（GitHub Releases） |
| **依赖要求** | Node.js + Bun | 无（原生二进制） |
| **跨平台** | macOS/Linux/Windows (WSL) | macOS/Linux/Windows |

### 开源状态对比及影响

| 维度 | Claude Code（闭源） | Codex CLI（Apache 2.0 开源） |
|------|---------------------|---------------------------|
| **代码可见性** | 不可查看源码 | 完全透明 |
| **社区贡献** | 不可 | 421+ 贡献者活跃参与 |
| **自定义修改** | 不可能 | 自由 fork 和修改 |
| **安全审计** | 依赖厂商 | 社区 + 独立审计 |
| **学习价值** | 仅通过逆向分析 | 可直接学习架构设计 |
| **迭代速度** | 厂商主导 | 社区驱动 + 厂商支持 |
| **生态扩展** | 受限于官方 API | 可自由扩展和集成 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| 功能极其丰富：40+ 工具、87+ 斜杠命令、70+ Hooks | 闭源，无法审查或修改源码 |
| 多 API 后端支持（Anthropic/Bedrock/Vertex/Foundry） | 依赖 Bun 运行时，部署有额外要求 |
| React/Ink 终端 UI 体验成熟 | TypeScript 运行时性能不如原生二进制 |
| 四层渐进式压缩策略，上下文管理精细 | 代码规模庞大（~50 万行），维护复杂度高 |
| 持久记忆系统（CLAUDE.md + Dream Task） | 不支持本地模型，必须依赖 Anthropic API |
| Prompt caching 显著降低长对话成本 | 计费较高（Opus $15/$75 per 1M tokens） |
| MCP 协议自研实现，支持 4 种传输方式 | 社区生态受限，无法接受外部贡献 |
| 特性标志系统（GrowthBook）灵活控制功能发布 | 无可复现构建支持 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| Apache 2.0 完全开源，社区活跃（75K+ Stars） | 工具数量较少（25+ vs 40+） |
| Rust 原生二进制，性能优异、内存安全 | 从 TypeScript 迁移而来，部分设计仍遗留 TS 痕迹 |
| 支持本地模型（Ollama/LM Studio），可免费使用 | 仅支持 OpenAI API（不含 Bedrock/Vertex 等多云） |
| 60+ Crate 微服务化架构，编译隔离、职责单一 | 60+ Crate 带来较高的架构复杂度 |
| Nix flake 支持可复现构建 | 无持久记忆系统（无 CLAUDE.md 等价物） |
| Bazel 9 CI 构建系统，大型项目工程化成熟 | 压缩策略相对简单（自动压缩 + 截断） |
| 多入口模式（CLI/TUI/Exec）灵活适配不同场景 | 无斜杠命令、无 React Hooks 等高级交互特性 |
| 前缀缓存优化，无状态请求设计 | 上下文管理精度不如 Claude Code 四层策略 |
| WebSocket 双向通信支持，流式中断更优雅 | apply_patch 格式对模型生成质量要求较高 |
| 沙箱安全体系完善（Linux/Windows 平台适配） | UI 框架（Ratatui）功能不如 React/Ink 丰富 |
# 整体架构对比：Claude Code vs Codex CLI

## Claude Code 实现

### 六层分层架构

Claude Code 采用**六层架构**，而非传统的 MVC 模式。这种设计的根本原因是需要管理三种不同生命周期的状态：**进程级**（State/基础设施）、**会话级**（UI/Hooks）、**轮次级**（Query/Services/Tools）。

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CLAUDE CODE CLI                                 │
│                                                                         │
│  ┌──────────┐  ┌────────────┐  ┌──────────┐  ┌──────────────────────┐  │
│  │  main.tsx │─▶│ init()     │─▶│ launchRe │─▶│  <App>               │  │
│  │ (entry)   │  │ (bootstrap)│  │ pl()     │  │   └─ <REPL>          │  │
│  └──────────┘  └────────────┘  └──────────┘  │       └─ PromptInput  │  │
│                                               │       └─ Messages    │  │
│  ┌──────────────────────────────────────────┐ └──────────────────────┘  │
│  │           QueryEngine (per session)       │                          │
│  │  ┌──────────────────────────────────┐     │                          │
│  │  │  query() — async generator loop  │     │                          │
│  │  │  ┌────────────────────────────┐  │     │                          │
│  │  │  │ queryModelWithStreaming()  │  │     │                          │
│  │  │  │  ├─ Build system prompt    │  │     │                          │
│  │  │  │  ├─ Normalize messages     │  │     │                          │
│  │  │  │  ├─ Stream API response    │  │     │                          │
│  │  │  │  └─ Yield events           │  │     │                          │
│  │  │  └────────────────────────────┘  │     │                          │
│  │  │  ┌────────────────────────────┐  │     │                          │
│  │  │  │ runTools() orchestration   │  │     │                          │
│  │  │  │  ├─ Permission check       │  │     │                          │
│  │  │  │  ├─ Hook execution         │  │     │                          │
│  │  │  │  ├─ Concurrent/serial exec │  │     │                          │
│  │  │  │  └─ Yield tool results     │  │     │                          │
│  │  │  └────────────────────────────┘  │     │                          │
│  │  │  ┌────────────────────────────┐  │     │                          │
│  │  │  │ Compaction (auto/micro)    │  │     │                          │
│  │  │  │  ├─ Token budget tracking  │  │     │                          │
│  │  │  │  ├─ Auto-compact trigger   │  │     │                          │
│  │  │  └─ Message summarization  │  │     │                          │
│  │  └──────────────────────────────────┘     │                          │
│  └───────────────────────────────────────────┘                          │
│                                                                         │
│  ┌──────────────┐ ┌────────────┐ ┌───────────┐ ┌────────────────────┐  │
│  │ Tool Registry │ │ Permission │ │ Hook      │ │ MCP Clients        │  │
│  │ (40+ tools)   │ │ Engine     │ │ Engine    │ │ (stdio/sse/ws/local) │  │
│  └──────────────┘ └────────────┘ └───────────┘ └────────────────────┘  │
│                                                                         │
│  ┌──────────────┐ ┌────────────┐ ┌───────────┐ ┌────────────────────┐  │
│  │ Skill Loader  │ │ Plugin Mgr │ │ Analytics │ │ State Store        │  │
│  │ (fs/bundled/  │ │ (builtin/  │ │ (OTel +   │ │ (AppState +        │  │
│  │  mcp/managed) │ │  market)   │ │  1P logs) │ │  React contexts)   │  │
│  └──────────────┘ └────────────┘ └───────────┘ └────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 六层职责说明

| 层级 | 位置 | 职责 | 生命周期 |
|------|------|------|----------|
| **UI 层** | `src/components/`, `src/screens/` | 纯渲染层，React/Ink 组件，声明式终端 UI | 会话级 |
| **Hooks 层** | `src/hooks/` (70+ hooks) | 封装副作用和状态逻辑，可被多个 UI 组件复用 | 会话级 |
| **State 层** | `src/state/`, `src/bootstrap/state.ts` | 进程级生命周期状态（全局单例 + Zustand store） | 进程级 |
| **Query 层** | `src/query.ts`, `src/QueryEngine.ts` | 单轮对话的瞬态状态管理，async generator 核心循环 | 轮次级 |
| **Services 层** | `src/services/` (13 个子系统) | 无状态能力提供者（API 客户端、压缩算法、MCP 协议等） | 进程级 |
| **Tools 层** | `src/tools/` (40+ 工具) | 具有身份标识的执行单元（名称、描述、权限要求） | 轮次级 |

### 层间数据流

六层之间的数据流遵循严格的单向依赖原则，上层可以调用下层，但下层不能直接回调上层（通过事件/yield 机制向上传递）：

```
┌─────────────────────────────────────────────────────────────────┐
│                        数据流方向                                │
│                                                                 │
│  UI 层 ──────▶ Hooks 层 ──────▶ State 层                       │
│    ▲              │                │                            │
│    │              │                ▼                            │
│    │              │           Query 层 ──────▶ Services 层      │
│    │              │                │                │            │
│    │              │                ▼                ▼            │
│    │              │           Tools 层 ◀────────────────        │
│    │              │                │                             │
│    │              │                ▼                             │
│    └──────────────┴──── yield StreamEvent ─────────┘            │
│                                                                 │
│  关键数据流路径：                                                 │
│  1. 用户输入 → UI → Query → API (Services) → StreamEvent → UI  │
│  2. 工具调用 → Query → Permission (Services) → Tools → Result  │
│  3. 状态变更 → State → React Context → UI 重渲染                │
│  4. 压缩触发 → Query → Compact (Services) → 消息截断 → Query   │
└─────────────────────────────────────────────────────────────────┘
```

**核心数据流路径详解：**

1. **用户输入流**：用户在 `PromptInput` 组件输入文本 -> `useSendMessage` Hook 处理 -> 调用 `QueryEngine.sendMessage()` -> 进入 `query()` async generator -> 调用 `queryModelWithStreaming()` (Services 层 API 客户端) -> yield `StreamEvent` -> UI 层通过 `useQueryEvents` Hook 消费事件并渲染
2. **工具执行流**：模型返回 `tool_use` block -> `StreamingToolExecutor` 收集完整参数 -> `runTools()` 编排 -> 权限检查 (Services 层) -> 工具执行 (Tools 层) -> 结果 yield 回 Query 层 -> 追加到 messages 数组
3. **状态传播流**：Bootstrap 初始化 `AppStateStore` -> React Context Provider 注入 -> 各 UI 组件通过 `useStore()` 消费 -> 状态变更触发 React 重渲染
4. **压缩数据流**：每轮 API 调用后检查 token 预算 -> 超阈值触发 `microCompact()` 或 `autoCompact()` -> 修改 messages 数组（原地截断/摘要替换）-> 下一轮 API 调用使用压缩后的上下文

### 状态生命周期管理

Claude Code 的状态管理核心挑战在于三种不同粒度的生命周期需要协调运作：

```
┌─────────────────────────────────────────────────────────────────┐
│                     状态生命周期分层                              │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  进程级状态 (Process-Lifetime)                           │   │
│  │  生命周期：从 main.tsx 启动到进程退出                     │   │
│  │  存储位置：bootstrap/state.ts (全局单例)                  │   │
│  │  包含内容：                                               │   │
│  │    • OAuth token、API 密钥                               │   │
│  │    • GrowthBook 特性标志实例                             │   │
│  │    • 全局设置 (settings.json)                            │   │
│  │    • OpenTelemetry 导出器                                │   │
│  │    • MCP 客户端连接池                                    │   │
│  │    • 文件系统缓存 (LRU, 100文件/25MB)                    │   │
│  │  特点：跨会话持久化，进程重启后需重新初始化                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  会话级状态 (Session-Lifetime)                           │   │
│  │  生命周期：从 /clear 或启动到 /clear 或退出               │   │
│  │  存储位置：React Context + Zustand store                  │   │
│  │  包含内容：                                               │   │
│  │    • 对话消息历史 (messages[])                           │   │
│  │    • 工具注册表 (当前可用工具)                            │   │
│  │    • 权限模式 (default/auto/bypass/plan)                 │   │
│  │    • 成本追踪器 (costTracker)                            │   │
│  │    • UI 状态 (输入焦点、通知、模态框)                     │   │
│  │    • 会话元数据 (session ID, 启动时间)                   │   │
│  │  特点：/clear 时重置，可持久化到磁盘恢复                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  轮次级状态 (Turn-Lifetime)                              │   │
│  │  生命周期：单次 query() 调用（用户发送到 Claude 完成响应） │   │
│  │  存储位置：query() async generator 闭包变量               │   │
│  │  包含内容：                                               │   │
│  │    • 当前轮次的 AbortController                          │   │
│  │    • StreamingToolExecutor 实例                          │   │
│  │    • 流式响应累积缓冲区                                  │   │
│  │    • 工具结果收集数组                                    │   │
│  │    • max-output-tokens 重试计数器                        │   │
│  │    • 临时文件句柄                                        │   │
│  │  特点：query() 返回后即被 GC 回收，不跨轮次保留          │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 为什么选择六层而非传统 MVC

| 传统 MVC 假设 | Claude Code 实际情况 | 六层架构的应对 |
|---------------|---------------------|---------------|
| 无状态请求 | 长对话需要维护完整消息历史 | **Query 层**：有状态的 async generator，持有跨轮次的消息数组 |
| 同步请求-响应 | 流式 SSE 响应 + 并行工具执行 | **Services 层**：流式 API 客户端 + StreamingToolExecutor |
| 单一数据模型 | 三种生命周期的状态混合 | **State 层**：分离进程级/会话级/轮次级状态 |
| 服务端渲染 | 终端实时 UI 更新 | **UI 层 + Hooks 层**：React/Ink 声明式渲染，yield 事件驱动更新 |
| 固定功能集 | 40+ 工具可动态加载/卸载 | **Tools 层**：具有身份标识的执行单元，支持延迟加载 |
| 单一入口 | CLI/SDK/MCP 服务器/IDE Bridge 多入口 | **分层解耦**：Query 层可被多种入口复用 |

**核心设计原则**：
1. **关注点分离**：每层只关心自己的职责，UI 不需要知道工具如何执行，Tools 不需要知道消息如何渲染
2. **生命周期隔离**：不同层的状态有不同的生命周期，避免"僵尸状态"泄漏
3. **可测试性**：Services 层无状态，可以独立单元测试；Tools 层有明确接口，可以 mock
4. **可扩展性**：新工具只需实现 Tool 接口；新 Hook 只需注册事件回调；新 MCP 服务器只需配置

---

## Codex CLI 实现

### Cargo Workspace 微服务化架构

Codex CLI 采用 **Cargo Workspace** 组织，包含 **60+ 个 Crate**，实现了高粒度的模块化。每个 Crate 职责单一，通过 workspace 依赖管理。

```
┌─────────────────────────────────────────────────────────────────┐
│                     Codex CLI (Rust)                             │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    入口层 (Entry Points)                  │   │
│  │  ┌─────────┐    ┌─────────┐    ┌─────────┐              │   │
│  │  │   cli   │    │   tui   │    │   exec  │              │   │
│  │  │ 多工具  │    │ 全屏 UI │    │ 无头    │              │   │
│  │  │ 入口    │    │ Ratatui │    │ 模式    │              │   │
│  │  └────┬────┘    └────┬────┘    └────┬────┘              │   │
│  └───────┼──────────────┼──────────────┼───────────────────┘   │
│          │              │              │                        │
│          └──────────────┼──────────────┘                        │
│                         ▼                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    核心层 (Core)                          │   │
│  │  ┌──────────────────────────────────────────────────┐   │   │
│  │  │  codex.rs — 主结构体 + 事件循环                   │   │   │
│  │  │  agent/ — Agent 循环核心逻辑                      │   │   │
│  │  │  tools/ — 内置工具实现                            │   │   │
│  │  │  sandboxing/ — 沙箱策略                           │   │   │
│  │  │  context_manager/ — 上下文管理                    │   │   │
│  │  │  guardian/ — 安全守护                             │   │   │
│  │  │  client.rs — API 客户端                           │   │   │
│  │  └──────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    基础设施层 (Infrastructure)             │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │   │
│  │  │sandboxing│ │  mcp-*   │ │  state   │ │  config   │  │   │
│  │  │linux/win │ │server/   │ │persist   │ │  loader   │  │   │
│  │  │          │ │client/   │ │          │ │           │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────────┘  │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │   │
│  │  │ codex-   │ │ models-  │ │  login   │ │ analytics │  │   │
│  │  │ client   │ │  manager  │ │  auth    │ │  otel     │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Workspace 组织策略

Codex CLI 的 Cargo Workspace 采用**分层组织策略**，将 60+ 个 Crate 按职责划分为清晰的层级：

| 层级 | Crate 数量 | 说明 |
|------|-----------|------|
| **入口层** | 3 | `cli`、`tui`、`exec` — 提供不同的用户交互模式 |
| **核心层** | 5 | `core`、`tools`、`protocol`、`codex-api`、`codex-client` — 业务逻辑主体 |
| **沙箱层** | 6 | `sandboxing`、`linux-sandbox`、`windows-sandbox-rs`、`process-hardening`、`execpolicy`、`shell-escalation` |
| **MCP 层** | 3 | `mcp-server`、`rmcp-client`、`mcp-types` |
| **模型层** | 5 | `codex-client`、`codex-api`、`chatgpt`、`lmstudio`、`ollama`、`models-manager` |
| **状态层** | 3 | `rollout`、`codex-state`、`state-db` |
| **配置层** | 2 | `codex-config`、`codex-features` |
| **认证层** | 2 | `codex-login`、`codex-keyring-store` |
| **遥测层** | 2 | `codex-analytics`、`codex-otel` |
| **工具层** | 3 | `codex-tools`、`codex-apply-patch`、`codex-shell-command` |
| **网络层** | 2 | `codex-network-proxy`、`codex-exec-server` |
| **其他** | ~20 | `codex-utils-*`、`codex-git-utils`、`codex-terminal-detection` 等 |

### Crate 依赖关系

```
                    ┌─────────┐
                    │   cli   │
                    └────┬────┘
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
         ┌────────┐ ┌────────┐ ┌────────┐
         │  tui   │ │  exec  │ │  core  │
         └───┬────┘ └───┬────┘ └───┬────┘
             │          │          │
             └──────────┼──────────┘
                        ▼
              ┌──────────────────┐
              │    protocol      │  ◄── 类型定义、Op/Event 枚举
              └────────┬─────────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐
   │ codex-   │  │  tools   │  │sandboxing│
   │  api     │  │          │  │          │
   └────┬─────┘  └────┬─────┘  └────┬─────┘
        │             │             │
        ▼             ▼             ▼
   ┌──────────┐  ┌──────────┐  ┌──────────────┐
   │codex-    │  │apply-    │  │linux-sandbox │
   │client    │  │patch     │  │windows-      │
   └──────────┘  └──────────┘  │sandbox-rs    │
                               └──────────────┘
```

**依赖规则**：
- `protocol` 是最底层的类型定义 crate，几乎所有 crate 都依赖它
- `core` 依赖 `protocol`、`codex-api`、`tools`、`sandboxing`
- `cli`/`tui`/`exec` 仅依赖 `core` 和少量基础设施 crate
- 沙箱 crate 之间互不依赖，通过 `sandboxing` 抽象层桥接

### 为什么选择 60+ Crate 微服务化

选择如此高粒度的 Crate 拆分有以下几个关键原因：

1. **编译隔离**：修改 `apply-patch` 的解析器不需要重新编译 `core`，大幅缩短开发迭代时间。Rust 编译器以 crate 为增量编译单元，60+ crate 意味着 60+ 个并行编译任务。

2. **职责单一**：每个 crate 有明确的职责边界。例如 `codex-network-proxy` 只处理网络代理，`codex-keyring-store` 只处理密钥存储，便于独立测试和维护。

3. **条件编译**：平台特定代码可以独立为 crate，通过 `#[cfg(target_os)]` 控制编译。`linux-sandbox` 仅在 Linux 上编译，`windows-sandbox-rs` 仅在 Windows 上编译。

4. **依赖最小化**：`protocol` crate 不依赖任何网络库，可以安全地在轻量级上下文中使用。`apply-patch` 可以作为独立可执行文件运行（`standalone_executable`）。

5. **渐进式迁移**：从 TypeScript 迁移到 Rust 时，高粒度 crate 允许逐个模块迁移，降低大规模重构风险。

6. **安全审计**：安全关键代码（沙箱、认证）隔离在独立 crate 中，便于安全审计和代码审查。

---

## 对比分析

### 架构模式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **架构风格** | 六层分层架构（单体） | Cargo Workspace 微服务化（60+ Crate） |
| **组织单位** | 目录/文件（1884 个 TS 文件） | Crate（60+ 独立编译单元） |
| **模块化粒度** | 中等（按功能目录划分） | 极高（每个 crate 职责单一） |
| **编译隔离** | 无（Bun bundle 整体打包） | 有（crate 级增量编译） |
| **依赖管理** | npm package.json | Cargo.toml workspace |

### 技术选型对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **语言** | TypeScript（严格模式） | Rust（Edition 2024） |
| **运行时** | Bun（JS 运行时） | 原生二进制（无运行时依赖） |
| **UI 框架** | React + Ink（声明式） | Ratatui + crossterm（命令式） |
| **异步模型** | Bun 内置 async/await | Tokio 1 async runtime |
| **构建系统** | Bun bundle | Bazel 9（CI）+ Cargo（开发） |
| **可复现构建** | 无 | Nix flake.nix |

### 状态管理对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **状态分层** | 三层（进程级/会话级/轮次级） | 两层（Session/ActiveTurn） |
| **进程级状态** | 全局单例 + Zustand store | Arc\<Session\> + watch channel |
| **会话级状态** | React Context + Zustand | Arc\<RwLock\<SessionState\>\> |
| **轮次级状态** | async generator 闭包 | TurnContext + CancellationToken |
| **状态传播** | yield StreamEvent + React Context | Event Queue (rx_event) |

### 入口模式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **入口数量** | 单一入口（main.tsx） | 多入口（cli/tui/exec） |
| **交互模式** | REPL 交互式 | CLI/TUI/无头 三种模式 |
| **多入口支持** | 通过 SDK/MCP 桥接 | 原生多二进制入口 |

### 架构哲学差异

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **设计哲学** | 功能优先，单体集中 | 模块优先，微服务解耦 |
| **复杂度管理** | 层级抽象（六层） | crate 隔离（60+） |
| **扩展方式** | 实现 Tool 接口 + Hook 注册 | 新建 crate + 实现 trait |
| **平台适配** | Bun 运行时抽象 | 条件编译 `#[cfg(target_os)]` |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| 六层架构清晰分离关注点，职责边界明确 | 单体架构，修改任一层可能影响整体 |
| 三种状态生命周期精细管理，避免状态泄漏 | 50 万行代码规模庞大，理解成本高 |
| React/Ink 声明式 UI 开发效率高 | 依赖 Bun 运行时，部署和分发受限 |
| async generator 流式模型优雅，事件驱动 | 无编译隔离，任意文件修改需全量重构建 |
| 丰富的 Hooks 层（70+）实现逻辑复用 | 闭源，无法审查内部架构决策 |
| 功能丰富（40+ 工具、87+ 命令、13 子系统） | 单一入口模式，无头执行需额外适配 |
| GrowthBook 特性标志灵活控制功能发布 | 无可复现构建，环境差异可能导致问题 |
| MCP 协议自研实现支持 4 种传输 | TypeScript 运行时性能不如 Rust 原生 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| 60+ Crate 微服务化，编译隔离极致 | 架构复杂度高，新贡献者学习曲线陡峭 |
| Rust 原生二进制，零运行时依赖 | Ratatui 命令式 UI 开发效率低于 React/Ink |
| 增量编译大幅缩短开发迭代时间 | crate 间接口维护成本高 |
| 条件编译优雅处理平台差异 | 60+ crate 的依赖关系管理复杂 |
| 多入口模式（CLI/TUI/Exec）灵活适配 | 功能相对精简（25+ 工具，无斜杠命令） |
| Nix flake 支持可复现构建 | 从 TypeScript 迁移，部分设计仍有 TS 痕迹 |
| 安全关键代码隔离在独立 crate，便于审计 | 无 Hooks 层等价物，逻辑复用机制较弱 |
| apply-patch 可独立运行为可执行文件 | protocol crate 作为底层依赖，变更影响面广 |
| Bazel 9 CI 构建系统成熟 | 无特性标志系统（无 GrowthBook 等价物） |
# Agent 循环对比：Claude Code vs Codex CLI

## Claude Code 实现

### query() async generator 核心结构

`query()` 是 Claude Code 的核心函数，以 async generator 的形式实现了一个完整的 ReAct（Reasoning + Acting）循环。它的设计精妙之处在于：通过 `yield` 将流式事件实时推送给 UI，同时保持内部循环状态。

```typescript
// query.ts -- query() 完整伪代码
export async function* query(
  params: QueryParams,
): AsyncGenerator<StreamEvent | Message, void, undefined> {
  const {
    messages,           // 消息历史（可变引用）
    tools,              // 可用工具列表
    systemPrompt,       // 系统提示词
    abortController,    // 中止控制器
    model,              // 模型名称
    maxTokens,          // 最大输出 token 数
    microCompact,       // 微压缩函数
    autoCompact,        // 自动压缩函数
    callModel,          // API 调用函数（依赖注入）
  } = params;

  let maxOutputTokensRetries = 0;
  const MAX_OUTPUT_TOKENS_RETRIES = 3;

  // ═══════════════════════════════════════════════════════════════
  // 主循环 -- 持续运行直到模型决定结束或用户中止
  // ═══════════════════════════════════════════════════════════════
  while (true) {
    // ─── 步骤 1: 检查中止信号 ─────────────────────────────────
    if (abortController.signal.aborted) {
      yield { type: 'aborted' };
      return;
    }

    // ─── 步骤 2: 微压缩 ──────────────────────────────────────
    // 每次迭代前检查是否需要微压缩
    // 微压缩是轻量级的：仅清除旧的工具结果，不调用 LLM
    if (microCompact) {
      const compacted = microCompact(messages);
      if (compacted) {
        yield { type: 'micro_compact', clearedCount: compacted.clearedCount };
      }
    }

    // ─── 步骤 3: 调用 LLM API（流式） ─────────────────────────
    // yield* 将子 generator 的事件直接传递给调用者
    const response = yield* queryModelWithStreaming({
      model,
      maxTokens,
      messages,
      tools: tools.map(t => ({
        name: t.name,
        description: t.description,
        input_schema: t.inputJSONSchema,
      })),
      systemPrompt,
      abortController,
    });

    // ─── 步骤 4: 处理响应 ─────────────────────────────────────
    const { message, stop_reason } = response;

    // 将助手消息追加到历史
    messages.push(message);

    // ─── 步骤 5: 检查终止条件 ─────────────────────────────────
    if (stop_reason === 'end_turn') {
      // 模型认为任务完成
      yield { type: 'end_turn', message };
      return;
    }

    if (stop_reason === 'max_tokens') {
      // 模型输出被截断，需要恢复
      maxOutputTokensRetries++;
      if (maxOutputTokensRetries >= MAX_OUTPUT_TOKENS_RETRIES) {
        yield { type: 'max_tokens_exceeded', message };
        return;
      }
      // 继续循环，让模型从截断处继续
      yield { type: 'max_tokens_retry', attempt: maxOutputTokensRetries };
      continue;
    }

    // ─── 步骤 6: 收集工具调用 ─────────────────────────────────
    const toolUseBlocks = message.content.filter(
      (block): block is ToolUseBlock => block.type === 'tool_use'
    );

    if (toolUseBlocks.length === 0) {
      // 没有工具调用且不是 end_turn，异常终止
      yield { type: 'unexpected_stop', stop_reason, message };
      return;
    }

    // ─── 步骤 7: 执行工具 ─────────────────────────────────────
    // yield* 将工具执行的事件也传递给调用者
    const toolResults = yield* runTools({
      toolUseBlocks,
      tools,
      messages,
      abortController,
    });

    // ─── 步骤 8: 将工具结果追加到消息历史 ─────────────────────
    messages.push({
      role: 'user',
      content: toolResults.map(result => ({
        type: 'tool_result' as const,
        tool_use_id: result.toolUseId,
        content: result.content,
        is_error: result.isError,
      })),
    });

    // ─── 步骤 9: 检查自动压缩 ─────────────────────────────────
    // 自动压缩是重量级的：调用 LLM 生成摘要
    if (autoCompact && shouldAutoCompact(messages)) {
      const compacted = await autoCompact(messages);
      if (compacted) {
        messages.length = 0;
        messages.push(...compacted);
        yield { type: 'auto_compact', newMessageCount: compacted.length };
      }
    }

    // ─── 继续循环 ─────────────────────────────────────────────
    // 工具结果已追加，下一轮迭代将让模型看到工具执行结果
  }
}
```

**关键设计决策：**

1. **`yield*` 委托**：`queryModelWithStreaming()` 和 `runTools()` 都是 async generator，使用 `yield*` 将它们的事件直接传递给外层消费者，避免事件缓冲延迟
2. **原地修改 messages**：`messages` 数组是通过引用传递的，`query()` 直接修改它，调用者（QueryEngine）持有的引用也会看到变化
3. **微压缩在循环头部**：确保每次 API 调用前上下文尽可能精简
4. **自动压缩在循环尾部**：在工具结果追加后检查，因为工具结果可能显著增加上下文大小
5. **max_tokens 恢复**：最多重试 3 次，避免无限循环

### queryModelWithStreaming() 实现

该函数负责构建 API 请求、处理流式响应、并 yield 事件：

```typescript
// services/api/claude.ts -- queryModelWithStreaming() 完整实现
async function* queryModelWithStreaming(
  params: StreamParams,
): AsyncGenerator<StreamEvent, ApiResponse, undefined> {
  const {
    model, maxTokens, messages, tools,
    systemPrompt, abortController,
  } = params;

  // ─── 构建 API 请求体 ──────────────────────────────────────
  const request: CreateMessageRequest = {
    model,
    max_tokens: maxTokens,
    messages: normalizeMessages(messages),
    system: buildSystemPrompt(systemPrompt),
    tools: tools.map(tool => ({
      name: tool.name,
      description: tool.description,
      input_schema: tool.input_schema,
      cache_control: { type: 'ephemeral' },
    })),
    thinking: model.includes('claude-3.7') ? {
      type: 'enabled',
      budget_tokens: Math.floor(maxTokens * 0.8),
    } : undefined,
    metadata: { user_id: getUserId() },
  };

  // ─── 系统提示词缓存优化 ───────────────────────────────────
  if (request.system && Array.isArray(request.system)) {
    request.system.forEach((block, index) => {
      if (block.type === 'text') {
        if (index === 0) {
          block.cache_control = { type: 'ephemeral' };
        }
      }
    });
  }

  // ─── 调用 Anthropic SDK stream API ─────────────────────────
  let stream: AnthropicStream;
  try {
    stream = await anthropic.messages.stream(request, {
      signal: abortController.signal,
      headers: { 'anthropic-beta': 'interleaved-thinking-2025-05-14' },
    });
  } catch (error) {
    yield { type: 'api_error', error };
    throw error;
  }

  // ─── 处理流式响应 ─────────────────────────────────────────
  const contentBlocks: ContentBlock[] = [];
  let currentBlock: Partial<ContentBlock> | null = null;
  let stopReason: StopReason | null = null;

  try {
    for await (const event of stream) {
      if (abortController.signal.aborted) {
        stream.abort();
        yield { type: 'aborted' };
        return;
      }

      switch (event.type) {
        case 'message_start':
          yield { type: 'message_start', message: event.message,
                  usage: event.usage };
          break;

        case 'content_block_start':
          currentBlock = {
            type: event.content_block.type,
            id: event.content_block.id,
          };
          if (event.content_block.type === 'tool_use') {
            currentBlock.name = event.content_block.name;
            currentBlock.input = {};
            yield { type: 'tool_use_start',
                    id: event.content_block.id,
                    name: event.content_block.name };
          } else if (event.content_block.type === 'thinking') {
            yield { type: 'thinking_start',
                    id: event.content_block.id };
          } else if (event.content_block.type === 'text') {
            yield { type: 'text_start',
                    id: event.content_block.id };
          }
          break;

        case 'content_block_delta':
          if (event.delta.type === 'text_delta') {
            yield { type: 'text_delta', delta: event.delta.text,
                    id: currentBlock?.id };
            if (currentBlock) {
              currentBlock.text = (currentBlock.text || '') + event.delta.text;
            }
          } else if (event.delta.type === 'thinking_delta') {
            yield { type: 'thinking_delta', delta: event.delta.thinking,
                    id: currentBlock?.id };
          } else if (event.delta.type === 'input_json_delta') {
            if (currentBlock) {
              currentBlock.inputJson =
                (currentBlock.inputJson || '') + event.delta.partial_json;
            }
            yield { type: 'tool_input_delta', id: currentBlock?.id,
                    delta: event.delta.partial_json };
          }
          break;

        case 'content_block_stop':
          if (currentBlock) {
            if (currentBlock.type === 'tool_use' && currentBlock.inputJson) {
              try {
                currentBlock.input = JSON.parse(currentBlock.inputJson);
              } catch {
                currentBlock.input = {};
                currentBlock.parseError = true;
              }
            }
            contentBlocks.push(currentBlock as ContentBlock);
            currentBlock = null;
          }
          break;

        case 'message_delta':
          stopReason = event.delta.stop_reason;
          yield { type: 'message_delta', delta: event.delta,
                  usage: event.usage };
          break;

        case 'message_stop':
          yield { type: 'message_stop' };
          break;
      }
    }
  } catch (error) {
    if (abortController.signal.aborted) {
      yield { type: 'aborted' };
      return;
    }
    yield { type: 'stream_error', error };
    throw error;
  }

  return {
    message: { role: 'assistant', content: contentBlocks },
    stop_reason: stopReason!,
    usage: stream.finalMessage()?.usage,
  };
}
```

### StreamingToolExecutor 并行执行机制

`StreamingToolExecutor`（~530行）是 Claude Code 中最精巧的组件之一。它实现了一个**事件驱动的状态机**，能够在模型仍在生成响应时就开始执行已完成的工具调用，显著减少端到端延迟。

#### 设计原理

```
传统模式：
  模型生成 [tool1参数...] [tool2参数...] [tool3参数...] -> 全部完成 -> 执行tool1 -> 执行tool2 -> 执行tool3

StreamingToolExecutor 模式：
  模型生成 [tool1参数完成] -> 立即执行tool1
           [tool2参数完成] -> 立即执行tool2（与tool1并行）
           [tool3参数完成] -> 等待tool1/tool2完成后执行tool3（如果不安全并行）
```

#### 核心数据结构

```typescript
// StreamingToolExecutor.ts -- 核心数据结构

// 正在接收参数的工具（参数尚未完整）
interface PendingBlock {
  id: string;              // content_block id
  name: string;            // 工具名称
  inputJson: string;       // 已接收的 JSON 字符串片段
  startedAt: number;       // 开始时间戳
}

// 正在运行的工具任务
interface RunningTool {
  id: string;              // content_block id
  name: string;            // 工具名称
  input: unknown;          // 已解析的完整输入
  promise: Promise<ToolResult>; // 执行 Promise
  abortController: AbortController; // 独立的中止控制器
  startedAt: number;       // 开始执行时间戳
}

// 执行结果（带顺序信息）
interface ToolResultWithOrder {
  id: string;
  name: string;
  result: ToolResult;
  order: number;           // 原始出现顺序（保证确定性）
}

export class StreamingToolExecutor {
  // 状态机核心
  private pendingBlocks: Map<string, PendingBlock> = new Map();
  private runningTools: RunningTool[] = [];
  private completedResults: ToolResultWithOrder[] = [];
  private nextOrder: number = 0;

  // 并发控制
  private maxConcurrent: number = 10;
  private siblingAbortController: AbortController;

  // 事件回调
  private onToolStart?: (id: string, name: string) => void;
  private onToolComplete?: (id: string, name: string, result: ToolResult) => void;
  private onToolError?: (id: string, name: string, error: Error) => void;
}
```

#### 事件驱动状态机

```typescript
class StreamingToolExecutor {
  onEvent(event: StreamEvent): void {
    switch (event.type) {
      case 'content_block_start':
        if (event.content_block.type === 'tool_use') {
          const block: PendingBlock = {
            id: event.content_block.id,
            name: event.content_block.name,
            inputJson: '',
            startedAt: Date.now(),
          };
          this.pendingBlocks.set(block.id, block);
        }
        break;

      case 'content_block_delta':
        if (event.delta.type === 'input_json_delta') {
          const block = this.pendingBlocks.get(event.id);
          if (block) {
            block.inputJson += event.delta.partial_json;
          }
        }
        break;

      case 'content_block_stop':
        const block = this.pendingBlocks.get(event.id);
        if (block) {
          this.pendingBlocks.delete(event.id);
          let input: unknown;
          try {
            input = JSON.parse(block.inputJson);
          } catch (error) {
            this.completedResults.push({
              id: block.id, name: block.name,
              result: { content: `Error: Invalid JSON input: ${error.message}`,
                        isError: true },
              order: this.nextOrder++,
            });
            break;
          }
          this.scheduleExecution(block.id, block.name, input);
        }
        break;
    }
  }
}
```

#### 并发安全的调度执行

```typescript
class StreamingToolExecutor {
  private async scheduleExecution(
    id: string, name: string, input: unknown,
  ): Promise<void> {
    const tool = this.findTool(name);
    const isSafe = tool?.isConcurrencySafe(input) ?? false;

    // 如果不安全且已有工具在运行，等待它们完成
    if (!isSafe && this.runningTools.length > 0) {
      await Promise.all(this.runningTools.map(t => t.promise));
    }

    // 检查并发上限
    if (this.runningTools.length >= this.maxConcurrent) {
      await Promise.race(this.runningTools.map(t => t.promise));
    }

    const abortController = new AbortController();
    const onSiblingAbort = () => abortController.abort();
    this.siblingAbortController.signal.addEventListener('abort', onSiblingAbort);
    const order = this.nextOrder++;

    const promise = this.executeTool(id, name, input, abortController)
      .then(result => {
        this.completedResults.push({ id, name, result, order });
        this.onToolComplete?.(id, name, result);
        return result;
      })
      .catch(error => {
        if (abortController.signal.aborted) return;
        const errorResult: ToolResult = {
          content: `Error: ${error.message}`, isError: true,
        };
        this.completedResults.push({ id, name, result: errorResult, order });
        this.onToolError?.(id, name, error);
      })
      .finally(() => {
        this.runningTools = this.runningTools.filter(t => t.id !== id);
        this.siblingAbortController.signal.removeEventListener('abort', onSiblingAbort);
      });

    this.runningTools.push({
      id, name, input, promise, abortController, startedAt: Date.now(),
    });
    this.onToolStart?.(id, name);
  }
}
```

#### 结果缓冲与顺序保证

```typescript
class StreamingToolExecutor {
  /**
   * 等待所有工具执行完成，并按原始顺序返回结果
   * 这保证了即使并行执行，结果的顺序也是确定性的
   */
  async collectResults(): Promise<ToolResultWithOrder[]> {
    await Promise.all(this.runningTools.map(t => t.promise));
    return [...this.completedResults].sort((a, b) => a.order - b.order);
  }

  abortAll(): void {
    this.siblingAbortController.abort();
    for (const tool of this.runningTools) {
      tool.abortController.abort();
    }
  }
}
```

**结果缓冲机制的关键设计**：
- `nextOrder` 计数器在 `content_block_start` 时递增，记录工具调用的原始顺序
- 并行执行的工具可能以任意顺序完成，但 `collectResults()` 按原始顺序返回
- 这保证了模型看到的工具结果顺序与它发出调用的顺序一致，避免混淆

### max-output-tokens 恢复机制

```typescript
// 在 query() 的 while-true 循环中：
if (stop_reason === 'max_tokens') {
  maxOutputTokensRetries++;

  if (maxOutputTokensRetries >= MAX_OUTPUT_TOKENS_RETRIES) {
    yield {
      type: 'max_tokens_exceeded',
      message: 'Model output was truncated after 3 retries. ' +
               'Consider breaking your task into smaller steps.',
    };
    return;
  }

  // 构建恢复消息：告诉模型从截断处继续
  messages.push({
    role: 'user',
    content: '[Model output was truncated due to max_tokens limit. ' +
             'Please continue from where you left off.]',
  });

  yield {
    type: 'max_tokens_retry',
    attempt: maxOutputTokensRetries,
  };

  continue; // 下一轮迭代将让模型看到截断提示并继续生成
}
```

**恢复机制的设计考虑**：
- **最多 3 次重试**：避免无限循环
- **用户消息而非系统消息**：使用 `role: 'user'` 提示模型继续
- **透明通知**：通过 yield 事件通知 UI 层显示重试状态
- **不增加 max_tokens**：重试时使用相同的 max_tokens 值

### StreamEvent 类型定义

```typescript
type StopReason = 'end_turn' | 'max_tokens' | 'stop_sequence' | 'tool_use';

type StreamEvent =
  | MessageStartEvent | TextStartEvent | TextDeltaEvent
  | ThinkingStartEvent | ThinkingDeltaEvent
  | ToolUseStartEvent | ToolInputDeltaEvent
  | MessageDeltaEvent | MessageStopEvent
  | ToolExecutionStartEvent | ToolExecutionCompleteEvent
  | ToolExecutionErrorEvent | MicroCompactEvent | AutoCompactEvent
  | AbortedEvent | MaxTokensRetryEvent | MaxTokensExceededEvent
  | ApiErrorEvent | StreamErrorEvent;
```

### QueryEngine 外层循环

QueryEngine 是 `query()` 的外层包装器，提供会话级的管理能力：

```typescript
export class QueryEngine {
  private messages: Message[] = [];
  private fileCache: LRUCache<string, FileContent>;
  private costTracker: CostTracker;
  private abortController: AbortController = new AbortController();
  private permissionState: PermissionState;
  private tokenBudget: TokenBudget;
  private retryState: RetryState = { count: 0, lastError: null, nextRetryAt: 0 };

  async sendMessage(userMessage: string): Promise<void> {
    this.messages.push({ role: 'user', content: userMessage });

    const estimatedTokens = estimateMessageTokens(this.messages);
    if (estimatedTokens > this.tokenBudget.getThreshold('compact')) {
      await this.autoCompact();
    }

    let attempt = 0;
    const maxAttempts = 3;

    while (attempt < maxAttempts) {
      try {
        const queryGenerator = query({
          messages: this.messages,
          tools: this.getAvailableTools(),
          abortController: this.abortController,
          model: this.config.model,
          maxTokens: this.config.maxOutputTokens || 16384,
          systemPrompt: this.buildSystemPrompt(),
          microCompact: this.microCompact.bind(this),
          autoCompact: this.shouldAutoCompact.bind(this),
          callModel: this.deps.callModel,
        });

        for await (const event of queryGenerator) {
          this.handleStreamEvent(event);
        }

        this.retryState.count = 0;
        return;

      } catch (error) {
        const category = classifyError(error);
        switch (category) {
          case 'rate_limit':
            await this.sleep(this.getRetryAfter(error));
            attempt++; continue;
          case 'overloaded':
            await this.sleep(Math.pow(2, attempt) * 1000);
            attempt++; continue;
          case 'context_overflow':
            await this.forceCompact();
            attempt++; continue;
          case 'auth_error':
          case 'forbidden':
            throw error;
          default:
            if (attempt >= maxAttempts - 1) throw error;
            attempt++; continue;
        }
      }
    }
  }
}
```

---

## Codex CLI 实现

### Op/Event 提交-事件模式

Codex CLI 的核心通信架构采用 **SQ/EQ（Submission Queue / Event Queue）模式**，这是整个系统的通信骨干。

```
┌─────────────────────────────────────────────────────────────────┐
│                    SQ/EQ 通信架构                                │
│                                                                  │
│  ┌──────────┐     Submission      ┌──────────────┐              │
│  │  Client   │────Queue (tx_sub)──▶│   Codex      │              │
│  │ (TUI/Exec)│                     │   Agent      │              │
│  │           │◀──Event Queue──────│   Loop       │              │
│  │           │     (rx_event)      │              │              │
│  └──────────┘                      └──────────────┘              │
│       │                                  │                       │
│       │  Op::UserInput                  │ EventMsg              │
│       │  Op::Interrupt                  │ EventMsg::ItemStarted  │
│       │  Op::ExecApproval               │ EventMsg::ItemCompleted│
│       │  Op::Shutdown                   │ EventMsg::TurnAborted  │
│       ▼                                  ▼                       │
│  ┌──────────┐                      ┌──────────────┐              │
│  │ 用户输入   │                      │ 模型响应     │              │
│  │ 审批决策   │                      │ 工具执行结果  │              │
│  │ 中断信号   │                      │ 状态变更通知  │              │
│  └──────────┘                      └──────────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

**核心数据结构 -- Submission：**

```rust
/// Submission Queue Entry - requests from user
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Submission {
    /// Unique id for this Submission to correlate with Events
    pub id: String,
    /// Payload
    pub op: Op,
    /// Optional W3C trace carrier propagated across async submission handoffs.
    pub trace: Option<W3cTraceContext>,
}
```

**核心数据结构 -- Codex 结构体：**

`codex.rs`（~2000 行）定义了整个 Agent 系统的核心结构体：

```rust
pub struct Codex {
    // 通信通道
    tx_sub: Sender<Submission>,      // 提交队列发送端
    rx_event: Receiver<EventMsg>,    // 事件队列接收端

    // 状态管理
    agent_status: Arc<watch::Sender<AgentStatus>>,  // Agent 状态广播
    session: Arc<Session>,           // 会话实例

    // 配置
    config: Config,                  // 全局配置

    // 服务
    services: Arc<SessionServices>,  // 会话级服务集合
}
```

**核心数据结构 -- Session 和 ActiveTurn：**

```rust
pub struct Session {
    pub conversation_id: ThreadId,
    pub state: Arc<RwLock<SessionState>>,
    pub services: Arc<SessionServices>,
}

pub struct ActiveTurn {
    pub turn_id: String,
    pub cancellation_token: CancellationToken,
    pub turn_context: Arc<TurnContext>,
}
```

### 提交循环 (Submission Loop)

提交循环是 Codex 结构体的核心事件循环，持续从 `rx_sub` 接收 Submission 并分发处理。

```
┌──────────────────────────────────────────────────────────────┐
│                    Submission Loop                            │
│                                                               │
│   loop {                                                      │
│       match rx_sub.recv().await {                             │
│           Op::UserInput { items, .. } => {                   │
│               spawn_task(items, turn_context)                 │
│               // 构建上下文 → 调用 API → 流式处理 → 工具执行  │
│           }                                                   │
│                                                               │
│           Op::Interrupt => {                                  │
│               abort_all_tasks()                               │
│               // 取消所有正在运行的任务                         │
│               // 发送 TurnAborted 事件                         │
│           }                                                   │
│                                                               │
│           Op::ExecApproval { approved } => {                  │
│               resolve_pending_approval(approved)              │
│               // 将用户的审批决策传递给等待中的工具执行          │
│           }                                                   │
│                                                               │
│           Op::Shutdown => {                                   │
│               shutdown()                                      │
│               break;                                          │
│           }                                                   │
│       }                                                       │
│   }                                                           │
└──────────────────────────────────────────────────────────────┘
```

**Op 枚举完整定义（核心变体）：**

```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Op {
    Interrupt,
    CleanBackgroundTerminals,
    RealtimeConversationStart(ConversationStartParams),
    RealtimeConversationAudio(ConversationAudioParams),
    RealtimeConversationText(ConversationTextParams),
    RealtimeConversationClose,
    RealtimeConversationListVoices,
    UserInput {
        items: Vec<UserInput>,
        final_output_json_schema: Option<Value>,
        responsesapi_client_metadata: Option<HashMap<String, String>>,
    },
    UserTurn {
        items: Vec<UserInput>,
        cwd: PathBuf,
        approval_policy: AskForApproval,
        approvals_reviewer: Option<ApprovalsReviewer>,
        sandbox_policy: SandboxPolicy,
        model: String,
        effort: Option<ReasoningEffortConfig>,
        summary: Option<ReasoningSummaryConfig>,
        service_tier: Option<Option<ServiceTier>>,
        final_output_json_schema: Option<Value>,
    },
    ExecApproval { id: String, approved: bool },
    Shutdown,
}
```

### Responses API 调用格式

Codex CLI 使用 OpenAI **Responses API**（而非传统的 Chat Completions API），每次请求发送完整的 JSON 负载。

```json
{
  "instructions": "系统指令内容...",
  "tools": [...],
  "input": [...]
}
```

| 字段 | 说明 | 来源 |
|------|------|------|
| `instructions` | 系统指令（base_instructions） | 模型默认指令 + 沙箱权限说明 + 开发者指令 |
| `tools` | 可用工具列表 | 内置工具 + API 提供的工具 + MCP 工具 |
| `input` | 对话输入序列 | 按序插入的多条消息 |

**input 按序插入规则：**

```
input 序列:
├── [0] developer 消息 — 沙箱权限说明
├── [1] developer 消息 — 开发者指令
├── [2] user 消息 — 用户指令
├── [3] user 消息 — 环境上下文
├── [4] user 消息 — 用户实际输入
├── [5+] 历史消息（压缩后的对话历史）
└── [最后] 当前用户输入
```

### SSE 流式处理

Codex CLI 通过 **Server-Sent Events (SSE)** 接收模型的流式响应。

**关键 SSE 事件类型：**

| 事件类型 | 说明 | 处理方式 |
|----------|------|----------|
| `response.reasoning_summary_text.delta` | 推理摘要增量文本 | 累积并显示推理过程 |
| `response.output_item.added` | 输出项添加（工具调用开始） | 创建新的工具调用上下文 |
| `response.output_text.delta` | 输出文本增量 | 实时显示给用户 |
| `response.completed` | 响应完成 | 提取最终结果，判断是否需要继续循环 |
| `response.function_call_arguments.delta` | 函数调用参数增量 | 累积函数调用参数 |
| `response.output_item.done` | 输出项完成 | 触发工具执行 |

**SSE 消费实现（简化）：**

```rust
// codex-api/src/sse/responses.rs 中的核心流处理逻辑
async fn stream_response(
    response: Response,
    tx_event: Sender<EventMsg>,
) -> Result<()> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        buffer.push_str(&chunk?);

        while let Some(pos) = buffer.find("\n\n") {
            let event_text = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            for line in event_text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    let event: ResponseEvent = serde_json::from_str(data)?;
                    match event {
                        ResponseEvent::ReasoningSummaryTextDelta { delta } => {
                            tx_event.send(EventMsg::ReasoningContentDelta(
                                ReasoningContentDeltaEvent { content: delta }
                            )).await?;
                        }
                        ResponseEvent::OutputItemAdded { item } => {
                            tx_event.send(EventMsg::ItemStarted(
                                ItemStartedEvent { item }
                            )).await?;
                        }
                        ResponseEvent::OutputTextDelta { delta } => {
                            tx_event.send(EventMsg::AgentMessageContentDelta(
                                AgentMessageContentDeltaEvent { delta }
                            )).await?;
                        }
                        ResponseEvent::Completed { response } => {
                            // 处理完成事件
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
```

**WebSocket 传输支持：**

除了标准 SSE，Codex CLI 还支持通过 WebSocket 传输响应流：

- **双向通信**：支持在流式传输过程中发送中断信号
- **增量请求**：WebSocket 支持增量式请求更新（incremental request tracking），减少重复传输
- **粘性路由**：WebSocket 连接保持会话亲和性，确保请求路由到同一后端实例

### 工具调用结果反馈

当模型发出工具调用后，Codex CLI 执行工具并将结果以 `function_call_output` 格式反馈给模型。

```json
{
  "type": "function_call_output",
  "call_id": "call_abc123",
  "output": "工具执行结果文本..."
}
```

### 无状态请求 + 前缀缓存

**关键设计 -- 旧提示词是新提示词的精确前缀：**

```
请求 N 的 input:
  [msg_0, msg_1, msg_2, ..., msg_n]

请求 N+1 的 input:
  [msg_0, msg_1, msg_2, ..., msg_n, tool_result_n+1]
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  精确前缀匹配 — API 可以复用之前计算的 KV 缓存
```

**不使用 `previous_response_id` 的原因：**

1. **零数据保留（ZDR）**：不依赖服务端状态意味着可以实现零数据保留模式
2. **简化实现**：不需要处理服务端状态不一致、会话过期等问题
3. **前缀缓存优化**：精确前缀匹配在现代 LLM 推理引擎中已经非常高效

**stream_request 函数（核心请求流程）：**

```rust
async fn stream_request(
    sess: &Session,
    turn_context: &TurnContext,
    client_session: &mut ModelClientSession,
    prompt: &Prompt,
) -> Result<()> {
    // 1. 构建完整的请求
    let request = build_request(prompt, turn_context);

    // 2. 发送请求并处理流式响应
    let response = client_session
        .stream_request(request, turn_context.cancellation_token.clone())
        .await?;

    // 3. 消费流式响应
    drain_to_completed(sess, turn_context, client_session, &prompt, response).await
}

async fn drain_to_completed(
    sess: &Session,
    turn_context: &TurnContext,
    client_session: &mut ModelClientSession,
    prompt: &Prompt,
) -> Result<()> {
    // 处理流式事件直到收到 completed 事件
    // 解析工具调用
    // 执行工具
    // 将工具结果追加到历史
    // 如果有工具调用，递归调用 stream_request
}
```

---

## 对比分析

### 循环模型对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **循环模式** | 流式异步生成器（AsyncGenerator） | 提交-事件模式（SQ/EQ） |
| **核心函数** | `query()` async generator | Submission Loop + `stream_request()` |
| **事件传递** | `yield` / `yield*` 委托 | Event Queue (channel) |
| **用户输入** | 直接调用 `sendMessage()` | 通过 `Op::UserInput` 提交 |
| **中止机制** | AbortController 层级传递 | CancellationToken + Op::Interrupt |
| **状态管理** | 有状态（闭包变量 + messages 引用） | 无状态请求 + 前缀缓存 |

### 流式处理对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **流式协议** | Anthropic SSE（SDK 封装） | OpenAI SSE + WebSocket |
| **事件类型** | 18+ StreamEvent 类型 | 6+ ResponseEvent 类型 |
| **中间件** | StreamingToolExecutor 状态机 | 直接 SSE 解析 + Event Queue |
| **双向通信** | 不支持（单向 SSE） | 支持（WebSocket） |
| **流式中断** | AbortController.abort() | WebSocket 中断 + CancellationToken |

### 工具执行对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **执行时机** | 流式中并行启动（StreamingToolExecutor） | 响应完成后串行执行 |
| **并发策略** | isConcurrencySafe 判断 + 批次执行 | Guardian 审批 + 沙箱执行 |
| **最大并发** | 10 个工具并行 | 无显式并发限制 |
| **顺序保证** | nextOrder 计数器 + 排序 | 按工具调用顺序串行 |
| **错误传播** | siblingAbortController 级联 | CancellationToken 取消 |

### API 调用对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **API 类型** | Anthropic Messages API | OpenAI Responses API |
| **请求格式** | messages + system + tools | instructions + tools + input |
| **缓存策略** | cache_control: ephemeral | 前缀缓存（精确前缀匹配） |
| **状态依赖** | 有状态（messages 数组累积） | 无状态（每次发送完整历史） |
| **max_tokens 处理** | 最多 3 次自动恢复 | 上下文窗口超时处理 |

### 错误处理与重试对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **重试策略** | 指数退避（rate_limit/overloaded） | 移除最旧历史项后重试 |
| **上下文溢出** | forceCompact() + reactiveCompact() | 移除最旧历史项 |
| **错误分类** | 5 类（rate_limit/overloaded/context_overflow/auth/other） | ContextWindowExceeded 等 |
| **最大重试** | 3 次（max_tokens）/ 3 次（API 错误） | 动态（直到历史可压缩） |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| AsyncGenerator 模型优雅，yield 事件零延迟传递给 UI | 有状态设计，messages 引用传递增加复杂度 |
| StreamingToolExecutor 实现流式并行工具执行，显著降低延迟 | StreamingToolExecutor 状态机复杂（~530 行），调试困难 |
| 18+ StreamEvent 类型覆盖所有场景，类型安全 | 事件类型过多，新增类型需修改多处代码 |
| max_tokens 自动恢复机制，对用户透明 | 恢复机制最多 3 次，复杂任务可能被截断 |
| QueryEngine 外层包装提供会话级重试和错误分类 | 重试逻辑与核心循环分离，状态同步需小心 |
| 微压缩 + 自动压缩集成在循环中，上下文管理无缝 | 压缩逻辑嵌入循环，增加了循环的复杂度 |
| AbortController 层级传播，中止信号精确控制 | 层级嵌套的 AbortController 管理复杂 |
| Prompt caching（ephemeral）显著降低重复输入成本 | 缓存仅限 Anthropic API，不适用于其他后端 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| SQ/EQ 模式解耦客户端和 Agent 循环，架构清晰 | 通道通信引入间接性，调试不如直接调用直观 |
| 无状态请求设计，零数据保留，隐私友好 | 每次发送完整历史，带宽消耗较大 |
| 前缀缓存优化，利用现代推理引擎的 KV 缓存 | 缓存效果依赖 API 服务端实现，不可控 |
| WebSocket 双向通信，支持流式中断 | WebSocket 连接管理复杂，需处理重连 |
| CancellationToken 协作式取消，Tokio 原生支持 | 取消粒度较粗，无法精确控制单个工具 |
| Op 枚举类型安全，serde 自动序列化/反序列化 | Op 变体较多，新增变体需修改多处匹配 |
| Responses API 简洁的三字段格式（instructions/tools/input） | 仅支持 OpenAI API，不兼容其他提供商 |
| Zstd 压缩支持，减少超长对话的传输数据量 | 压缩/解压增加 CPU 开销 |
| 远程压缩判断（should_use_remote_compact_task）灵活 | 远程压缩依赖 OpenAI 服务端，本地模型不可用 |
| drain_to_completed 递归模式简洁清晰 | 递归深度不受限，极端情况可能栈溢出 |
# 工具系统对比：Claude Code vs Codex CLI

## Claude Code 实现

### 工具注册表

```
┌─────────────────────────────────────────────────────────────┐
│                      工具注册表                               │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 核心工具 (始终加载)                                   │    │
│  │  BashTool, FileReadTool, FileEditTool, FileWriteTool│    │
│  │  GlobTool, GrepTool, AgentTool, SkillTool           │    │
│  │  TaskCreate/Get/Update/List/Output/Stop             │    │
│  │  EnterPlanMode, ExitPlanMode, WebFetch, WebSearch   │    │
│  │  ToolSearchTool, SendMessageTool                    │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 特性门控工具                                         │    │
│  │  KAIROS:        SendUserFile, PushNotification       │    │
│  │  MONITOR_TOOL:  MonitorTool                          │    │
│  │  COORDINATOR:   TeamCreate, TeamDelete               │    │
│  │  AGENT_TRIGGERS:CronCreate, CronDelete, CronList     │    │
│  │  WORKFLOW:      WorkflowTool                         │    │
│  │  WEB_BROWSER:   WebBrowserTool                      │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 延迟加载工具 (通过 ToolSearchTool)                    │    │
│  │  MCP 工具 (全部)                                      │    │
│  │  shouldDefer=true 的工具                              │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 动态 MCP 工具                                         │    │
│  │  运行时从连接的 MCP 服务器发现                         │    │
│  │  名称规范化: mcp__{server}__{tool}                    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

**工具加载流程**：

```
getAllBaseTools()
    |
    +-- 创建核心工具实例（BashTool, FileReadTool, ...）
    |       每个工具通过 buildTool() 工厂函数创建
    |
    +-- 检查特性标志（GrowthBook）
    |       if (!feature('COORDINATOR_MODE')) -> 过滤 TeamCreate/TeamDelete
    |       if (!feature('WEB_BROWSER')) -> 过滤 WebBrowserTool
    |
    +-- 加载 MCP 工具
    |       从配置中读取 MCP 服务器列表
    |       连接每个服务器并获取工具列表
    |       创建 MCPTool 代理（shouldDefer=true）
    |
    +-- 返回工具池
            assembleToolPool(coreTools, mcpTools, featureGatedTools)
```

### Tool 接口完整定义

```typescript
// Tool.ts -- 完整的 Tool 接口定义

export interface ToolUseContext {
  cwd: string;
  homeDir: string;
  abortController: AbortController;
  toolUseId: string;
  sessionId: string;
  fileCache: LRUCache<string, FileContent>;
  costTracker: CostTracker;
  permissionMode: PermissionMode;
  isNonInteractive: boolean;
  model: string;
}

export interface ToolResult {
  content: string;
  isError?: boolean;
  metadata?: {
    fileChanges?: FileChange[];
    exitCode?: number;
    duration?: number;
  };
}

export type PermissionResult =
  | { allowed: true }
  | { allowed: false; reason: string; block?: boolean };

export interface Tool {
  name: string;
  description: string;
  inputJSONSchema: JSONSchema;
  call(input: Record<string, unknown>, context: ToolUseContext): Promise<ToolResult>;
  validateInput?(input: unknown): { valid: boolean; error?: string; };
  checkPermissions?(input: Record<string, unknown>, context: ToolUseContext): PermissionResult;
  isConcurrencySafe(input: Record<string, unknown>): boolean;
  isReadOnly?: boolean;
  isDestructive?: boolean;
  isEnabled?(): boolean;
  canUseInNonInteractive?: boolean;
  shouldDefer?: boolean;
  alwaysLoad?: boolean;
  toAutoClassifierInput?(input: Record<string, unknown>): string;
}
```

### buildTool() 工厂函数和默认值

```typescript
const TOOL_DEFAULTS: Partial<Tool> = {
  isReadOnly: false,
  isDestructive: false,
  canUseInNonInteractive: true,
  shouldDefer: false,
  alwaysLoad: false,
  isEnabled: () => true,
};

function buildTool(
  definition: Partial<Tool> & Pick<Tool, 'name' | 'call'>
): Tool {
  return {
    ...TOOL_DEFAULTS,
    ...definition,
    description: definition.description || '',
    inputJSONSchema: definition.inputJSONSchema || { type: 'object', properties: {} },
    isConcurrencySafe: definition.isConcurrencySafe || (() => false),
  };
}

// FileReadTool -- 只读、并发安全
const FileReadTool = buildTool({
  name: 'Read',
  description: 'Reads a file from the local filesystem...',
  inputJSONSchema: fileReadSchema,
  call: async (input, context) => { /* ... */ },
  isReadOnly: true,
  isConcurrencySafe: () => true,
});

// BashTool -- 非只读、非并发安全、破坏性
const BashTool = buildTool({
  name: 'Bash',
  description: 'Executes a bash command...',
  inputJSONSchema: bashSchema,
  call: async (input, context) => { /* ... */ },
  isReadOnly: false,
  isDestructive: true,
  isConcurrencySafe: () => false,
  checkPermissions: (input, context) => {
    const command = input.command as string;
    if (isDangerousCommand(command)) {
      return { allowed: false, reason: 'Dangerous command detected' };
    }
    return { allowed: true };
  },
});
```

### runTools() 编排实现

```typescript
async function* runTools(
  params: RunToolsParams,
): AsyncGenerator<StreamEvent, ToolResult[], undefined> {
  const { toolUseBlocks, tools, messages, abortController,
          permissionMode, hooks } = params;

  const siblingAbortController = new AbortController();
  const onMainAbort = () => siblingAbortController.abort();
  abortController.signal.addEventListener('abort', onMainAbort);

  try {
    const safeBatch: ToolUseBlock[] = [];
    const unsafeBatch: ToolUseBlock[] = [];

    for (const block of toolUseBlocks) {
      const tool = tools.find(t => t.name === block.name);
      if (!tool) {
        yield { type: 'tool_execution_error', id: block.id,
                name: block.name, error: new Error(`Unknown tool: ${block.name}`) };
        continue;
      }

      const permission = await checkToolPermission(tool, block.input, permissionMode, hooks);
      if (!permission.allowed) {
        yield { type: 'tool_execution_error', id: block.id,
                name: block.name, error: new Error(permission.reason) };
        if (permission.block) break;
        continue;
      }

      const hookResult = await hooks.run('PreToolUse', {
        toolName: block.name, toolInput: block.input,
      });
      if (hookResult?.blocked) {
        yield { type: 'tool_execution_error', id: block.id,
                name: block.name, error: new Error(hookResult.reason || 'Blocked by hook') };
        continue;
      }

      if (tool.isConcurrencySafe(block.input)) {
        safeBatch.push(block);
      } else {
        unsafeBatch.push(block);
      }
    }

    const results: ToolResult[] = [];

    // 并行执行安全批次
    const safeResults = await Promise.all(
      safeBatch.map(async (block) => {
        const tool = tools.find(t => t.name === block.name)!;
        yield { type: 'tool_execution_start', id: block.id,
                name: block.name, input: block.input };
        try {
          const result = await tool.call(block.input, {
            cwd: params.cwd, abortController: siblingAbortController,
            toolUseId: block.id,
          });
          yield { type: 'tool_execution_complete', id: block.id,
                  name: block.name, result };
          await hooks.run('PostToolUse', {
            toolName: block.name, toolInput: block.input, toolResult: result,
          });
          return { ...result, toolUseId: block.id };
        } catch (error) {
          if (!siblingAbortController.signal.aborted && !(error instanceof AbortError)) {
            siblingAbortController.abort();
          }
          yield { type: 'tool_execution_error', id: block.id,
                  name: block.name, error: error as Error };
          return { content: `Error: ${(error as Error).message}`,
                   isError: true, toolUseId: block.id };
        }
      })
    );
    results.push(...safeResults);

    // 串行执行不安全批次
    for (const block of unsafeBatch) {
      if (siblingAbortController.signal.aborted) break;
      const tool = tools.find(t => t.name === block.name)!;
      yield { type: 'tool_execution_start', id: block.id,
              name: block.name, input: block.input };
      try {
        const result = await tool.call(block.input, {
          cwd: params.cwd, abortController: siblingAbortController,
          toolUseId: block.id,
        });
        yield { type: 'tool_execution_complete', id: block.id,
                name: block.name, result };
        results.push({ ...result, toolUseId: block.id });
      } catch (error) {
        yield { type: 'tool_execution_error', id: block.id,
                name: block.name, error: error as Error };
        results.push({ content: `Error: ${(error as Error).message}`,
                         isError: true, toolUseId: block.id });
        if (!(error instanceof AbortError)) siblingAbortController.abort();
      }
    }

    return results;
  } finally {
    abortController.signal.removeEventListener('abort', onMainAbort);
  }
}
```

### isConcurrencySafe 判断逻辑

```typescript
const CONCURRENCY_SAFE_TOOLS = new Set([
  'Read', 'Glob', 'Grep', 'WebFetch', 'WebSearch',
  'TaskGet', 'TaskList', 'TaskOutput', 'ToolSearch', 'EnterPlanMode',
]);

const CONCURRENCY_UNSAFE_TOOLS = new Set([
  'Bash', 'Write', 'Edit', 'AgentTool', 'SkillTool',
  'TaskCreate', 'TaskUpdate', 'TaskStop', 'SendMessage',
  'TeamCreate', 'TeamDelete', 'ExitPlanMode',
]);

function isConcurrencySafe(toolName: string, input: unknown): boolean {
  if (toolName.startsWith('mcp__')) return false;
  if (CONCURRENCY_SAFE_TOOLS.has(toolName)) return true;
  if (CONCURRENCY_UNSAFE_TOOLS.has(toolName)) return false;
  return false; // 未知工具默认不安全
}
```

### siblingAbortController 错误传播

```
中止信号传播链：

用户按 Ctrl+C
  -> mainAbortController.abort()
    -> query() 检测到中止 -> yield 'aborted' -> return
    -> siblingAbortController.abort() (通过事件监听)
      -> RunningTool1.abortController.abort()
      -> RunningTool2.abortController.abort()
      -> RunningTool3.abortController.abort()

工具 A 执行失败（非 AbortError）
  -> siblingAbortController.abort()
    -> RunningToolB.abortController.abort() -> 工具 B 收到 AbortError
    -> RunningToolC.abortController.abort() -> 工具 C 收到 AbortError

注意：AbortError 不触发兄弟中止（避免级联失败）

中止控制器层级关系：

mainAbortController (进程级)
  -> siblingAbortController (批次级，每次 runTools 创建新的)
     -> toolAbortController_1 (工具级)
     -> toolAbortController_2 (工具级)
     -> toolAbortController_3 (工具级)
```

### 延迟加载（ToolSearchTool）

```
┌─────────────────────────────────────────────────────────────────┐
│                    延迟加载流程                                   │
│                                                                 │
│  系统提示（仅包含工具名称列表，无 schema）                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Available tools:                                        │   │
│  │   Read, Write, Edit, Bash, Glob, Grep, ...             │   │
│  │   mcp__slack__send_message (use ToolSearch for schema)  │   │
│  │   mcp__github__create_issue (use ToolSearch for schema) │   │
│  │   mcp__jira__update_ticket (use ToolSearch for schema)  │   │
│  │   ... (50+ more MCP tools)                              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  模型决定使用 mcp__slack__send_message                           │
│       |                                                         │
│       v                                                         │
│  Step 1: 调用 ToolSearch({ query: "select:mcp__slack__send_message"})│
│       |                                                         │
│       v                                                         │
│  Step 2: ToolSearch 返回完整 schema (name, description, input_schema)│
│       |                                                         │
│       v                                                         │
│  Step 3: 模型使用获取的 schema 调用工具                          │
│                                                                 │
│  Token 节省估算：                                                 │
│  * 50 个 MCP 工具，每个 schema ~500 tokens                       │
│  * 全部加载：50 * 500 = 25,000 tokens（每次 API 调用）           │
│  * 延迟加载：仅 ~100 tokens（名称列表）+ 按需 ~500 tokens       │
│  * 节省：~24,400 tokens/轮次（对于未使用的工具）                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Codex CLI 实现

### 内置工具

Codex CLI 提供以下内置工具：

| 工具名称 | 说明 | 实现位置 |
|----------|------|----------|
| **`shell`** | 在沙箱中执行 Shell 命令 | `core/src/tools/shell.rs` |
| **`apply_patch`** | 对文件应用差异补丁（核心文件编辑能力） | `apply-patch/src/` |
| **`view_image`** | 查看图片 | `core/src/tools/view_image.rs` |
| **`js_repl`** | JavaScript REPL 执行（实验性，feature-gated） | `core/src/tools/js_repl.rs` |

**shell 工具执行流程：**

```
模型发出 shell 调用
    │
    ▼
Guardian 审批检查
    │
    ├── canAutoApprove → 自动执行
    │
    └── 需要审批 → 发送 ExecApprovalRequestEvent
                      │
                      ▼
                 用户确认/拒绝
                      │
                      ▼
              在沙箱中执行命令
                      │
                      ▼
              收集 stdout/stderr
                      │
                      ▼
              截断输出（防止上下文膨胀）
                      │
                      ▼
              返回 function_call_output
```

### 工具 Schema 系统

工具 Schema 系统定义在 `codex-tools` crate 中，提供了多层抽象：

```rust
/// 工具规范 — 定义工具的名称、描述和参数 schema
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}

/// 工具定义 — 包含工具规范和执行逻辑
pub struct ToolDefinition {
    pub spec: ToolSpec,
    pub handler: Box<dyn Fn(ToolInput) -> ToolOutput>,
}

/// 配置后的工具规范 — 经过配置系统处理后的工具
pub struct ConfiguredToolSpec {
    pub spec: ToolSpec,
    pub is_enabled: bool,
    pub requires_approval: bool,
}

/// Responses API 工具适配 — 转换为 API 格式
pub struct ResponsesApiTool {
    pub type_: String,        // "function"
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub strict: bool,
}

/// 自由格式工具 — 用于 js_repl 等非标准工具
pub struct FreeformTool {
    pub name: String,
    pub description: String,
    pub input_schema: Option<Value>,
}
```

**tool_search / tool_suggest 动态发现：**

```rust
/// 工具搜索 — 按需发现工具的完整 schema
pub async fn tool_search(query: &str) -> Vec<ToolSpec> {
    // 1. 在已注册的工具中搜索匹配的工具
    // 2. 在 MCP 工具中搜索
    // 3. 返回匹配工具的完整 schema
}

/// 工具建议 — 基于上下文推荐可能需要的工具
pub async fn tool_suggest(context: &TurnContext) -> Vec<ToolSpec> {
    // 基于当前对话上下文，推荐可能需要的工具
}
```

### apply_patch.rs 文件编辑

`apply-patch` 是 Codex CLI 的核心文件编辑能力，独立为 `codex-apply-patch` crate（parser.rs: 741 行 + lib.rs: 1672 行）。

#### Patch 格式语法

Patch 格式使用自定义的标记语言，由 Lark 语法定义：

```
start: begin_patch hunk+ end_patch

begin_patch:  "*** Begin Patch" LF
end_patch:    "*** End Patch" LF?

hunk: add_hunk | delete_hunk | update_hunk

add_hunk:    "*** Add File: " filename LF add_line+
delete_hunk: "*** Delete File: " filename LF
update_hunk: "*** Update File: " filename LF change_move? change?

change_move: "*** Move to: " filename LF
change: (change_context | change_line)+ eof_line?

change_context: ("@@" | "@@ " /(.+)/) LF
change_line: ("+" | "-" | " ") /(.+)/ LF
eof_line: "*** End of File" LF
```

**完整示例：**

```diff
*** Begin Patch
*** Add File: new_module.rs
+ pub fn new_function() -> i32 {
+     42
+ }
*** Update File: src/main.rs
@@ fn main() {
     println!("Hello");
-    old_line();
+    new_line();
 }
*** Delete File: deprecated.rs
*** End Patch
```

#### 解析器实现

解析器支持两种模式：

```rust
/// 解析模式
enum ParseMode {
    /// 严格模式 — 完全按照语法规范解析
    Strict,

    /// 宽松模式 — 兼容 GPT-4.1 的 heredoc 格式
    ///
    /// GPT-4.1 可能生成如下格式：
    /// ```json
    /// ["apply_patch", "<<'EOF'\n*** Begin Patch\n...\n*** End Patch\nEOF\n"]
    /// ```
    /// 在宽松模式下，解析器会检测并剥离 heredoc 包装
    Lenient,
}
```

**解析流程：**

```
输入: patch 文本
    │
    ▼
check_patch_boundaries_strict()
    │
    ├── 成功 → 解析 hunk
    │
    └── 失败 → check_patch_boundaries_lenient()
                    │
                    ├── 检测 heredoc 标记 (<<'EOF' ... EOF)
                    │   │
                    │   ├── 成功 → 剥离 heredoc，重新检查边界
                    │   │
                    │   └── 失败 → 返回 ParseError
                    │
                    └── 解析 hunk
```

**Hunk 数据结构：**

```rust
#[derive(Debug, PartialEq, Clone)]
pub enum Hunk {
    /// 添加新文件
    AddFile {
        path: PathBuf,
        contents: String,
    },

    /// 删除文件
    DeleteFile {
        path: PathBuf,
    },

    /// 更新文件（支持重命名）
    UpdateFile {
        path: PathBuf,
        move_path: Option<PathBuf>,     // 重命名目标路径
        chunks: Vec<UpdateFileChunk>,   // 按顺序排列的修改块
    },
}

/// 文件更新块
#[derive(Debug, PartialEq, Clone)]
pub struct UpdateFileChunk {
    /// 上下文行 — 用于定位修改位置（通常是类/方法/函数定义）
    pub change_context: Option<String>,

    /// 旧行 — 需要被替换的行
    pub old_lines: Vec<String>,

    /// 新行 — 替换后的行
    pub new_lines: Vec<String>,

    /// 是否在文件末尾
    pub is_end_of_file: bool,
}
```

#### 冲突处理策略

当 patch 无法干净地应用到文件时，系统采用多级策略处理：

1. **上下文匹配**：使用 `seek_sequence` 算法在文件中查找 `change_context` 指定的上下文行，精确定位修改位置

```rust
/// 在文件行中搜索目标序列
fn seek_sequence(
    lines: &[String],
    target: &[String],
    start_from: usize,
    eof: bool,
) -> Option<usize> {
    // 从 start_from 位置开始搜索
    // 如果 eof=true，也在文件末尾搜索
}
```

2. **多级 @@ 标记**：支持多个 `@@` 上下文标记来精确定位修改位置，减少歧义

3. **EOF 标记**：`*** End of File` 标记指示修改在文件末尾，系统会容忍尾部换行差异

4. **路径解析**：所有相对路径都基于 `cwd` 解析为绝对路径，使用 `AbsolutePathBuf` 确保路径安全

```rust
impl Hunk {
    pub fn resolve_path(&self, cwd: &AbsolutePathBuf) -> AbsolutePathBuf {
        let path = match self {
            Hunk::UpdateFile { path, .. } => path,
            Hunk::AddFile { .. } | Hunk::DeleteFile { .. } => self.path(),
        };
        AbsolutePathBuf::resolve_path_against_base(path, cwd)
    }
}
```

---

## 对比分析

### 工具数量与分类对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **内置工具总数** | 40+ | 25+ |
| **核心文件操作** | Read, Write, Edit（search-and-replace） | apply_patch（unified diff） |
| **Shell 执行** | Bash（直接执行） | shell（沙箱执行） |
| **搜索工具** | Glob, Grep | 无独立搜索工具（依赖 shell） |
| **任务管理** | TaskCreate/Get/Update/List/Output/Stop | 无 |
| **计划模式** | EnterPlanMode, ExitPlanMode | 无 |
| **Web 工具** | WebFetch, WebSearch | web_search（API 提供） |
| **MCP 工具** | 自研实现（4 种传输） | rmcp 0.12 |
| **Agent 工具** | AgentTool, SkillTool | 无 |
| **特性门控工具** | 6+（KAIROS/COORDINATOR/WORKFLOW 等） | js_repl（feature-gated） |
| **斜杠命令** | 87+ | N/A |

### 文件编辑策略对比

| 维度 | Claude Code（search-and-replace） | Codex CLI（apply_patch / unified diff） |
|------|----------------------------------|----------------------------------------|
| **编辑方式** | 搜索旧内容 -> 替换为新内容 | 声明式 diff（+/- 行标记） |
| **定位机制** | 精确字符串匹配 | 上下文行（@@ 标记）+ seek_sequence |
| **多文件编辑** | 每次调用编辑一个文件 | 单次 patch 可编辑多个文件 |
| **文件创建** | Write 工具 | `*** Add File` hunk |
| **文件删除** | Bash 工具 | `*** Delete File` hunk |
| **文件重命名** | Bash 工具 | `*** Move to` 支持 |
| **冲突处理** | 精确匹配失败则报错 | 多级 @@ 标记 + EOF 容错 + 宽松模式 |
| **模型友好度** | 高（简单直观） | 中（需学习 patch 格式） |
| **原子性** | 单文件原子操作 | 多文件原子 patch |

### 工具接口设计对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **接口定义** | TypeScript interface（13 个字段） | Rust struct（ToolSpec + ToolDefinition） |
| **输入验证** | validateInput? 可选方法 | serde JSON Schema 自动验证 |
| **权限检查** | checkPermissions? 可选方法 | Guardian 审批系统 |
| **并发安全** | isConcurrencySafe() 方法 | 无显式并发控制 |
| **工厂模式** | buildTool() 工厂函数 | ToolDefinition handler 闭包 |
| **延迟加载** | shouldDefer + ToolSearchTool | tool_search / tool_suggest |
| **特性门控** | GrowthBook feature flags | Rust cfg features |

### 并发模型对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **并发策略** | 安全/不安全批次分离 | 无显式并发（串行执行） |
| **最大并发数** | 10 | N/A |
| **安全判断** | 静态工具名白名单 | N/A |
| **错误传播** | siblingAbortController 级联 | CancellationToken 取消 |
| **顺序保证** | nextOrder 计数器 + 排序 | 按调用顺序串行 |
| **流式并行** | StreamingToolExecutor（边生成边执行） | 响应完成后执行 |

### 延迟加载策略对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **实现方式** | ToolSearchTool（专用工具） | tool_search / tool_suggest 函数 |
| **触发方式** | 模型主动调用 ToolSearch | 按需搜索 / 上下文推荐 |
| **Token 节省** | ~24,400 tokens/轮次（50 MCP 工具） | 类似效果 |
| **加载粒度** | 单个工具 schema | 搜索结果集 |
| **MCP 支持** | 全部 MCP 工具延迟加载 | MCP 工具动态发现 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| 40+ 工具覆盖全面，功能极其丰富 | 工具数量多导致系统提示词膨胀 |
| search-and-replace 编辑方式直观，模型易学 | 单次调用只能编辑一个文件，多文件修改效率低 |
| isConcurrencySafe 白名单 + StreamingToolExecutor 实现真正的流式并行 | 并发安全判断基于静态工具名，无法根据输入动态判断 |
| ToolSearchTool 延迟加载显著节省 token（~24,400/轮次） | 延迟加载增加了一轮额外的工具调用开销 |
| buildTool() 工厂函数统一默认值，工具开发简洁 | Tool 接口字段过多（13 个），实现完整工具成本高 |
| Hook 系统（PreToolUse/PostToolUse）提供工具执行前后拦截 | Hook 执行增加延迟，可能影响工具并发执行 |
| siblingAbortController 三级错误传播，中止控制精细 | 层级嵌套的 AbortController 管理复杂 |
| GrowthBook 特性门控灵活控制工具可用性 | 特性标志依赖远程服务，离线时可能失效 |
| 87+ 斜杠命令提供丰富的快捷操作 | 斜杠命令与工具系统分离，维护两套体系 |
| TaskCreate/Get/Update/List/Output/Stop 完整的任务管理 | 任务管理工具增加系统复杂度 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| apply_patch 支持单次多文件原子编辑，效率高 | patch 格式对模型生成质量要求高，格式错误率高 |
| 独立 crate（codex-apply-patch），可单独测试和运行 | 解析器复杂（parser.rs 741 行 + lib.rs 1672 行） |
| 宽松模式（Lenient）兼容 GPT-4.1 heredoc 格式 | 宽松模式增加解析器复杂度 |
| seek_sequence + @@ 标记精确定位，冲突处理多级 | 上下文匹配不如精确字符串匹配可靠 |
| `*** Move to` 原生支持文件重命名 | 重命名功能依赖 patch 格式正确性 |
| Guardian 审批系统与工具执行深度集成 | 审批流程增加工具执行延迟 |
| 沙箱执行所有 shell 命令，安全性高 | 沙箱限制可能阻止合法操作 |
| serde JSON Schema 自动验证输入 | 验证错误信息不如 Zod 友好 |
| tool_search / tool_suggest 双模式动态发现 | 动态发现机制不如 ToolSearchTool 成熟 |
| 工具数量精简（25+），系统提示词紧凑 | 功能覆盖面不如 Claude Code（缺少搜索、任务管理等） |
| shell 工具统一执行入口，简洁高效 | 缺少独立的 Glob/Grep 工具，搜索依赖 shell 命令 |
# 上下文与记忆管理对比：Claude Code vs Codex CLI

## Claude Code 实现

### 多层记忆架构

```
┌─────────────────────────────────────────────────────┐
│                 上下文窗口 (默认 200K, 扩展至 1M)     │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │ 系统提示 (每个会话固定)                           │  │
│  │  |- 基础 CLI 指令                               │  │
│  │  |- 工具描述（非延迟加载的）                      │  │
│  │  |- MCP 服务器指令                              │  │
│  │  └- 模式特定指导                                │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │ 用户上下文 (以 system-reminder 注入)             │  │
│  │  |- CLAUDE.md 层级 (项目记忆)                   │  │
│  │  |    /etc/claude-code/CLAUDE.md  (全局)        │  │
│  │  |    ~/.claude/CLAUDE.md         (用户)        │  │
│  │  |    ./CLAUDE.md                 (项目)        │  │
│  │  |    ./.claude/CLAUDE.md         (项目)        │  │
│  │  |    ./.claude/rules/*.md        (项目规则)    │  │
│  │  |    ./CLAUDE.local.md           (本地)        │  │
│  │  |- Git 状态 (分支、最近提交)                     │  │
│  │  └- 当前日期                                    │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │ 对话消息                                        │  │
│  │  |- [压缩边界 -- 旧消息摘要]                      │  │
│  │  |- 用户消息                                    │  │
│  │  |- 助手消息 (文本 + tool_use)                  │  │
│  │  |- 工具结果                                    │  │
│  │  └- ... (随每轮增长)                            │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │ 输出预留 (~20K tokens)                          │  │
│  └────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 四层压缩策略

```
Token 使用率 ──────────────────────────────────────────▶

0%              80%        85%        90%       98%
|               |          |          |          |
|  正常         | 微压缩    | 自动压缩  | 会话记忆  | 反应式
|  运行         | (清除     | (完整     | 压缩     | 压缩
|               |  旧工具   |  摘要     | (提取    | (API
|               |  结果)    |  旧消息)  |  到记忆) | 错误触发)
```

1. **微压缩（Micro-Compact）**：清除旧工具结果（替换为 `[Old tool result content cleared]`），针对 FileRead、Bash、Grep 等工具，基于时间阈值
2. **自动压缩（Auto-Compact）**：在约 167K tokens 时触发，发送旧消息给模型进行摘要，替换为压缩边界标记
3. **会话记忆压缩（Session Memory Compact）**：提取关键信息到持久会话记忆，保持 10K-40K tokens
4. **反应式压缩（Reactive Compact）**：由 API 的 `prompt_too_long` 错误触发，截断最旧的消息组

### microCompact 实现细节

```typescript
const MICRO_COMPACT_CONFIG = {
  maxClearCount: 20,
  maxAgeInTurns: 3,
  maxContentSize: 10000,
  persistThreshold: 50000,
};

export function microCompact(messages: Message[]): {
  clearedCount: number;
  persistedFiles: string[];
} {
  let clearedCount = 0;
  const persistedFiles: string[] = [];
  const currentTurnIndex = messages.length;

  for (let i = 0; i < messages.length
       && clearedCount < MICRO_COMPACT_CONFIG.maxClearCount; i++) {
    const msg = messages[i];
    if (msg.role !== 'user' || !Array.isArray(msg.content)) continue;

    const newContent = msg.content.map(block => {
      if (block.type !== 'tool_result') return block;
      const turnAge = currentTurnIndex - i;
      if (turnAge <= MICRO_COMPACT_CONFIG.maxAgeInTurns) return block;

      const contentSize = typeof block.content === 'string'
        ? block.content.length : JSON.stringify(block.content).length;
      if (contentSize <= MICRO_COMPACT_CONFIG.maxContentSize) return block;

      if (contentSize > MICRO_COMPACT_CONFIG.persistThreshold) {
        const filePath = persistToolResult(block.tool_use_id, block.content);
        persistedFiles.push(filePath);
      }

      clearedCount++;
      return { ...block, content: '[Old tool result content cleared]' };
    });

    messages[i] = { ...msg, content: newContent };
  }

  return { clearedCount, persistedFiles };
}
```

### autoCompact 摘要提示词

```typescript
const COMPACT_SYSTEM_PROMPT = `You are a conversation summarizer. Your task is to
create a concise but comprehensive summary of the conversation so far.

The summary MUST preserve the following information:
1. **User's original request**: What the user asked for, including any specific
   requirements or constraints
2. **Work completed**: What actions were taken, files modified, commands run,
   and their outcomes
3. **Current state**: Where we are in the task -- what's done, what's in
   progress, what's pending
4. **Key decisions**: Any important decisions made during the conversation
   (architecture choices, approaches selected/rejected)
5. **Errors encountered**: Any errors that occurred and how they were resolved
6. **Context for continuation**: Any information needed to continue the task
   without losing progress

Format the summary as a structured document with clear sections.`;

const COMPACT_BOUNDARY_MARKER = `[COMPACT_SUMMARY]
The conversation above this line has been summarized to save context space.
The summary preserves all essential information needed to continue the task.

--- SUMMARY START ---
{summary}
--- SUMMARY END ---`;
```

### reactiveCompact 截断策略

```typescript
export function reactiveCompact(
  messages: Message[], maxTokens: number,
): Message[] {
  const targetTokens = Math.floor(maxTokens * 0.8);
  let tailTokens = 0;
  let cutIndex = messages.length;

  for (let i = messages.length - 1; i >= 0; i--) {
    const msgTokens = estimateMessageTokens([messages[i]]);
    if (tailTokens + msgTokens > targetTokens) {
      cutIndex = i + 1; break;
    }
    tailTokens += msgTokens;
  }

  if (cutIndex >= messages.length) return messages.slice(-5);

  return [
    { role: 'user',
      content: '[Previous messages were truncated due to context length limits.]' },
    ...messages.slice(cutIndex),
  ];
}
```

### tokenBudget 计算方式

```typescript
/**
 * 估算消息的 token 数量
 * 公式：字符数 / 3（字符数/4 * 4/3 的简化）
 * 已知精度问题：
 * - 对纯中文文本会低估（中文 1 字符约等于 1-2 tokens）
 * - 对大量代码会高估（代码 token 效率较高）
 * - 误差范围：约 -20% 到 +50%
 */
export function estimateMessageTokens(messages: Message[]): number {
  let totalChars = 0;
  for (const msg of messages) {
    if (typeof msg.content === 'string') {
      totalChars += msg.content.length;
    } else if (Array.isArray(msg.content)) {
      for (const block of msg.content) {
        if (block.type === 'text') totalChars += block.text?.length || 0;
        else if (block.type === 'tool_use') totalChars += JSON.stringify(block.input).length;
        else if (block.type === 'tool_result')
          totalChars += typeof block.content === 'string'
            ? block.content.length : JSON.stringify(block.content).length;
        else if (block.type === 'thinking') totalChars += block.thinking?.length || 0;
      }
    }
  }
  return Math.max(1, Math.ceil(totalChars / 3));
}

export class TokenBudget {
  private maxTokens: number;
  private usedTokens: number = 0;
  constructor(maxTokens: number) { this.maxTokens = maxTokens; }
  getThreshold(type: 'compact'): number {
    return type === 'compact' ? Math.floor(this.maxTokens * 0.80) : this.maxTokens;
  }
  update(usage: Usage): void {
    this.usedTokens += usage.input_tokens + usage.output_tokens;
  }
  getUsageRatio(): number { return this.usedTokens / this.maxTokens; }
  shouldAutoCompact(): boolean { return this.getUsageRatio() >= 0.85; }
}
```

### 持久记忆（memdir）

```
~/.claude/projects/<project-slug>/memory/
├── MEMORY.md           # 索引文件 (最多 200 行)
├── user_role.md        # 用户类型：角色、偏好
├── feedback_testing.md # 反馈类型：要重复/避免的行为
├── project_auth.md     # 项目类型：持续工作上下文
└── reference_docs.md   # 参考类型：外部系统指针
```

### 提示词缓存优化

通过 `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__` 标记将静态指令（全局缓存）与动态会话内容分离，最大化 API 提示词缓存命中率。

```typescript
const systemPrompt = [
  {
    type: 'text',
    text: STATIC_SYSTEM_INSTRUCTIONS,
    cache_control: { type: 'ephemeral' },  // 标记为可缓存
  },
  {
    type: 'text',
    text: '__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__',  // 动态边界标记
  },
  {
    type: 'text',
    text: dynamicSessionContent,  // 不标记 cache_control
  },
];
```

**缓存效果**：静态部分（~30K tokens）在首次请求后被 API 缓存，后续请求只需发送动态部分（~5K tokens）+ 缓存引用，节省约 85% 的输入 token 成本和延迟。

### LRU 文件状态缓存

```typescript
this.fileCache = new LRUCache<string, FileContent>({
  max: 100,           // 最多缓存 100 个文件
  maxSize: 25 * 1024 * 1024,  // 总大小上限 25MB
  sizeCalculation: (value: FileContent) => {
    return typeof value.content === 'string'
      ? value.content.length : Buffer.byteLength(value.content);
  },
});
// 使用场景：FileReadTool 读取时先查缓存，FileEditTool 编辑时更新缓存，
// 跨轮次保持文件状态，避免冗余读取和变更检测
```

### Dream Task

独特的后台记忆整合机制，在用户空闲时自动回顾会话并更新记忆文件，实现跨会话学习持久化：

```
用户空闲（超过 5 分钟无交互）
  -> autoDream 服务启动（后台子 Agent）
  -> 回顾当前会话历史，提取关键信息
  -> 更新 ~/.claude/projects/<slug>/memory/ 下的记忆文件
  -> 下次会话启动时，Claude 已经"记住"了之前的学习
```

---

## Codex CLI 实现

### compact.rs 压缩算法

核心实现在 `codex-rs/core/src/compact.rs`（442 行），负责在对话历史过长时自动压缩早期对话。

**两种压缩模式：**

```rust
/// 控制压缩后是否需要注入初始上下文
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InitialContextInjection {
    /// 在最后一条用户消息之前注入初始上下文
    /// 用于 mid-turn 压缩（模型训练时看到的压缩摘要位置）
    BeforeLastUserMessage,

    /// 不注入初始上下文
    /// 用于 pre-turn/manual 压缩（下一个常规轮次会完整注入初始上下文）
    DoNotInject,
}
```

| 模式 | 触发时机 | 初始上下文注入 | 说明 |
|------|----------|---------------|------|
| **Pre-turn / Manual** | 用户手动触发或轮次开始前 | `DoNotInject` | 替换历史为摘要，清除 `reference_context_item`，下一轮会完整注入 |
| **Mid-turn** | 轮次进行中自动触发 | `BeforeLastUserMessage` | 在摘要前注入初始上下文，保持模型训练时的位置预期 |

**本地压缩流程（4 步）：**

```
┌──────────────────────────────────────────────────────────────┐
│                    本地压缩流程                                │
│                                                               │
│  步骤 1: 构建压缩请求                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ history = sess.clone_history()                       │   │
│  │ history.record_items(&[initial_input_for_turn])      │   │
│  │ prompt = Prompt {                                    │   │
│  │     input: history.for_prompt(...),                  │   │
│  │     base_instructions: SUMMARIZATION_PROMPT,         │   │
│  │ }                                                   │   │
│  └──────────────────────────────────────────────────────┘   │
│                          │                                   │
│                          ▼                                   │
│  步骤 2: 调用模型生成摘要                                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ drain_to_completed(&sess, turn_context, &prompt)     │   │
│  │ // 流式调用 API，收集模型生成的摘要                    │   │
│  │ // 如果上下文窗口溢出，移除最旧的历史项后重试          │   │
│  └──────────────────────────────────────────────────────┘   │
│                          │                                   │
│                          ▼                                   │
│  步骤 3: 构建压缩后的历史                                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ summary_text = format!("{SUMMARY_PREFIX}\n{summary}")│   │
│  │ user_messages = collect_user_messages(history_items) │   │
│  │ new_history = build_compacted_history(               │   │
│  │     Vec::new(), &user_messages, &summary_text        │   │
│  │ )                                                   │   │
│  └──────────────────────────────────────────────────────┘   │
│                          │                                   │
│                          ▼                                   │
│  步骤 4: 替换历史并发送事件                                   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ sess.replace_compacted_history(                      │   │
│  │     new_history, reference_context_item,             │   │
│  │     compacted_item                                  │   │
│  │ )                                                   │   │
│  │ // 发送 CompactedItem 事件                           │   │
│  │ // 发送 WarningEvent（建议新开线程）                   │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

**远程压缩判断：**

```rust
/// 判断是否应使用远程压缩（云端 API）
pub(crate) fn should_use_remote_compact_task(provider: &ModelProviderInfo) -> bool {
    provider.is_openai()  // 仅 OpenAI 提供商使用远程压缩
}
```

### 摘要提示词模板

压缩使用专门的摘要提示词模板：

```rust
pub const SUMMARIZATION_PROMPT: &str = include_str!("../templates/compact/prompt.md");
pub const SUMMARY_PREFIX: &str = include_str!("../templates/compact/summary_prefix.md");
```

**摘要文本格式：**

```
{SUMMARY_PREFIX}

{模型生成的摘要内容}
```

摘要内容包含：
- 用户的原始请求
- 已完成的工作摘要
- 重要的决策和推理过程
- 待完成的工作（如果有）

### Token 感知截断

核心实现在 `codex-rs/core/src/truncate.rs`（363 行），提供基于字节估算的 token 截断功能。

**关键常量：**

```rust
/// 每个 token 的近似字节数（用于 UTF-8 文本的粗略估算）
pub const APPROX_BYTES_PER_TOKEN: usize = 4;

/// 压缩后用户消息的最大 token 数
pub const COMPACT_USER_MESSAGE_MAX_TOKENS: usize = 20_000;
```

**TruncationPolicy 枚举：**

```rust
pub enum TruncationPolicy {
    /// 不截断
    NoTruncation,

    /// 按字节估算截断到指定 token 数
    TokenLimit {
        max_tokens: usize,
    },

    /// 按行数截断
    LineLimit {
        max_lines: usize,
    },
}
```

**truncate_with_byte_estimate 算法：**

```rust
/// 基于字节估算的截断算法
///
/// 策略：50/50 分配前缀和后缀
/// - 前缀保留开头内容（提供上下文）
/// - 后缀保留结尾内容（提供最新信息）
/// - 在 UTF-8 字符边界处安全切割
pub fn truncate_with_byte_estimate(
    text: &str,
    max_tokens: usize,
) -> String {
    let max_bytes = max_tokens * APPROX_BYTES_PER_TOKEN;

    if text.len() <= max_bytes {
        return text.to_string();
    }

    let half = max_bytes / 2;

    // 找到前缀的安全 UTF-8 切割点
    let prefix_end = find_safe_utf8_boundary(text, half);

    // 找到后缀的安全 UTF-8 切割点
    let suffix_start = find_safe_utf8_boundary_from_end(text, half);

    format!(
        "{}\n\n... [truncated {} bytes] ...\n\n{}",
        &text[..prefix_end],
        text.len() - max_bytes,
        &text[suffix_start..]
    )
}

/// 近似 token 计数
pub fn approx_token_count(text: &str) -> usize {
    (text.len() + APPROX_BYTES_PER_TOKEN - 1) / APPROX_BYTES_PER_TOKEN
}
```

**build_compacted_history_with_limit 重建逻辑：**

```rust
/// 构建压缩后的历史，确保总 token 数不超过限制
pub fn build_compacted_history_with_limit(
    user_messages: &[String],
    summary_text: &str,
    max_tokens: usize,
) -> Vec<ResponseItem> {
    let mut history = Vec::new();

    // 1. 添加摘要
    history.push(ResponseItem::from(summary_text));

    // 2. 从最新消息开始，向前添加用户消息
    let mut remaining_tokens = max_tokens
        .saturating_sub(approx_token_count(summary_text));

    for msg in user_messages.iter().rev() {
        let msg_tokens = approx_token_count(msg);
        if msg_tokens > remaining_tokens {
            // 截断消息以适应剩余预算
            let truncated = truncate_with_byte_estimate(
                msg,
                remaining_tokens
            );
            history.push(ResponseItem::from(truncated));
            break;
        }
        remaining_tokens -= msg_tokens;
        history.push(ResponseItem::from(msg.clone()));
    }

    // 3. 反转以保持时间顺序
    history.reverse();
    history
}
```

### 上下文窗口超时处理

当对话历史超过模型上下文窗口时，系统采用移除最旧历史项的重试策略：

```rust
// compact.rs 中的上下文窗口超时处理
Err(e @ CodexErr::ContextWindowExceeded) => {
    if turn_input_len > 1 {
        // 从开头移除最旧的历史项
        // 保留缓存（基于前缀），保持最近消息完整
        error!(
            "Context window exceeded while compacting; \
             removing oldest history item. Error: {e}"
        );
        history.remove_first_item();
        truncated_count += 1;
        retries = 0;
        continue;  // 重试
    }
    // 如果只剩一条消息仍然超限，报告错误
    sess.set_total_tokens_full(turn_context.as_ref()).await;
    let event = EventMsg::Error(e.to_error_event(None));
    sess.send_event(&turn_context, event).await;
    return Err(e);
}
```

---

## 对比分析

### 压缩策略对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **压缩层数** | 四层渐进式（微/自动/记忆/反应式） | 两层（自动压缩 + 截断） |
| **微压缩** | 有（清除旧工具结果，不调用 LLM） | 无 |
| **自动压缩** | 有（~167K tokens 触发，LLM 摘要） | 有（本地 + 远程两种模式） |
| **记忆压缩** | 有（提取到持久记忆文件） | 无 |
| **反应式压缩** | 有（API 错误触发） | 有（ContextWindowExceeded 触发） |
| **压缩触发阈值** | 80%/85%/90%/98% 四级 | 动态（上下文窗口溢出时） |
| **压缩粒度** | 工具结果级 + 消息级 | 消息级 |

### 压缩实现对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **微压缩实现** | microCompact() 函数，原地修改 messages | N/A |
| **微压缩策略** | 基于轮次年龄 + 内容大小阈值 | N/A |
| **大结果持久化** | >50K 字符的工具结果持久化到磁盘 | N/A |
| **自动压缩实现** | autoCompact() + COMPACT_SYSTEM_PROMPT | compact.rs 4 步流程 |
| **摘要提示词** | 内联 TypeScript 模板 | 外部 .md 模板文件 |
| **压缩边界标记** | `[COMPACT_SUMMARY]` + SUMMARY START/END | SUMMARY_PREFIX 常量 |
| **初始上下文注入** | 无 | BeforeLastUserMessage / DoNotInject |

### Token 计算对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **估算方法** | 字符数 / 3 | 字节数 / 4 |
| **适用文本** | 通用（英文为主） | UTF-8 文本 |
| **中文精度** | 低估（1 字符约 1-2 tokens，但按 1/3 计算） | 较好（UTF-8 中文 3 字节/token） |
| **代码精度** | 高估（代码 token 效率较高） | 较好（代码 ASCII 1 字节/token） |
| **误差范围** | -20% 到 +50% | 类似范围 |
| **精确计数** | 使用 API 返回的 usage 字段 | 使用 API 返回的 usage 字段 |
| **预算管理** | TokenBudget 类（阈值 80%/85%） | TruncationPolicy 枚举 |

### 持久记忆对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **持久记忆** | 有（memdir 目录结构） | 无 |
| **记忆文件** | MEMORY.md + 4 个分类文件 | N/A |
| **记忆类型** | 用户角色/反馈/项目上下文/参考文档 | N/A |
| **CLAUDE.md** | 6 级层级（全局/用户/项目/规则/本地） | N/A |
| **Dream Task** | 有（后台自动记忆整合） | N/A |
| **跨会话学习** | 支持（记忆文件持久化） | 不支持 |

### 缓存策略对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **提示词缓存** | cache_control: ephemeral（Anthropic API） | 前缀缓存（OpenAI API） |
| **静态/动态分离** | `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__` 标记 | 无显式分离 |
| **缓存节省** | ~85% 输入 token 成本 | 依赖 API 服务端前缀缓存 |
| **文件缓存** | LRU（100 文件 / 25MB） | 无显式文件缓存 |
| **Zstd 压缩** | 无 | 支持（超长对话历史） |

### 上下文窗口管理对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **默认窗口** | 200K tokens（扩展至 1M） | 取决于模型（128K-1M） |
| **输出预留** | ~20K tokens | 动态 |
| **溢出处理** | reactiveCompact 截断最旧消息 | 移除最旧历史项后重试 |
| **重试策略** | forceCompact + reactiveCompact | remove_first_item + continue |
| **最大重试** | 动态（直到可压缩） | 动态（直到只剩一条消息） |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| 四层渐进式压缩策略精细，覆盖从轻量到紧急的全场景 | 四层策略实现复杂，各层触发条件需协调 |
| 微压缩不调用 LLM，零成本清除旧工具结果 | 微压缩仅清除工具结果，不处理文本消息膨胀 |
| 大工具结果（>50K 字符）持久化到磁盘，避免丢失 | 持久化路径管理增加复杂度 |
| autoCompact 摘要提示词详细，保留 6 类关键信息 | 摘要质量依赖模型能力，可能丢失细节 |
| CLAUDE.md 6 级层级提供灵活的项目记忆 | 6 级加载顺序复杂，冲突规则不明确 |
| Dream Task 后台记忆整合，实现跨会话学习 | Dream Task 消耗额外 API 调用，增加成本 |
| TokenBudget 类提供精确的阈值管理（80%/85%） | 字符数/3 的估算对中文严重低估 |
| cache_control: ephemeral 提示词缓存节省 ~85% 成本 | 缓存仅限 Anthropic API，多云切换时失效 |
| LRU 文件缓存（100 文件/25MB）减少冗余 I/O | 缓存一致性需手动维护（编辑时更新） |
| 静态/动态提示词分离最大化缓存命中率 | `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__` 标记不够优雅 |
| reactiveCompact 作为最后防线处理 API 错误 | 截断策略简单（保留尾部 80%），可能丢失关键上下文 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| compact.rs 442 行实现简洁，逻辑清晰 | 仅两层压缩（自动 + 截断），缺少微压缩层 |
| 两种压缩模式（Pre-turn/Mid-turn）适配不同场景 | 无微压缩，旧工具结果持续占用上下文 |
| 远程压缩判断（should_use_remote_compact_task）灵活 | 远程压缩仅支持 OpenAI，本地模型不可用 |
| 外部 .md 模板文件，摘要提示词易于修改和版本管理 | 摘要提示词内容不如 Claude Code 详细（仅 4 类信息） |
| truncate_with_byte_estimate 50/50 前后缀策略合理 | 50/50 分配对长消息可能截断关键中间内容 |
| UTF-8 安全切割，避免乱码 | 字节数/4 估算对纯英文高估（ASCII 1 字节/token） |
| build_compacted_history_with_limit 从最新消息向前填充 | 无 TokenBudget 类等价物，阈值管理不够精细 |
| 上下文窗口超时处理移除最旧项后重试，保持前缀缓存 | 重试无上限，极端情况可能移除所有历史 |
| Zstd 压缩支持减少超长对话传输数据量 | Zstd 压缩/解压增加 CPU 开销 |
| 前缀缓存利用现代推理引擎 KV 缓存 | 缓存效果完全依赖 API 服务端，不可控 |
| TruncationPolicy 枚举提供多种截断策略 | 无持久记忆系统，跨会话信息丢失 |
| COMPACT_USER_MESSAGE_MAX_TOKENS (20K) 限制单条消息 | 无 CLAUDE.md 等价物，项目上下文管理薄弱 |
| InitialContextInjection 枚举明确控制注入行为 | 无 Dream Task 等价物，无后台记忆整合 |
# 安全与权限系统对比

## Claude Code 实现

### 权限管道

Claude Code 采用**软件层权限治理**方案，通过多层权限管道对工具调用进行逐级过滤。整个管道从工具调用到达开始，经过模式检查、规则匹配、LLM 分类器，最终到用户确认。

```
工具调用到达
     |
     v
┌──────────────┐    ┌────────────┐
│ 检查模式     │───>│ bypass     │──> 允许（跳过所有检查）
│              │    │ Permissions │
│              │    └────────────┘
│              │    ┌────────────┐
│              │───>│ dontAsk    │──> 拒绝（阻止所有）
└──────┬───────┘    └────────────┘
       |
       v
┌──────────────┐
│ 应用规则     │
│  1. Deny     │──> 匹配 -> 拒绝
│  2. Allow    │──> 匹配 -> 允许
│  3. Ask      │──> 匹配 -> 提示用户
└──────┬───────┘
       | 无规则匹配
       v
┌──────────────┐
│ Auto 模式?   │──YES──> LLM 分类器
│              │         |- 白名单工具? -> 允许
│              │         |- 分类器说安全? -> 允许
│              │         └- 分类器说不安全? -> 拒绝
│              │             (>3 连续或 >20 总计 -> 回退到 ASK)
└──────┬───────┘
       | 非 auto 模式
       v
┌──────────────┐
│ 模式特定默认 │──> ASK 用户
│  acceptEdits │──> 允许 cwd 内文件编辑，其他 ASK
│  plan        │──> 暂停并显示计划
└──────────────┘
```

### 6 种权限模式

| 模式 | 符号 | 行为 |
|------|------|------|
| `default` | `>` | 所有非只读工具都需要询问 |
| `acceptEdits` | `>>` | 自动允许 cwd 内文件编辑 |
| `plan` | `?` | 在工具调用之间暂停以供审查 |
| `bypassPermissions` | `!` | 跳过所有检查（危险） |
| `auto` | `A` | LLM 分类器决定（特性门控） |

### 权限管道完整代码流程

```typescript
export async function checkToolPermission(
  tool: Tool, input: Record<string, unknown>,
  mode: PermissionMode, rules: PermissionRule[],
  hooks: HookEngine, autoClassifier: AutoClassifier | null,
  stats: PermissionStats,
): Promise<PermissionResult> {
  // 第 1 层：模式前置检查
  if (mode === 'bypassPermissions') {
    if (isBypassDisabledByRemote()) {
      return { allowed: false, reason: 'Bypass mode disabled by remote policy' };
    }
    return { allowed: true };
  }
  if (mode === 'dontAsk') {
    if (tool.isReadOnly) return { allowed: true };
    return { allowed: false, reason: 'dontAsk mode: non-readonly tools blocked' };
  }

  // 第 2 层：只读工具自动放行
  if (tool.isReadOnly) return { allowed: true };

  // 第 3 层：工具级权限检查
  if (tool.checkPermissions) {
    const toolPermission = tool.checkPermissions(input, context);
    if (!toolPermission.allowed) return toolPermission;
  }

  // 第 4 层：规则匹配
  for (const rule of rules) {
    if (matchesRule(tool.name, input, rule)) {
      switch (rule.action) {
        case 'deny':
          stats.deniedCount++; stats.consecutiveDenials++;
          return { allowed: false, reason: rule.reason || 'Denied by rule' };
        case 'allow':
          stats.consecutiveDenials = 0;
          return { allowed: true };
        case 'ask': break;
      }
    }
  }

  // 第 5 层：acceptEdits 模式特殊处理
  if (mode === 'acceptEdits') {
    if ((tool.name === 'Write' || tool.name === 'Edit') && isWithinCwd(input.file_path)) {
      return { allowed: true };
    }
  }

  // 第 6 层：Auto 模式 -- LLM 分类器
  if (mode === 'auto' && autoClassifier) {
    if (isAutoModeDisabledByRemote()) return await askUser(tool, input);
    if (AUTO_WHITELIST_TOOLS.has(tool.name)) return { allowed: true };
    if (stats.consecutiveDenials >= 3 || stats.deniedCount >= 20) {
      return await askUser(tool, input);
    }
    const classifierInput = tool.toAutoClassifierInput?.(input)
      || `${tool.name}: ${JSON.stringify(input)}`;
    const isSafe = await autoClassifier.classify(classifierInput);
    if (isSafe) { stats.consecutiveDenials = 0; return { allowed: true }; }
    else { stats.deniedCount++; stats.consecutiveDenials++;
            return { allowed: false, reason: 'Auto-classifier: unsafe operation' }; }
  }

  // 第 7 层：用户确认
  return await askUser(tool, input);
}
```

### 7 层安全机制详细说明

| 层级 | 机制 | 实现位置 | 说明 |
|------|------|----------|------|
| **1** | 危险文件保护 | `utils/permissions/dangerousFiles.ts` | `.gitconfig`、`.bashrc`、`.zshrc`、`.mcp.json`、`/etc/` 下的系统文件被阻止修改 |
| **2** | 危险命令检测 | `BashTool.checkPermissions()` | `rm -rf /`、`git push --force`、`DROP TABLE`、`chmod 777`、`> /dev/sda` 等模式匹配 |
| **3** | Bypass 权限终止开关 | `growthbook.ts` + `isBypassDisabledByRemote()` | GrowthBook 特性门可以远程禁用 bypass 模式 |
| **4** | Auto 模式断路器 | Statsig 实时门 | Statsig 门可以远程禁用 auto 模式，连续拒绝 >3 或总计 >20 时自动回退到 ASK |
| **5** | 拒绝追踪 | `PermissionStats` | 跟踪 `deniedCount` 和 `consecutiveDenials`，超过阈值时切换到更保守的模式 |
| **6** | 技能范围收窄 | `SkillTool` | 编辑 `.claude/skills/X/` 时提供窄范围权限 |
| **7** | MCP Shell 阻止 | `MCPTool` | MCP 来源的技能永不执行 shell 命令，防止远程代码注入 |

### LLM 自动分类器（auto 模式）

Claude Code 的 auto 模式使用一个轻量级 LLM 分类器（Claude 3 Haiku）来判断工具调用是否安全：

```typescript
const AUTO_WHITELIST_TOOLS = new Set([
  'Read', 'Glob', 'Grep', 'WebFetch', 'WebSearch',
  'TaskGet', 'TaskList', 'TaskOutput', 'ToolSearch',
  'EnterPlanMode', 'ExitPlanMode',
]);

export class AutoClassifier {
  private consecutiveDenials: number = 0;
  private totalDenials: number = 0;

  async classify(input: string): Promise<boolean> {
    if (this.consecutiveDenials >= 3 || this.totalDenials >= 20) return false;
    if (isAutoModeDisabledByRemote()) return false;

    try {
      const response = await anthropic.messages.create({
        model: 'claude-3-haiku-20240307',
        max_tokens: 1,
        messages: [{
          role: 'user',
          content: `Is this tool call safe to execute without user confirmation?
Tool call: ${input}
Respond with only "Y" (safe) or "N" (unsafe).`,
        }],
      });

      const isSafe = response.content[0].text.trim() === 'Y';
      if (!isSafe) { this.consecutiveDenials++; this.totalDenials++; }
      else { this.consecutiveDenials = 0; }
      return isSafe;
    } catch { return false; }
  }
}
```

**关键设计要点：**
- 使用 `claude-3-haiku-20240307`（最轻量模型）降低延迟和成本
- `max_tokens: 1` 限制输出为单个字符，确保响应速度
- 断路器机制：连续拒绝 >=3 或总计 >=20 时自动回退到 ASK 模式
- 远程终止开关：GrowthBook/Statsig 可实时禁用 auto 模式

---

## Codex CLI 实现

### 沙箱系统

Codex CLI 采用**平台原生 OS 内核级沙箱**方案，实现真正的进程级隔离。这是其最重要的安全特性。

#### macOS Seatbelt 沙箱

macOS 使用 Apple 原生的 **Seatbelt** (`sandbox-exec`) 沙箱系统。核心实现在 `core/src/seatbelt.rs`（623 行）。

**SBPL 策略结构：**

```
┌─────────────────────────────────────────────────────────────┐
│                    SBPL 策略结构                             │
│                                                              │
│  (version 1)                                                 │
│  (deny default)                                              │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 基础策略                                              │   │
│  │  (allow process-exec)                                 │   │
│  │  (allow sysctl-read)                                  │   │
│  │  (allow file-read*)                                   │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 文件读取策略                                          │   │
│  │  (allow file-read* (subpath "/usr"))                  │   │
│  │  (allow file-read* (subpath "/bin"))                  │   │
│  │  (allow file-read* (subpath "/opt/homebrew"))         │   │
│  │  (allow file-read* (subpath "{cwd}"))                 │   │
│  │  (allow file-read* (subpath "{home}"))                │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 文件写入策略（workspace-write 模式）                   │   │
│  │  (allow file-write* (subpath "{cwd}"))                │   │
│  │  (deny file-write* (subpath "{cwd}/.git"))            │   │
│  │  (deny file-write* (subpath "{cwd}/.codex"))          │   │
│  │  (deny file-write* (subpath "{cwd}/.hg"))             │   │
│  │  (deny file-write* (subpath "{cwd}/.svn"))            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 网络策略                                              │   │
│  │  read-only 模式: (deny network*)                      │   │
│  │  workspace-write 模式: (deny network*)                │   │
│  │  danger-full-access 模式: (allow network*)            │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**create_seatbelt_command_args 代码示例：**

```rust
// codex-rs/sandboxing/src/seatbelt.rs
pub fn create_seatbelt_command_args_for_policies(
    command: Vec<String>,
    fs_policy: &FileSystemSandboxPolicy,
    network_policy: NetworkSandboxPolicy,
    sandbox_policy_cwd: &Path,
    enforce_managed_network: bool,
    network: Option<&NetworkProxy>,
) -> Vec<String> {
    let mut args = Vec::new();

    // 构建 SBPL 配置文件
    let profile = build_sbpl_profile(fs_policy, network_policy, sandbox_policy_cwd);

    args.push("-p".to_string());  // 使用内联配置文件
    args.push(profile);

    // 传递原始命令
    args.extend(command);

    args
}
```

**特殊保护目录：**

| 目录 | 保护原因 |
|------|----------|
| `.git/` | Git 版本控制数据，防止破坏版本历史 |
| `.codex/` | Codex 自身的配置和状态数据 |
| `.hg/` | Mercurial 版本控制数据 |
| `.svn/` | Subversion 版本控制数据 |

#### Linux Landlock + Bubblewrap

Linux 沙箱采用**三层架构**，核心实现在 `codex-rs/linux-sandbox/src/landlock.rs`（约 300 行）。

```
┌─────────────────────────────────────────────────────────────┐
│                 Linux 三层沙箱架构                            │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第一层: Landlock (内核 5.13+)                         │   │
│  │  - 文件系统访问控制（只读/读写目录）                    │   │
│  │  - 在当前线程上应用，子进程继承                         │   │
│  │  - 适用于简单策略（read-only, workspace-write）        │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第二层: seccomp (系统调用过滤)                         │   │
│  │  - 限制可用的系统调用                                  │   │
│  │  - 阻止危险系统调用（如 ptrace、mount）                │   │
│  │  - 与 Landlock 配合使用                               │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第三层: Bubblewrap (用户命名空间隔离)                   │   │
│  │  - 完整的 Linux 命名空间隔离                           │   │
│  │  - 独立的 mount/IPC/network/PID 命名空间               │   │
│  │  - 适用于复杂策略（需要细粒度控制时自动路由）            │   │
│  │  - codex-linux-sandbox 独立进程                        │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  策略路由:                                                   │
│  简单策略 -> Landlock 直接处理                                │
│  复杂策略 -> 自动路由到 Bubblewrap                            │
└─────────────────────────────────────────────────────────────┘
```

**apply_sandbox_policy_to_current_thread 代码：**

```rust
// codex-rs/linux-sandbox/src/landlock.rs
pub fn apply_sandbox_policy_to_current_thread(
    fs_policy: &FileSystemSandboxPolicy,
) -> Result<(), SandboxErr> {
    // 1. 检查 Landlock 是否可用
    if !landlock_available() {
        return Err(SandboxErr::LandlockRestrict(
            "Landlock not available on this kernel".into()
        ));
    }

    // 2. 创建 Landlock 规则集
    let mut ruleset = Ruleset::new()
        .handle_access(Access::from_bits(
            AccessFS::READ_FILE | AccessFS::READ_DIR
        ))?;

    // 3. 添加只读目录规则
    for read_dir in &fs_policy.read_roots {
        ruleset.add_rule(PathFd::new(read_dir)?, AccessFS::READ_FILE)?;
    }

    // 4. 添加读写目录规则
    for write_dir in &fs_policy.write_roots {
        ruleset.add_rule(PathFd::new(write_dir)?,
            AccessFS::READ_FILE | AccessFS::WRITE_FILE)?;
    }

    // 5. 限制自身（应用沙箱）
    ruleset.restrict_self()?;

    Ok(())
}
```

**codex-linux-sandbox 独立进程：**

```bash
# codex-linux-sandbox 通过 Bubblewrap 创建隔离环境
codex-linux-sandbox \
    --ro-bind /usr /usr \
    --ro-bind /bin /bin \
    --bind /path/to/cwd /path/to/cwd \
    --dev-bind /dev/null /dev/null \
    --unshare-net \
    --seccomp <seccomp-filter> \
    -- <command>
```

#### Windows Restricted Token

Windows 使用**限制令牌（Restricted Token）**模式实现沙箱：

```
┌─────────────────────────────────────────────────────────────┐
│                 Windows 沙箱策略                              │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Restricted Token 模式                                 │   │
│  │  - 创建限制令牌，移除特权 SID                          │   │
│  │  - 限制文件系统访问权限                                │   │
│  │  - 适用于简单策略                                      │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Elevated Runner 模式                                   │   │
│  │  - 提升权限的后端进程                                  │   │
│  │  - 支持更细粒度的文件系统控制                          │   │
│  │  - 通过 IPC 与主进程通信                               │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

#### 沙箱策略注入系统提示词

沙箱策略不仅通过操作系统内核执行，还会注入到系统提示词中，让模型在尝试操作前就知道其约束条件：

```markdown
<permissions instructions>
## File System Permissions

You have the following file system permissions:
- **Read access**: You can read files in the current working directory and its subdirectories.
- **Write access**: You can create and modify files in the current working directory and its subdirectories.
- **Protected paths**: You CANNOT write to `.git/`, `.codex/`, `.hg/`, `.svn/` directories.

## Network Permissions

- **Network access**: DISABLED. You cannot make network requests.
</permissions instructions>
```

#### 网络访问控制

```
┌─────────────────────────────────────────────────────────────┐
│                 网络访问控制多层实现                           │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第一层: Seatbelt 网络规则 (macOS)                     │   │
│  │  (deny network*) 或 (allow network*)                  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第二层: seccomp 过滤 (Linux)                           │   │
│  │  阻止 socket/connect/bind 系统调用                     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第三层: NetworkProxy                                   │   │
│  │  - 应用层网络代理                                      │   │
│  │  - 审计所有网络请求                                    │   │
│  │  - 支持域名白名单/黑名单                               │   │
│  │  - 记录网络访问日志                                    │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  默认行为:                                                   │
│  read-only 模式 -> 网络完全禁止                               │
│  workspace-write 模式 -> 网络完全禁止                         │
│  danger-full-access 模式 -> 网络完全允许                       │
└─────────────────────────────────────────────────────────────┘
```

### Guardian 安全守护系统

#### 审批策略

Guardian 系统通过 `AskForApproval` 枚举定义四级审批策略：

```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub enum AskForApproval {
    /// 不信任模式 -- 所有命令都需要用户审批
    Untrusted,

    /// 失败时审批 -- 仅当命令失败时需要审批
    OnFailure,

    /// 按需审批 -- 模型认为需要时请求审批
    OnRequest,

    /// 从不审批 -- 所有命令自动执行（危险）
    Never,
}
```

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| `untrusted` | 所有命令都需要用户确认 | 安全敏感环境 |
| `on-failure` | 命令失败时才需要确认 | 开发环境 |
| `on-request` | 模型自行判断是否需要确认 | 日常使用 |
| `never` | 所有命令自动执行 | CI/CD、容器环境 |

#### canAutoApprove 判断逻辑

Guardian 的核心判断函数 `canAutoApprove` 维护一个已知安全命令的白名单：

```rust
fn is_known_safe_command(command: &str) -> bool {
    // 完全匹配的安全命令
    const SAFE_COMMANDS: &[&str] = &[
        "cat", "cd", "echo", "grep", "ls", "pwd", "wc",
    ];

    // Git 安全子命令
    const SAFE_GIT_COMMANDS: &[&str] = &[
        "git status", "git log", "git diff", "git show",
        "git branch", "git stash list",
    ];

    // 受限的 find 命令（仅允许 -name、-type 参数）
    // 受限的 ripgrep 命令（仅允许基本搜索参数）

    // 复合命令解析（bash -lc "..."）
    if command.starts_with("bash -lc") {
        // 解析内层命令并递归检查
    }

    // ... 白名单匹配逻辑
}
```

**复合命令解析：**

```rust
/// 解析复合命令，提取实际执行的命令
fn parse_command(command: &str) -> ParsedCommand {
    // 处理 bash -lc "..." 格式
    if command.starts_with("bash -lc") {
        if let Some(inner) = extract_quoted_arg(command) {
            return parse_command(inner);
        }
    }

    // 处理管道命令 -- 取第一个命令
    if let Some(first) = command.split('|').next() {
        return ParsedCommand {
            primary: first.trim().to_string(),
            has_pipe: true,
        };
    }

    ParsedCommand {
        primary: command.to_string(),
        has_pipe: false,
    }
}
```

#### 用户确认 UI

当命令需要用户审批时，TUI 层通过 `BottomPane` 的 `view_stack` 显示审批覆盖层：

```
┌──────────────────────────────────────────────────────────────┐
│  TUI 主界面                                                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ 聊天消息区域                                            │  │
│  │ ...                                                    │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ BottomPane (view_stack)                                │  │
│  │ ┌──────────────────────────────────────────────────┐  │  │
│  │ │ ApprovalOverlay                                  │  │  │
│  │ │                                                   │  │  │
│  │ │  命令需要审批                                     │  │  │
│  │ │                                                   │  │  │
│  │ │  命令: rm -rf /tmp/build                          │  │  │
│  │ │  工作目录: /home/user/project                      │  │  │
│  │ │                                                   │  │  │
│  │ │  [y] 允许  [n] 拒绝  [a] 始终允许此类命令          │  │  │
│  │ └──────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

用户确认后，通过 `Op::ExecApproval` 发送审批决策：

```rust
Op::ExecApproval {
    id: "approval_request_id".to_string(),
    approved: true,  // 或 false
}
```

---

## 对比分析

### 安全架构范式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **安全模型** | 软件层权限治理（应用层） | OS 内核级沙箱（系统层） |
| **隔离粒度** | 工具调用级别 | 进程/系统调用级别 |
| **执行前检查** | 7 层权限管道 | SBPL/seccomp/Landlock 规则 |
| **执行时保护** | 无（依赖执行前检查） | OS 内核强制执行 |
| **权限决策者** | LLM 分类器 + 人类用户 | OS 内核 + 白名单 + 人类用户 |
| **远程控制** | GrowthBook/Statsig 特性门 | 无（本地策略决定） |
| **跨平台** | 统一 TypeScript 实现 | 平台特定实现（Seatbelt/Landlock/Restricted Token） |

### 权限模式对比

| Claude Code 模式 | Codex CLI 等价 | 差异说明 |
|------------------|----------------|----------|
| `default` (`>`) | `untrusted` | 两者都要求所有非只读操作需用户确认 |
| `acceptEdits` (`>>`) | `on-request` | Claude Code 仅限 cwd 内编辑；Codex CLI 由模型自行判断 |
| `plan` (`?`) | 无直接等价 | Claude Code 独有的暂停审查模式 |
| `auto` (`A`) | `on-failure` | Claude Code 用 LLM 分类器；Codex CLI 仅在失败时审批 |
| `bypassPermissions` (`!`) | `never` | 两者都是危险的全自动模式 |

### 攻击面分析

| 攻击面 | Claude Code | Codex CLI |
|--------|-------------|-----------|
| **恶意工具调用绕过** | 可能（软件层检查可被绕过） | 极低（OS 内核强制执行） |
| **Shell 注入** | 依赖危险命令模式匹配 | seccomp 系统调用过滤 + 命名空间隔离 |
| **文件系统越权** | 依赖路径检查和规则匹配 | Landlock/Seatbelt 内核级文件系统隔离 |
| **网络越权** | 无独立网络控制 | 多层网络控制（Seatbelt/seccomp/NetworkProxy） |
| **LLM 分类器被欺骗** | 可能（prompt injection） | 不适用（不依赖 LLM 做安全决策） |
| **远程策略下发** | GrowthBook CDN 轮询 | 无远程策略通道 |

### 安全保证层级

```
Claude Code 安全保证链:
  应用层规则 -> LLM 分类器 -> 人类确认 -> 执行
  [软件层]    [AI 层]       [人类层]    [无保护]

Codex CLI 安全保证链:
  Guardian 白名单 -> 人类确认 -> OS 沙箱 -> 执行
  [应用层]         [人类层]    [内核层]    [内核保护]
```

Claude Code 的安全保证在执行阶段终止，即一旦通过权限检查，命令在 OS 层面没有任何额外限制。而 Codex CLI 即使通过了所有应用层检查，OS 沙箱仍然在执行阶段提供内核级的强制隔离。

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| **灵活性极高**：6 种权限模式覆盖从严格到宽松的各种场景，用户可按需切换 | **安全性依赖应用层**：所有安全检查在软件层完成，一旦被绕过则无内核级保护 |
| **LLM 智能分类**：auto 模式使用 Claude 3 Haiku 进行语义级安全判断，能理解上下文意图 | **LLM 分类器可被欺骗**：prompt injection 可能导致分类器误判，存在安全风险 |
| **远程紧急响应**：通过 GrowthBook/Statsig 可实时禁用危险模式（如 bypass），快速响应安全事件 | **无 OS 级隔离**：命令执行后不受沙箱保护，恶意命令可能影响整个系统 |
| **丰富的规则系统**：支持 Deny/Allow/Ask 三种规则动作，可按工具名、路径模式等灵活配置 | **网络控制薄弱**：没有独立的网络访问控制层，无法阻止工具执行时的网络请求 |
| **人类始终在环**：default 模式下所有非只读操作都需要人类确认，安全意识强 | **bypass 模式风险高**：`!` 模式跳过所有检查，一旦启用则完全失去保护 |
| **断路器机制**：连续拒绝计数器自动回退到更保守模式，防止 auto 模式失控 | **危险命令检测有限**：基于模式匹配，可能遗漏变种命令 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| **OS 内核级安全保证**：Seatbelt/Landlock/seccomp 在内核层面强制执行，即使应用层被绕过也无法越权 | **功能受限**：沙箱严格限制文件系统和网络访问，某些合法操作可能被阻止 |
| **跨平台沙箱**：macOS (Seatbelt)、Linux (Landlock+Bubblewrap)、Windows (Restricted Token) 全平台覆盖 | **Linux 内核版本要求**：Landlock 需要内核 5.13+，旧系统无法使用最优沙箱 |
| **多层网络控制**：Seatbelt 网络规则 + seccomp 系统调用过滤 + NetworkProxy 应用层代理，三重保障 | **审批策略粒度较粗**：4 级审批策略相比 Claude Code 的 6 种模式灵活性不足 |
| **白名单机制简单可靠**：`is_known_safe_command` 基于确定性匹配，不存在 LLM 被欺骗的风险 | **无 LLM 智能判断**：不能像 Claude Code 的 auto 模式那样理解上下文语义 |
| **提示词注入沙箱策略**：将权限约束注入系统提示词，减少模型尝试被阻止操作的无用 API 调用 | **无远程紧急响应**：没有类似 GrowthBook 的远程策略下发能力，无法实时调整安全策略 |
| **进程级隔离**：Bubblewrap 提供完整的命名空间隔离（mount/IPC/network/PID），安全性极高 | **白名单维护成本**：安全命令白名单需要手动维护，新增安全命令需要更新代码 |
| **特殊目录保护**：`.git/`、`.codex/` 等目录在沙箱层面被保护，防止版本控制数据被破坏 | **danger-full-access 模式风险**：该模式允许网络访问和完整文件系统权限，安全保证大幅降低 |
# 多 Agent 系统对比

## Claude Code 实现

Claude Code 支持**三个级别的多 Agent 执行**，从简单的子 Agent 到复杂的持久化团队，形成完整的层级体系。

### 三级架构

```
┌────────────────────────────────────────────────────────────────┐
│  级别 1: 子 Agent (AgentTool)                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ 主 Agent 通过 AgentTool 生成子 Agent                      │  │
│  │  * 隔离的文件缓存（从父 Agent 克隆）                      │  │
│  │  * 独立的 AbortController                                 │  │
│  │  * 独立的转录记录 (JSONL 侧链)                            │  │
│  │  * 过滤的工具池（按 Agent 定义）                          │  │
│  │  * 以文本形式返回结果给父 Agent                           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  级别 2: 协调器模式 (多 Worker)                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ CLAUDE_CODE_COORDINATOR_MODE=1                           │  │
│  │  * 系统提示重写为编排模式                                  │  │
│  │  * 通过 AgentTool 生成受限工具的 Worker                   │  │
│  │  * XML task-notification 协议传递结果                    │  │
│  │  * 协调器聚合并响应用户                                   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  级别 3: 团队模式 (持久化团队)                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ TeamCreateTool 创建命名团队                               │  │
│  │  * 团队文件持久化到 ~/.claude/teams/{name}.json           │  │
│  │  * InProcessTeammates 在同一进程中运行                    │  │
│  │  * SendMessageTool 在队友间路由消息                       │  │
│  │  * 共享 scratchpad 文件系统进行知识交换                    │  │
│  │  * 结构化关闭协议 (request -> approve)                     │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### 级别 1：子 Agent（AgentTool）

子 Agent 是最基础的多 Agent 单元，由主 Agent 通过 `AgentTool` 动态生成：

- **隔离机制**：每个子 Agent 拥有从父 Agent 克隆的独立文件缓存，确保子 Agent 的文件操作不影响父 Agent 的视图
- **生命周期管理**：独立的 `AbortController` 允许父 Agent 单独中止某个子 Agent
- **审计追踪**：每个子 Agent 维护独立的 JSONL 转录记录（侧链），便于事后审计
- **工具池过滤**：子 Agent 的可用工具可以按定义进行过滤，限制其能力范围
- **结果传递**：子 Agent 的执行结果以文本形式返回给父 Agent，由父 Agent 决定后续操作

### 级别 2：协调器模式

协调器模式通过环境变量 `CLAUDE_CODE_COORDINATOR_MODE=1` 激活：

- **系统提示重写**：激活后，系统提示词被重写为编排模式，主 Agent 变为协调器角色
- **Worker 生成**：协调器通过 AgentTool 生成多个 Worker，每个 Worker 拥有受限的工具集
- **XML 通信协议**：Worker 之间通过 `task-notification` XML 协议传递执行结果
- **结果聚合**：协调器负责收集所有 Worker 的结果，进行聚合后统一响应用户

### 级别 3：团队模式

团队模式是最复杂的多 Agent 形态，支持持久化的团队协作：

- **持久化**：团队配置持久化到 `~/.claude/teams/{name}.json`，跨会话存活
- **进程内运行**：`InProcessTeammates` 在同一进程中运行，共享内存空间
- **消息路由**：`SendMessageTool` 在队友之间路由消息，支持点对点通信
- **知识交换**：共享 scratchpad 文件系统，团队成员可以通过文件交换知识
- **结构化关闭**：采用 `request -> approve` 的结构化关闭协议，确保优雅退出

### Fork 缓存优化

当从同一上下文生成多个 Agent 时，Claude Code 使用 **Fork 机制**最大化 API 提示缓存命中：

- 所有子 Agent 共享相同的前缀（父对话历史）
- 仅最后一条指令不同
- Fork 操作在 API 层面复用已缓存的 prompt 前缀，大幅降低 token 成本和延迟

---

## Codex CLI 实现

Codex CLI 支持完整的多 Agent 编排系统，每个 Agent 运行在独立的沙箱容器中。

### 核心原语

| 原语 | 说明 |
|------|------|
| `spawn` | 创建新的 Agent 实例 |
| `resume` | 恢复已暂停的 Agent |
| `wait` | 等待 Agent 完成 |
| `close` | 终止 Agent |
| `send-message` | 向 Agent 发送消息 |
| `list` | 列出所有 Agent |
| `assign-task` | 向 Agent 分配任务 |

### 批量 Agent 任务

Codex CLI 支持从 CSV 文件批量创建 Agent 任务，每个 CSV 行定义一个独立的 Agent 任务，所有 Agent 并行运行：

```rust
/// 从 CSV 文件批量创建 Agent 任务
/// 每个 CSV 行定义一个独立的 Agent 任务
/// 所有 Agent 并行运行，各自拥有独立的沙箱环境
async fn spawn_agents_on_csv(
    csv_path: &Path,
    task_template: &str,
    sandbox_policy: SandboxPolicy,
) -> Vec<AgentHandle> {
    let tasks = parse_csv(csv_path).await?;
    let mut handles = Vec::new();

    for task in tasks {
        let handle = spawn_agent(
            task.prompt,
            task.cwd,
            sandbox_policy.clone(),
        ).await?;
        handles.push(handle);
    }

    handles
}
```

**关键设计要点：**
- **独立沙箱**：每个 Agent 拥有独立的沙箱环境（`sandbox_policy.clone()`），实现进程级隔离
- **并行执行**：所有 Agent 并行运行，充分利用系统资源
- **CSV 驱动**：通过 CSV 文件定义任务，便于批量处理和自动化

### Guardian 守护者模式

Guardian 作为安全中间层，在多 Agent 场景中决定哪些操作可以自动执行：

```
┌──────────────────────────────────────────────────────────────┐
│                 Guardian 守护者模式                            │
│                                                               │
│  Agent 请求                                                   │
│      │                                                        │
│      v                                                        │
│  ┌──────────────┐                                            │
│  │ Guardian     │                                            │
│  │ 评估请求      │                                            │
│  │              │                                            │
│  │ 1. 检查命令白名单                                           │
│  │ 2. 评估风险等级                                             │
│  │ 3. 检查审批策略                                             │
│  └──────┬───────┘                                            │
│         │                                                     │
│    ┌────┴────┐                                               │
│    v         v                                                │
│  自动执行   请求用户审批                                       │
│    │         │                                                │
│    v         v                                                │
│  沙箱执行   ApprovalOverlay                                   │
│              │                                                │
│              v                                                │
│         用户确认/拒绝                                          │
└──────────────────────────────────────────────────────────────┘
```

**Guardian 在多 Agent 中的角色：**
- 每个 Agent 的操作请求都经过 Guardian 评估
- Guardian 根据审批策略（`AskForApproval`）决定是否需要用户介入
- 白名单内的安全命令自动放行，减少用户审批疲劳
- 非白名单命令根据策略决定是否请求用户审批

---

## 对比分析

### 架构范式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **架构模型** | 三级递进架构（子 Agent -> 协调器 -> 团队） | 扁平原语 + Guardian 守护者 |
| **隔离机制** | 应用层隔离（独立缓存、AbortController） | OS 内核级隔离（独立沙箱容器） |
| **通信方式** | 文本返回 / XML 协议 / 消息路由 | 原语调用（spawn/send-message/wait） |
| **持久化** | 团队文件持久化（JSON） | 会话 Rollout 持久化（JSONL + SQLite） |
| **批量执行** | Fork 机制（共享前缀缓存） | CSV 批量生成（独立沙箱） |
| **安全层** | 工具池过滤 + 权限管道 | Guardian 白名单 + OS 沙箱 |

### Agent 生命周期管理对比

| 操作 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **创建** | `AgentTool` 调用 | `spawn` 原语 |
| **暂停/恢复** | `AbortController` 中止 | `resume` 原语 |
| **终止** | AbortController 中止 | `close` 原语 |
| **等待完成** | Promise 链 | `wait` 原语 |
| **消息传递** | `SendMessageTool` / XML 协议 | `send-message` 原语 |
| **任务分配** | 协调器模式 / 团队模式 | `assign-task` 原语 |
| **列表查看** | 团队文件读取 | `list` 原语 |

### 隔离机制深度对比

```
Claude Code 隔离层级:
  子 Agent
  ├── 独立文件缓存（应用层）
  ├── 独立 AbortController（应用层）
  ├── 独立 JSONL 转录（应用层）
  └── 过滤的工具池（应用层）
  [全部在同一个 Node.js 进程中运行]

Codex CLI 隔离层级:
  Agent
  ├── 独立沙箱容器（OS 内核层）
  │   ├── Landlock 文件系统隔离
  │   ├── seccomp 系统调用过滤
  │   └── Bubblewrap 命名空间隔离
  ├── Guardian 审批（应用层）
  └── 独立 Rollout 记录（应用层）
  [运行在独立进程/命名空间中]
```

### 通信模型对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **父子通信** | 文本返回（子 -> 父） | 原语调用（双向） |
| **兄弟通信** | XML task-notification 协议 | send-message 原语 |
| **团队通信** | SendMessageTool 路由 | send-message 原语 |
| **知识共享** | 共享 scratchpad 文件系统 | 独立沙箱（无共享） |
| **结果聚合** | 协调器聚合 | 父 Agent 等待收集 |

### 规模化能力对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **批量创建** | Fork 机制（手动） | `spawn_agents_on_csv`（自动化） |
| **并行度** | 受限于单进程内存 | 受限于系统资源（独立进程） |
| **资源隔离** | 共享进程资源 | 独立进程资源 |
| **故障隔离** | 子 Agent 崩溃可能影响父进程 | Agent 崩溃不影响其他 Agent |
| **API 缓存优化** | Fork 共享前缀缓存 | 无 API 缓存优化 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| **三级架构灵活**：从简单的子 Agent 到复杂的持久化团队，按需选择合适的复杂度级别 | **隔离深度不足**：所有 Agent 运行在同一 Node.js 进程中，子 Agent 崩溃可能影响父进程 |
| **Fork 缓存优化**：多个子 Agent 共享 API 提示前缀缓存，大幅降低 token 成本和延迟 | **无 OS 级隔离**：Agent 之间仅靠应用层隔离，无法防止恶意 Agent 影响系统 |
| **团队持久化**：团队配置持久化到磁盘，跨会话存活，支持长期协作场景 | **进程内运行限制**：InProcessTeammates 共享进程资源，大规模并行时可能成为瓶颈 |
| **丰富的通信方式**：支持文本返回、XML 协议、消息路由等多种通信模式 | **知识共享风险**：共享 scratchpad 文件系统可能导致 Agent 间意外干扰 |
| **协调器模式**：专门的编排模式，支持 Worker 结果聚合和统一响应 | **批量创建不够自动化**：相比 CSV 驱动的批量创建，Fork 机制更依赖手动编排 |
| **结构化关闭协议**：团队模式采用 request -> approve 协议，确保优雅退出 | **XML 协议复杂度**：task-notification XML 协议增加了通信复杂度 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| **OS 级隔离**：每个 Agent 运行在独立沙箱容器中，进程级隔离保证故障不扩散 | **通信原语较基础**：spawn/send-message/wait 等原语功能单一，复杂编排需要额外逻辑 |
| **批量自动化**：`spawn_agents_on_csv` 从 CSV 文件批量创建 Agent，适合大规模并行任务 | **无 API 缓存优化**：每个 Agent 独立发起 API 请求，无法共享提示缓存 |
| **Guardian 统一安全**：所有 Agent 的操作都经过 Guardian 评估，安全策略一致 | **无持久化团队**：Agent 生命周期与会话绑定，没有跨会话的团队持久化机制 |
| **故障完全隔离**：Agent 崩溃在沙箱内，不影响其他 Agent 或主进程 | **无知识共享机制**：独立沙箱意味着 Agent 之间无法直接共享文件或状态 |
| **原语简洁**：7 个核心原语覆盖完整的 Agent 生命周期，API 设计清晰 | **无协调器角色**：缺少专门的协调器模式，复杂编排需要用户自行实现 |
| **独立 Rollout 记录**：每个 Agent 有独立的会话记录，便于独立审计和调试 | **Guardian 白名单局限**：基于确定性匹配的白名单可能过于严格，限制 Agent 自主性 |
# 配置系统与状态管理对比

## Claude Code 实现

### CLAUDE.md 加载优先级和合并策略

CLAUDE.md 文件是 Claude Code 的项目级指令系统，支持 4 级目录层级和惰性加载机制：

```
┌─────────────────────────────────────────────────────────────────┐
│                 CLAUDE.md 4 级目录层级                           │
│                                                                 │
│  优先级    位置                              作用域              │
│  ──────    ────                              ────              │
│  最高      /etc/claude-code/CLAUDE.md       系统级（管理员）     │
│  高        ~/.claude/CLAUDE.md              用户级（全局）       │
│  中        ./.claude/CLAUDE.md              项目级（团队共享）   │
│  中        ./.claude/rules/*.md             项目规则（按文件匹配）│
│  低        ./CLAUDE.md                      项目级（本地）       │
│  最低      ./CLAUDE.local.md                本地级（不提交 Git） │
│                                                                 │
│  惰性加载机制：                                                 │
│  1. 启动时仅加载系统级和用户级 CLAUDE.md                        │
│  2. 项目级 CLAUDE.md 在进入项目目录时加载                      │
│  3. rules/*.md 按文件路径匹配条件加载                          │
│  4. CLAUDE.local.md 仅在本地存在时加载                          │
│  5. 文件变更通过 settingsChangeDetector + debounce 触发重新加载 │
└─────────────────────────────────────────────────────────────────┘
```

```typescript
// CLAUDE.md 加载逻辑（简化示意）
async function loadClaudeMdFiles(cwd: string): Promise<ClaudeMdContent[]> {
  const files: ClaudeMdContent[] = [];

  // 1. 系统级
  const systemPath = '/etc/claude-code/CLAUDE.md';
  if (await exists(systemPath)) {
    files.push({ path: systemPath, content: await readFile(systemPath), priority: 0 });
  }

  // 2. 用户级
  const userPath = path.join(os.homedir(), '.claude', 'CLAUDE.md');
  if (await exists(userPath)) {
    files.push({ path: userPath, content: await readFile(userPath), priority: 1 });
  }

  // 3. 项目级
  const projectPaths = [
    path.join(cwd, '.claude', 'CLAUDE.md'),
    path.join(cwd, 'CLAUDE.md'),
  ];
  for (const p of projectPaths) {
    if (await exists(p)) {
      files.push({ path: p, content: await readFile(p), priority: 2 });
    }
  }

  // 4. 项目规则（按路径匹配）
  const rulesDir = path.join(cwd, '.claude', 'rules');
  if (await exists(rulesDir)) {
    const ruleFiles = await readdir(rulesDir);
    for (const ruleFile of ruleFiles.filter(f => f.endsWith('.md'))) {
      files.push({
        path: path.join(rulesDir, ruleFile),
        content: await readFile(path.join(rulesDir, ruleFile)),
        priority: 2,
        isRule: true,
      });
    }
  }

  // 5. 本地级
  const localPath = path.join(cwd, 'CLAUDE.local.md');
  if (await exists(localPath)) {
    files.push({ path: localPath, content: await readFile(localPath), priority: 3 });
  }

  return files.sort((a, b) => a.priority - b.priority);
}
```

### settings.json 5 级优先级

```
┌─────────────────────────────────────────────────────────────────┐
│                 settings.json 5 级优先级                         │
│                                                                 │
│  优先级（从低到高）                                             │
│                                                                 │
│  1. flagSettings      特性标志默认值                            │
│     来源：代码内硬编码的默认值                                  │
│                                                                 │
│  2. localSettings     本地设置                                  │
│     来源：.claude/settings.local.json                           │
│     用途：开发者个人偏好，不提交到 Git                          │
│                                                                 │
│  3. projectSettings   项目设置                                  │
│     来源：.claude/settings.json                                 │
│     用途：团队共享的项目配置，提交到 Git                        │
│                                                                 │
│  4. userSettings      用户设置                                  │
│     来源：~/.claude/settings.json                               │
│     用途：用户全局偏好，跨项目生效                              │
│                                                                 │
│  5. policySettings    策略设置（最高优先级）                     │
│     来源：MDM 策略文件                                          │
│     用途：企业管理员强制策略，不可被用户覆盖                    │
│                                                                 │
│  合并策略：mergeWith + settingsMergeCustomizer                  │
│  - 数组字段：高优先级替换低优先级（非追加）                     │
│  - 对象字段：深度合并                                          │
│  - 布尔字段：高优先级覆盖低优先级                               │
│                                                                 │
│  热重载：settingsChangeDetector + debounce                      │
│  - 文件监视器检测 settings.json 变更                            │
│  - debounce 300ms 避免频繁重载                                  │
│  - 重载后触发 ConfigChange Hook                                │
└─────────────────────────────────────────────────────────────────┘
```

```typescript
// settings.ts -- 合并策略
import { mergeWith } from 'lodash';
import { settingsMergeCustomizer } from './settingsMerge';

function settingsMergeCustomizer(objValue: any, srcValue: any, key: string) {
  // 数组字段：替换而非追加
  if (Array.isArray(objValue) && Array.isArray(srcValue)) {
    return srcValue;
  }
  // 其他情况：使用 lodash 默认的深度合并
}

export async function loadSettings(): Promise<Settings> {
  const [flagSettings, localSettings, projectSettings,
         userSettings, policySettings] = await Promise.all([
    loadFeatureFlagSettings(),
    loadLocalSettings(),
    loadProjectSettings(),
    loadUserSettings(),
    loadPolicySettings(),
  ]);

  return mergeWith(
    {}, flagSettings, localSettings, projectSettings,
    userSettings, policySettings, settingsMergeCustomizer,
  );
}
```

### GrowthBook 特性标志系统

```
┌─────────────────────────────────────────────────────────────────┐
│                 GrowthBook 特性标志系统                          │
│                                                                 │
│  初始化属性：                                                   │
│  {                                                              │
│    organizationUUID: string,    // 组织 ID                     │
│    accountUUID: string,         // 账户 ID                     │
│    email: string,               // 用户邮箱                    │
│    platform: string,            // 操作系统 (darwin/linux/win32)│
│    claudeCodeVersion: string,   // Claude Code 版本号          │
│  }                                                              │
│                                                                 │
│  编译时 DCE vs 运行时远程配置轮询：                             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 编译时 (bun:bundle feature())                           │   │
│  │  - 构建时完全剥离未激活的代码分支                        │   │
│  │  - 减小打包体积（如 VOICE_MODE ~200KB）                 │   │
│  │  - 适用于永久性功能开关                                  │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │ 运行时 (GrowthBook.isOn())                              │   │
│  │  - 从 CDN 轮询最新配置                                  │   │
│  │  - 支持实时开关（无需重新构建）                          │   │
│  │  - 支持 A/B 测试和灰度发布                               │   │
│  │  - 适用于需要灵活控制的开关                              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Feature flag 随机词对命名（防止猜测）：                        │
│  - tengu_frond_boric        -> 禁用 bypass 权限                │
│  - alpine_ripple_flux       -> 控制 auto 模式可用性            │
│  - cobalt_glade_sprout      -> 会话镜像功能开关                │
│  - ember_pine_quartz        -> 协调器模式开关                  │
│                                                                 │
│  关键远程控制能力：                                             │
│  1. 禁用 bypass 权限（安全紧急响应）                           │
│  2. 控制 auto 模式可用性（安全策略调整）                       │
│  3. 会话镜像（企业审计）                                       │
│  4. 功能灰度发布（按用户百分比）                               │
│  5. A/B 测试（不同配置的性能对比）                             │
└─────────────────────────────────────────────────────────────────┘
```

### Bootstrap State 完整结构

```typescript
export interface BootstrapState {
  // --- 目录与会话 ---
  cwd: string;                    // 当前工作目录
  homeDir: string;                // 用户主目录
  sessionId: string;              // 会话唯一 ID
  conversationId: string;         // 对话 ID

  // --- 成本追踪 ---
  totalCost: number;              // 累计成本（美元）
  totalInputTokens: number;       // 累计输入 token 数
  totalOutputTokens: number;      // 累计输出 token 数
  totalCacheReadTokens: number;   // 累计缓存读取 token 数
  totalCacheCreationTokens: number; // 累计缓存创建 token 数

  // --- 性能指标 ---
  startupTime: number;            // 启动耗时（ms）
  apiLatencyP50: number;          // API 延迟 P50（ms）
  apiLatencyP99: number;          // API 延迟 P99（ms）
  toolExecutionCount: number;     // 工具执行总次数
  toolExecutionErrors: number;    // 工具执行错误次数

  // --- 认证安全 ---
  authToken: string | null;       // OAuth token
  authMethod: 'oauth' | 'api_key' | 'bare' | null;
  mdmPolicy: MDMPolicy | null;    // MDM 策略

  // --- 遥测 ---
  otelExporter: OTelExporter | null;
  statsigClient: StatsigClient | null;
  growthBook: GrowthBook | null;

  // --- Hooks ---
  hooks: Map<HookEvent, Hook[]>;

  // --- 特性标志 ---
  featureFlags: Map<string, boolean>;

  // --- 设置 ---
  settings: Settings;
}
```

### QueryEngine 状态管理

```typescript
interface QueryEngineState {
  // 预算执行
  tokenBudget: TokenBudget;
  estimatedTokens: number;

  // 重试状态
  retryCount: number;
  lastRetryError: Error | null;
  nextRetryAt: number;

  // 权限状态
  permissionMode: PermissionMode;
  deniedCount: number;
  consecutiveDenials: number;
  lastDeniedTool: string | null;

  // 会话统计
  messageCount: number;
  toolCallCount: number;
  compactCount: number;
}
```

### React Context 使用方式

Claude Code 使用自定义的极简 Store 实现替代 Zustand，通过 React Context 注入：

```typescript
// store.ts -- createStore (~20行)
type Listener<T> = (state: T) => void;

export function createStore<T extends object>(initialState: T) {
  let state = initialState;
  const listeners = new Set<Listener<T>>();

  return {
    getState: () => state,
    setState: (partial: Partial<T>) => {
      state = { ...state, ...partial };
      listeners.forEach(listener => listener(state));
    },
    subscribe: (listener: Listener<T>) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
  };
}

// AppStateProvider -- 防嵌套机制
export function AppStateProvider({ children }: { children: React.ReactNode }) {
  const storeRef = useRef<AppStore | null>(null);

  if (!storeRef.current) {
    storeRef.current = createStore<AppState>(defaultAppState);
  }

  // 防嵌套：如果已经存在 Provider，直接复用
  const existingStore = useContext(AppStoreContext);
  if (existingStore) {
    return <>{children}</>;
  }

  return (
    <AppStoreContext.Provider value={storeRef.current}>
      {children}
    </AppStoreContext.Provider>
  );
}
```

**核心 Context 列表**：

| Context | 用途 | 生命周期 |
|---------|------|----------|
| `AppStoreContext` | 全局应用状态（消息、权限、成本） | 会话级 |
| `NotificationsContext` | 通知队列和显示 | 会话级 |
| `StatsContext` | 会话统计信息 | 会话级 |
| `ModalContext` | 模态框状态管理 | 会话级 |
| `OverlayContext` | 覆盖层（diff viewer 等） | 会话级 |
| `VoiceContext` | 语音输入/输出状态 | 会话级 |
| `MailboxContext` | 多 Agent 消息路由 | 会话级 |

---

## Codex CLI 实现

### 配置文件发现与加载

Codex CLI 使用 **5 级优先级** 配置系统：

```
优先级从高到低:
┌──────────────────────────────────────────────────────────────┐
│  1. 环境变量 (CODEX_*)                                       │
│     CODEX_MODEL=gpt-4o                                       │
│     CODEX_SANDBOX=workspace-write                             │
│     OPENAI_API_KEY=sk-...                                    │
│                                                               │
│  2. CLI 标志 (--model, --sandbox, --approval-policy)          │
│     codex --model gpt-4o --sandbox workspace-write            │
│                                                               │
│  3. Profile (codex --profile <name>)                         │
│     ~/.codex/profiles/<name>.toml                             │
│                                                               │
│  4. 全局 config.toml                                          │
│     ~/.codex/config.toml                                      │
│                                                               │
│  5. 内置默认值                                                │
│     model = "o4-mini"                                         │
│     sandbox_mode = "read-only"                                │
│     approval_policy = "on-request"                            │
└──────────────────────────────────────────────────────────────┘
```

**ConfigLayerStack 合并：**

```rust
/// 配置层栈 -- 按优先级从低到高合并
pub struct ConfigLayerStack {
    layers: Vec<ConfigLayer>,
}

pub struct ConfigLayer {
    pub name: ConfigLayerSource,
    pub config: TomlValue,
    pub is_disabled: bool,
}

pub enum ConfigLayerSource {
    /// 内置默认值
    Builtin,
    /// 全局配置文件
    Global { path: PathBuf },
    /// 项目配置文件
    Project { path: PathBuf },
    /// 环境变量
    Environment,
    /// CLI 标志
    CliFlags,
    /// Profile
    Profile { name: String },
}
```

### AGENTS.md / codex.md 发现机制

核心实现在 `codex-rs/core/src/project_doc.rs`（315 行）。

**发现算法（4 步）：**

```
步骤 1: 确定项目根目录
┌──────────────────────────────────────────────────────────────┐
│  从当前工作目录向上遍历，查找 project_root_markers            │
│  默认标记: [".git"]                                          │
│  可配置: project_root_markers = [".git", "package.json"]     │
│                                                               │
│  /home/user/project/src/  <- cwd                              │
│  /home/user/project/      <- 找到 .git，确定为项目根          │
│  /home/user/                                                   │
│  /home/                                                       │
│  /                                                            │
└──────────────────────────────────────────────────────────────┘

步骤 2: 从项目根到 cwd 收集 AGENTS.md
┌──────────────────────────────────────────────────────────────┐
│  搜索路径（从根到 cwd）:                                      │
│  /home/user/project/AGENTS.md          <- 项目级指令          │
│  /home/user/project/src/AGENTS.md      <- 子目录指令          │
│                                                               │
│  候选文件名（按优先级）:                                       │
│  1. AGENTS.override.md  (本地覆盖，最高优先级)                │
│  2. AGENTS.md           (标准文件名)                          │
│  3. codex.md            (备用文件名，可配置)                   │
└──────────────────────────────────────────────────────────────┘

步骤 3: 读取并拼接内容
┌──────────────────────────────────────────────────────────────┐
│  大小限制: 32 KiB (project_doc_max_bytes)                     │
│  如果总大小超过限制，从最早发现的文件开始截断                   │
│  多个文件之间用 "\n\n" 分隔拼接                                │
└──────────────────────────────────────────────────────────────┘

步骤 4: 组装用户指令
┌──────────────────────────────────────────────────────────────┐
│  最终指令 = Config::instructions                              │
│            + "\n\n--- project-doc ---\n\n"                    │
│            + AGENTS.md 内容                                    │
│            + JS REPL 指令（如果启用）                           │
│            + 子 Agent 指令（如果启用）                          │
└──────────────────────────────────────────────────────────────┘
```

### 配置文件格式

完整的 TOML 配置文件示例：

```toml
# ~/.codex/config.toml

# 模型配置
model = "o4-mini"
model_provider = "openai"

# 沙箱模式
sandbox_mode = "workspace-write"

# 审批策略
approval_policy = "on-request"

# 模型提供商配置
[model_provider]
name = "openai"

# MCP 服务器配置
[mcp_servers]
my-server = { command = "npx", args = ["-y", "@my/mcp-server"] }
another-server = { command = "python", args = ["-m", "my_mcp"] }

# 项目文档配置
project_doc_max_bytes = 32768
project_doc_fallback_filenames = ["codex.md", "CODEX.md"]
project_root_markers = [".git", "package.json", "Cargo.toml"]

# 需求配置
[requirements]
# 定义项目特定的约束条件

# OpenTelemetry 配置
[otel]
enabled = true
endpoint = "http://localhost:4318"
```

### 模型路由

**ModelProviderInfo 注册表：**

```rust
/// 模型提供商信息
pub struct ModelProviderInfo {
    /// 提供商标识符
    pub id: String,

    /// 提供商名称
    pub name: String,

    /// 基础指令模板
    pub base_instructions: BaseInstructions,

    /// 支持的模型列表
    pub models: Vec<ModelInfo>,

    /// 是否为 OpenAI 提供商
    pub is_openai: bool,

    /// 流式请求最大重试次数
    pub stream_max_retries: usize,
}
```

**认证方式路由：**

| 认证方式 | 说明 | 配置 |
|----------|------|------|
| **ChatGPT OAuth** | 通过浏览器登录 ChatGPT 账户 | `codex login` |
| **API Key** | 直接使用 OpenAI API Key | `codex login --api-key` 或 `OPENAI_API_KEY` |
| **本地模型** | Ollama / LM Studio | `model_provider = "ollama"` |
| **Azure** | Azure OpenAI Service | `model_provider = "azure"` |

### 双层存储架构

Codex CLI 使用**双层存储架构**持久化会话状态：

```
┌──────────────────────────────────────────────────────────────┐
│                 双层存储架构                                   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第一层: JSONL Rollout 文件                            │   │
│  │  ~/.codex/sessions/YYYY/MM/DD/                       │   │
│  │  rollout-2025-05-07T17-24-21-{uuid}.jsonl            │   │
│  │                                                       │   │
│  │  - 完整的会话事件流                                   │   │
│  │  - 可用 jq/fx 等工具直接查看                          │   │
│  │  - 支持会话恢复和分叉                                 │   │
│  │  - 人类可读的 JSON Lines 格式                         │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 第二层: SQLite 数据库                                 │   │
│  │  ~/.codex/state.db                                    │   │
│  │                                                       │   │
│  │  - 线程元数据索引                                     │   │
│  │  - 快速搜索和过滤                                     │   │
│  │  - 线程列表分页                                       │   │
│  │  - 从 Rollout 文件 read-repair                        │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### Rollout 文件格式

**RolloutLine 格式：**

每行是一个 JSON 对象，包含时间戳和事件类型：

```json
{"timestamp":"2025-05-07T17:24:21.123Z","item":{"SessionMeta":{...}}}
{"timestamp":"2025-05-07T17:24:22.456Z","item":{"ResponseItem":{...}}}
{"timestamp":"2025-05-07T17:24:23.789Z","item":{"EventMsg":{...}}}
{"timestamp":"2025-05-07T17:24:24.012Z","item":{"TurnContext":{...}}}
{"timestamp":"2025-05-07T17:24:25.345Z","item":{"Compacted":{...}}}
```

**RolloutItem 类型表：**

| 类型 | 说明 | 包含内容 |
|------|------|----------|
| `ResponseItem` | 模型响应项 | 文本消息、工具调用、工具结果 |
| `EventMsg` | 事件消息 | 执行命令开始/结束、审批请求等 |
| `TurnContext` | 轮次上下文 | 模型信息、沙箱策略、审批策略等 |
| `Compacted` | 压缩记录 | 压缩摘要、替换历史 |
| `SessionMeta` | 会话元数据 | 会话 ID、时间戳、cwd、Git 信息等 |

### RolloutRecorder

核心实现在 `codex-rs/rollout/src/recorder.rs`（1111 行）。

**异步写入架构：**

```rust
/// Rollout 记录器 -- 异步写入会话事件
#[derive(Clone)]
pub struct RolloutRecorder {
    /// 命令发送通道（有界队列，容量 256）
    tx: Sender<RolloutCmd>,

    /// 后台写入任务状态
    writer_task: Arc<RolloutWriterTask>,

    /// Rollout 文件路径
    pub(crate) rollout_path: PathBuf,

    /// SQLite 数据库句柄
    state_db: Option<StateDbHandle>,

    /// 事件持久化模式
    event_persistence_mode: EventPersistenceMode,
}

/// 写入命令
enum RolloutCmd {
    /// 添加事件项
    AddItems(Vec<RolloutItem>),

    /// 持久化（创建文件并写入所有缓冲项）
    Persist { ack: oneshot::Sender<std::io::Result<()>> },

    /// 刷新（将缓冲项写入已打开的文件）
    Flush { ack: oneshot::Sender<std::io::Result<()>> },

    /// 关闭（写入所有缓冲项后停止）
    Shutdown { ack: oneshot::Sender<std::io::Result<()>> },
}
```

### 会话恢复与分叉

**恢复流程：**

```
┌──────────────────────────────────────────────────────────────┐
│                 会话恢复流程                                   │
│                                                               │
│  1. 加载 Rollout 文件                                         │
│     load_rollout_items(path)                                  │
│     ├── 读取 JSONL 文件                                       │
│     ├── 逐行解析 RolloutLine                                  │
│     ├── 提取 SessionMeta 获取 thread_id                       │
│     └── 收集所有 RolloutItem                                  │
│                                                               │
│  2. 重放 RolloutItem 流                                       │
│     ├── SessionMeta -> 恢复会话元数据                          │
│     ├── ResponseItem -> 重建消息历史                           │
│     ├── TurnContext -> 恢复轮次上下文                           │
│     ├── Compacted -> 恢复压缩历史                              │
│     └── EventMsg -> 重建事件状态                               │
│                                                               │
│  3. 创建新的 Codex 实例                                       │
│     ├── 使用恢复的历史初始化 Session                          │
│     ├── RolloutRecorder 以 Resume 模式打开（追加写入）        │
│     └── 继续正常的 SQ/EQ 事件循环                             │
└──────────────────────────────────────────────────────────────┘
```

**分叉流程：**

```
┌──────────────────────────────────────────────────────────────┐
│                 会话分叉流程                                   │
│                                                               │
│  1. 加载原始 Rollout 文件                                     │
│     load_rollout_items(original_path)                         │
│                                                               │
│  2. 创建新的线程 ID                                           │
│     new_thread_id = Uuid::new_v4()                            │
│                                                               │
│  3. 继承配置                                                 │
│     ├── 模型配置                                             │
│     ├── 沙箱策略                                             │
│     ├── 审批策略                                             │
│     └── 工具配置                                             │
│                                                               │
│  4. 创建新的 Rollout 文件                                     │
│     RolloutRecorder::new(                                    │
│         RolloutRecorderParams::Create {                      │
│             conversation_id: new_thread_id,                  │
│             forked_from_id: Some(original_thread_id),        │
│             source: SessionSource::Fork,                     │
│             ...                                              │
│         }                                                    │
│     )                                                        │
│                                                               │
│  5. 不修改原始 Rollout 文件                                   │
│     原始会话保持不变，可以继续使用                             │
└──────────────────────────────────────────────────────────────┘
```

### 记忆系统

```
~/.codex/
├── sessions/                  # 会话 Rollout 文件
│   ├── 2025/
│   │   ├── 05/
│   │   │   ├── 07/
│   │   │   │   └── rollout-2025-05-07T17-24-21-{uuid}.jsonl
│   │   │   └── ...
│   │   └── ...
│   └── archived/              # 已归档会话
│       └── ...
├── state.db                   # SQLite 状态数据库
├── config.toml                # 全局配置
├── profiles/                  # 配置 Profile
│   └── dev.toml
└── memories/                  # 跨会话记忆
    └── ...
```

记忆系统通过 `memories` 模块实现跨会话知识保持。当 `generate_memories` 配置启用时，系统会在会话结束时自动提取关键信息并持久化，在后续会话中作为上下文注入。

---

## 对比分析

### 配置文件格式对比

| 维度 | Claude Code (JSON) | Codex CLI (TOML) |
|------|--------------------|--------------------|
| **格式** | JSON (settings.json) | TOML (config.toml) |
| **可读性** | 较差（大括号嵌套、需引号） | 优秀（简洁、支持注释） |
| **注释支持** | 不支持（需 JSONC 扩展） | 原生支持 `#` 注释 |
| **类型安全** | 运行时验证 | 编译时类型检查（Rust serde） |
| **合并策略** | lodash mergeWith + 自定义 customizer | ConfigLayerStack 按优先级合并 |
| **热重载** | settingsChangeDetector + debounce 300ms | 无明确热重载机制 |

### 项目指令文件对比

| 维度 | CLAUDE.md | AGENTS.md |
|------|-----------|-----------|
| **文件名** | `CLAUDE.md` / `CLAUDE.local.md` | `AGENTS.md` / `AGENTS.override.md` / `codex.md` |
| **层级数** | 4 级（系统/用户/项目/本地） | 多级（项目根到 cwd 逐级收集） |
| **加载策略** | 惰性加载（按需） | 启动时一次性加载 |
| **大小限制** | 无明确限制 | 32 KiB (project_doc_max_bytes) |
| **规则匹配** | `.claude/rules/*.md` 按文件路径匹配 | 无独立规则匹配机制 |
| **拼接方式** | 按优先级排序后合并 | 按发现顺序用 `\n\n` 分隔拼接 |
| **覆盖机制** | 高优先级覆盖低优先级 | AGENTS.override.md 覆盖 AGENTS.md |

### 配置优先级对比

| 优先级 | Claude Code | Codex CLI |
|--------|-------------|-----------|
| **最高** | MDM 策略 (policySettings) | 环境变量 (CODEX_*) |
| **高** | 用户设置 (~/.claude/settings.json) | CLI 标志 (--model, --sandbox) |
| **中** | 项目设置 (.claude/settings.json) | Profile (~/.codex/profiles/) |
| **低** | 本地设置 (.claude/settings.local.json) | 全局配置 (~/.codex/config.toml) |
| **最低** | 特性标志默认值 (flagSettings) | 内置默认值 |

### 状态管理架构对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **状态管理** | React Context + 自定义 Store | Rust Mutex + Arc 共享状态 |
| **响应式** | 发布-订阅模式（listeners） | Mutex 锁保护 |
| **Context 数量** | 7 个核心 Context | 无明确分层 |
| **线程安全** | 单线程（Node.js） | 编译时保证（Rust 所有权） |
| **状态持久化** | JSONL 转录文件 | JSONL + SQLite 双层 |
| **索引/搜索** | sessions.json 手动索引 | SQLite 自动索引 |
| **跨会话记忆** | 无内置机制 | memories 模块 |

### 热重载机制对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **配置热重载** | settingsChangeDetector + debounce 300ms | 无明确热重载 |
| **指令热重载** | debounce 触发 CLAUDE.md 重新加载 | 无明确热重载 |
| **Hook 触发** | ConfigChange Hook | 无 |
| **特性标志** | GrowthBook CDN 实时轮询 | 无远程特性标志 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| **React Context 响应式**：发布-订阅模式确保 UI 实时响应状态变化，开发体验好 | **JSON 格式可读性差**：settings.json 不支持注释，配置维护成本高 |
| **GrowthBook 远程控制**：支持实时特性标志轮询、A/B 测试、灰度发布，运维能力强 | **无 SQLite 索引**：会话索引依赖手动维护的 sessions.json，搜索和过滤能力弱 |
| **惰性加载**：CLAUDE.md 按需加载，启动速度快，资源占用低 | **7 个 Context 复杂度**：多个 Context 增加了代码复杂度和理解成本 |
| **MDM 策略支持**：企业管理员可通过 MDM 策略强制配置，适合企业部署 | **无跨会话记忆**：缺少类似 Codex CLI 的 memories 模块，跨会话知识保持能力弱 |
| **热重载机制**：配置和指令文件变更自动检测并重载，无需重启 | **无 Profile 机制**：缺少配置 Profile，不同场景需要手动修改配置 |
| **规则匹配**：`.claude/rules/*.md` 按文件路径匹配，实现细粒度的目录级指令 | **无大小限制**：CLAUDE.md 无明确大小限制，可能导致 prompt 过长 |
| **防嵌套 Provider**：AppStateProvider 内置防嵌套机制，避免 Context 重复初始化 | **JSON 无注释**：配置文件无法添加说明注释，团队协作时理解成本高 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| **TOML 格式可读性优秀**：原生支持注释、语法简洁，配置维护成本低 | **无热重载**：配置变更需要重启会话，开发体验不如 Claude Code |
| **SQLite 索引**：state.db 提供高效的会话搜索、过滤和分页能力 | **无远程特性标志**：缺少 GrowthBook 类似的远程控制能力，运维灵活性不足 |
| **双层存储架构**：JSONL（人类可读）+ SQLite（高效索引），兼顾可读性和性能 | **无 MDM 策略**：缺少企业管理员强制策略机制，企业部署能力弱 |
| **Profile 机制**：支持命名 Profile（dev/staging/prod），一键切换配置场景 | **无惰性加载**：AGENTS.md 启动时一次性加载，可能影响启动速度 |
| **跨会话记忆**：memories 模块自动提取和注入跨会话知识，长期协作能力强 | **32 KiB 大小限制**：AGENTS.md 有严格大小限制，大型项目可能不够用 |
| **Rust 类型安全**：serde 编译时类型检查，配置错误在编译阶段就能发现 | **无规则匹配**：缺少按文件路径匹配的规则机制，指令粒度较粗 |
| **多种认证方式**：支持 ChatGPT OAuth、API Key、本地模型、Azure 等多种认证方式 | **无 ConfigChange Hook**：缺少配置变更的 Hook 机制，扩展性受限 |
| **模型提供商注册表**：ModelProviderInfo 统一管理多个模型提供商，切换方便 | **AGENTS.md 发现算法简单**：仅按目录层级收集，缺少更灵活的匹配策略 |
# 错误处理与会话恢复对比

## Claude Code 实现

### withRetry 重试策略

Claude Code 的重试机制采用**指数退避 + 随机抖动**策略，并尊重服务端的 `retry-after` 头：

```typescript
// services/api/withRetry.ts -- 指数退避 + 抖动

export interface RetryOptions {
  maxRetries?: number;        // 默认 3
  baseDelay?: number;         // 默认 1000ms
  maxDelay?: number;          // 默认 30000ms
  jitter?: number;            // 默认 1000ms
  retryableStatuses?: number[]; // 默认 [429, 529, 500, 502, 503]
}

export async function withRetry<T>(
  fn: () => Promise<T>,
  options: RetryOptions = {},
): Promise<T> {
  const {
    maxRetries = 3,
    baseDelay = 1000,
    maxDelay = 30000,
    jitter = 1000,
    retryableStatuses = [429, 529, 500, 502, 503],
  } = options;

  let lastError: Error;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;
      const apiError = error as ApiError;
      const status = apiError?.status;

      // 不可重试的错误
      if (status && !retryableStatuses.includes(status)) {
        throw error;
      }

      // 最后一次尝试，不再重试
      if (attempt >= maxRetries) {
        throw error;
      }

      // 计算延迟：指数退避 + 随机抖动
      const exponentialDelay = baseDelay * Math.pow(2, attempt);
      const jitterDelay = Math.random() * jitter;
      const delay = Math.min(exponentialDelay + jitterDelay, maxDelay);

      // 尊重 retry-after 头（如果有）
      const retryAfter = apiError?.headers?.['retry-after'];
      const actualDelay = retryAfter
        ? Math.max(parseInt(retryAfter) * 1000, delay)
        : delay;

      await sleep(actualDelay);
    }
  }

  throw lastError!;
}
```

### 错误分类（classifyError）

Claude Code 将错误分为 **6 大类**，每类有不同的处理策略：

```typescript
// services/api/errors.ts -- 错误分类

export type ErrorCategory =
  | 'rate_limit'      // 429 - 速率限制
  | 'overloaded'      // 529 - 服务过载
  | 'server_error'    // 5xx - 服务器错误
  | 'auth_error'      // 401 - 认证失败
  | 'forbidden'       // 403 - 禁止访问
  | 'context_overflow' // prompt_too_long
  | 'network_error'   // 网络连接错误
  | 'timeout'         // 请求超时
  | 'unknown';        // 未知错误

export function classifyError(error: unknown): ErrorCategory {
  // 非 API 错误
  if (!(error instanceof ApiError)) {
    if (error instanceof TypeError && error.message.includes('fetch')) {
      return 'network_error';
    }
    if (error instanceof DOMException && error.name === 'AbortError') {
      return 'timeout';
    }
    return 'network_error';
  }

  const { status, message } = error;

  if (status === 429) return 'rate_limit';
  if (status === 529) return 'overloaded';
  if (status >= 500 && status < 600) return 'server_error';
  if (status === 401) return 'auth_error';
  if (status === 403) return 'forbidden';
  if (message?.includes('prompt_too_long')) return 'context_overflow';
  if (message?.includes('timeout')) return 'timeout';

  return 'unknown';
}
```

### 速率限制处理

```typescript
// 速率限制处理策略

// 1. 尊重 retry-after 头
function getRetryAfterMs(error: ApiError): number | null {
  const retryAfter = error.headers?.['retry-after'];
  if (!retryAfter) return null;

  // retry-after 可以是秒数或 HTTP 日期
  const seconds = parseInt(retryAfter, 10);
  if (!isNaN(seconds)) return seconds * 1000;

  const date = new Date(retryAfter);
  if (!isNaN(date.getTime())) {
    return Math.max(0, date.getTime() - Date.now());
  }

  return null;
}

// 2. 组织级限制检测
function isOrgRateLimit(error: ApiError): boolean {
  return error.status === 429
    && error.message?.includes('rate limit')
    && error.message?.includes('organization');
}

// 组织级限制不重试，直接通知用户升级计划
if (isOrgRateLimit(error)) {
  throw new OrgRateLimitError(
    'Organization rate limit reached. Please upgrade your plan or wait.'
  );
}
```

### 工具执行失败处理

```typescript
// 工具执行失败的差异化处理

try {
  const result = await tool.call(input, context);
  // ...
} catch (error) {
  if (error instanceof AbortError) {
    // AbortError：被 siblingAbortController 或用户中止
    // 不触发兄弟中止，静默处理
    return { content: '[Tool execution aborted]', isError: true };
  }

  if (error instanceof PermissionDeniedError) {
    // 权限拒绝：记录但不中止其他工具
    return { content: `Permission denied: ${error.message}`, isError: true };
  }

  if (error instanceof ValidationError) {
    // 输入验证失败：记录错误但不中止
    return { content: `Validation error: ${error.message}`, isError: true };
  }

  // 其他错误：触发兄弟中止
  siblingAbortController.abort();
  return { content: `Error: ${error.message}`, isError: true };
}
```

### 会话持久化到磁盘

```
┌─────────────────────────────────────────────────────────────────┐
│                 会话持久化文件结构                                │
│                                                                 │
│  ~/.claude/projects/<project-slug>/                            │
│  ├── sessions.json              # 会话索引                      │
│  │   {                                                          │
│  │     "sessions": [                                          │
│  │       {                                                      │
│  │         "id": "abc123",                                     │
│  │         "title": "Implement auth feature",                  │
│  │         "createdAt": "2026-04-13T10:00:00Z",                │
│  │         "lastUpdated": "2026-04-13T11:30:00Z",              │
│  │         "messageCount": 42,                                 │
│  │         "totalTokens": 156000,                              │
│  │         "totalCost": 0.42,                                  │
│  │         "transcriptPath": "sessions/abc123.jsonl"           │
│  │       },                                                     │
│  │       ...                                                    │
│  │     ]                                                        │
│  │   }                                                          │
│  │                                                              │
│  └── sessions/                                                │
│      └── abc123.jsonl            # JSONL 转录文件                │
│          {"type":"message","ts":...,"msg":{...}}               │
│          {"type":"tool_call","ts":...,"tool":{...}}            │
│          {"type":"checkpoint","ts":...,"state":{...}}          │
│          ...                                                    │
└─────────────────────────────────────────────────────────────────┘
```

```typescript
// SessionMetadata 接口定义
export interface SessionMetadata {
  id: string;
  title: string;
  createdAt: string;          // ISO 8601
  lastUpdated: string;        // ISO 8601
  messageCount: number;
  totalTokens: number;
  totalCost: number;
  transcriptPath: string;
  model: string;
  permissionMode: PermissionMode;
}

// JSONL 转录事件类型
type TranscriptEvent =
  | { type: 'message'; timestamp: number; message: Message; }
  | { type: 'tool_call'; timestamp: number; toolName: string;
      input: unknown; result: ToolResult; }
  | { type: 'checkpoint'; timestamp: number; state: CheckpointState; }
  | { type: 'compact'; timestamp: number; clearedCount: number; }
  | { type: 'permission_decision'; timestamp: number;
      toolName: string; allowed: boolean; reason?: string; };
```

### 持久化时机

```
┌─────────────────────────────────────────────────────────────────┐
│                 持久化时机                                       │
│                                                                 │
│  1. 每次消息交换完成后                                         │
│     - 用户消息发送后                                           │
│     - 助手响应完成后                                           │
│     追加到 JSONL 转录文件                                     │
│                                                                 │
│  2. 每次工具调用完成后                                         │
│     - 工具名称、输入、结果                                     │
│     - 执行时长、退出码                                         │
│     追加到 JSONL 转录文件                                     │
│                                                                 │
│  3. Checkpoint 时刻                                             │
│     - 自动压缩后                                               │
│     - 会话空闲超过阈值                                         │
│     - 创建完整的消息快照                                       │
│     追加 checkpoint 事件到 JSONL                               │
│                                                                 │
│  4. sessions.json 更新时机                                     │
│     - 每次消息交换后更新 lastUpdated 和统计                    │
│     - 使用 debounce 5s 避免频繁写入                            │
└─────────────────────────────────────────────────────────────────┘
```

### 恢复会话时的状态重建

```typescript
// 4 步恢复过程

export async function restoreSession(
  sessionId: string,
): Promise<SessionRestoreResult> {
  // 步骤 1: 读取会话元数据
  const metadata = await readSessionMetadata(sessionId);
  if (!metadata) {
    throw new Error(`Session ${sessionId} not found`);
  }

  // 步骤 2: 读取并解析 JSONL 转录
  const transcriptPath = path.join(
    getProjectDir(), 'sessions', `${sessionId}.jsonl`
  );
  const transcriptLines = await readFile(transcriptPath, 'utf-8');
  const events = transcriptLines
    .split('\n')
    .filter(line => line.trim())
    .map(line => JSON.parse(line));

  // 步骤 3: 重建消息历史
  const messages: Message[] = [];
  let lastCheckpoint: CheckpointState | null = null;

  for (const event of events) {
    switch (event.type) {
      case 'message':
        messages.push(event.message);
        break;
      case 'checkpoint':
        lastCheckpoint = event.state;
        break;
      // 其他事件类型用于审计，不参与消息重建
    }
  }

  // 如果有 checkpoint，从 checkpoint 恢复（跳过中间事件）
  if (lastCheckpoint) {
    messages.length = 0;
    messages.push(...lastCheckpoint.messages);
  }

  // 步骤 4: 恢复工具注册表和权限状态
  const tools = await rebuildToolRegistry(metadata.model);
  const permissionState = {
    mode: metadata.permissionMode || 'default',
    deniedCount: 0,
    consecutiveDenials: 0,
  };

  return {
    messages,
    tools,
    permissionState,
    metadata,
    stats: {
      totalTokens: metadata.totalTokens,
      totalCost: metadata.totalCost,
      messageCount: metadata.messageCount,
    },
  };
}
```

**已知脆弱性：**
- **attachment 不持久化**：用户上传的文件附件（如图片）不会被持久化到磁盘，恢复会话时这些内容会丢失
- **fork-session 缓存前缀问题**：从 fork 的子 Agent 恢复会话时，API 提示词缓存前缀可能与原始会话不同，导致缓存未命中

---

## Codex CLI 实现

### 错误类型体系

核心实现在 `codex-rs/core/src/error.rs`（659 行），定义了完整的错误类型枚举，包含 **15+ 变体**：

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum CodexErr {
    /// 轮次被中止（用户中断）
    #[error("Turn aborted")]
    TurnAborted,

    /// 流式响应错误
    #[error("Stream error: {0}")]
    Stream(String),

    /// 上下文窗口超限
    #[error("Context window exceeded")]
    ContextWindowExceeded,

    /// 请求超时
    #[error("Request timeout")]
    Timeout,

    /// 操作被中断
    #[error("Operation interrupted")]
    Interrupted,

    /// 意外的 HTTP 状态码
    #[error("Unexpected status: {0}")]
    UnexpectedStatus(u16),

    /// 无效请求
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// 使用限制达到
    #[error("Usage limit reached")]
    UsageLimitReached,

    /// 服务器过载
    #[error("Server overloaded")]
    ServerOverloaded,

    /// 配额超限
    #[error("Quota exceeded")]
    QuotaExceeded,

    /// 重试次数超限
    #[error("Retry limit reached")]
    RetryLimit,

    /// 内部服务器错误
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    /// 沙箱错误
    #[error("Sandbox error: {0}")]
    Sandbox(#[from] SandboxErr),

    // ... 更多变体
}
```

### 可重试错误判断

```rust
impl CodexErr {
    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            CodexErr::Stream(_) => true,
            CodexErr::Timeout => true,
            CodexErr::ServerOverloaded => true,
            CodexErr::InternalServerError(_) => true,
            CodexErr::UnexpectedStatus(429) => true,  // Rate limit
            CodexErr::UnexpectedStatus(502) => true,  // Bad Gateway
            CodexErr::UnexpectedStatus(503) => true,  // Service Unavailable
            _ => false,
        }
    }
}
```

### 重试策略

**指数退避 backoff() 函数：**

```rust
/// 指数退避计算
/// 重试 1: ~1s, 重试 2: ~2s, 重试 3: ~4s, ...
pub fn backoff(retry_count: usize) -> Duration {
    let base_ms = 1000u64;
    let max_ms = 30_000u64;  // 最大 30 秒
    let delay_ms = base_ms * 2u64.saturating_pow(retry_count as u32 - 1);
    Duration::from_millis(delay_ms.min(max_ms))
}
```

**Stream 错误特殊处理：**

```rust
// Stream 错误返回 Option<Duration> 表示是否应该重试以及等待时间
fn handle_stream_error(err: &CodexErr) -> Option<Duration> {
    match err {
        CodexErr::Stream(msg) if msg.contains("connection reset") => {
            Some(Duration::from_secs(1))  // 立即重试
        }
        CodexErr::Stream(msg) if msg.contains("timeout") => {
            Some(Duration::from_secs(2))  // 等待后重试
        }
        CodexErr::ServerOverloaded => {
            Some(backoff(1))  // 指数退避
        }
        _ => None,  // 不可重试
    }
}
```

**ContextWindowExceeded 移除最旧历史项重试：**

```rust
// 在压缩过程中，如果上下文窗口仍然超限
Err(e @ CodexErr::ContextWindowExceeded) => {
    if turn_input_len > 1 {
        // 移除最旧的历史项（保留前缀缓存）
        history.remove_first_item();
        truncated_count += 1;
        continue;  // 重试
    }
    // 无法继续缩减，报告错误
}
```

**使用限制错误的升级建议：**

```rust
CodexErr::UsageLimitReached => {
    // 发送升级建议事件
    sess.send_event(&turn_context, EventMsg::Error(ErrorEvent {
        message: "API usage limit reached. Consider upgrading your plan or waiting for the limit to reset.".to_string(),
        codex_error_info: Some(CodexErrorInfo::UsageLimitReached),
    })).await;
}
```

### 沙箱执行失败处理

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum SandboxErr {
    /// 操作被沙箱拒绝
    #[error("Operation denied by sandbox: {0}")]
    Denied(String),

    /// seccomp 过滤器安装失败
    #[error("Failed to install seccomp filter: {0}")]
    SeccompInstall(String),

    /// 命令执行超时
    #[error("Command timed out after {0:?}")]
    Timeout(Duration),

    /// 命令被信号终止
    #[error("Command terminated by signal: {0}")]
    Signal(i32),

    /// Landlock 限制应用失败
    #[error("Failed to apply Landlock restrictions: {0}")]
    LandlockRestrict(String),
}
```

### 会话恢复流程

Codex CLI 的会话恢复基于 JSONL Rollout 文件和 SQLite 双层存储：

```
┌──────────────────────────────────────────────────────────────┐
│                 会话恢复流程                                   │
│                                                               │
│  1. 加载 Rollout 文件                                         │
│     load_rollout_items(path)                                  │
│     ├── 读取 JSONL 文件                                       │
│     ├── 逐行解析 RolloutLine                                  │
│     ├── 提取 SessionMeta 获取 thread_id                       │
│     └── 收集所有 RolloutItem                                  │
│                                                               │
│  2. 重放 RolloutItem 流                                       │
│     ├── SessionMeta -> 恢复会话元数据                          │
│     ├── ResponseItem -> 重建消息历史                           │
│     ├── TurnContext -> 恢复轮次上下文                           │
│     ├── Compacted -> 恢复压缩历史                              │
│     └── EventMsg -> 重建事件状态                               │
│                                                               │
│  3. 创建新的 Codex 实例                                       │
│     ├── 使用恢复的历史初始化 Session                          │
│     ├── RolloutRecorder 以 Resume 模式打开（追加写入）        │
│     └── 继续正常的 SQ/EQ 事件循环                             │
└──────────────────────────────────────────────────────────────┘
```

**RolloutItem 类型覆盖：**

| 类型 | 恢复时处理 |
|------|-----------|
| `SessionMeta` | 恢复会话元数据（thread_id、cwd、Git 信息等） |
| `ResponseItem` | 重建消息历史（文本消息、工具调用、工具结果） |
| `TurnContext` | 恢复轮次上下文（模型信息、沙箱策略、审批策略） |
| `Compacted` | 恢复压缩历史（压缩摘要、替换历史） |
| `EventMsg` | 重建事件状态（执行命令、审批请求等） |

### 会话分叉

```
┌──────────────────────────────────────────────────────────────┐
│                 会话分叉流程                                   │
│                                                               │
│  1. 加载原始 Rollout 文件                                     │
│     load_rollout_items(original_path)                         │
│                                                               │
│  2. 创建新的线程 ID                                           │
│     new_thread_id = Uuid::new_v4()                            │
│                                                               │
│  3. 继承配置                                                 │
│     ├── 模型配置                                             │
│     ├── 沙箱策略                                             │
│     ├── 审批策略                                             │
│     └── 工具配置                                             │
│                                                               │
│  4. 创建新的 Rollout 文件                                     │
│     RolloutRecorder::new(                                    │
│         RolloutRecorderParams::Create {                      │
│             conversation_id: new_thread_id,                  │
│             forked_from_id: Some(original_thread_id),        │
│             source: SessionSource::Fork,                     │
│             ...                                              │
│         }                                                    │
│     )                                                        │
│                                                               │
│  5. 不修改原始 Rollout 文件                                   │
│     原始会话保持不变，可以继续使用                             │
└──────────────────────────────────────────────────────────────┘
```

**TUI 恢复选择器：**

```
┌──────────────────────────────────────────────────────────────┐
│  会话恢复选择器                                               │
│                                                               │
│  最近会话 (1/3):                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ #1  [2025-05-07 17:24]  "重构认证模块"                  │  │
│  │     /home/user/project  main  o4-mini                  │  │
│  │                                                        │  │
│  │ #2  [2025-05-07 15:10]  "修复 CI 管道"                  │  │
│  │     /home/user/project  fix-ci  o4-mini                │  │
│  │                                                        │  │
│  │ #3  [2025-05-06 09:30]  "添加单元测试"                  │  │
│  │     /home/user/project  main  gpt-4o                   │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  分页: 每页 25 条 | 过滤: 支持搜索 | 排序: 按时间/按名称     │
│                                                               │
│  [上下键] 选择  [Enter] 恢复  [/] 搜索  [q] 退出              │
└──────────────────────────────────────────────────────────────┘
```

### 记忆系统

```
~/.codex/
├── sessions/                  # 会话 Rollout 文件
│   ├── 2025/
│   │   ├── 05/
│   │   │   ├── 07/
│   │   │   │   └── rollout-2025-05-07T17-24-21-{uuid}.jsonl
│   │   │   └── ...
│   │   └── ...
│   └── archived/              # 已归档会话
│       └── ...
├── state.db                   # SQLite 状态数据库
├── config.toml                # 全局配置
├── profiles/                  # 配置 Profile
│   └── dev.toml
└── memories/                  # 跨会话记忆
    └── ...
```

记忆系统通过 `memories` 模块实现跨会话知识保持。当 `generate_memories` 配置启用时，系统会在会话结束时自动提取关键信息并持久化，在后续会话中作为上下文注入。

---

## 对比分析

### 错误分类粒度对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **错误分类数** | 6 大类（ErrorCategory 枚举） | 15+ 变体（CodexErr 枚举） |
| **分类方式** | 基于 HTTP 状态码 + 错误消息模式匹配 | Rust 枚举 + thiserror 派生 |
| **类型安全** | 运行时分类（TypeScript string union） | 编译时类型检查（Rust enum） |
| **沙箱错误** | 无独立分类 | `SandboxErr` 独立枚举（5 变体） |
| **上下文溢出** | `context_overflow` 类别 | `ContextWindowExceeded` 变体 |
| **使用限制** | 包含在 `rate_limit` 中 | 独立 `UsageLimitReached` + `QuotaExceeded` |
| **流式错误** | 包含在 `network_error` 中 | 独立 `Stream(String)` 变体 |

### 错误分类详细映射

| 错误场景 | Claude Code 分类 | Codex CLI 分类 |
|----------|-----------------|----------------|
| HTTP 429 | `rate_limit` | `UnexpectedStatus(429)` |
| HTTP 529 | `overloaded` | `ServerOverloaded` |
| HTTP 5xx | `server_error` | `InternalServerError` |
| HTTP 401 | `auth_error` | `UnexpectedStatus(401)` |
| HTTP 403 | `forbidden` | `UnexpectedStatus(403)` |
| 上下文过长 | `context_overflow` | `ContextWindowExceeded` |
| 网络断开 | `network_error` | `Stream("connection reset")` |
| 请求超时 | `timeout` | `Timeout` |
| 用户中断 | 无独立分类 | `TurnAborted` / `Interrupted` |
| 沙箱拒绝 | 无独立分类 | `Sandbox(Denied)` |
| 沙箱超时 | 无独立分类 | `Sandbox(Timeout)` |
| 使用限制 | `rate_limit`（组织级） | `UsageLimitReached` / `QuotaExceeded` |
| 重试超限 | 无独立分类 | `RetryLimit` |
| 无效请求 | 无独立分类 | `InvalidRequest` |

### 重试策略对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **退避算法** | 指数退避 + 随机抖动 | 指数退避（无抖动） |
| **基础延迟** | 1000ms | 1000ms |
| **最大延迟** | 30000ms | 30000ms |
| **抖动** | 有（0-1000ms 随机） | 无 |
| **retry-after 支持** | 支持（秒数 + HTTP 日期） | 不明确 |
| **可重试状态码** | [429, 529, 500, 502, 503] | 429, 502, 503 + Stream/Timeout/ServerOverloaded |
| **组织级限制** | 检测并跳过重试，通知升级 | `UsageLimitReached` 发送升级建议事件 |
| **上下文溢出** | 无特殊处理 | 移除最旧历史项后重试 |
| **Stream 错误** | 统一为 network_error | 按错误内容差异化处理（connection reset vs timeout） |

### 会话持久化格式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **持久化格式** | JSONL | JSONL |
| **存储位置** | `~/.claude/projects/<slug>/sessions/` | `~/.codex/sessions/YYYY/MM/DD/` |
| **索引机制** | sessions.json（手动维护） | SQLite state.db（自动索引） |
| **事件类型** | message / tool_call / checkpoint / compact / permission_decision | ResponseItem / EventMsg / TurnContext / Compacted / SessionMeta |
| **元数据** | SessionMetadata 接口 | SessionMeta RolloutItem |
| **写入方式** | 同步追加 | 异步写入（有界队列，容量 256） |
| **debounce** | 5s（sessions.json 更新） | 无（异步通道缓冲） |
| **Checkpoint** | 支持（完整消息快照） | 支持（Compacted 记录） |

### 恢复可靠性对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **恢复步骤** | 4 步（元数据 -> JSONL -> 消息重建 -> 工具/权限恢复） | 3 步（加载 Rollout -> 重放事件 -> 创建实例） |
| **Checkpoint 优化** | 从最新 checkpoint 恢复，跳过中间事件 | 从 Compacted 记录恢复压缩历史 |
| **权限状态恢复** | 恢复权限模式，重置计数器 | 通过 TurnContext 恢复审批策略 |
| **工具注册表** | 重建工具注册表 | 通过 TurnContext 恢复 |
| **分叉支持** | 有（但存在缓存前缀问题） | 有（独立 Rollout 文件，原始会话不受影响） |
| **TUI 选择器** | 无明确描述 | 有（分页、搜索、排序） |
| **已知脆弱性** | attachment 不持久化、fork 缓存前缀问题 | 无明确记录 |

### 会话恢复完整性对比

```
Claude Code 恢复内容:
  [x] 消息历史（从 checkpoint 或逐条重建）
  [x] 工具注册表
  [x] 权限模式
  [x] 成本统计
  [x] 消息数量
  [ ] 文件附件（已知丢失）
  [ ] 权限拒绝计数器（重置为 0）
  [ ] API 提示缓存（fork 时可能失效）

Codex CLI 恢复内容:
  [x] 消息历史（ResponseItem 重放）
  [x] 模型信息（TurnContext）
  [x] 沙箱策略（TurnContext）
  [x] 审批策略（TurnContext）
  [x] 压缩历史（Compacted）
  [x] 事件状态（EventMsg）
  [x] Git 信息（SessionMeta）
  [ ] 文件附件（未明确）
```

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| **随机抖动**：指数退避 + 随机抖动，有效避免重试风暴（thundering herd） | **错误分类粒度粗**：仅 6 大类，缺少沙箱错误、流式错误等细粒度分类 |
| **retry-after 双格式**：支持秒数和 HTTP 日期两种 retry-after 格式，兼容性好 | **无沙箱错误处理**：没有独立的沙箱错误分类，无法区分沙箱拒绝和普通权限拒绝 |
| **组织级限制检测**：`isOrgRateLimit` 专门检测组织级限制，避免无意义重试 | **无上下文溢出自动恢复**：context_overflow 错误没有自动移除历史项重试的机制 |
| **工具失败差异化**：AbortError/PermissionDeniedError/ValidationError 分别处理，策略清晰 | **attachment 不持久化**：恢复会话时文件附件丢失，影响用户体验 |
| **Checkpoint 优化**：从最新 checkpoint 恢复，跳过中间事件，恢复速度快 | **fork 缓存前缀问题**：从 fork 的子 Agent 恢复时 API 缓存可能失效 |
| **debounce 写入**：sessions.json 更新使用 5s debounce，避免频繁 IO | **同步写入**：JSONL 追加是同步操作，可能阻塞主线程 |
| **权限决策审计**：`permission_decision` 事件记录每次权限决策，审计追踪完整 | **无 TUI 恢复选择器**：缺少可视化的会话恢复选择界面 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| **错误分类精细**：15+ 枚举变体覆盖所有错误场景，类型安全（Rust 编译时检查） | **无随机抖动**：纯指数退避缺少抖动，多实例并发重试时可能产生重试风暴 |
| **沙箱错误独立**：`SandboxErr` 独立枚举（5 变体），能精确定位沙箱层面的问题 | **无 retry-after 支持**：不明确支持 HTTP retry-after 头，可能过早重试 |
| **上下文溢出自动恢复**：`ContextWindowExceeded` 时自动移除最旧历史项并重试 | **使用限制处理简单**：`UsageLimitReached` 仅发送升级建议，无组织级限制区分 |
| **Stream 错误差异化**：按错误内容（connection reset vs timeout）差异化处理，策略精准 | **无权限决策审计**：缺少类似 Claude Code 的 `permission_decision` 事件记录 |
| **异步写入架构**：RolloutRecorder 使用有界队列（容量 256）异步写入，不阻塞主线程 | **恢复步骤较少**：3 步恢复相比 Claude Code 的 4 步，可能遗漏某些状态 |
| **SQLite 索引**：state.db 提供高效的会话搜索、过滤和分页，恢复选择体验好 | **无 debounce**：异步通道缓冲但无 debounce，高频事件可能导致频繁写入 |
| **TUI 恢复选择器**：内置分页、搜索、排序的会话恢复选择界面，用户体验优秀 | **无明确脆弱性记录**：缺少已知脆弱性的文档记录，维护者可能忽视潜在问题 |
| **会话分叉可靠**：独立 Rollout 文件，原始会话不受影响，分叉操作安全 | **无 attachment 持久化**：同样未明确记录文件附件的持久化策略 |
| **thiserror 派生**：错误类型自动生成 Display 实现，错误信息格式统一 | **错误消息粒度不一**：部分变体携带详细消息（String），部分仅有固定描述 |
# Hook、技能与 MCP 集成对比

## Claude Code 实现

### Hook 系统

Hooks 是用户定义的生命周期事件动作，是 Claude Code 的**可扩展性骨干**。

#### Hook 事件类型（13 种）

| 事件 | 触发时机 |
|------|----------|
| `SessionStart` | 启动/恢复/清除/压缩时 |
| `Stop` | Claude 结束响应前 |
| `UserPromptSubmit` | 用户提交时（exit code 2 = 阻止提交） |
| `PreToolUse` | 工具执行前（exit code 2 = 阻止并显示 stderr） |
| `PostToolUse` | 成功执行后 |
| `PostToolUseFail` | 失败执行后 |
| `SubagentStart/Stop` | 子 Agent 生成/完成 |
| `TaskCreated/Completed` | 任务注册/到达终态 |
| `PermissionDenied` | auto 模式拒绝时 |
| `ConfigChange` | 设置变更时 |
| `CwdChanged` | 工作目录变更时 |
| `FileChanged` | 监视文件变更时 |
| `Notification` | 通知发送时 |

#### Hook 类型（5 种）

| 类型 | 实现 | 特点 |
|------|------|------|
| **Command Hook** | Shell 命令 (bash/zsh) | Exit code 0=ok, 2=block, N=error |
| **Prompt Hook** | LLM 评估条件 (Haiku 模型) | 返回 `{ok: true/false, reason}` |
| **Agent Hook** | 完整 Agent + 工具 | 超时 60s，无递归 |
| **HTTP Hook** | HTTP 请求到端点 | JSON body + context |
| **Function Hook** | TS 回调（内存中） | 仅会话级，不持久化 |

### 技能系统

#### 技能来源（4 级优先级）

| 来源 | 位置 | 优先级 |
|------|------|--------|
| 托管技能 | 企业策略控制 | 最高 |
| 项目技能 | `./.claude/skills/` | 高 |
| 用户技能 | `~/.claude/skills/` | 中 |
| 内置技能 | 编译打包 | 最低 |

#### 技能执行模式

- **内联模式（默认）**：技能内容注入到当前对话中，模型视为当前轮次的一部分
- **Fork 模式（`context: "fork"`）**：创建子 Agent 独立执行，拥有自己的上下文和预算，返回结果文本给父 Agent

#### 条件技能（路径过滤）

技能可以配置 `paths` 字段，仅当模型编辑匹配的文件时才激活。

### MCP 集成

#### MCP 客户端架构

支持 4 种传输协议：
- **stdio**：本地进程 stdin/stdout 管道
- **SSE/HTTP**：远程 HTTP + EventSource + OAuth
- **WebSocket**：持久连接 + 二进制帧 + TLS/代理
- **local**：本地协议

工具名称规范化：服务器 `my-server` 的工具 `send_message` -> `mcp__my_server__send_message`

#### MCP 配置作用域（5 级）

| 作用域 | 位置 | 用例 |
|--------|------|------|
| `local` | `.claude/settings.local.json` | 用户本地服务器 |
| `user` | `~/.claude/settings.json` | 用户全局服务器 |
| `project` | `.claude/settings.json` | 团队共享服务器 |
| `dynamic` | 运行时注册 | 编程式服务器 |
| `enterprise` | MDM 策略 | 管理员管理服务器 |

---

## Codex CLI 实现

### MCP 集成

#### codex-rmcp-client

MCP 客户端通过 `codex-rmcp-client` crate 实现，核心是 `McpConnectionManager`：

```rust
/// MCP 连接管理器 -- 管理所有 MCP 服务器连接
pub struct McpConnectionManager {
    /// 已连接的 MCP 服务器
    connections: HashMap<String, McpConnection>,

    /// 工具信息缓存
    tool_info_cache: HashMap<String, Vec<ToolInfo>>,
}

/// MCP 连接
pub struct McpConnection {
    /// 服务器名称
    pub server_name: String,

    /// 服务器配置
    pub config: McpServerConfig,

    /// 客户端实例
    pub client: rmcp::Client,
}
```

#### config.toml 配置方式

```toml
[mcp_servers]
# stdio 传输
my-local-server = { command = "npx", args = ["-y", "@my/mcp-server"] }

# SSE 传输
my-remote-server = { url = "https://my-server.com/mcp" }

# 带环境变量的服务器
my-auth-server = {
    command = "python",
    args = ["-m", "my_mcp"],
    env = { API_KEY = "${MY_API_KEY}" }
}
```

#### MCP 工具暴露控制

MCP 工具通过 `McpToolSnapshot` 暴露给模型：

```rust
/// MCP 工具快照 -- 某个时刻的 MCP 工具列表
pub struct McpToolSnapshot {
    /// 服务器名称
    pub server_name: String,

    /// 该服务器提供的工具列表
    pub tools: Vec<McpTool>,
}

/// 工具命名规范: mcp__{server_name}__{tool_name}
/// 例如: mcp__my_server__search_files
pub fn mcp_tool_name(server_name: &str, tool_name: &str) -> String {
    format!("mcp__{}__{}", server_name, tool_name)
}
```

**重要设计 -- MCP 工具不受沙箱保护：**

MCP 工具在 MCP 服务器进程中执行，**不受 Codex CLI 沙箱策略的约束**。这意味着：
- MCP 工具可以访问文件系统的任意位置
- MCP 工具可以发起网络请求
- 安全性依赖于 MCP 服务器自身的安全实现

这是有意的设计选择，因为 MCP 工具通常需要访问外部资源（如数据库、API），沙箱限制会使其无法正常工作。

#### MCP Server 模式

Codex CLI 自身也可以作为 MCP 服务器运行：

```bash
# 启动 Codex CLI 作为 MCP 服务器
codex mcp-server
```

```rust
/// Codex CLI 作为 MCP 服务器的实现
pub struct RmcpServer {
    /// 内部 Codex 实例
    codex: Arc<Codex>,

    /// 暴露的工具
    tools: Vec<McpTool>,
}

/// 暴露给外部客户端的工具
pub struct CodexMcpTool {
    /// 工具名称
    pub name: String,

    /// 工具描述
    pub description: String,
}

/// run_codex 工具 -- 允许外部 MCP 客户端通过 Codex 执行任务
pub struct RunCodexTool {
    pub prompt: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
}
```

### Hook 系统

Codex CLI **没有独立的 Hook 系统**。其可扩展性主要通过以下机制实现：
- **插件系统**：通过 `plugin/*` API 管理插件
- **Guardian 守护者**：安全中间层决定操作是否自动执行
- **沙箱策略**：可配置的执行策略

### 技能系统

Codex CLI **没有独立的技能系统**。类似功能通过以下方式实现：
- **AGENTS.md**：项目级指令文件（类似 CLAUDE.md）
- **提示词模板**：通过配置文件定义
- **插件**：扩展工具能力

---

## 对比分析

### Hook 系统对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **Hook 事件数量** | 13 种生命周期事件 | 无独立 Hook 系统 |
| **Hook 类型** | 5 种（Command/Prompt/Agent/HTTP/Function） | 无 |
| **阻止能力** | 支持（exit code 2 阻止操作） | 通过 Guardian 实现 |
| **LLM 评估** | Prompt Hook（Haiku 模型评估） | 无 |
| **HTTP 集成** | HTTP Hook（请求外部端点） | 无 |
| **编程式扩展** | Function Hook（TS 回调） | 通过插件系统 |

### 技能系统对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **技能来源** | 4 级（托管/项目/用户/内置） | 无独立技能系统 |
| **执行模式** | 内联 + Fork 两种 | N/A |
| **路径过滤** | 支持（paths 字段） | N/A |
| **条件激活** | 支持文件匹配触发 | N/A |
| **企业控制** | 托管技能（最高优先级） | N/A |

### MCP 集成对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **传输协议** | 4 种（stdio/SSE/WebSocket/local） | 2 种（stdio/SSE） |
| **配置作用域** | 5 级（local/user/project/dynamic/enterprise） | 1 级（config.toml） |
| **工具命名** | `mcp__{server}__{tool}` | `mcp__{server}__{tool}`（一致） |
| **MCP Server 模式** | 不支持 | 支持（`codex mcp-server`） |
| **实现方式** | 自研实现 | 基于 rmcp 0.12 |
| **沙箱保护** | N/A | MCP 工具不受沙箱约束 |
| **工具缓存** | 延迟加载（ToolSearchTool） | McpToolSnapshot 快照 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| Hook 系统极其丰富（13 事件 x 5 类型），提供全方位生命周期控制 | Hook 系统复杂度高，学习曲线陡峭 |
| Prompt Hook 可利用 LLM 智能评估条件，灵活度极高 | Agent Hook 有 60s 超时限制，复杂任务可能不够 |
| 技能系统支持 4 级优先级和企业托管，适合团队协作 | 技能系统与 Hook 系统重叠，概念边界模糊 |
| MCP 支持 4 种传输协议，覆盖本地和远程场景 | 不支持作为 MCP Server 运行，无法被其他工具调用 |
| 5 级配置作用域精细控制 MCP 服务器可见性 | 自研 MCP 实现，与社区标准可能有偏差 |
| 延迟加载 MCP 工具 Schema，大幅节省 token | MCP 配置分散在多个 JSON 文件中，管理复杂 |
| Function Hook 支持内存中 TS 回调，开发体验好 | Function Hook 不持久化，仅限当前会话 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| 支持作为 MCP Server 运行，可被其他工具集成调用 | 没有独立 Hook 系统，可扩展性受限 |
| 基于 rmcp 0.12 标准，与社区生态兼容性好 | 仅支持 2 种传输协议（stdio/SSE），缺少 WebSocket |
| MCP 配置集中在 config.toml，管理简单 | 配置作用域单一，无法区分用户/项目/企业级别 |
| 工具信息缓存（McpToolSnapshot）提升查询效率 | MCP 工具不受沙箱保护，存在安全隐患 |
| 零依赖原生二进制，MCP 客户端无需额外运行时 | 没有独立技能系统，缺乏 Claude Code 的路径过滤等高级功能 |
| 插件系统提供了一定程度的可扩展性 | 插件系统无法实现 Hook 级别的生命周期控制 |
# 认证与遥测对比

## Claude Code 实现

### 遥测系统

#### 三层遥测架构

```
┌─────────────────────────────────────────────────────────────────┐
│                 三层遥测架构                                     │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 第 1 层：Statsig（运营指标）                              │   │
│  │  - 用户行为分析                                          │   │
│  │  - 功能使用统计                                          │   │
│  │  - A/B 实验数据                                          │   │
│  │  - 实时仪表盘                                            │   │
│  │  数据流向：Claude Code -> Statsig CDN                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 第 2 层：Sentry（错误日志）                               │   │
│  │  - 未捕获异常                                            │   │
│  │  - API 错误                                              │   │
│  │  - 工具执行失败                                          │   │
│  │  - 崩溃报告                                              │   │
│  │  数据流向：Claude Code -> Sentry CDN                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 第 3 层：OpenTelemetry（管理员监控）                      │   │
│  │  - 管理员可配置的导出器                                  │   │
│  │  - 自定义 Span 和 Metric                                 │   │
│  │  - 与企业可观测性平台集成                                │   │
│  │  数据流向：Claude Code -> OTEL Collector                 │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

#### 8 种核心指标

| 指标名称 | 类型 | 说明 |
|----------|------|------|
| `claude_code.session.count` | Counter | 会话总数 |
| `lines_of_code.count` | Counter | 生成/修改的代码行数 |
| `pull_request.count` | Counter | 创建的 PR 数量 |
| `commit.count` | Counter | 创建的 commit 数量 |
| `cost.usage` | Gauge | 当前会话成本（美元） |
| `token.usage` | Gauge | 当前会话 token 使用量 |
| `code_edit_tool.decision` | Histogram | 代码编辑工具的决策时间 |
| `active_time.total` | Gauge | 用户活跃时间（秒） |

#### 5 种核心事件

| 事件名称 | 触发时机 | 包含数据 |
|----------|----------|----------|
| `user_prompt` | 用户提交消息 | 消息长度、模式类型 |
| `tool_result` | 工具执行完成 | 工具名称、执行时长、是否成功 |
| `api_request` | API 调用完成 | 模型、token 用量、延迟、缓存命中率 |
| `api_error` | API 调用失败 | 错误类型、状态码、重试次数 |
| `tool_decision` | 权限决策完成 | 工具名称、决策结果（允许/拒绝/询问） |

#### 隐私保护措施

```
┌─────────────────────────────────────────────────────────────────┐
│                 隐私保护措施                                     │
│                                                                 │
│  1. Prompt 默认脱敏                                            │
│     - 用户 prompt 内容在发送到遥测前进行哈希处理                │
│     - 仅记录 prompt 的 token 数量，不记录内容                   │
│     - 工具输入中的敏感信息（路径、命令）进行模式替换             │
│                                                                 │
│  2. OTEL_LOG_USER_PROMPTS 环境变量                             │
│     - 默认 false：不记录用户 prompt 内容                        │
│     - 设为 true：记录完整的 prompt（仅用于调试）                │
│     - 仅在用户明确启用时生效                                    │
│                                                                 │
│  3. Bedrock/Vertex 禁用非必要流量                              │
│     - 使用 AWS Bedrock 或 GCP Vertex 时                        │
│     - 自动禁用 Statsig 和 GrowthBook 遥测                      │
│     - 仅保留 OpenTelemetry（由企业管理员控制）                  │
│     - 确保数据不离开企业基础设施                                │
│                                                                 │
│  4. 数据最小化原则                                            │
│     - 仅收集必要的指标和事件                                    │
│     - 不记录对话内容（除非用户明确启用）                        │
│     - 错误日志中自动脱敏文件路径和命令内容                      │
└─────────────────────────────────────────────────────────────────┘
```

### 认证系统

#### OAuth 2.0 PKCE 流程（8 步）

Claude Code 使用 OAuth 2.0 PKCE（Proof Key for Code Exchange）进行认证，而非传统的 OAuth 授权码流程。PKCE 的选择理由：
- **安全性**：即使授权码被截获，攻击者也无法交换 token（没有 code_verifier）
- **无需 client_secret**：CLI 应用无法安全存储 client_secret，PKCE 消除了这个需求
- **公共客户端友好**：原生应用和 CLI 工具的最佳实践

```
┌─────────────────────────────────────────────────────────────────┐
│                 OAuth 2.0 PKCE 8 步流程                         │
│                                                                 │
│  Step 1: 生成 PKCE 参数                                        │
│    code_verifier = randomURLSafeString(128)                    │
│    code_challenge = SHA256(code_verifier) -> base64url          │
│                                                                 │
│  Step 2: 打开浏览器授权页面                                    │
│    GET https://console.anthropic.com/oauth/authorize            │
│      ?client_id=claude-code                                    │
│      &response_type=code                                       │
│      &redirect_uri=http://localhost:PORT/callback              │
│      &code_challenge={code_challenge}                          │
│      &code_challenge_method=S256                               │
│      &scope=openid profile email                                │
│                                                                 │
│  Step 3: 用户在浏览器中登录并授权                               │
│    Anthropic 授权服务器验证用户身份                              │
│                                                                 │
│  Step 4: 回调重定向到本地 HTTP 服务器                           │
│    Claude Code 启动临时 HTTP 服务器监听回调                     │
│    GET http://localhost:PORT/callback?code=AUTH_CODE            │
│                                                                 │
│  Step 5: 用授权码交换 token                                     │
│    POST https://console.anthropic.com/oauth/token               │
│      {                                                          │
│        grant_type: 'authorization_code',                        │
│        code: AUTH_CODE,                                         │
│        redirect_uri: 'http://localhost:PORT/callback',          │
│        client_id: 'claude-code',                                │
│        code_verifier: CODE_VERIFIER  // PKCE 关键步骤           │
│      }                                                          │
│                                                                 │
│  Step 6: 获取 access_token + refresh_token                     │
│    {                                                          │
│      access_token: 'eyJ...',                                   │
│      refresh_token: 'eyJ...',                                  │
│      expires_in: 3600,                                         │
│      token_type: 'Bearer'                                      │
│    }                                                          │
│                                                                 │
│  Step 7: 存储 token 到安全存储                                  │
│    macOS -> Keychain                                           │
│    Windows/Linux -> ~/.claude/credentials.json (加密)           │
│                                                                 │
│  Step 8: 关闭临时 HTTP 服务器                                   │
└─────────────────────────────────────────────────────────────────┘
```

#### Token 管理

```typescript
// Token 管理策略

interface TokenManager {
  accessToken: string | null;
  refreshToken: string | null;
  expiresAt: number | null;

  // 5 分钟缓冲：在 token 过期前 5 分钟主动刷新
  REFRESH_BUFFER_MS: 300_000;

  async getValidToken(): Promise<string> {
    // 检查是否有有效 token
    if (this.accessToken && this.expiresAt) {
      const now = Date.now();
      const buffer = this.REFRESH_BUFFER_MS;

      if (now < this.expiresAt - buffer) {
        return this.accessToken; // token 仍然有效
      }

      // 即将过期，主动刷新
      try {
        await this.refreshAccessToken();
        return this.accessToken!;
      } catch (error) {
        // 刷新失败，使用旧 token 直到它真正过期
        if (now < this.expiresAt) {
          return this.accessToken!;
        }
        throw error;
      }
    }

    // 没有 token，需要重新认证
    throw new Error('No valid token. Please run claude auth login.');
  }
}
```

#### 认证解析链（6 级优先级）

```
┌─────────────────────────────────────────────────────────────────┐
│                 认证解析链（6 级优先级）                          │
│                                                                 │
│  优先级    来源                    说明                          │
│  ──────    ────                    ────                          │
│  最高      3P context              第三方集成（IDE/SDK）提供     │
│            的预配置认证                                           │
│                                                                 │
│  高        bare mode                CLAUDE_CODE_BARE=1           │
│            无认证模式              直接使用 API（无 OAuth）       │
│                                                                 │
│  中高      managed OAuth            企业 MDM 管理的 OAuth        │
│            MDM 策略提供            策略强制指定认证方式          │
│                                                                 │
│  中        explicit tokens          环境变量显式指定              │
│            ANTHROPIC_API_KEY        API key（直接使用）          │
│            ANTHROPIC_AUTH_TOKEN     OAuth token（直接使用）      │
│                                                                 │
│  中低      OAuth                    本地存储的 OAuth token       │
│            Keychain/credentials     自动刷新                    │
│                                                                 │
│  最低      API key                  ~/.claude/credentials        │
│            本地存储的 API key       中的 api_key 字段            │
└─────────────────────────────────────────────────────────────────┘
```

#### 安全存储

```
┌─────────────────────────────────────────────────────────────────┐
│                 安全存储策略                                     │
│                                                                 │
│  macOS:                                                         │
│    - 使用系统 Keychain 存储 OAuth token 和 refresh token        │
│    - 服务名称: "com.anthropic.claude-code"                      │
│    - Keychain 缓存 TTL: 5 分钟（避免频繁 Keychain 访问）       │
│    - Stale-while-error: Keychain 访问失败时使用内存缓存         │
│                                                                 │
│  Windows / Linux:                                               │
│    - 使用 ~/.claude/credentials.json（明文）                    │
│    - 文件权限设置为 600（仅用户可读写）                         │
│    - 无 Keychain 缓存（每次直接读取文件）                       │
│    - 未来计划支持 libsecret (Linux) 和 Credential Manager (Win) │
│                                                                 │
│  Token 刷新调度：                                               │
│    - 启动时检查 token 有效性                                    │
│    - 每次 API 调用前检查 token 有效性                          │
│    - 过期前 5 分钟主动刷新                                      │
│    - 刷新失败时使用旧 token 直到真正过期                        │
│    - refresh_token 过期时触发重新认证流程                       │
└─────────────────────────────────────────────────────────────────┘
```

#### MDM 策略（3 平台路径）

```
┌─────────────────────────────────────────────────────────────────┐
│                 MDM 策略（3 平台路径）                           │
│                                                                 │
│  macOS:                                                         │
│    /Library/Managed Preferences/com.anthropic.claude-code.plist  │
│    - 通过 MDM 配置文件推送                                      │
│    - 支持强制权限模式、禁用功能、配置 MCP 服务器               │
│                                                                 │
│  Windows:                                                       │
│    HKLM\Software\Policies\Anthropic\ClaudeCode                  │
│    - 通过注册表推送                                             │
│    - 支持 Group Policy 配置                                     │
│                                                                 │
│  Linux:                                                         │
│    /etc/claude-code/policy.json                                 │
│    - 通过配置管理工具推送                                       │
│    - 支持 Ansible/Puppet/Chef 集成                              │
│                                                                 │
│  MDM 可控制的策略：                                             │
│  - permissionMode: 强制权限模式                                 │
│  - disabledFeatures: 禁用功能列表                               │
│  - allowedMcpServers: 允许的 MCP 服务器白名单                   │
│  - maxCostPerSession: 单会话最大成本                            │
│  - auditLogEnabled: 是否启用审计日志                            │
│  - telemetryEnabled: 是否启用遥测                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Codex CLI 实现

### 遥测系统

#### 内置 OpenTelemetry SDK

Codex CLI 内置了 OpenTelemetry SDK，通过 `codex-otel` crate 实现：

```toml
# config.toml 中的 OTel 配置
[otel]
enabled = true
endpoint = "http://localhost:4318"  # OTLP exporter endpoint
```

#### 5 种核心指标

| 指标名称 | 类型 | 说明 |
|----------|------|------|
| `feature.state` | Gauge | 特性标志状态 |
| `approval.requested` | Counter | 审批请求次数 |
| `tool.call` | Counter | 工具调用次数 |
| `conversation.turn.count` | Counter | 对话轮次计数 |
| `shell_snapshot` | Histogram | Shell 命令执行快照 |

#### AnalyticsEventsClient

```rust
/// 分析事件客户端
pub struct AnalyticsEventsClient {
    /// 事件发送端点
    endpoint: String,

    /// 安装 ID
    installation_id: String,
}

impl AnalyticsEventsClient {
    /// 发送分析事件
    pub async fn track_event(&self, event: AnalyticsEvent) {
        // POST /codex/analytics-events/events
    }

    /// 跟踪压缩事件
    pub async fn track_compaction(&self, event: CodexCompactionEvent) {
        // 记录压缩触发原因、持续时间、token 变化等
    }
}
```

**插件遥测事件：**

```rust
// 插件安装/禁用事件
AnalyticsEvent::PluginInstalled { plugin_id, version }
AnalyticsEvent::PluginDisabled { plugin_id, reason }
```

#### 隐私保护

- **默认不记录 prompt**：系统提示和用户输入不会被发送到遥测端点
- **LLM_CLI_TELEMETRY_DISABLED**：设置此环境变量可完全禁用遥测
- **本地优先**：所有会话数据存储在本地 `~/.codex/` 目录

### 认证系统

#### ChatGPT 账户登录（3 种流程）

**浏览器环境（localhost:1455）：**

```
┌──────────────────────────────────────────────────────────────┐
│  ChatGPT OAuth 流程（浏览器环境）                             │
│                                                               │
│  1. codex login                                              │
│     │                                                        │
│     ▼                                                        │
│  2. 启动本地 HTTP 服务器 (localhost:1455)                     │
│     │                                                        │
│     ▼                                                        │
│  3. 打开默认浏览器，跳转到 ChatGPT 授权页面                    │
│     │                                                        │
│     ▼                                                        │
│  4. 用户在浏览器中登录并授权                                   │
│     │                                                        │
│     ▼                                                        │
│  5. ChatGPT 重定向到 localhost:1455/callback?code=...        │
│     │                                                        │
│     ▼                                                        │
│  6. 本地服务器接收授权码，交换 access_token                    │
│     │                                                        │
│     ▼                                                        │
│  7. 存储 token 到 keyring                                     │
└──────────────────────────────────────────────────────────────┘
```

**无浏览器环境（设备码认证流程 3 步）：**

```
1. codex login
   -> 显示设备码: XXXX-XXXX
   -> 显示验证 URL: https://chatgpt.com/device

2. 用户在其他设备上打开 URL，输入设备码

3. codex login 轮询等待授权完成
   -> 授权成功，存储 token
```

**远程服务器（SSH 端口转发）：**

```bash
# 在远程服务器上
codex login

# 在本地机器上建立端口转发
ssh -L 1455:localhost:1455 user@remote-server

# 然后在本地浏览器中完成授权
```

#### API Key 登录

```bash
# 方式 1: 通过命令行
codex login --api-key sk-...

# 方式 2: 通过环境变量
export OPENAI_API_KEY=sk-...
```

#### keyring-store 密钥管理

密钥管理通过 `codex-keyring-store` crate 实现，支持 4 个平台：

| 平台 | 后端 | 说明 |
|------|------|------|
| **macOS** | Keychain | 系统密钥链 |
| **Windows** | Credential Manager | Windows 凭据管理器 |
| **Linux** | DBus + keyutils | 通过 DBus 协议访问系统密钥环 |
| **FreeBSD** | DBus | 通过 DBus 协议 |

**三种存储模式：**

```rust
pub enum OAuthCredentialsStoreMode {
    /// 自动选择（优先 Keyring，回退到文件）
    Auto,

    /// 仅使用 Keyring
    Keyring,

    /// 仅使用文件存储
    File,
}
```

**安全设计：**

```rust
/// 密钥服务名称
const KEYRING_SERVICE: &str = "codex-cli";

/// 使用 SHA-256 哈希作为存储键
fn hash_key(key: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 刷新令牌时间偏移（30 秒）
const REFRESH_SKEW_MILLIS: u64 = 30_000;
```

---

## 对比分析

### 遥测架构对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **架构层次** | 三层（Statsig + Sentry + OTel） | 单层（内置 OTel + Analytics） |
| **核心指标数** | 8 种 | 5 种 |
| **核心事件数** | 5 种 | 通过 AnalyticsEventsClient 自定义 |
| **A/B 测试** | 支持（Statsig + GrowthBook） | 不支持 |
| **错误追踪** | Sentry 集成 | 无独立错误追踪 |
| **企业可观测性** | OTel 管理员可配置导出器 | OTel endpoint 配置 |
| **完全禁用** | 无全局开关 | `LLM_CLI_TELEMETRY_DISABLED` 环境变量 |
| **Bedrock/Vertex** | 自动禁用非必要遥测 | N/A |
| **Prompt 脱敏** | 默认哈希处理 | 默认不记录 |

### 认证流程对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **OAuth 流程** | PKCE 8 步 | 标准 OAuth + 设备码 + SSH 转发 |
| **认证流程数** | 1 种（PKCE） | 3 种（浏览器/设备码/SSH） |
| **认证解析链** | 6 级优先级 | 未明确分层 |
| **API Key 支持** | 环境变量 | 命令行 + 环境变量 |
| **Bare Mode** | 支持（CLAUDE_CODE_BARE=1） | 无对应模式 |
| **MDM 策略** | 3 平台路径 | 无 |
| **Token 缓冲** | 5 分钟 | 30 秒 |

### 密钥存储方案对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **macOS** | Keychain（5 分钟缓存） | Keychain |
| **Windows** | credentials.json（权限 600） | Credential Manager |
| **Linux** | credentials.json（权限 600） | DBus + keyutils |
| **FreeBSD** | 不支持 | DBus |
| **存储模式** | 固定（平台决定） | 3 种（Auto/Keyring/File） |
| **密钥哈希** | 无 | SHA-256 哈希存储键 |
| **Stale-while-error** | 支持（Keychain 缓存） | 未提及 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| 三层遥测架构覆盖运营、错误、企业监控，全面性强 | 遥测架构复杂，依赖多个外部服务（Statsig/Sentry/OTel） |
| 8 种核心指标 + 5 种核心事件，覆盖业务全链路 | 无全局遥测禁用开关，隐私控制粒度不够 |
| Bedrock/Vertex 自动禁用非必要遥测，企业友好 | Statsig 和 Sentry 数据发送到第三方 CDN |
| OAuth PKCE 8 步流程安全性高，无 client_secret 泄露风险 | 仅支持 1 种认证流程，无设备码/SSH 转发支持 |
| 6 级认证解析链覆盖所有使用场景（IDE/MDM/API Key/OAuth） | Windows/Linux 使用明文 credentials.json 存储 |
| MDM 策略支持 3 平台，企业部署友好 | Linux/Windows 缺少系统级密钥链支持 |
| 5 分钟 Token 刷新缓冲，减少认证中断 | Keychain 缓存仅 macOS 支持 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| 3 种认证流程（浏览器/设备码/SSH），适应各种环境 | 无 PKCE 保护（依赖标准 OAuth 流程） |
| `LLM_CLI_TELEMETRY_DISABLED` 全局禁用开关，隐私友好 | 遥测架构简单，缺少 A/B 测试和错误追踪 |
| keyring-store 支持 4 平台，包括 FreeBSD | 仅 5 种核心指标，覆盖面有限 |
| 3 种存储模式（Auto/Keyring/File）灵活可控 | 无 MDM 策略支持，企业部署能力弱 |
| SHA-256 哈希存储键，安全性更高 | 无认证解析链，优先级不明确 |
| Windows 使用 Credential Manager，安全性优于明文文件 | 30 秒刷新偏移较短，可能频繁触发刷新 |
| API Key 支持命令行直接传入，使用便捷 | 无 bare mode 或第三方集成认证支持 |
# IDE 集成与 LSP 对比

## Claude Code 实现

### IDE 集成 -- Bridge 协议

#### Bridge 协议架构

```
┌─────────────────────────────────────────────────────────────────┐
│                 Bridge 协议架构                                   │
│                                                                 │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────────┐ │
│  │ IDE          │     │ Bridge Layer │     │ Claude Code Core │ │
│  │ Extension    │<───>│ (33 files)   │<───>│ (query.ts, etc)  │ │
│  │              │     │              │     │                  │ │
│  │ - VS Code    │     │ - Protocol   │     │ - QueryEngine    │ │
│  │ - JetBrains  │     │ - Transport  │     │ - Tool Registry  │ │
│  │ - Neovim     │     │ - Auth       │     │ - Permission     │ │
│  │ - Emacs      │     │ - Lifecycle  │     │ - Hooks          │ │
│  └──────────────┘     └──────────────┘     └──────────────────┘ │
│                                                                 │
│  通信协议：JSON-RPC over stdio / WebSocket                      │
│                                                                 │
│  核心文件列表（13 个关键文件）：                                 │
│  bridge/                                                        │
│  ├── protocol.ts           # Bridge 协议定义                    │
│  ├── transport.ts          # 传输层抽象（stdio/ws）              │
│  ├── auth.ts               # Bridge 认证                        │
│  ├── lifecycle.ts          # 生命周期管理                       │
│  ├── messageRouter.ts      # 消息路由                           │
│  ├── sessionManager.ts     # 会话管理                           │
│  ├── notificationHandler.ts # 通知处理                          │
│  ├── fileWatcher.ts        # 文件监视同步                       │
│  ├── diffViewer.ts         # Diff 查看器集成                    │
│  ├── planMode.ts           # Plan 模式集成                      │
│  ├── ideMcpServer.ts       # IDE MCP 服务器                     │
│  ├── contextProvider.ts    # 上下文提供者                       │
│  └── healthCheck.ts        # 健康检查                           │
└─────────────────────────────────────────────────────────────────┘
```

#### VS Code 集成

- **官方扩展**：Claude Code 提供 VS Code 扩展，通过 Bridge 协议与核心通信
- **IDE MCP 服务器**：每个 IDE 实例启动一个 MCP 服务器，Claude Code 通过 MCP 获取 IDE 上下文（打开的文件、选中的代码、诊断信息等）
- **实时同步**：文件变更、光标位置、选区变更实时同步到 Claude Code
- **diff viewer**：工具执行结果中的文件变更通过 IDE 的 diff viewer 展示
- **Plan 模式**：在 IDE 中显示 Claude 的执行计划，用户可以审查后批准

### LSP 集成

#### LSPClient.ts

Claude Code 内置了 LSP（Language Server Protocol）客户端，可以直接与语言服务器通信获取代码智能信息：

```typescript
// services/lsp/LSPClient.ts -- JSON-RPC over stdio

export class LSPClient {
  private process: ChildProcess;
  private messageId: number = 0;
  private pendingRequests: Map<number, {
    resolve: (result: any) => void;
    reject: (error: Error) => void;
  }> = new Map();
  private eventQueue: LSPEvent[] = [];
  private initialized: boolean = false;

  constructor(
    private serverCommand: string,
    private serverArgs: string[],
    private rootUri: string,
  ) {}

  async start(): Promise<void> {
    // 启动语言服务器进程
    this.process = spawn(this.serverCommand, this.serverArgs, {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: this.rootUri,
    });

    // 监听 stdout 的 JSON-RPC 消息
    this.process.stdout.on('data', (data: Buffer) => {
      const messages = parseLSPMessages(data);
      for (const msg of messages) {
        this.handleMessage(msg);
      }
    });

    // 发送 initialize 请求
    await this.sendRequest('initialize', {
      processId: process.pid,
      rootUri: this.rootUri,
      capabilities: {
        textDocument: {
          completion: { completionItem: { snippetSupport: false } },
          hover: { contentFormat: ['markdown', 'plaintext'] },
          definition: { linkSupport: true },
          references: {},
          typeDefinition: { linkSupport: true },
          callHierarchy: { prepareSupport: true },
        },
      },
    });

    // 发送 initialized 通知
    this.sendNotification('initialized', {});
    this.initialized = true;
  }

  // 延迟队列：确保服务器初始化完成后再处理请求
  private async ensureInitialized(): Promise<void> {
    if (!this.initialized) {
      await new Promise(resolve => {
        const check = () => {
          if (this.initialized) resolve();
          else setTimeout(check, 100);
        };
        check();
      });
    }
  }
}
```

#### LSPServerManager.ts

管理多个 LSP 服务器实例，按文件扩展名路由到对应的语言服务器：

```typescript
// services/lsp/LSPServerManager.ts -- 多实例管理

export class LSPServerManager {
  private clients: Map<string, LSPClient> = new Map();
  private extensionMap: Map<string, string> = new Map();

  constructor() {
    // 文件扩展名 -> 语言服务器映射
    this.extensionMap.set('.ts', 'typescript');
    this.extensionMap.set('.tsx', 'typescript');
    this.extensionMap.set('.js', 'typescript');
    this.extensionMap.set('.jsx', 'typescript');
    this.extensionMap.set('.py', 'python');
    this.extensionMap.set('.go', 'gopls');
    this.extensionMap.set('.rs', 'rust-analyzer');
    this.extensionMap.set('.java', 'jdtls');
    this.extensionMap.set('.cpp', 'clangd');
    this.extensionMap.set('.c', 'clangd');
  }

  // 按需启动：第一次访问某语言时才启动对应的服务器
  async getClientForFile(filePath: string): Promise<LSPClient | null> {
    const ext = path.extname(filePath);
    const language = this.extensionMap.get(ext);

    if (!language) return null;

    if (!this.clients.has(language)) {
      const serverConfig = this.getServerConfig(language);
      if (!serverConfig) return null;

      const client = new LSPClient(
        serverConfig.command,
        serverConfig.args,
        this.rootUri,
      );
      await client.start();
      this.clients.set(language, client);
    }

    return this.clients.get(language)!;
  }

  // 关闭所有服务器
  async dispose(): Promise<void> {
    for (const client of this.clients.values()) {
      await client.dispose();
    }
    this.clients.clear();
  }
}
```

#### 支持的 LSP 操作（10 种）

| 操作 | LSP 方法 | 用途 |
|------|----------|------|
| 代码导航 | `textDocument/definition` | 跳转到定义 |
| 代码信息 | `textDocument/hover` | 获取类型信息、文档 |
| 查找引用 | `textDocument/references` | 查找所有引用位置 |
| 类型定义 | `textDocument/typeDefinition` | 跳转到类型定义 |
| 调用层次 | `callHierarchy/incomingCalls` | 查找调用者 |
| 调用层次 | `callHierarchy/outgoingCalls` | 查找被调用者 |
| 符号搜索 | `workspace/symbol` | 全局符号搜索 |
| 文件生命周期 | `textDocument/didOpen` | 通知服务器文件已打开 |
| 文件生命周期 | `textDocument/didChange` | 通知服务器文件已变更 |
| 文件生命周期 | `textDocument/didClose` | 通知服务器文件已关闭 |

---

## Codex CLI 实现

### IDE 集成 -- app-server 协议

Codex CLI 通过 `app-server` 协议与 IDE 集成，使用 **JSON-RPC 2.0** 双向通信。

#### 传输层

| 传输方式 | 说明 | 状态 |
|----------|------|------|
| **stdio** | 通过标准输入/输出通信（默认） | 稳定 |
| **websocket** | 通过 WebSocket 通信 | 实验性 |

#### 核心原语

| 原语 | 说明 |
|------|------|
| **Thread** | 对话线程（对应一个会话） |
| **Turn** | 对话轮次（一次用户输入到模型完成响应） |
| **Item** | 轮次中的项目（消息、工具调用、工具结果） |

#### 关键 API 列表

| API | 方法 | 说明 |
|-----|------|------|
| `thread/start` | POST | 创建新对话线程 |
| `thread/resume` | POST | 恢复已有线程 |
| `thread/fork` | POST | 分叉线程 |
| `turn/start` | POST | 开始新轮次 |
| `turn/steer` | POST | 引导当前轮次 |
| `turn/interrupt` | POST | 中断当前轮次 |
| `plugin/*` | POST | 插件管理 |
| `config/*` | POST | 配置管理 |
| `review/start` | POST | 开始代码审查 |
| `feedback/upload` | POST | 上传用户反馈 |

#### 初始化握手

```json
// 客户端 -> 服务器
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "clientInfo": { "name": "vscode-codex", "version": "1.0.0" },
    "capabilities": {}
  },
  "id": 1
}

// 服务器 -> 客户端
{
  "jsonrpc": "2.0",
  "result": {
    "serverInfo": { "name": "codex-app-server", "version": "0.1.0" },
    "capabilities": {}
  },
  "id": 1
}

// 客户端 -> 服务器（确认初始化完成）
{
  "jsonrpc": "2.0",
  "method": "initialized",
  "params": {}
}
```

#### 背压处理

当客户端无法及时处理事件时，服务器通过有界队列实现背压：

```rust
/// 有界事件队列（防止内存溢出）
const EVENT_QUEUE_CAPACITY: usize = 1024;

/// 背压错误码
const BACKPRESSURE_ERROR_CODE: i32 = -32001;

// 当队列满时，返回背压错误
if event_queue.is_full() {
    return Err(JsonRpcError {
        code: BACKPRESSURE_ERROR_CODE,
        message: "Event queue full, client is too slow".to_string(),
        data: None,
    });
}
```

### LSP 集成

Codex CLI **没有内置 LSP 客户端**。代码智能功能依赖 IDE 自身的语言服务器提供。Codex CLI 的 IDE 集成主要通过 app-server 协议与 IDE 扩展通信，但不直接与语言服务器交互。

---

## 对比分析

### IDE 集成协议对比

| 维度 | Claude Code (Bridge) | Codex CLI (app-server) |
|------|---------------------|----------------------|
| **协议** | JSON-RPC over stdio/WebSocket | JSON-RPC 2.0 over stdio/WebSocket |
| **文件规模** | 33 文件，13 个关键文件 | 未公开详细文件数 |
| **认证** | JWT 认证 | capabilities 交换 |
| **IDE 支持** | VS Code、JetBrains、Neovim、Emacs | VS Code（社区扩展） |
| **实时同步** | 文件变更、光标位置、选区变更 | 无明确实时同步 |
| **diff viewer** | 集成 IDE diff viewer | 无 |
| **Plan 模式** | IDE 中显示执行计划 | 无 |
| **MCP 服务器** | IDE MCP 服务器提供上下文 | 无 IDE MCP |
| **线程管理** | 会话管理 | thread/start/resume/fork |
| **轮次控制** | 无 | turn/start/steer/interrupt |
| **背压处理** | 未明确 | 有界队列（1024 容量） |
| **代码审查** | 无 | review/start API |

### LSP 集成对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **内置 LSP 客户端** | 有（LSPClient.ts） | 无 |
| **多实例管理** | 有（LSPServerManager） | N/A |
| **支持语言数** | 10+ 种（TS/Python/Go/Rust/Java/C++ 等） | N/A |
| **LSP 操作数** | 10 种 | N/A |
| **按需启动** | 支持（首次访问时启动） | N/A |
| **代码导航** | 跳转定义、类型定义、引用查找 | 依赖 IDE |
| **调用层次** | incomingCalls/outgoingCalls | 依赖 IDE |
| **符号搜索** | workspace/symbol | 依赖 IDE |
| **文件生命周期** | didOpen/didChange/didClose | 依赖 IDE |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| Bridge 协议 33 文件，IDE 集成最为完善 | Bridge 层代码量大，维护成本高 |
| 内置 LSP 客户端，独立于 IDE 提供代码智能 | LSP 服务器需用户本地安装，增加依赖 |
| 支持 4 种 IDE（VS Code/JetBrains/Neovim/Emacs） | 非 VS Code IDE 的集成质量可能不一致 |
| 实时同步光标位置和选区，交互体验优秀 | 实时同步增加通信开销 |
| IDE MCP 服务器提供丰富上下文（诊断/打开文件/选区） | 无背压处理机制，IDE 卡顿可能影响 CLI |
| diff viewer 集成，代码变更可视化效果好 | Plan 模式仅在部分 IDE 中支持 |
| 10 种 LSP 操作覆盖代码导航全链路 | LSPServerManager 按需启动有冷启动延迟 |
| JWT 认证保护 Bridge 通信安全 | JWT 密钥管理复杂度较高 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| JSON-RPC 2.0 标准协议，实现简洁 | IDE 集成功能相对简单 |
| 有界队列背压处理，防止内存溢出 | 无内置 LSP 客户端，代码智能依赖 IDE |
| thread/start/resume/fork 线程管理灵活 | 无实时同步（光标/选区/文件变更） |
| turn/start/steer/interrupt 轮次控制精细 | 无 diff viewer 集成 |
| review/start API 支持代码审查工作流 | 无 IDE MCP 服务器，上下文获取能力弱 |
| WebSocket 传输支持远程 IDE 连接 | WebSocket 传输仍为实验性 |
| capabilities 交换机制标准化 | capabilities 未充分利用（当前为空对象） |
| 轻量级协议，资源消耗低 | 仅 VS Code 有社区扩展，IDE 覆盖面窄 |
# UI/UX 实现对比

## Claude Code 实现

### React + Ink 终端渲染框架

Claude Code 采用 **React + Ink** 声明式终端 UI 框架，这是目前终端应用中最为成熟的声明式渲染方案。

```
┌─────────────────────────────────────────────────────────────────┐
│                 Claude Code UI 架构                              │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ React/Ink 渲染引擎                                      │   │
│  │  - 声明式组件模型                                        │   │
│  │  - 虚拟 DOM 差异更新                                     │   │
│  │  - Hooks 状态管理                                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  组件规模：                                                      │
│  - 140+ React/Ink 组件                                        │
│  - 70+ React Hooks                                             │
│  - 完整 React 生态模式                                          │
│                                                                 │
│  目录结构：                                                      │
│  src/                                                           │
│  ├── components/              # UI 组件库 (~140 组件)           │
│  ├── screens/                 # 全屏视图组件                    │
│  │   ├── REPL.tsx             # 主交互式 REPL                   │
│  │   ├── Doctor.tsx           # 环境诊断                       │
│  │   └── ResumeConversation.tsx # 会话恢复                     │
│  ├── hooks/                   # React Hooks (70+)               │
│  │   ├── useCanUseTool.tsx    # 核心权限决策 Hook               │
│  │   ├── useTextInput.ts      # 文本输入                       │
│  │   ├── useVimInput.ts       # Vim 模式输入                   │
│  │   ├── toolPermission/      # 工具权限 UI 子系统              │
│  │   └── notifs/              # 通知子系统                      │
│  ├── vim/                     # Vim 模式实现（完整状态机）       │
│  ├── voice/                   # 语音系统                        │
│  ├── keybindings/             # 键绑定系统（50+ 动作）           │
│  └── ink/                     # Ink 渲染引擎扩展                 │
└─────────────────────────────────────────────────────────────────┘
```

### REPL 主界面

REPL（Read-Eval-Print Loop）是 Claude Code 的主交互界面，由核心组件组成：

- **PromptInput**：用户输入组件，支持多行输入、斜杠命令自动补全
- **Messages**：消息列表组件，渲染 AI 响应、工具调用结果、系统消息
- **useSendMessage** Hook：处理用户输入到 QueryEngine 的消息流
- **useQueryEvents** Hook：消费 QueryEngine 的流式事件并驱动 UI 更新

### Vim 模式

Claude Code 内置了完整的 Vim 编辑模式，实现为独立的状态机：

- **位置**：`src/vim/` 目录
- **状态机**：完整的 Vim 模式切换（Normal/Insert/Visual/Command）
- **Hook 集成**：`useVimInput.ts` Hook 将 Vim 模式集成到输入系统
- **键绑定**：与 Claude Code 的键绑定系统无缝集成

### 语音模式

Claude Code 支持语音输入/输出功能：

- **特性标志**：`VOICE_MODE`（通过 `bun:bundle` 特性标志控制）
- **位置**：`src/voice/` 目录
- **服务入口**：`voice.ts` 语音服务
- **构建优化**：未激活时通过死代码消除（DCE）完全剥离，节省约 200KB

### 键绑定系统

- **位置**：`src/keybindings/` 目录
- **规模**：50+ 动作映射
- **可扩展性**：支持用户自定义键绑定
- **Vim 集成**：与 Vim 模式共享键绑定基础设施

### Doctor 环境诊断界面

- **位置**：`screens/Doctor.tsx`
- **功能**：检测运行环境问题（Bun 版本、API 认证、网络连接等）
- **输出**：结构化的诊断报告，包含问题和建议

### Plan 模式

- **markdown 文档内联评论**：在 IDE 中显示 Claude 的执行计划
- **用户审查**：用户可以在执行前审查和修改计划
- **IDE 集成**：通过 Bridge 协议与 IDE 同步

### 状态管理

Claude Code 使用自定义极简 Store 实现（替代 Zustand），通过 React Context 注入：

```typescript
// React Context 使用方式
export function AppStateProvider({ children }: { children: React.ReactNode }) {
  // 自定义 Store 注入
  return (
    <AppContext.Provider value={store}>
      {children}
    </AppContext.Provider>
  );
}

// 组件中消费状态
function MyComponent() {
  const store = useStore();
  // ...
}
```

---

## Codex CLI 实现

### Ratatui + crossterm 命令式终端 UI

Codex CLI 采用 **Ratatui 0.29 + crossterm 0.28** 命令式终端 UI 框架，这是 Rust 生态中最流行的终端 UI 方案。

```
┌─────────────────────────────────────────────────────────────────┐
│                 Codex CLI UI 架构                                │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Ratatui 渲染引擎                                        │   │
│  │  - 命令式渲染模型                                        │   │
│  │  - 即时模式（Immediate Mode）绘制                        │   │
│  │  - crossterm 跨平台终端控制                              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  入口模式：                                                      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                      │
│  │   cli   │    │   tui   │    │   exec  │                      │
│  │ 多工具  │    │ 全屏 UI │    │ 无头    │                      │
│  │ 入口    │    │ Ratatui │    │ 模式    │                      │
│  └─────────┘    └─────────┘    └─────────┘                      │
│                                                                 │
│  TUI 核心组件：                                                  │
│  - BottomPane: 底部面板（view_stack 视图栈）                     │
│  - ApprovalOverlay: 审批覆盖层                                  │
│  - 聊天消息区域                                                  │
│  - 键盘输入处理                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 全屏 TUI 模式（codex tui）

`codex tui` 启动全屏终端 UI，基于 Ratatui 构建：

- **聊天消息区域**：显示 AI 响应和工具调用结果
- **BottomPane**：底部面板，使用 `view_stack` 管理多个视图
- **ApprovalOverlay**：工具执行审批覆盖层

#### 用户确认 UI

当命令需要用户审批时，TUI 层通过 `BottomPane` 的 `view_stack` 显示审批覆盖层：

```
┌──────────────────────────────────────────────────────────────┐
│  TUI 主界面                                                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ 聊天消息区域                                            │  │
│  │ ...                                                    │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ BottomPane (view_stack)                                │  │
│  │ ┌──────────────────────────────────────────────────┐  │  │
│  │ │ ApprovalOverlay                                  │  │  │
│  │ │                                                   │  │  │
│  │ │  命令需要审批                                     │  │  │
│  │ │                                                   │  │  │
│  │ │  命令: rm -rf /tmp/build                          │  │  │
│  │ │  工作目录: /home/user/project                      │  │  │
│  │ │                                                   │  │  │
│  │ │  [y] 允许  [n] 拒绝  [a] 始终允许此类命令          │  │  │
│  │ └──────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

用户确认后，通过 `Op::ExecApproval` 发送审批决策：

```rust
Op::ExecApproval {
    id: "approval_request_id".to_string(),
    approved: true,  // 或 false
}
```

### 无头模式（codex exec）

`codex exec` 提供无头（headless）执行模式，适合自动化和脚本集成：

- **无 UI 渲染**：直接输出文本结果
- **脚本友好**：适合 CI/CD 管道
- **管道集成**：支持 stdin/stdout 数据流

### 实时语音（WebRTC + Opus）

Codex CLI 的实时语音通信采用 **WebRTC** 技术：

```
┌──────────────────────────────────────────────────────────────┐
│                 WebRTC 实时通信架构                            │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 音频编解码: opus-rs                                   │   │
│  │  - 低延迟音频编解码                                   │   │
│  │  - 自适应比特率                                       │   │
│  │  - 适用于语音交互场景                                  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Data Channel: 控制消息                                │   │
│  │  - 文本消息传输                                       │   │
│  │  - 会话控制命令                                       │   │
│  │  - 可靠有序传输                                       │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ RTP Track: 音频流                                     │   │
│  │  - 实时音频传输                                       │   │
│  │  - 适用于语音对话场景                                  │   │
│  │  - 低延迟优先                                         │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  性能对比：                                                   │
│  | 维度       | WebSocket | WebRTC |                         │
│  | 延迟       | ~50-100ms | ~10-30ms |                       │
│  | 音频质量   | 一般      | 优秀（opus 编解码）|              │
│  | NAT 穿透   | 需要代理  | 内置 ICE/STUN/TURN |             │
└──────────────────────────────────────────────────────────────┘
```

### TUI 恢复选择器

Codex CLI 提供 TUI 恢复选择器，用于恢复之前的会话：

- **分页显示**：每页 25 条会话记录
- **过滤排序**：支持按时间、项目等条件过滤和排序
- **会话恢复**：选择后恢复到之前的对话状态

### 代码高亮

Codex CLI 使用 **tree-sitter** 进行代码语法高亮，相比 Claude Code 的内置实现，tree-sitter 提供更精确的语法解析和更丰富的语言支持。

---

## 对比分析

### UI 框架对比

| 维度 | Claude Code (React + Ink) | Codex CLI (Ratatui + crossterm) |
|------|--------------------------|-------------------------------|
| **渲染模型** | 声明式（虚拟 DOM） | 命令式（即时模式） |
| **组件数量** | 140+ 组件 | 未公开（相对较少） |
| **Hooks 数量** | 70+ React Hooks | N/A（Rust 无对应概念） |
| **状态管理** | React Context + 自定义 Store | Rust 所有权 + 事件队列 |
| **开发效率** | 高（React 生态成熟） | 中（Ratatui 学习曲线） |
| **运行时性能** | 依赖 Bun 优化 | Rust 原生性能 |
| **终端样式** | Chalk | crossterm |
| **代码高亮** | 内置实现 | tree-sitter |

### 交互模式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **主界面** | REPL（PromptInput + Messages） | 全屏 TUI（聊天区域 + BottomPane） |
| **Vim 模式** | 完整状态机（src/vim/） | 无 |
| **语音模式** | VOICE_MODE 标志（src/voice/） | WebRTC + Opus 实时语音 |
| **无头模式** | SDK/QueryEngine 入口 | codex exec |
| **键绑定** | 50+ 动作（src/keybindings/） | 基础键盘处理 |
| **环境诊断** | Doctor 界面 | 无 |
| **Plan 模式** | markdown 内联评论 | 无 |
| **会话恢复** | ResumeConversation 屏幕 | TUI 恢复选择器（分页 25 条） |
| **审批 UI** | 工具权限子系统（toolPermission/） | ApprovalOverlay |
| **通知系统** | notifs/ 子系统 | 无独立通知系统 |
| **斜杠命令** | 87+ 斜杠命令 | N/A |

### 语音功能对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **实现方式** | VOICE_MODE 标志控制 | WebRTC + opus-rs |
| **音频编解码** | 未公开 | Opus（自适应比特率） |
| **延迟** | 未公开 | ~10-30ms（WebRTC） |
| **NAT 穿透** | N/A | 内置 ICE/STUN/TURN |
| **构建优化** | 死代码消除（~200KB） | 始终编译（零成本抽象） |
| **特性控制** | bun:bundle feature() | 无独立标志 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| React + Ink 声明式渲染，开发效率高，生态成熟 | 依赖 Bun 运行时，非原生性能 |
| 140+ 组件 + 70+ Hooks，UI 功能极为丰富 | 组件数量庞大，代码维护成本高 |
| 完整 Vim 模式（状态机实现），Vim 用户友好 | Vim 模式增加代码复杂度 |
| 87+ 斜杠命令，操作入口丰富 | 斜杠命令数量多，学习成本高 |
| Doctor 环境诊断界面，问题排查方便 | 诊断功能与主界面耦合 |
| VOICE_MODE 特性标志 + DCE，按需加载 | 语音实现细节未公开，技术选型不明 |
| Plan 模式 markdown 内联评论，执行计划可视化 | Plan 模式依赖 IDE 集成 |
| 50+ 键绑定动作，可自定义 | 键绑定与 Vim 模式可能冲突 |
| 通知子系统独立，用户体验完整 | 通知系统增加 UI 层复杂度 |
| 工具权限 UI 子系统精细，权限决策可视化 | 权限 UI 流程复杂，可能影响操作效率 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| Rust 原生性能，终端渲染流畅 | Ratatui 命令式开发效率低于 React |
| WebRTC + Opus 实时语音，延迟 ~10-30ms | 无 Vim 模式，Vim 用户不友好 |
| tree-sitter 代码高亮，语法精确 | 组件丰富度远不及 Claude Code |
| codex exec 无头模式，CI/CD 友好 | 无斜杠命令系统，操作入口单一 |
| ApprovalOverlay 审批 UI 简洁直观 | 无 Doctor 环境诊断界面 |
| TUI 恢复选择器支持分页和过滤排序 | 无 Plan 模式，缺少执行计划可视化 |
| 零依赖原生二进制，无需额外运行时 | 无独立通知子系统 |
| crossterm 跨平台终端控制 | 无键绑定自定义系统 |
| 即时模式渲染，无虚拟 DOM 开销 | UI 组件数量有限，交互体验不如 Claude Code 丰富 |
| view_stack 视图栈管理灵活 | 缺少工具权限可视化子系统 |
# 构建系统与部署对比

## Claude Code 实现

### Bun bundle 单一构建

Claude Code 使用 **Bun** 作为运行时和构建工具，采用 `bun:bundle` 进行单一构建：

```
┌─────────────────────────────────────────────────────────────────┐
│                 Claude Code 构建系统                              │
│                                                                 │
│  构建工具：Bun bundle                                           │
│  运行时：Bun（非 Node.js）                                      │
│  包管理：Bun 内置                                               │
│                                                                 │
│  构建流程：                                                      │
│  ┌──────────────────────────────────────────────────────┐      │
│  │  TypeScript 源码 (~50 万行, 1884 个 TS 文件)          │      │
│  │       │                                               │      │
│  │       ▼                                               │      │
│  │  bun:bundle 特性标志死代码消除 (DCE)                   │      │
│  │  - feature('VOICE_MODE') == false -> 剥离语音代码      │      │
│  │  - feature('BRIDGE_MODE') == false -> 剥离 IDE 代码    │      │
│  │  - feature('DAEMON') == false -> 剥离守护进程代码      │      │
│  │  - 未激活特性在构建时完全剥离，零运行时开销            │      │
│  │       │                                               │      │
│  │       ▼                                               │      │
│  │  单一打包产物                                          │      │
│  └──────────────────────────────────────────────────────┘      │
│                                                                 │
│  特性标志列表：                                                  │
│  | 标志            | 功能                    | 预估节省 |       │
│  | PROACTIVE       | 主动 Agent 模式          | ~150KB   |       │
│  | KAIROS          | Kairos 子系统            | ~200KB   |       │
│  | BRIDGE_MODE     | IDE bridge 集成          | ~300KB   |       │
│  | DAEMON          | 后台守护进程模式          | ~100KB   |       │
│  | VOICE_MODE      | 语音输入/输出            | ~200KB   |       │
│  | AGENT_TRIGGERS  | 触发式 Agent 动作         | ~50KB    |       │
│  | MONITOR_TOOL    | 监控工具                 | ~80KB    |       │
│  | COORDINATOR_MODE| 多 Agent 协调器           | ~120KB   |       │
│  | WORKFLOW_SCRIPTS| 工作流自动化脚本          | ~100KB   |       │
└─────────────────────────────────────────────────────────────────┘
```

### 特性标志与死代码消除

```typescript
import { feature } from 'bun:bundle'

// 编译时 DCE：bun:bundle 的 feature() 在构建时剥离
if (feature('VOICE_MODE')) {
  // 此代码在构建时被完全剥离（如果标志未激活）
  // 实现语音输入/输出功能
}
```

### 分发方式

| 方式 | 命令 | 说明 |
|------|------|------|
| **npm 分发** | `npm install -g @anthropic-ai/claude-code` | 主要分发渠道 |
| **单一二进制** | Bun 打包 | 自包含可执行文件 |

### 构建限制

- **无 Nix 支持**：没有可复现构建配置
- **无 Bazel/Cross**：不支持跨平台编译
- **依赖 Bun 运行时**：目标机器需要 Bun 环境（或使用 npm 安装的打包版本）
- **无 monorepo 工作区**：单一项目结构

---

## Codex CLI 实现

### 双构建系统策略

Codex CLI 采用 **Bazel + Cargo** 双构建系统，辅以 Nix 可复现构建：

```
┌──────────────────────────────────────────────────────────────┐
│                 双构建系统策略                                 │
│                                                               │
│  开发环境:                                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ cargo build                                           │   │
│  │ cargo test                                            │   │
│  │ cargo run --bin codex -- tui                          │   │
│  │ cargo clippy                                          │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  CI/CD:                                                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ bazel build //codex-rs:cli                            │   │
│  │ bazel test //codex-rs/...                             │   │
│  │ bazel run //codex-rs:cli -- --release                 │   │
│  │ 跨平台: Linux x86_64, macOS ARM64, Windows x86_64     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  可复现构建:                                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ nix build                                             │   │
│  │ nix develop                                           │   │
│  │ 确保所有依赖版本锁定                                   │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### 构建工具矩阵

| 工具 | 用途 | 说明 |
|------|------|------|
| **Bazel 9** | 主力构建系统，CI/CD，跨平台编译 | 生产级构建，支持远程缓存和分布式构建 |
| **Cargo** | Rust 包管理和开发构建 | 日常开发首选，编译速度快 |
| **Nix** | 可复现构建（flake.nix） | 确保任何机器构建出完全相同的二进制 |
| **just** | 任务运行器 | 统一的开发任务入口 |
| **pnpm** | npm 生态包管理（monorepo） | 管理 JavaScript/TypeScript 相关依赖 |

### 项目文件结构

```
codex-rs/
├── BUILD.bazel            # Bazel 构建配置
├── MODULE.bazel           # Bazel 模块定义
├── flake.nix              # Nix 构建支持
├── justfile               # just 任务运行器配置
├── pnpm-workspace.yaml    # pnpm monorepo 工作区定义
├── Cargo.toml             # Cargo workspace 定义
└── codex-rs/              # 60+ Crate 源码
```

### Cargo Workspace 组织

```
┌──────────────────────────────────────────────────────────────┐
│                 Cargo Workspace 分层组织                      │
│                                                               │
│  层级          Crate 数量    说明                              │
│  ────          ──────────    ────                              │
│  入口层        3            cli, tui, exec                     │
│  核心层        5            core, tools, protocol, ...         │
│  沙箱层        6            sandboxing, linux-sandbox, ...     │
│  MCP 层        3            mcp-server, rmcp-client, ...       │
│  模型层        5            codex-client, chatgpt, ollama, ... │
│  状态层        3            rollout, codex-state, state-db     │
│  配置层        2            codex-config, codex-features       │
│  认证层        2            codex-login, codex-keyring-store   │
│  遥测层        2            codex-analytics, codex-otel        │
│  工具层        3            codex-tools, codex-apply-patch, .. │
│  网络层        2            codex-network-proxy, ...           │
│  其他          ~20          codex-utils-*, codex-git-utils, .. │
│                                                               │
│  总计: 60+ Crate                                             │
└──────────────────────────────────────────────────────────────┘
```

### 分发方式

| 方式 | 说明 |
|------|------|
| **零依赖原生可执行文件** | Rust 编译产物，无需运行时 |
| **npm 分发** | `npm install -g @openai/codex` |
| **cargo 分发** | `cargo install codex-cli` |
| **pnpm monorepo** | 工作区内包管理 |

### 跨平台编译

Bazel 支持以下平台的交叉编译：
- Linux x86_64
- macOS ARM64
- Windows x86_64

---

## 对比分析

### 构建复杂度对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **构建工具数量** | 1 个（Bun bundle） | 5 个（Bazel/Cargo/Nix/just/pnpm） |
| **构建配置文件** | 少量 | BUILD.bazel + MODULE.bazel + flake.nix + justfile + Cargo.toml |
| **学习曲线** | 低（Bun 简单易用） | 高（Bazel + Nix 学习成本大） |
| **日常开发** | `bun run` | `cargo build` |
| **CI/CD** | `bun bundle` | `bazel build` |
| **构建速度** | 快（Bun 原生优化） | 中（Cargo 增量编译快，Bazel 首次慢） |

### 跨平台支持对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **跨平台编译** | 不支持 | Bazel 跨平台编译 |
| **目标平台** | 依赖 Bun 运行时 | 零依赖原生二进制 |
| **平台覆盖** | macOS/Linux/Windows（需 Bun） | Linux x86_64, macOS ARM64, Windows x86_64 |
| **运行时依赖** | Bun 运行时 | 无（原生二进制） |
| **分发体积** | 中等（Bun 打包） | 小（Rust 静态链接） |

### 可复现性对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **可复现构建** | 无 | Nix（flake.nix） |
| **依赖锁定** | Bun lockfile | Cargo.lock + Nix flake lock |
| **环境一致性** | 依赖 Bun 版本 | Nix 确保完全一致 |
| **远程缓存** | 无 | Bazel 远程缓存 |
| **分布式构建** | 无 | Bazel 分布式构建 |

### 特性标志对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **特性标志系统** | GrowthBook + bun:bundle DCE | `feature.state` OTel 指标 |
| **死代码消除** | 构建时剥离（零运行时开销） | 无构建时 DCE |
| **远程控制** | GrowthBook 远程特性开关 | 无远程控制 |
| **A/B 测试** | 支持 | 不支持 |
| **标志数量** | 9+ 个特性标志 | 未公开 |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| Bun bundle 构建简单，学习成本低 | 无 Nix/可复现构建支持 |
| bun:bundle DCE 构建时剥离未激活代码，零运行时开销 | 无跨平台编译能力 |
| GrowthBook 远程特性开关 + A/B 测试，产品迭代灵活 | 依赖 Bun 运行时，非原生二进制 |
| npm 分发便捷，安装简单 | 单一构建系统，CI/CD 灵活性不足 |
| 9+ 特性标志精细控制功能发布 | 无 Bazel 远程缓存和分布式构建 |
| TypeScript + Bun 开发效率高 | ~50 万行代码，构建产物体积较大 |
| 原生 JSX/TSX 支持无需转译 | 无 monorepo 工作区管理 |
| 构建速度快（Bun 原生优化） | 无法确保跨机器构建一致性 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| Bazel 9 支持跨平台编译和远程缓存 | 5 个构建工具，学习曲线陡峭 |
| Nix flake.nix 提供完全可复现构建 | 构建配置复杂（BUILD.bazel + MODULE.bazel + flake.nix + ...） |
| 零依赖原生可执行文件，分发简单 | Bazel 首次构建慢，配置复杂 |
| Cargo + pnpm monorepo 双生态支持 | Nix 学习成本高，社区相对小 |
| 60+ Crate 微服务化，编译隔离 | 无构建时死代码消除 |
| pnpm monorepo 工作区管理规范 | 无远程特性开关和 A/B 测试 |
| just 任务运行器统一开发入口 | Rust 编译时间较长（相比 Bun） |
| 双构建系统（Bazel + Cargo）灵活切换 | Bazel 和 Nix 的组合增加了 CI/CD 复杂度 |
| ~8 万行 Rust 实现 ~50 万行 TS 等效功能 | pnpm 仅管理 JS/TS 依赖，Rust 依赖由 Cargo 管理 |
| Bazel 分布式构建加速大型项目 | 构建系统维护成本高（需同时维护 Bazel 和 Cargo 配置） |
# 计费与生态系统对比

## Claude Code 实现

### API 按 Token 计费

Claude Code 通过 Anthropic API 按使用量计费，支持多种 API 后端：

| 模型 | 输入价格 | 输出价格 | 上下文窗口 |
|------|----------|----------|------------|
| **Claude Sonnet 4** | $3 / 1M tokens | $15 / 1M tokens | 200K |
| **Claude Opus 4** | $15 / 1M tokens | $75 / 1M tokens | 200K |
| **Claude Haiku 3.5** | $0.80 / 1M tokens | $4 / 1M tokens | 200K |

### API 后端支持

Claude Code 支持 4 个 API 后端，计费方式各不相同：

| 后端 | 计费方式 | 说明 |
|------|----------|------|
| **Anthropic API** | 按 token 计费 | 直接使用 Anthropic API |
| **AWS Bedrock** | 独立计费 | 通过 AWS 账户计费，支持企业协议价 |
| **GCP Vertex** | 独立计费 | 通过 GCP 账户计费，支持企业协议价 |
| **Anthropic Foundry** | 独立计费 | 定制模型微调 |

### 企业订阅

| 计划 | 价格 | 说明 |
|------|------|------|
| **Claude Max** | $100/月/seat | 个人高级订阅 |
| **Claude Team** | $200/月/seat | 团队协作订阅 |

### 成本追踪

Claude Code 内置成本追踪功能：

- **`cost.usage` 指标**：实时追踪当前会话成本（美元）
- **`token.usage` 指标**：实时追踪 token 使用量
- **`maxCostPerSession` MDM 策略**：企业管理员可设置单会话最大成本

### 闭源模式

- **许可证**：闭源
- **自托管**：不支持
- **代码透明度**：无（仅通过逆向分析了解架构）
- **社区贡献**：不接受外部贡献

---

## Codex CLI 实现

### ChatGPT 订阅

| 计划 | 价格 | 说明 |
|------|------|------|
| **ChatGPT Plus** | $20/月 | 含 Codex CLI 使用（有使用限额） |
| **ChatGPT Pro** | $200/月 | 更高使用限额 |

### API 按 Token 计费

Codex CLI 通过 OpenAI API 按使用量计费：

| 模型 | 输入价格 | 输出价格 | 上下文窗口 |
|------|----------|----------|------------|
| **GPT-4.1** | $2 / 1M tokens | $8 / 1M tokens | 1M |
| **GPT-4.1 mini** | $0.40 / 1M tokens | $1.60 / 1M tokens | 1M |
| **GPT-4.1 nano** | $0.10 / 1M tokens | $0.40 / 1M tokens | 1M |
| **o3** | $10 / 1M tokens | $40 / 1M tokens | 200K |
| **o4-mini** | $1.10 / 1M tokens | $4.40 / 1M tokens | 200K |

### 本地模型（免费）

Codex CLI 原生支持本地模型运行，**完全免费**：

| 提供商 | 说明 |
|--------|------|
| **Ollama** | 开源本地模型运行时，支持 Llama、Mistral 等 |
| **LM Studio** | 桌面应用，支持下载和运行开源模型 |

本地模型的优势：
- **零 API 成本**：所有推理在本地完成
- **数据隐私**：代码不离开本地机器
- **无网络依赖**：离线可用
- **无限使用**：不受 API 限额限制

### 开源模式

- **许可证**：Apache 2.0 开源
- **自托管**：完全支持
- **代码透明度**：完全透明（~8 万行 Rust 源码公开）
- **社区贡献**：421+ 贡献者，75,000+ GitHub Stars

---

## 对比分析

### 计费模式对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **免费使用** | 无（需 API Key 或订阅） | 有（本地模型免费） |
| **订阅入口** | Claude Max $100/月 | ChatGPT Plus $20/月 |
| **订阅高端** | Claude Team $200/月 | ChatGPT Pro $200/月 |
| **最低 API 价格** | Haiku: $0.80/$4 per 1M | GPT-4.1 nano: $0.10/$0.40 per 1M |
| **最高 API 价格** | Opus 4: $15/$75 per 1M | o3: $10/$40 per 1M |
| **主流模型价格** | Sonnet 4: $3/$15 per 1M | GPT-4.1: $2/$8 per 1M |
| **上下文窗口** | 200K | 最高 1M（GPT-4.1 系列） |
| **企业 API** | Bedrock/Vertex 独立计费 | 无独立企业 API |
| **成本追踪** | 内置（cost.usage 指标） | 无内置成本追踪 |
| **会话成本限制** | MDM maxCostPerSession | 无 |

### 生态系统对比

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **许可证** | 闭源 | Apache 2.0 开源 |
| **GitHub Stars** | 未公开 | 75,000+ |
| **贡献者** | 未公开 | 421+ |
| **MCP 生态** | 自研 MCP 实现（4 种传输） | rmcp 0.12 标准 |
| **插件系统** | 有（plugins/） | 有（plugin/* API） |
| **IDE 集成** | VS Code/JetBrains/Neovim/Emacs | VS Code（社区扩展） |
| **本地模型** | 不支持 | Ollama + LM Studio |
| **模型提供商** | Anthropic（4 后端） | OpenAI + Ollama + LM Studio |
| **社区生态** | Anthropic 官方主导 | 开源社区驱动 |
| **自托管** | 不支持 | 完全支持 |
| **企业部署** | MDM 策略 + Bedrock/Vertex | 开源 + 自定义部署 |

### API 成本效率对比

以处理 100K input tokens + 50K output tokens 为例：

| 模型 | 平台 | 输入成本 | 输出成本 | 总成本 |
|------|------|----------|----------|--------|
| Claude Sonnet 4 | Anthropic | $0.30 | $0.75 | **$1.05** |
| Claude Opus 4 | Anthropic | $1.50 | $3.75 | **$5.25** |
| Claude Haiku 3.5 | Anthropic | $0.08 | $0.20 | **$0.28** |
| GPT-4.1 | OpenAI | $0.20 | $0.40 | **$0.60** |
| GPT-4.1 mini | OpenAI | $0.04 | $0.08 | **$0.12** |
| GPT-4.1 nano | OpenAI | $0.01 | $0.02 | **$0.03** |
| o3 | OpenAI | $1.00 | $2.00 | **$3.00** |
| o4-mini | OpenAI | $0.11 | $0.22 | **$0.33** |
| 本地模型 | Ollama/LM Studio | $0.00 | $0.00 | **$0.00** |

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| Claude Haiku 3.5 性价比高（$0.80/$4 per 1M） | 无免费使用选项，必须付费 |
| Bedrock/Vertex 企业 API 支持企业协议价 | Claude Opus 4 价格昂贵（$15/$75 per 1M） |
| 内置成本追踪（cost.usage + token.usage 指标） | 无本地模型支持，无法离线使用 |
| MDM maxCostPerSession 企业成本控制 | 闭源，无法审计代码安全性 |
| 4 个 API 后端灵活切换 | 无法自托管，数据必须经过 Anthropic |
| Claude Max/Team 订阅适合个人和团队 | 订阅价格较高（$100-200/月/seat） |
| 上下文窗口 200K，满足大多数场景 | 上下文窗口不及 GPT-4.1 的 1M |
| Anthropic 官方维护，质量有保障 | 社区无法贡献代码和功能 |
| MCP 生态成熟，4 种传输协议 | 插件生态相对封闭 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| 本地模型完全免费（Ollama/LM Studio） | o3 推理模型价格较高（$10/$40 per 1M） |
| ChatGPT Plus $20/月，入门门槛低 | ChatGPT Plus 使用限额可能不够 |
| GPT-4.1 nano 极低价格（$0.10/$0.40 per 1M） | 无企业级 API 后端（如 Bedrock/Vertex） |
| Apache 2.0 开源，代码完全透明 | 无内置成本追踪功能 |
| 可自托管，数据不出本地 | 无 MDM 策略，企业成本控制弱 |
| 75,000+ Stars，421+ 贡献者，社区活跃 | IDE 集成覆盖面较窄 |
| GPT-4.1 系列 1M 上下文窗口 | 本地模型需要 GPU 资源，硬件要求高 |
| 支持多模型提供商（OpenAI/Ollama/LM Studio） | 开源项目的长期维护依赖社区 |
| 零 API 成本运行本地模型 | 本地模型质量不及云端模型 |
| 完全可定制和可扩展 | 无官方企业支持（仅社区支持） |
# 设计模式与结论

## Claude Code 核心设计模式

| 设计模式 | 实现位置 | 说明 |
|----------|----------|------|
| **流式异步生成器** | `query.ts` | Agent 循环核心，yield 事件而非批量返回，实现实时流式交互。这是 Claude Code 区别于传统请求-响应模式的关键设计——生成器模型天然支持背压、取消和流式消费 |
| **工具使用循环** | QueryEngine | 模型提议工具调用 -> 执行 -> 反馈结果，经典 ReAct 模式的流式变体。通过 `StreamingToolExecutor` 在模型生成时就开始执行已完成的工具调用，显著减少端到端延迟 |
| **分层上下文** | context.ts | 系统提示 + CLAUDE.md + 对话 + 压缩，多层上下文叠加。通过 `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__` 标记分离静态和动态内容，最大化 API 缓存命中率 |
| **权限门控** | utils/permissions/ | 每个工具调用经过多层权限管道（模式 -> 规则 -> Hook -> LLM 分类器 -> 用户确认），灵活但复杂。7 层安全机制提供纵深防御 |
| **特性门控** | GrowthBook + bun:bundle | 运行时特性标志 + 构建时死代码消除，支持渐进式功能发布。未激活的特性在构建时被完全剥离，零运行时开销 |
| **延迟加载** | ToolSearchTool | 100+ MCP 工具按需加载 schema，大幅节省 token。系统提示仅包含工具名称列表，模型需要时再查询完整 schema |
| **多 Agent 隔离** | AgentTool | 克隆文件缓存、独立 AbortController、过滤工具池。Fork 机制共享前缀最大化 API 缓存命中 |
| **Fork 缓存优化** | AgentTool | 共享前缀最大化 API 提示缓存命中。多 Agent 场景下缓存命中率通常 >80% |
| **Hook 可扩展性** | hooks/ | 5 种 Hook 类型（Command/Prompt/Agent/HTTP/Function）覆盖 13 个生命周期事件。是 Claude Code 可扩展性的骨干 |
| **声明式终端 UI** | React/Ink | 140+ 组件、70+ Hooks、完整 React 生态模式。声明式渲染模型简化了复杂的终端 UI 开发 |
| **梦境任务** | autoDream/ | 后台记忆整合，会话学习持久化。在用户空闲时自动回顾会话并更新记忆文件，实现跨会话学习 |
| **LRU 文件缓存** | QueryEngine | 100 文件 / 25MB 上限，去重读取和变更检测。跨轮次跟踪文件内容，避免冗余读取 |

**深入分析：**

Claude Code 的设计模式体现了"**功能优先**"的哲学——通过丰富的设计模式实现尽可能多的功能，即使这增加了系统复杂性。流式异步生成器是最核心的模式，它不仅驱动了 Agent 循环，还影响了整个系统的架构——UI 层通过消费生成器事件来更新界面，遥测系统通过监听事件来收集指标，日志系统通过拦截事件来记录操作。

特性门控模式（GrowthBook + bun:bundle）是 Claude Code 作为闭源产品的独特优势——可以在不发布新版本的情况下远程控制功能的可用性，这对于快速迭代和 A/B 测试至关重要。

---

## Codex CLI 核心设计模式

| 设计模式 | 实现位置 | 说明 |
|----------|----------|------|
| **Crate 微服务化** | codex-rs/ | 60+ Crate 高粒度模块化，每个 Crate 职责单一。Cargo workspace 提供天然的编译隔离和依赖管理 |
| **事件驱动架构** | codex.rs | 事件循环通过 Submission Queue / Event Queue 将模型响应、工具执行、用户输入解耦。Rust 的所有权模型确保通道传递的安全性 |
| **委托模式** | codex_delegate.rs | 将具体操作委托给专门的子系统处理（沙箱、Guardian、上下文管理器），保持核心循环的简洁性 |
| **策略模式** | sandboxing/ | 沙箱策略、执行策略、审批策略均可配置和替换。平台抽象层屏蔽 macOS/Linux/Windows 差异 |
| **Guardian 守护者** | guardian/ | 安全中间层，决定哪些操作可以自动执行。与沙箱配合提供双层安全保障 |
| **平台抽象层** | sandboxing/ | 屏蔽 macOS/Linux/Windows 差异，上层代码无需关心平台细节。每个平台有独立的 Crate 实现 |
| **无状态请求** | client.rs | 每个请求携带完整历史，支持零数据保留配置。简化服务端逻辑，提高可恢复性 |
| **提示词前缀缓存** | client.rs | 后续请求是前一次的精确前缀，最大化缓存命中率。无状态设计天然适合前缀缓存 |
| **双构建系统** | BUILD.bazel + Cargo.toml | Bazel 用于生产 CI/CD 和跨平台编译，Cargo 用于日常开发。Nix 提供可复现构建环境 |
| **渐进式迁移** | tools/ crate | 从 core 渐进式提取共享代码到独立 Crate，避免大规模重构风险。体现了 Rust 生态中常见的演进式架构方法 |

**深入分析：**

Codex CLI 的设计模式体现了"**安全优先**"的哲学——每个设计决策都以安全为第一考量。Crate 微服务化不仅是模块化的需要，更是安全隔离的需要——沙箱相关的 Crate（`linux-sandbox`、`windows-sandbox-rs`、`process-hardening`）与核心逻辑完全隔离，减少了安全漏洞的 blast radius。

无状态请求模式是 Codex CLI 最独特的设计选择之一。在大多数 Agent 系统都采用有状态设计的背景下，Codex CLI 的无状态方案看似"倒退"，但实际上带来了显著的优势：服务端无需维护会话状态、进程崩溃可恢复、支持零数据保留配置。提示词前缀缓存弥补了无状态设计的性能劣势。

---

## 可借鉴的架构思想

### 1. 分层状态管理（Claude Code）

将进程级（全局单例、基础设施）、会话级（UI、Hooks）、轮次级（Query、Services、Tools）状态分离管理，避免生命周期混乱。Claude Code 的六层架构（UI -> Hooks -> State -> Query -> Services -> Tools）是一个优秀的分层设计参考。

### 2. OS 级沙箱（Codex CLI）

安全由操作系统内核保证而非软件层，更可靠且不可绕过。这是安全设计的黄金原则——将安全边界放在尽可能底层的可信组件中。

### 3. 延迟加载工具 Schema（Claude Code）

在工具数量庞大时有效节省 token，避免系统提示膨胀。ToolSearchTool 的"先发现后使用"模式是一个通用的 LLM 优化策略。

### 4. Crate 微服务化（Codex CLI）

高粒度模块化带来更好的编译隔离和职责清晰。60+ Crate 的组织方式展示了如何在 Rust 中实现"微服务化"的单体应用。

### 5. 渐进式压缩（Claude Code）

四层压缩策略优雅地处理上下文窗口限制。从微压缩到自动压缩到会话记忆压缩到反应式压缩，每一层处理不同严重程度的上下文压力。

### 6. 无状态请求 + 前缀缓存（Codex CLI）

简化服务端逻辑同时保持性能。无状态设计带来的可恢复性和可扩展性优势，可以通过前缀缓存来弥补性能劣势。

### 7. Fork 缓存优化（Claude Code）

多 Agent 场景下最大化 API 缓存利用率。Fork 机制确保子 Agent 共享相同的前缀，仅计算不同的后缀。

### 8. 平台抽象沙箱（Codex CLI）

统一接口屏蔽平台差异，上层代码无需关心底层实现。`sandboxing` Crate 定义统一的沙箱 trait，各平台 Crate 提供具体实现。

---

## 架构演进趋势

### 从 TypeScript 到 Rust

Codex CLI 从 TypeScript 迁移到 Rust 的决策反映了 AI 编码助手领域的一个重要趋势：**对性能和安全的要求正在推动语言层面的升级**。

- **性能**：Rust 的零成本抽象和原生性能对于频繁执行沙箱操作、文件 I/O 和网络请求的 AI 编码助手至关重要
- **安全性**：Rust 的内存安全保证在系统级编程中提供了编译时安全保证
- **并发**：Rust 的 `Send + Sync` trait 和无数据竞争的并发模型天然适合多 Agent 系统
- **代码规模**：~8 万行 Rust 实现了 ~50 万行 TypeScript 的等效功能

### 从权限治理到 OS 沙箱

安全模型的演进方向正在从**软件层权限治理**向**OS 级沙箱隔离**转变。

- **Claude Code 潜在演进**：引入轻量级沙箱（如 macOS Seatbelt）、将 MCP 工具纳入沙箱保护
- **Codex CLI 潜在演进**：细化沙箱策略粒度、支持自定义沙箱配置文件

### 从有状态到无状态

Agent 循环的状态管理趋势正在从**有状态设计**向**无状态设计**演进。未来的 AI 编码助手可能会采用**混合状态管理**策略——核心 Agent 循环保持无状态，但在本地维护缓存层以优化性能。

### MCP 协议标准化

MCP（Model Context Protocol）正在成为 AI 工具生态的**事实标准协议**。

- **工具接口统一**：MCP 提供了标准化的工具描述和调用协议
- **传输协议收敛**：从多种传输协议向少数标准协议收敛
- **安全标准化**：MCP 工具的安全保护正在成为共识
- **未来展望**：工具市场可能出现，MCP 可能成为 AI Agent 互操作的通用协议

---

## 综合对比

### 核心发现总结

**1. 安全哲学的分野是两者最根本的架构区别。** Claude Code 采用"信任但验证"的软件层权限治理，Codex CLI 采用"零信任"的 OS 级沙箱隔离。两者代表了 AI 编码助手安全设计的两个极端。

**2. Agent Loop 的设计反映了技术栈的深层影响。** Claude Code 的流式异步生成器（TypeScript AsyncGenerator）和 Codex CLI 的提交-事件模式（Rust Channel）都是各自技术栈下的最优选择。

**3. 工具系统的设计选择体现了不同的工程权衡。** Claude Code 的 search-and-replace 编辑方式降低了模型认知负担但增加了 token 消耗；Codex CLI 的 unified diff patch 方式更适合代码审查但增加了模型出错概率。

**4. 上下文管理是 AI 编码助手的核心竞争力。** Claude Code 的四层渐进式压缩是目前业界最完善的上下文管理方案，Dream Task 的跨会话学习机制更是独树一帜。

**5. 语言选择对架构有深远影响。** TypeScript 的 ~50 万行 vs Rust 的 ~8 万行实现了等效功能，这不仅是代码量的差异，更是架构复杂度、维护成本和安全保证的差异。

### 综合对比表

| 维度 | Claude Code | Codex CLI |
|------|-------------|-----------|
| **安全哲学** | 权限治理（信任权限系统） | OS 级沙箱（不信任 AI） |
| **Agent Loop** | 流式异步生成器，双层循环 | 提交-事件模式，无状态循环 |
| **工具并发** | 只读并行（最多 10 个），写串行 | 全部串行（沙箱约束） |
| **文件编辑** | 搜索并替换（search-and-replace） | 统一差异补丁（unified diff） |
| **上下文压缩** | 四层渐进式压缩 + LRU 缓存 | 本地 + 远程压缩 + Token 截断 |
| **持久记忆** | MEMORY.md + Dream Task 后台整合 | AGENTS.md + rollout 持久化 |
| **多 Agent** | 三级架构（子 Agent / 协调器 / 团队） | 完整多 Agent + 守护者模式 + 批量 CSV |
| **代码规模** | ~50 万行 TypeScript | ~8 万行 Rust |
| **开源** | 闭源 | Apache 2.0 |
| **提示词缓存** | 动态边界分割 + Fork 前缀共享 | 精确前缀匹配 + Zstd 压缩 |
| **配置系统** | JSON 5 级优先级 | TOML 5 级优先级 |
| **IDE 集成** | Bridge 协议（33 文件）+ JWT | app-server JSON-RPC 2.0 |
| **遥测** | OTel + Statsig + GrowthBook + Sentry | 内置 OTel + AnalyticsEventsClient |
| **错误处理** | 6 类错误分类 | 15+ 枚举变体 |
| **可复现构建** | 无 | Nix 支持 |
| **模型支持** | Claude 系列（4 后端） | OpenAI + Ollama + LM Studio |

### 适用场景建议

#### Claude Code 更适合

- **个人开发者和小团队**：丰富的工具集和灵活的权限系统适合快速迭代开发
- **需要深度 IDE 集成的场景**：Bridge 协议提供了业界最完善的 IDE 集成（光标同步、调试信息、LSP）
- **需要高级多 Agent 协作的场景**：三级多 Agent 架构（子 Agent / 协调器 / 团队）支持复杂的工作流
- **需要跨会话学习的场景**：Dream Task 后台记忆整合 + MEMORY.md 持久记忆
- **使用 Claude 模型的用户**：对 Claude 系列模型有深度优化，支持 4 个 API 后端
- **需要丰富扩展性的场景**：5 种 Hook 类型、技能系统、插件系统提供了强大的可扩展性

#### Codex CLI 更适合

- **安全敏感的企业环境**：OS 级沙箱提供了不可绕过的安全保证
- **需要多模型支持的场景**：支持 OpenAI、Ollama、LM Studio 等多个模型提供商
- **需要批量自动化处理的场景**：`spawn_agents_on_csv` 支持 CSV 驱动的批量 Agent 任务
- **开源偏好者**：Apache 2.0 许可证，完全透明的代码和决策过程
- **需要可复现构建的场景**：Nix + Bazel 提供了完全可复现的构建环境
- **需要本地模型运行的场景**：原生支持 Ollama 和 LM Studio 本地模型

---

## 优缺点总结

### Claude Code

| 优点 | 缺点 |
|------|------|
| 功能极为丰富（40+ 工具、87+ 命令、13 Hook 事件、5 Hook 类型） | 系统复杂度极高（~50 万行 TS），维护成本大 |
| 四层渐进式压缩 + Dream Task 跨会话学习，上下文管理业界领先 | 闭源，无法审计代码安全性和正确性 |
| Bridge 协议 + 内置 LSP 客户端，IDE 集成最完善 | 无 OS 级沙箱，安全性依赖软件层规则 |
| React/Ink 声明式 UI（140+ 组件、70+ Hooks），交互体验优秀 | 依赖 Bun 运行时，非原生二进制性能 |
| GrowthBook + bun:bundle 特性门控，支持远程 A/B 测试 | 无可复现构建（Nix），无跨平台编译 |
| 6 级认证解析链 + MDM 策略，企业部署友好 | Windows/Linux 密钥存储安全性不足 |
| 三层遥测架构（Statsig + Sentry + OTel），可观测性全面 | 遥测数据发送到第三方，隐私顾虑 |
| Claude Haiku 性价比高，Bedrock/Vertex 企业 API 灵活 | 无免费使用选项，无本地模型支持 |
| StreamingToolExecutor 边生成边执行，端到端延迟低 | 有状态设计，进程崩溃不可恢复 |
| Fork 缓存优化，多 Agent 缓存命中率 >80% | 权限管道复杂（7 层），配置和理解成本高 |

### Codex CLI

| 优点 | 缺点 |
|------|------|
| OS 级沙箱（Landlock/Seatbelt/Restricted Token），安全性不可绕过 | 功能相对精简（25+ 工具），扩展性不如 Claude Code |
| Apache 2.0 开源，代码完全透明，421+ 贡献者 | 无独立 Hook 系统，可扩展性受限 |
| 零依赖原生二进制，性能优秀（Rust 零成本抽象） | Ratatui 命令式 UI 开发效率低于 React/Ink |
| Nix + Bazel 可复现构建 + 跨平台编译 | 构建系统复杂（5 个工具），学习曲线陡峭 |
| 本地模型免费（Ollama/LM Studio），零 API 成本 | 本地模型需要 GPU 资源，质量不及云端 |
| 无状态请求设计，进程崩溃可恢复 | 无内置 LSP 客户端，代码智能依赖 IDE |
| keyring-store 4 平台密钥管理，安全性高 | 遥测架构简单，缺少 A/B 测试和错误追踪 |
| ChatGPT Plus $20/月入门门槛低 | o3 推理模型价格较高 |
| GPT-4.1 系列 1M 上下文窗口 | 无 Dream Task 跨会话学习机制 |
| 60+ Crate 微服务化，编译隔离，职责清晰 | 无 MDM 策略，企业部署能力弱 |
| 15+ 错误枚举变体，Rust 类型系统确保显式处理 | 无 Vim 模式，无斜杠命令系统 |
| WebRTC + Opus 实时语音，延迟 ~10-30ms | IDE 集成仅 VS Code 社区扩展 |
| 批量 CSV Agent 任务，自动化处理能力强 | 无 Plan 模式，无 Doctor 诊断界面 |

---

# 第 7 章 综合优缺点对比与替代方案分析

## 7.1 各维度优缺点汇总

### 7.1.1 编程语言选择（TypeScript/Bun vs Rust）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 开发效率高、前端生态丰富、React/Ink 成熟、团队技能匹配、快速迭代能力强 | 原生性能、零 GC 开销、内存安全编译时保证、零依赖二进制分发、系统级编程能力强 |
| **缺点** | 运行时性能受 V8 限制、依赖 Bun 运行时、非原生二进制、系统编程能力弱（需 FFI） | 编译速度慢、学习曲线陡峭、UI 框架（Ratatui）成熟度不及 React、开发迭代速度慢 |
| **更优方** | 各有千秋 | Claude Code 在开发效率和 UI 生态上更优，Codex CLI 在性能和安全上更优 |

### 7.1.2 架构设计（六层分层 vs 微服务化 Crate）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 六层分层清晰（API/Agent/Tool/UI/Config/Hook）、职责明确、模块间耦合度低 | 60+ Crate 微服务化、编译隔离、职责极度清晰、Rust 类型系统强制接口规范 |
| **缺点** | ~50 万行单仓库、模块间可能存在隐式依赖、维护成本高 | Crate 数量过多可能增加编译时间和认知负担、依赖管理复杂 |
| **更优方** | 各有千秋 | Claude Code 分层适合中型项目，Codex CLI 微服务化适合大型长期项目 |

### 7.1.3 UI 实现（React/Ink vs Ratatui）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 140+ 组件、70+ Hooks 声明式 UI、组件化复用、生态成熟、动画和交互丰富 | 命令式精确控制、渲染性能高、原生终端体验、内存占用低 |
| **缺点** | React 抽象层增加复杂度、Ink 框架更新可能带来兼容性问题 | 开发效率低、组件复用困难、缺乏声明式状态管理、动画支持有限 |
| **更优方** | Claude Code | React/Ink 在终端 UI 开发效率和可维护性上显著领先 |

### 7.1.4 Agent 循环（流式异步生成器 vs Op/Event）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 流式异步生成器天然适配 LLM 流式 API、边生成边执行（StreamingToolExecutor）、端到端延迟低 | Op/Event 提交-事件模式适合多线程 Rust、类型安全、状态转换可预测、易于测试 |
| **缺点** | 异步生成器调试复杂、错误传播链长 | 模式较为僵化、动态工具链支持不如流式方案灵活 |
| **更优方** | 各有千秋 | Claude Code 的流式方案更适合 LLM 交互，Codex CLI 的 Op/Event 更适合 Rust 并发 |

### 7.1.5 工具系统（search-replace vs unified diff）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | search-replace 语义清晰、对人类可读、不易出错、支持模糊匹配 | unified diff 标准 Unix 工具、可复用 git diff 知识、支持多文件批量修改 |
| **缺点** | 大范围修改时 search-replace 可能匹配不准确 | diff 格式对行号敏感、文件变更后 diff 失效、可读性差 |
| **更优方** | Claude Code | search-replace 在 AI 编辑场景下更健壮、更易审查 |

### 7.1.6 文件编辑方式
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | search-and-replace 精确匹配、支持部分匹配、容错性好 | unified diff patch 标准化、支持多文件原子操作、与 git 工作流一致 |
| **缺点** | 大规模重构效率不如 diff、多文件编辑需多次调用 | diff 依赖精确行号、文件变化后易失效、冲突处理复杂 |
| **更优方** | 各有千秋 | Claude Code 更适合精确局部编辑，Codex CLI 更适合大规模重构 |

### 7.1.7 上下文管理（四层压缩 vs 自动+截断）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 四层渐进式压缩（原始→摘要→关键→最小）、Dream Task 跨会话学习、压缩策略精细 | 自动压缩 + 智能截断、实现简洁、1M 上下文窗口缓冲大 |
| **缺点** | 压缩策略复杂、实现成本高、有状态设计导致崩溃不可恢复 | 压缩策略相对简单、无跨会话学习、信息损失可能更大 |
| **更优方** | Claude Code | 四层压缩 + Dream Task 在上下文管理上业界领先 |

### 7.1.8 安全模型（权限管道 vs OS 沙箱）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 7 层权限管道细粒度控制、支持 allowlist/denylist/自定义规则、Hook 可拦截操作 | OS 级沙箱（Landlock/Seatbelt/Restricted Token）不可绕过、安全性由内核保证 |
| **缺点** | 软件层规则可被绕过、配置复杂（7 层）、理解成本高 | 沙箱粒度较粗、平台差异大（Linux/Mac/Windows 实现不同）、灵活性低 |
| **更优方** | Codex CLI | OS 级沙箱的安全性从根本上优于软件层规则，不可绕过是关键优势 |

### 7.1.9 多 Agent 系统
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | OS 隔离的多 Agent（子进程）、Fork 缓存优化（命中率 >80%）、Agent 间通信完善 | 无状态请求设计、支持批量 CSV Agent 任务、自动化处理能力强 |
| **缺点** | 多 Agent 资源消耗大、进程管理复杂 | 无独立多 Agent 协作框架、Agent 间通信能力弱 |
| **更优方** | Claude Code | 多 Agent 架构更成熟、缓存优化更完善 |

### 7.1.10 配置系统（JSON vs TOML）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | JSON 通用性强、工具链支持好、前后端通用格式 | TOML 可读性好、支持注释、适合配置文件、Rust 生态标准 |
| **缺点** | JSON 不支持注释、冗余（大括号/引号多）、手写易出错 | TOML 通用性不如 JSON、部分语言原生支持差 |
| **更优方** | 各有千秋 | JSON 在通用性上更优，TOML 在配置可读性上更优 |

### 7.1.11 状态管理（React Context vs Rust Mutex）
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | React Context + Hooks 声明式状态管理、组件自动重渲染、状态变更可追踪 | Rust Mutex 编译时保证线程安全、无数据竞争、性能可控 |
| **缺点** | React Context 可能导致不必要的重渲染、状态管理分散在多个 Context 中 | Mutex 可能死锁、状态管理代码冗长、缺乏响应式更新 |
| **更优方** | 各有千秋 | React Context 在 UI 场景更自然，Rust Mutex 在并发场景更安全 |

### 7.1.12 错误处理
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | try/catch 统一处理、错误边界组件、用户友好的错误展示、Sentry 集成 | 15+ 错误枚举变体、Rust 类型系统强制显式处理、编译时穷举检查 |
| **缺点** | TypeScript 异常处理不够严格、可能遗漏错误分支 | 错误处理代码冗长、unwrap/expect 可能 panic |
| **更优方** | Codex CLI | Rust 的类型系统在错误处理上从根本上更安全 |

### 7.1.13 会话恢复
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | Dream Task 跨会话学习、会话历史持久化、上下文继承 | 无状态请求设计、进程崩溃可恢复、请求级别隔离 |
| **缺点** | 有状态设计导致进程崩溃不可恢复、会话状态复杂 | 无跨会话学习、每次启动从零开始 |
| **更优方** | 各有千秋 | Claude Code 在会话连续性上更优，Codex CLI 在崩溃恢复上更优 |

### 7.1.14 Hook 系统
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 13 个 Hook 事件、5 种 Hook 类型、支持 Pre/Post/Notification/Stop/SubAgent、可自定义规则 | 无独立 Hook 系统、通过事件机制实现部分功能 |
| **缺点** | Hook 配置复杂、调试困难、可能影响性能 | 缺乏灵活的扩展机制、无法拦截和修改操作 |
| **更优方** | Claude Code | Hook 系统在可扩展性和自定义能力上远超 Codex CLI |

### 7.1.15 MCP 集成
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 内置 MCP 客户端、支持 stdio/SSE/WebSocket/local 传输、工具自动发现、与 40+ 工具无缝集成 | 支持 MCP（stdio/SSE 传输） |
| **缺点** | MCP 依赖外部服务器、增加复杂度 | 缺乏标准化工具扩展协议 |
| **更优方** | Claude Code | MCP 集成在工具扩展性上具有显著优势 |

### 7.1.16 认证系统
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 6 级认证解析链（API Key/OAuth/SSO/MDM/环境变量/配置文件）、企业 SSO 支持、MDM 策略 | keyring-store 4 平台密钥管理、安全性高、OAuth 2.0 PKCE 流程 |
| **缺点** | Windows/Linux 密钥存储安全性不足、认证链复杂 | 仅支持 OpenAI API 认证、无企业 SSO |
| **更优方** | Claude Code | 企业级认证和多 API 后端支持更完善 |

### 7.1.17 遥测
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 三层遥测架构（Statsig + Sentry + OTel）、A/B 测试、错误追踪全面、GrowthBook 特性门控 | 遥测简洁、隐私友好、无第三方数据共享 |
| **缺点** | 遥测数据发送到第三方（Statsig/Sentry）、隐私顾虑 | 遥测架构简单、缺少 A/B 测试和错误追踪能力 |
| **更优方** | 各有千秋 | Claude Code 在可观测性上更优，Codex CLI 在隐私保护上更优 |

### 7.1.18 IDE 集成
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | Bridge 协议、内置 LSP 客户端、VS Code/JetBrains 深度集成、实时文件同步 | VS Code 社区扩展、基本编辑器集成 |
| **缺点** | 集成复杂度高、依赖 Bridge 进程 | IDE 集成仅 VS Code、功能有限、无 LSP 客户端 |
| **更优方** | Claude Code | IDE 集成深度和广度显著领先 |

### 7.1.19 LSP 支持
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 内置 LSP 客户端、支持多语言代码智能（诊断/补全/悬停/定义）、与工具系统联动 | 无内置 LSP 客户端 |
| **缺点** | LSP 客户端增加复杂度和资源消耗 | 代码智能完全依赖 IDE |
| **更优方** | Claude Code | 内置 LSP 在独立终端使用场景下具有不可替代的优势 |

### 7.1.20 构建系统
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | Bun 内置打包（bun:bundle）、构建简单、开发体验好 | Nix + Bazel 可复现构建、跨平台编译、5 个工具链覆盖全平台 |
| **缺点** | 无可复现构建（Nix）、无跨平台编译、依赖 Bun 运行时 | 构建系统复杂（5 个工具）、学习曲线陡峭、编译时间长 |
| **更优方** | 各有千秋 | Claude Code 在开发体验上更优，Codex CLI 在可复现性上更优 |

### 7.1.21 计费模式
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | Claude Haiku 性价比高、Bedrock/Vertex 企业 API 灵活、按量计费 | ChatGPT Plus $20/月入门门槛低、本地模型零 API 成本 |
| **缺点** | 无免费使用选项、无本地模型支持、API 调用费用可能较高 | o3 推理模型价格较高、本地模型需要 GPU 资源 |
| **更优方** | 各有千秋 | Claude Code 在企业灵活性上更优，Codex CLI 在低成本入门上更优 |

### 7.1.22 开源策略
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 闭源但功能丰富、商业支持稳定、更新节奏快 | Apache 2.0 开源、代码完全透明、421+ 贡献者社区活跃 |
| **缺点** | 闭源无法审计、依赖 Anthropic 持续维护、用户无法自定义核心逻辑 | 开源可能导致碎片化、安全漏洞公开暴露 |
| **更优方** | Codex CLI | 开源在透明度和可信度上具有根本优势 |

### 7.1.23 本地模型支持
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | 无（不支持本地模型） | Ollama/LM Studio 本地模型支持、零 API 成本、数据不出本地 |
| **缺点** | 完全依赖云端 API、数据必须上传、网络依赖 | 本地模型质量不及云端、需要 GPU 资源、模型管理复杂 |
| **更优方** | Codex CLI | 本地模型支持在隐私和成本上具有显著优势 |

### 7.1.24 生态扩展性
| | Claude Code | Codex CLI |
|---|---|---|
| **优点** | MCP 标准协议、40+ 内置工具、Hook 系统可扩展、npm 生态丰富 | 开源社区贡献、crates.io 生态、本地模型生态（Ollama） |
| **缺点** | 闭源限制社区贡献、扩展需通过 MCP/Hook 间接实现 | 工具数量较少、无 MCP 协议、社区规模较小 |
| **更优方** | Claude Code | 内置工具丰富度和 MCP 协议标准化使其扩展性更强 |

## 7.2 编程语言深度对比：TypeScript vs Rust

### 7.2.1 为什么 Claude Code 选择 TypeScript + Bun
- **开发效率**：前端生态丰富、React/Ink 成熟、npm 海量包可直接使用
- **快速迭代**：动态类型、热重载、无需编译等待，适合快速产品迭代
- **团队技能**：Anthropic 团队 TypeScript 经验丰富，招聘池大
- **UI 需求**：React 组件化适合复杂终端 UI，声明式编程提升开发效率
- **全栈统一**：前后端统一语言，降低团队沟通成本

### 7.2.2 为什么 Codex CLI 选择 Rust
- **性能**：原生二进制、零 GC 开销、启动速度快，适合 CLI 工具
- **安全**：内存安全、类型系统保证、无数据竞争，适合安全敏感场景
- **沙箱**：需要系统级编程能力（Landlock、seccomp、Seatbelt），Rust 是最佳选择
- **单文件分发**：零依赖二进制，用户无需安装运行时，部署极简
- **可靠性**：编译时穷举检查，减少运行时错误

### 7.2.3 语言选择的权衡分析
| 维度 | TypeScript + Bun | Rust |
|------|-----------------|------|
| 开发速度 | 快（动态类型、丰富生态） | 慢（编译时检查、生命周期） |
| 运行性能 | 中等（V8 引擎优化） | 极高（原生编译、零开销抽象） |
| 内存安全 | 运行时检查 | 编译时保证 |
| 二进制大小 | 大（需打包运行时） | 小（静态编译） |
| 生态丰富度 | 极高（npm） | 中等（crates.io） |
| 跨平台 | 好（Bun 支持） | 极好（交叉编译） |
| 系统编程 | 弱（需 FFI） | 强（原生能力） |
| UI 框架 | React/Ink 成熟 | Ratatui 成长中 |
| 学习曲线 | 低 | 高 |

### 7.2.4 是否有更好的语言选择？
分析以下替代方案的可行性：

- **Go**：编译快、二进制小、goroutine 适合并发、跨平台好。但缺乏高级类型系统、UI 框架不成熟、泛型支持较晚
- **Zig**：系统编程能力强、C 互操作好。但生态太新、缺乏包管理器成熟度、学习资源少
- **Swift**：Apple 生态好、性能优秀、安全性强。但 Linux 支持不完善、社区小、终端 UI 框架缺乏
- **Kotlin/Native**：JVM 生态、多平台支持。但二进制大、启动慢、内存占用高
- **结论**：对于 AI 编码助手这类需要系统编程（沙箱）+ 终端 UI + 快速迭代的项目，TypeScript（侧重 UI 和迭代速度）和 Rust（侧重安全和性能）都是合理选择。Go 可能是最佳折中方案——兼具编译速度、并发能力和跨平台分发，但 UI 框架和类型系统的不足限制了其在复杂终端应用中的表现

## 7.3 架构模式替代方案分析

### 7.3.1 Agent 循环替代方案
- **当前方案**：Claude Code（流式异步生成器）、Codex CLI（Op/Event 提交-事件）
- **替代方案 1：Actor 模型**（如 Erlang/Akka 风格）
  - 优点：天然并发、容错性好、支持热升级、消息隔离
  - 缺点：复杂度高、调试困难、消息传递开销
- **替代方案 2：CSP（Channel）模式**（如 Go channel）
  - 优点：简洁直观、避免共享状态、适合流水线处理
  - 缺点：不如 Actor 灵活、难以表达复杂状态机
- **替代方案 3：基于图的 DAG 执行**
  - 优点：工具依赖关系可视化、天然支持并行优化、执行顺序可推导
  - 缺点：静态图难以处理动态工具链、LLM 输出的不确定性导致图结构不稳定
- **结论**：当前两种方案都是实用主义选择，流式生成器更适合 LLM 流式 API 的特性，Op/Event 更适合多线程 Rust 的类型安全需求

### 7.3.2 安全模型替代方案
- **当前方案**：Claude Code（权限管道）、Codex CLI（OS 沙箱）
- **替代方案 1：eBPF 沙箱**（Linux 内核级可编程安全）
  - 优点：比 Landlock 更灵活、性能开销低、可动态加载安全策略
  - 缺点：仅 Linux、编程复杂、内核版本要求高
- **替代方案 2：WebAssembly 沙箱**
  - 优点：跨平台、语言无关、隔离性强
  - 缺点：无法直接访问文件系统、性能损耗、工具适配成本高
- **替代方案 3：gVisor 容器**
  - 优点：完整内核级隔离、安全性极高
  - 缺点：重量级、启动慢、资源消耗大、不适合 CLI 工具
- **结论**：理想方案是 Claude Code 的细粒度权限控制 + Codex CLI 的 OS 沙箱相结合——双层安全模型既保证不可绕过的隔离，又提供精细的操作控制

### 7.3.3 文件编辑替代方案
- **当前方案**：Claude Code（search-and-replace）、Codex CLI（unified diff patch）
- **替代方案 1：AST-based 编辑**（基于抽象语法树）
  - 优点：语义精确、不受格式变化影响、支持重构操作
  - 缺点：需要每种语言的 parser、复杂度高、非代码文件无法处理
- **替代方案 2：LSP TextEdit**
  - 优点：IDE 一致性、支持所有 LSP 语言、语义感知
  - 缺点：依赖 LSP 服务器、非 LSP 语言不支持
- **替代方案 3：CRDT-based 协作编辑**
  - 优点：支持多人实时协作、无冲突合并
  - 缺点：复杂度极高、AI 场景不需要多人协作、实现成本大
- **结论**：AST-based 编辑是未来方向，但当前 search-replace 和 diff 都足够实用。短期可结合 LSP TextEdit 提升语义精确度

### 7.3.4 上下文管理替代方案
- **当前方案**：Claude Code（四层渐进式压缩）、Codex CLI（自动压缩+截断）
- **替代方案 1：RAG（检索增强生成）**
  - 优点：无限上下文、精确检索、支持海量代码库
  - 缺点：需要向量数据库、增加延迟、检索质量依赖 embedding 模型
- **替代方案 2：分层记忆（episodic + semantic + procedural）**
  - 优点：更接近人类记忆模型、不同类型信息分别存储和检索
  - 缺点：实现复杂、需要精心设计记忆索引和检索策略
- **替代方案 3：滑动窗口 + 摘要链**
  - 优点：简单有效、实现成本低
  - 缺点：信息损失大、长距离依赖容易丢失
- **结论**：Claude Code 的四层压缩已接近最优实用方案，未来可结合 RAG 实现无限上下文，分层记忆模型是更长期的演进方向

## 7.4 竞品与替代产品分析

### 7.4.1 其他 AI 编码助手对比
| 产品 | 开发商 | 开源 | 语言 | 安全模型 | 特色 |
|------|--------|------|------|----------|------|
| **Claude Code** | Anthropic | 否 | TypeScript | 权限管道 | 40+ 工具、React UI、Dream Task |
| **Codex CLI** | OpenAI | 是 | Rust | OS 沙箱 | 本地模型、WebRTC 语音 |
| **Cline** | Saoud Rizwan | 是 | TypeScript | 权限确认 | VS Code 原生、简单易用 |
| **Aider** | Paul Gauthier | 是 | Python | Git 暂存区 | 命令行极简、多模型支持 |
| **Cursor** | Anysphere | 否 | TypeScript | IDE 内置 | IDE-first、Tab 补全 |
| **Windsurf** | Codeium | 否 | TypeScript | IDE 内置 | Cascade 多步推理 |
| **Continue** | Continue Dev | 是 | TypeScript | VS Code 扩展 | 开源可定制 |
| **OpenCode** | opencode-ai | 是 | Go | 权限确认 | Go 实现、TUI |

### 7.4.2 各产品定位分析
- **IDE-first 派**：Cursor、Windsurf — 适合不想离开 IDE 的开发者，提供无缝的编辑器内 AI 体验
- **终端原生派**：Claude Code、Codex CLI、Aider — 适合终端重度用户，提供完整的命令行 AI 编码能力
- **开源派**：Codex CLI、Cline、Aider、Continue、OpenCode — 适合自托管和安全敏感场景，代码完全透明
- **极简派**：Aider — 适合快速任务，命令行极简设计，上手零成本
- **全能派**：Claude Code — 适合复杂项目、多 Agent 协作，功能最丰富但学习曲线也最陡

### 7.4.3 推荐选择矩阵
| 场景 | 推荐产品 | 理由 |
|------|---------|------|
| 企业开发（安全优先） | Codex CLI | OS 沙箱、开源可审计、零依赖二进制 |
| 复杂项目（多功能） | Claude Code | 40+ 工具、多 Agent、IDE 集成、Dream Task |
| 个人开发者（免费） | Aider + 本地模型 | 零成本、多模型支持、命令行极简 |
| VS Code 用户 | Cursor 或 Cline | IDE 原生体验、Tab 补全、无缝集成 |
| 自托管需求 | Codex CLI 或 Continue | 开源、可定制、社区活跃 |
| 学习研究 | Claude Code 源码分析 | 架构最复杂、设计模式最丰富 |

## 7.5 未来演进方向

### 7.5.1 理想 AI 编码助手架构
结合两者优点，下一代 AI 编码助手应具备以下特征：

1. **安全**：OS 级沙箱 + 细粒度权限管道（双层安全模型，内核隔离 + 应用层控制）
2. **性能**：Rust 核心循环 + TypeScript UI 层（混合语言架构，各取所长）
3. **上下文**：四层压缩 + RAG 检索（兼顾实时性和历史知识，接近无限上下文）
4. **编辑**：AST-based 精确编辑 + diff 审查（语义精确性 + 人类可读性）
5. **Agent**：OS 隔离的多 Agent + Fork 缓存优化（安全隔离 + 高效缓存）
6. **生态**：MCP 标准协议 + 开源 + 本地模型支持（开放生态 + 灵活部署）
7. **计费**：API 计费 + 本地模型免费（灵活选择，按需付费）

### 7.5.2 关键技术趋势
- **多模态**：语音、图片、视频输入输出，AI 理解截图生成代码、语音描述需求
- **实时协作**：多人 + AI 共同编码，CRDT-based 协作编辑、实时冲突解决
- **自主 Agent**：长时间自主完成复杂任务，自主规划、执行、验证、修复
- **个性化**：学习用户编码风格和偏好，自适应代码生成、风格一致的输出
- **边缘部署**：本地小模型 + 云端大模型协同，隐私敏感操作本地执行、复杂推理云端处理

---

# 第 8 章 系统提示词文件路径拆解与创新点探究

> 基于 2026 年 3-4 月公开泄露的系统提示词，深入分析 Codex CLI 和 Claude Code 的提示词文件结构与创新性设计模式。

---

## 8.1 文件路径架构对比

### 高层结构差异

```
Codex CLI                          Claude Code
┌─────────────────────┐           ┌─────────────────────────────┐
│  单一整体式           │           │  模块化多层系统               │
│  系统提示词           │           │                             │
│                     │           │  ┌─ system-prompt-main ────┐ │
│  ┌───────────────┐  │           │  ├─ system-reminders (~40) │ │
│  │ 身份与        │  │           │  ├─ agent-prompts (37)     │ │
│  │ 人格          │  │           │  ├─ skill-prompts (20)     │ │
│  ├───────────────┤  │           │  ├─ data-prompts (30)      │ │
│  │ 工作流        │  │           │  ├─ tool-descriptions (24) │ │
│  │ 规范          │  │           │  └─ conditional-injections │ │
│  ├───────────────┤  │           └─────────────────────────────┘
│  │ 代码编辑      │  │
│  │ 规则          │  │           根据以下条件组合：
│  ├───────────────┤  │           • 环境（操作系统、Shell 等）
│  │ 沙箱与        │  │           • 用户配置
│  │ 审批          │  │           • 活跃工具
│  ├───────────────┤  │           • 会话模式（plan/auto 等）
│  │ 前端          │  │           • 子 Agent类型
│  │ 指令          │  │
│  ├───────────────┤  │
│  │ 格式化        │  │
│  │ 规则          │  │
│  └───────────────┘  │
└─────────────────────┘
```

| 维度 | Codex CLI | Claude Code |
|------|-----------|-------------|
| **架构** | 整体式单文件 | 模块化多文件，条件性组合 |
| **文件数量** | 1 个系统提示词（2 个版本） | 110+ 个独立提示词文件 |
| **命名规范** | 扁平（Codex.md） | 层级式前缀（`agent-prompt-*`、`skill-*`、`data-*`、`system-prompt-*`） |
| **组合方式** | 静态——始终相同 | 动态——根据上下文按会话组装 |
| **版本追踪** | 社区维护 | 官方 CHANGELOG 追踪 150+ 个版本 |
| **可定制性** | 未设计此功能 | tweakcc 工具支持按文件定制 |

---

## 8.2 OpenAI Codex CLI — 文件结构拆解

Codex 使用**单一整体式系统提示词**，内部分区清晰。已知有两个版本：

### 版本 1：Codex CLI（开源版）

**来源：** `guy915/System-Prompts/Codex.md`（342 行，23.3 KB）

```
Codex.md
├── 身份（Identity）
│   ├── "You are a coding agent running in the Codex CLI"
│   └── 能力描述（工作区、计划、函数调用）
│
├── 工作方式（How You Work）
│   ├── 人格（Personality）
│   │   ├── "简洁、直接、友好"
│   │   └── 前言消息指南（8-12 词，分组操作）
│   ├── 规划（Planning）
│   │   ├── update_plan 工具使用
│   │   ├── 高质量 vs 低质量计划示例
│   │   └── 何时使用计划（6 个条件）
│   ├── 任务执行（Task Execution）
│   │   ├── "持续工作直到问题完全解决"
│   │   ├── apply_patch 使用规则
│   │   └── 编码准则（根因修复、最小变更、风格一致）
│   ├── 测试（Testing Your Work）
│   │   ├── 从最具体开始，逐步扩展
│   │   ├── 仅在模式表明需要时添加测试
│   │   └── 格式化命令（最多迭代 3 次）
│   └── 沙箱与审批（Sandbox and Approvals）
│       ├── 文件系统沙箱（3 级）
│       ├── 网络沙箱（2 级）
│       └── 审批模式（4 级：untrusted/on-failure/on-request/never）
│
├── 野心与精度（Ambition vs. Precision）
│   ├── 新项目 → "大胆创新"
│   └── 现有代码库 → "手术级精度"
│
├── 进度共享（Sharing Progress Updates）
│   ├── 简洁句子（8-10 词）
│   └── 大工作量前的前言
│
└── 工作展示与最终消息（Presenting Your Work）
    ├── 自然的队友语气
    ├── 最终答案格式化指南
    └── 文件引用规则（绝对路径、Markdown 链接）
```

### 版本 2：GPT-5 Codex（2026-03-25）

**来源：** `meefs/leaked-system-prompts/openai-chatgpt5-codex_20260325.md`（127 行，14.4 KB）

```
openai-chatgpt5-codex_20260325.md
├── 系统提示词
│   └── "You are Codex, a coding agent based on GPT-5"
├── 人格（Personality）
│   └── "深度务实、高效的软件工程师"
├── 价值观（Values）
│   ├── 清晰（Clarity）
│   ├── 务实（Pragmatism）
│   └── 严谨（Rigor）
├── 交互风格（Interaction Style）
│   ├── 禁止 cheerleading / 激励性语言 / fluff
│   └── 升级处理指南
├── 通用（General）
│   ├── 偏好 rg 而非 grep
│   ├── 并行工具调用（multi_tool_use.parallel）
│   └── 文件编辑默认 ASCII
├── 编辑约束（Editing Constraints）
│   ├── 仅用 apply_patch
│   ├── 脏工作区处理
│   ├── 禁止破坏性 git 命令
│   └── 禁止版权头 / 内联注释 / 单字母变量
├── 特殊用户请求（Special User Requests）
│   ├── 简单请求 → 运行终端命令
│   └── "review" → 代码审查模式（Bug 优先，按严重程度排序）
├── 自主性与持久性（Autonomy and Persistence）
│   └── "持续工作直到任务端到端完全处理"
├── 前端任务（Frontend Tasks）
│   ├── 排版（禁用默认字体栈）
│   ├── 颜色（CSS 变量，禁用紫色偏好）
│   ├── 动效（有意义的动画）
│   ├── 背景（渐变、形状、纹理）
│   └── React 模式（useEffectEvent、startTransition）
├── 与用户协作（Working with the User）
│   ├── commentary 频道（中间更新）
│   └── final 频道（完成的工作）
├── 格式化规则（Formatting Rules）
│   ├── GitHub 风格 Markdown
│   ├── 禁止嵌套列表
│   ├── 短 Title Case 标题
│   ├── 禁止 emoji / em dash
│   └── 最终答案 50-70 行限制
├── 最终答案指令（Final Answer Instructions）
│   ├── 偏好短段落
│   ├── 禁止对话式开头
│   └── 禁止"保存/复制此文件"指令
└── 中间更新（Intermediary Updates）
    ├── 1-2 句用户更新
    ├── 每 30 秒一次
    └── 变化句子结构
```

---

## 8.3 Anthropic Claude Code — 文件结构拆解

Claude Code 的提示词系统按**4 大类别**组织，使用层级式命名规范。所有文件位于 `system-prompts/` 下。

### 8.3.1 代理提示词（37 个文件）

定义子 Agent和实用功能的行为。

```
system-prompts/
├── agent-prompt-explore.md                          (494 tks)   # 文件搜索专家（只读）
├── agent-prompt-plan-mode-enhanced.md               (636 tks)   # 规划子 Agent
├── agent-prompt-general-purpose.md                  (285 tks)   # 通用编码子 Agent
├── agent-prompt-worker-fork.md                      (258 tks)   # 分叉工作子 Agent
│
├── agent-prompt-agent-creation-architect.md         (1,110 tks) # 自定义 AI 代理创建
├── agent-prompt-agent-hook.md                       (133 tks)   # 代理生命周期钩子
├── agent-prompt-hook-condition-evaluator-stop.md    (145 tks)   # 钩子停止条件
│
├── agent-prompt-batch-slash-command.md              (1,106 tks) # /batch — 可并行化变更
├── agent-prompt-review-pr-slash-command.md          (211 tks)   # /review-pr — PR 审查
├── agent-prompt-schedule-slash-command.md           (2,486 tks) # /schedule — cron 触发器
├── agent-prompt-security-review-slash-command.md    (2,607 tks) # /security-review
│
├── agent-prompt-security-monitor-...-first-part.md  (3,101 tks) # 安全监控（规则）
├── agent-prompt-security-monitor-...-second-part.md (3,325 tks) # 安全监控（环境 + block/allow）
│
├── agent-prompt-verification-specialist.md          (2,938 tks) # 对抗性验证（PASS/FAIL）
│
├── agent-prompt-dream-memory-consolidation.md       (763 tks)   # 记忆巩固
├── agent-prompt-dream-memory-pruning.md             (456 tks)   # 记忆修剪
├── agent-prompt-memory-synthesis.md                 (402 tks)   # 记忆查询综合
├── agent-prompt-session-memory-update-instructions.md (756 tks) # 会话记忆更新
├── agent-prompt-determine-which-memory-files-to-attach.md (265 tks)
│
├── agent-prompt-conversation-summarization.md       (1,121 tks) # 对话压缩
├── agent-prompt-recent-message-summarization.md     (724 tks)   # 最近消息摘要
│
├── agent-prompt-claude-guide-agent.md               (734 tks)   # 帮助用户使用 Claude Code
├── agent-prompt-claudemd-creation.md                (384 tks)   # CLAUDE.md 生成
├── agent-prompt-onboarding-guide-generator.md       (1,135 tks) # ONBOARDING.md 生成
├── agent-prompt-managed-agents-onboarding-flow.md   (2,265 tks) # 托管代理设置
│
├── agent-prompt-coding-session-title-generator.md   (181 tks)   # 会话标题
├── agent-prompt-session-title-and-branch-generation.md (307 tks) # 标题 + 分支名
├── agent-prompt-rename-auto-generate-session-name.md  (?)       # 会话重命名
├── agent-prompt-session-search.md                   (158 tks)   # 历史会话搜索
│
├── agent-prompt-prompt-suggestion-generator-v2.md   (296 tks)   # 提示建议
├── agent-prompt-auto-mode-rule-reviewer.md          (257 tks)   # 自动模式分类器审查
│
├── agent-prompt-bash-command-description-writer.md  (207 tks)   # Bash 命令描述
├── agent-prompt-bash-command-prefix-detection.md    (823 tks)   # 命令注入检测
│
├── agent-prompt-quick-git-commit.md                 (510 tks)   # 快速 git 提交
├── agent-prompt-quick-pr-creation.md                (806 tks)   # 快速 PR 创建
├── agent-prompt-status-line-setup.md                (2,029 tks) # 状态栏配置
└── agent-prompt-webfetch-summarizer.md              (189 tks)   # WebFetch 输出摘要
```

### 8.3.2 系统提示词（6 个文件）

定义主代理行为的核心系统级提示词。

```
system-prompts/
├── system-prompt-auto-mode.md                       # 自动模式行为
├── system-prompt-advisor-tool-instructions.md       # 顾问工具指南
├── system-prompt-agent-memory-instructions.md       # 记忆系统指令
├── system-prompt-agent-summary-generation.md        # 代理摘要生成
├── system-prompt-agent-thread-notes.md              # 线程笔记管理
└── （主系统提示词 — 未单独提取，嵌入编译后的 JS 中）
```

### 8.3.3 技能提示词（20 个文件）

可按需加载的专项"技能"模块，代表 Claude Code 的**插件式架构**。

```
system-prompts/
├── skill-agent-design-patterns.md                   # 代理设计模式
├── skill-build-with-claude-api-reference-guide.md   # 使用 Claude API 构建指南
├── skill-building-llm-powered-applications.md       # LLM 应用构建指南
├── skill-computer-use-mcp.md                        # 通过 MCP 使用计算机
├── skill-create-verifier-skills.md                  # 验证技能创建
├── skill-debugging.md                               # 调试方法论
├── skill-dream-nightly-schedule.md                  # 夜间梦境调度
├── skill-dynamic-pacing-loop-execution.md           # 动态节奏循环
├── skill-init-claudemd-and-skill-setup-new-version.md # CLAUDE.md 初始化
├── skill-insights-report-output.md                  # 洞察报告生成
├── skill-loop-cloud-first-scheduling-offer.md       # 云优先调度
├── skill-loop-self-pacing-mode.md                   # 自节奏模式
├── skill-loop-slash-command.md                      # 循环斜杠命令
├── skill-loop-slash-command-dynamic-mode.md         # 动态循环模式
├── skill-schedule-recurring-cron-and-execute-immediately-compact.md # Cron 调度
├── skill-schedule-recurring-cron-and-run-immediately.md # Cron + 立即运行
├── skill-simplify.md                                # 代码简化
├── skill-stuck-slash-command.md                     # 卡住时的辅助
├── skill-team-onboarding-guide.md                   # 团队入职指南
├── skill-update-claude-code-config.md               # 配置更新
├── skill-update-config-7-step-verification-flow.md  # 7 步配置验证
├── skill-verify-cli-changes-example-for-verify-skill.md # CLI 变更验证
├── skill-verify-serverapi-changes-example.md        # 服务器 API 验证
├── skill-verify-skill-runtime-verification.md       # 运行时验证
└── skill-verify-skill.md                            # 验证技能
```

### 8.3.4 数据提示词（30 个文件）

直接嵌入提示词上下文的参考数据——本质上是**内置知识库**。

```
system-prompts/
├── data-claude-api-reference-c.md                   (4,341 tks) # C# SDK
├── data-claude-api-reference-curl.md                (2,174 tks) # cURL / 原始 HTTP
├── data-claude-api-reference-go.md                  (4,294 tks) # Go SDK
├── data-claude-api-reference-java.md                (4,506 tks) # Java SDK
├── data-claude-api-reference-php.md                 (3,486 tks) # PHP SDK
├── data-claude-api-reference-python.md              (3,549 tks) # Python SDK
├── data-claude-api-reference-ruby.md                (923 tks)   # Ruby SDK
├── data-claude-api-reference-typescript.md          (2,881 tks) # TypeScript SDK
├── data-claude-model-catalog.md                     (2,295 tks) # 模型目录 + 定价
├── data-files-api-reference-python.md               (1,334 tks) # Files API（Python）
├── data-files-api-reference-typescript.md           (797 tks)   # Files API（TypeScript）
├── data-github-actions-workflow-for-claude-mentions.md (527 tks) # GitHub Actions
├── data-github-app-installation-pr-description.md   (424 tks)   # GitHub App PR 模板
├── data-http-error-codes-reference.md               (1,922 tks) # HTTP 错误码
├── data-live-documentation-sources.md               (3,584 tks) # 文档 WebFetch URL
├── data-managed-agents-client-patterns.md           (2,685 tks) # 托管代理模式
├── data-managed-agents-core-concepts.md             (3,208 tks) # 托管代理概念
├── data-managed-agents-endpoint-reference.md        (4,526 tks) # 托管代理端点
├── data-managed-agents-environments-and-resources.md (2,909 tks) # 托管代理环境
├── data-managed-agents-events-and-steering.md       (2,428 tks) # 托管代理事件
├── data-managed-agents-overview.md                  (2,202 tks) # 托管代理概览
├── data-managed-agents-reference-curl.md            (2,641 tks) # 托管代理（cURL）
├── data-managed-agents-reference-python.md          (2,841 tks) # 托管代理（Python）
├── data-managed-agents-reference-typescript.md      (2,855 tks) # 托管代理（TS）
├── data-managed-agents-tools-and-skills.md          (3,844 tks) # 托管代理工具
├── data-message-batches-api-reference-python.md     (1,544 tks) # 批量 API
├── data-prompt-caching-design-optimization.md       (2,657 tks) # 提示缓存指南
├── data-session-memory-template.md                  (292 tks)   # 记忆模板
├── data-streaming-reference-python.md               (1,528 tks) # 流式（Python）
├── data-streaming-reference-typescript.md           (1,703 tks) # 流式（TypeScript）
├── data-tool-use-concepts.md                        (4,139 tks) # 工具使用概念
├── data-tool-use-reference-python.md                # 工具使用（Python）
└── data-tool-use-reference-typescript.md            # 工具使用（TypeScript）
```

---

## 8.4 创新点分析

### 8.4.1 Claude Code 的创新

#### 创新 1：模块化提示词组合架构

**是什么：** 110+ 个独立提示词文件，按会话**条件性组装**。

**为什么创新：**
- 传统系统提示词是整体式字符串。Claude Code 将提示词视为**可组合模块**——类似提示工程的微服务架构。
- 不同会话根据环境、工具、模式和用户配置加载不同的提示词组合。
- 支持**增量更新**——修改一个模块不需要重写整个提示词。
- tweakcc 工具允许用户自定义单个提示词文件，并提供冲突的 diff 管理。

**类比：** 如果 Codex 的提示词是单个 `.c` 文件，Claude Code 的就是一个组织良好的 npm 包，带有 `import` 语句。

#### 创新 2：独立安全监控子 Agent

**是什么：** 一个专用的 6,400+ token 安全分类器，实时评估每个代理操作。

**为什么创新：**
- 这不仅仅是"提示词中的安全规则"——它是一个**独立的 AI 驱动安全层**，独立运行。
- 定义了正式的**威胁模型**，包含三个类别：提示注入、范围蔓延、意外损坏。
- 实现了复杂的评估规则：
  - **复合操作：** 链式命令（`&&`、`||`、`;`）——任何部分危险则阻止全部
  - **已写文件执行：** 追踪之前写入的文件，在执行时评估其内容
  - **延迟效果：** 阻止会启用未来危害的操作（环境变量、cron 任务、权限授予）
  - **子 Agent委派：** 检查生成子 Agent的 `prompt` 字段是否包含危险指令
  - **分类器绕过检测：** 阻止试图操纵分类器本身的行为
  - **预防性阻止：** 即使当前操作无害，有明确危险意图也阻止
- **用户意图规则**尤为精细——区分用户请求和代理操作，认识到"问题不是同意"。

**Codex 没有对应功能**——Codex 依赖沙箱权限和审批模式，但没有 AI 驱动的实时安全评估。

#### 创新 3："梦境"记忆系统

**是什么：** 一个受生物学启发的记忆巩固系统，包含四个阶段。

**为什么创新：**
- **阶段 1 — 定向：** 读取现有记忆索引和主题文件
- **阶段 2 — 收集：** 搜索每日日志，检查矛盾事实，用窄关键词 grep 转录文件
- **阶段 3 — 巩固：** 将新信号合并到现有文件，将相对日期转为绝对日期，删除矛盾事实
- **阶段 4 — 修剪：** 更新索引（保持在最大行数以下），移除过时指针，解决矛盾

这模仿了人类睡眠中巩固记忆的过程——代理字面意义上地"做梦"来整理所学。系统包括：
- **记忆综合：** 查询时仅检索相关记忆
- **记忆修剪：** 自动删除过时/重复记忆
- **会话记忆：** 跨对话持久化的会话上下文

**Codex 完全没有记忆系统**——每次会话从零开始。

#### 创新 4：动态节奏循环

**是什么：** 一个自调节执行循环，根据观察到的活动调整唤醒间隔。

```
关键机制：
- 如果监控工具已启用 → 使用 1200-1800 秒后备心跳
- 如果没有监控 → 根据活动级别调整延迟
  - 安静分支？→ 等更久
  - 大量进行中？→ 等更短
- 事件驱动唤醒：任务通知绕过调度
- 当代理省略唤醒调用时循环停止
```

**为什么创新：** 这本质上是一个**自调节代理调度器**——AI 根据观察决定"检查"频率，类似人类开发者监控长时间运行的部署。

#### 创新 5：内置知识库（数据提示词）

**是什么：** 30+ 参考文档（8 种语言 SDK 指南、模型目录、HTTP 错误码等）直接嵌入提示词上下文。

**为什么创新：**
- 代理不需要搜索网络获取 API 文档——它已经内置了。
- 包含**实时文档源**（WebFetch URL），需要时可获取最新文档。
- 覆盖**整个 Claude API 生态**：Messages API、Files API、Streaming、Tool Use、Prompt Caching、Managed Agents、Message Batches。
- 这本质上是代理上下文中的**策展式、始终可用的技术图书馆**。

#### 创新 6：对抗性验证专家

**是什么：** 一个专用子 Agent（2,938 tokens），主动尝试破坏实现。

**为什么创新：**
- 不仅是"运行测试"——它执行**对抗性探测**：刻意寻找边界情况、破坏事物、利用漏洞。
- 发布正式的 **PASS / FAIL / PARTIAL** 裁定。
- 这更接近代码验证的**红队**方法。

#### 创新 7：代理创建架构师

**是什么：** 一个元提示词（1,110 tokens），帮助用户创建具有详细规范的自定义 AI 代理。

**为什么创新：** Claude Code 可以**创建新代理**——它不仅是编码助手，更是代理工厂。这实现了递归能力，工具可以扩展自身功能。

### 8.4.2 Codex 的创新

#### 创新 1：人格驱动的工程文化

**是什么：** Codex 显式定义工程价值观（清晰、务实、严谨）和反模式（禁 fluff、禁 cheerleading）。

**为什么创新：**
- 超越了典型的"乐于助人"指令——定义了一个**完整的工程人格**，包含具体行为约束。
- "禁 fluff"规则执行粒度极细：禁对话式开头、禁激励性语言、禁 emoji、禁嵌套列表。
- **升级（Escalation）**概念独特——Codex 被明确告知可以挑战用户的技术决策，但绝不居高临下。

#### 创新 2：双频道通信

**是什么：** 分离的 `commentary` 和 `final` 频道用于不同类型的消息。

**为什么创新：**
- 中间更新发送到 `commentary`——简短、频繁、进行中。
- 完成的工作发送到 `final`——结构化、全面。
- 这种分离允许 UI 以不同方式渲染不同消息类型，创造更清晰的用户体验。

#### 创新 3：四级审批矩阵

**是什么：** 一个 3×4 的文件系统 × 审批配置矩阵。

**为什么创新：**
- `never` 模式特别有趣——它强制代理完全自主，在不询问用户的情况下绕过约束。
- `on-request` 模式赋予代理决定何时需要提升权限的自主权。
- 这是一个适应不同风险容忍度的**分级信任模型**。

#### 创新 4：前端"反 AI Slop"指令

**是什么：** 避免通用、模板化前端设计的详细指令。

**为什么创新：**
- 这是一个**元认知**问题——OpenAI 认识到 LLM 倾向于产生同质化设计，并明确指示避免。
- 具体到可操作："禁用 Inter、Roboto、Arial"、"禁用紫色-on-white"、"禁用纯色背景"。
- "跨输出变化主题"指令对抗模型重复模式的倾向。

#### 创新 5：脏工作区感知

**是什么：** 处理预存未提交变更的明确指令。

**为什么创新：**
- 展示了对真实开发工作流的感知——工作区很少是干净的。
- 代理被告知**绝不回退未做出的变更**——尊重用户的工作。
- 如果出现意外变更，假设是用户做出的，除非直接冲突。

### 8.4.3 共有模式

| 模式 | Codex | Claude Code |
|------|-------|-------------|
| **apply_patch 编辑** | ✅ 强制 | ✅（通过 Write/Edit 工具） |
| **只读探索** | ❌（无独立模式） | ✅（专用 Explore 代理） |
| **计划跟踪** | ✅ update_plan 工具 | ✅ TodoWrite 工具 |
| **并行工具调用** | ✅ multi_tool_use.parallel | ✅ 批量工具调用 |
| **测试指导** | ✅ 从具体到广泛 | ✅ 验证专家 |
| **Git 安全** | ✅ 禁止破坏性命令 | ✅ 快速提交/PR 子 Agent |
| **简洁输出** | ✅ 50-70 行限制 | ✅（较宽松） |
| **不过度工程** | ✅ "Don't gold-plate" | ✅ "Don't gold-plate, but don't leave it half-done" |

---

## 8.5 架构哲学对比

```
CODEX 哲学                           CLAUDE CODE 哲学
━━━━━━━━━━━━━━━━                    ━━━━━━━━━━━━━━━━━━━━━━

"一个卓越的工程师"                    "一个专家团队"

单一提示词 = 单一思维                 模块化提示词 = 组织架构

通过规则建立信任                     通过监督建立信任

人格驱动行为                         角色驱动行为

"了解规则，遵守规则"                  "了解规则，监控合规"

静态组合                             动态、上下文感知组合

无记忆（无状态）                     持久化记忆（有状态）

用户审批操作                         安全监控审批操作

聚焦代码                             代码 + DevOps + 安全 + 记忆
```

---

## 8.6 Token 预算分析

| 类别 | Codex CLI | Claude Code |
|------|-----------|-------------|
| 主系统提示词 | ~6,000 tks | ~15,000+ tks（估算主提示词） |
| 子 Agent提示词 | 无 | ~25,000 tks（37 个文件） |
| 技能提示词 | 无 | ~10,000 tks（估算，20 个文件） |
| 数据/参考提示词 | 无 | ~70,000 tks（30 个文件） |
| 系统提醒 | 无 | ~8,000 tks（~40 个提醒） |
| 工具描述 | 嵌入式 | ~15,000 tks（24 个工具） |
| **总可用量** | **~6,000 tks** | **~143,000+ tks** |
| **典型会话加载** | **~6,000 tks** | **~30,000-50,000 tks**（条件性） |

**关键洞察：** Claude Code 有约 24 倍更多的可用提示词内容，但每次会话仅根据上下文加载一部分。这就是模块化组合的优势——获得特异性而不产生膨胀。

---

## 8.7 与前文架构分析的关联

本章揭示的提示词架构创新与前文各章分析的代码实现高度一致：

| 前文章节 | 对应提示词创新 |
|----------|---------------|
| 第 3 章 Agent 循环 | Codex 的"持续工作直到完全解决"指令驱动了其 Op/Event 循环设计；Claude Code 的模块化提示词支撑了其流式异步生成器架构 |
| 第 4 章 工具系统 | Codex 的"仅用 apply_patch"规则对应其 unified diff 实现；Claude Code 的延迟加载工具提示词对应 ToolSearchTool |
| 第 5 章 上下文管理 | Claude Code 的四层压缩策略与其"梦境"记忆系统提示词直接对应；Codex 的无记忆设计与其简洁的压缩算法一致 |
| 第 6 章 安全系统 | Claude Code 的 7 层权限管道与其独立安全监控子 Agent提示词形成纵深防御；Codex 的 OS 沙箱与其 4 级审批模式互补 |
| 第 7 章 多 Agent | Claude Code 的三级 Agent 架构与其 37 个子 Agent提示词一一对应；Codex 的无状态设计与无子 Agent提示词一致 |

**结论：** 系统提示词不仅是"对模型的指令"，更是整个产品架构的**蓝图**。Claude Code 的模块化提示词架构反映了其"功能优先、复杂但全面"的产品哲学；Codex 的整体式提示词反映了其"简洁、高效、安全"的工程哲学。两者各有千秋，但 Claude Code 在提示词工程方面的创新更为激进和前沿。

---

## 8.8 信息来源

- [Claude Code 系统提示词仓库 (Piebald-AI)](https://github.com/Piebald-AI/claude-code-system-prompts) — v2.1.107，2026 年 4 月
- [Codex CLI 系统提示词 (guy915/System-Prompts)](https://github.com/guy915/System-Prompts/blob/main/Codex.md)
- [GPT-5 Codex 系统提示词 (meefs/leaked-system-prompts)](https://github.com/meefs/leaked-system-prompts/blob/main/openai-chatgpt5-codex_20260325.md)
- [tweakcc — Claude Code 提示词定制工具](https://github.com/Piebald-AI/tweakcc)
- [Claude Code 源码泄露分析](https://www.implicator.ai/anthropic-built-an-operating-system-for-code-then-shipped-the-blueprints-to-npm-2/)
