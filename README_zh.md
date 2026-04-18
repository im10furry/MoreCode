# MoreCode Agent

> 基于 Rust + Ratatui 构建的多 Agent 编排 AI 编码助手，通过专职 Agent 协作将复杂编码任务自动化。

---

## 功能特性

- **多 Agent 编排**：10 个专职 Agent（Coordinator、Explorer、Impact Analyzer、Planner、Coder、Reviewer、Tester、Research、DocWriter、Debugger）按能力类型协作，职责分离、上下文隔离、并行执行
- **递归编排（Map-Filter-Reduce）**：任何 Agent 可作为"子协调者"将复杂任务拆分为子任务并行处理，支持动态 Token 预算分配和深度控制
- **智能路由决策**：四级复杂度路由（简单/中等/复杂/研究），80% 请求无需 LLM 即可完成路由，记忆感知路由利用项目知识加速决策
- **双层安全沙箱**：OS 层（Landlock + Seccomp）作为安全基座 + WASM 层（Wasmtime + WASI）提供函数级隔离和跨平台能力
- **四层渐进式上下文压缩**：微压缩 → LLM 摘要 → 记忆压缩 → 反应式截断，配合 Focus 主动压缩创新方向
- **Letta 式分层记忆系统**：Core/Working/Recall/Archival 四层记忆，支持跨会话复用、增量更新和 Sleep-Time Compute
- **多 LLM Provider 支持**：OpenAI 兼容统一层覆盖 DeepSeek/智谱/通义/Moonshot/Ollama 等，可选 Anthropic、Google Gemini，内置语义缓存中间件
- **MCP 协议集成**：基于 rmcp v1.4+ 官方 SDK，同时作为 MCP 客户端和服务端，支持 Stdio/HTTP/Unix Socket 三种传输
- **五层 Prompt 缓存**：全局/组织/项目/会话/轮次分层缓存，预期节省 50-67% 输入 Token 成本
- **Ratatui 终端界面**：实时展示 Agent 执行状态、Token 消耗、通信拓扑，支持进度流/代码流/确认流三种反馈模式
- **Daemon 模式**：支持 7×24 无人值守自主运行，Checkpoint 恢复、成本控制、通知系统

---

## 技术栈

| 类别 | 技术 | 说明 |
|------|------|------|
| 主语言 | **Rust** | 高性能、内存安全、零成本抽象，编译为单一二进制 |
| 并发模型 | **Actor + CSP Channel** | 每个 Agent 为独立 Actor，通过 Channel 异步通信 |
| 异步运行时 | **Tokio** | Reactor 模式调度，Rust 生态事实标准 |
| UI 框架 | **Ratatui** | 终端 TUI 框架，即时模式渲染 |
| 安全沙箱 | **Landlock + Seccomp + WASM** | 双层沙箱设计 |
| 工具协议 | **MCP (rmcp v1.4+)** | AI 工具互操作事实标准 |
| 代码解析 | **Tree-sitter** | 增量语法解析，生成 Repo Map |
| 可观测性 | **tracing + OpenTelemetry + Langfuse** | Agent 专用 Span + 开源自托管 |

---

## 架构概览

```
┌─────────────────────────────────────────────────────────────────────┐
│                          用户输入 (CLI / TUI)                        │
│                      Ratatui 终端界面层                              │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     ┌─────────────────┐                             │
│                     │   Coordinator   │                             │
│                     │   (主协调者)     │                             │
│                     └────────┬────────┘                             │
│              ┌───────────────┼───────────────┐                      │
│     ┌────────▼──────┐ ┌─────▼──────┐ ┌──────▼────────┐             │
│     │   Explorer    │ │  Planner   │ │ ImpactAnalyzer│             │
│     └───────────────┘ └─────┬──────┘ └───────────────┘             │
│              ┌─────────────┼─────────────┐                         │
│     ┌────────▼──────┐ ┌───▼────────┐ ┌──▼──────────┐              │
│     │    Coder      │ │  Reviewer  │ │   Tester    │              │
│     └───────────────┘ └────────────┘ └─────────────┘              │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│     ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│     │   Research   │ │  DocWriter   │ │   Debugger   │             │
│     └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│  工具层：文件读写 | 代码搜索 | Git 操作 | 终端执行 | LLM 调用       │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│  状态管理层：记忆管理器 | 配置管理 | Checkpoint | 审计日志            │
└─────────────────────────────────────────────────────────────────────┘
```

### 设计原则

1. **先理解再分析后执行** — 任何编码任务都必须经过 Explorer → Impact Analyzer → Coder 流程
2. **不是所有任务都需要多 Agent** — 简单任务直接路由到单个 Agent，避免不必要的编排开销
3. **上下文按需注入** — 每个 Agent 只接收完成其职责所需的最小上下文
4. **摘要通信** — Agent 间通过结构化摘要（JSON）通信，而非传递原始上下文

---

## 项目结构

```
morecode-agent/
├── Cargo.toml                    # Workspace 根配置
├── core/                         # 核心类型与 trait（零外部依赖）
├── coordinator/                  # 协调器（意图理解、路由决策、执行监控）
├── agent/                        # Agent trait + 10 个 Agent 实现
├── communication/                # 四级通信系统（控制/状态/数据/广播）
├── llm/                          # LLM Provider 抽象层 + 多后端实现
├── context/                      # 上下文管理与四层压缩策略
├── memory/                       # 项目记忆系统（Letta 式四层分层）
├── prompt/                       # Prompt 模板与五层缓存管理
├── tool/                         # 工具注册与内置工具实现
├── config/                       # 多级配置管理（全局/项目/环境变量）
├── sandbox/                      # Guardian 双层沙箱
├── recursive/                    # 递归编排引擎（Map-Filter-Reduce）
├── daemon/                       # Daemon 模式 + 生命周期管理
├── tui/                          # Ratatui 终端界面
├── cli/                          # CLI 入口（二进制 crate）
├── prompts/                      # Prompt 模板文件（system/tools/org/project）
└── tests/                        # 集成测试与端到端测试
```

---

## 快速开始

### 环境要求

- Rust stable (最新版)
- <!-- TODO: 补充具体最低 Rust 版本 -->

### 构建

```bash
# 克隆项目
git clone https://github.com/<!-- TODO: 补充仓库地址 -->/morecode-agent.git
cd morecode-agent

# 最小构建（无可选功能）
cargo build -p cli --no-default-features

# 完整构建（含所有 Provider 和沙箱后端）
cargo build --all-features
```

### 基本用法

```bash
# 交互式运行
morecode

# Daemon 模式
morecode daemon start

# 环境诊断
morecode doctor
```

### Feature Flags

| Feature | 说明 |
|---------|------|
| `tui` | Ratatui 终端界面（默认启用） |
| `daemon` | Daemon 模式（默认启用） |
| `anthropic` | Anthropic Claude Provider |
| `google` | Google Gemini Provider |
| `landlock` | Linux Landlock 文件系统沙箱 |
| `seccomp` | Seccomp 系统调用过滤 |
| `wasm` | WASM 沙箱（Wasmtime + WASI） |
| `mock` | Mock LLM Provider（测试用） |

---

## 配置

配置采用多级合并策略：**项目级 > 全局级 > 代码默认值**

```bash
# 全局配置目录
~/.morecode/
├── config.toml          # 全局默认配置
├── routing.toml         # 路由规则
├── daemon.toml          # Daemon 配置
├── providers.toml       # LLM Provider 配置
└── cost.toml            # 成本预算配置

# 项目配置目录（覆盖全局）
.morecode/
├── config.toml
└── routing.toml
```

### LLM Provider 配置示例

```toml
# ~/.morecode/providers.toml
[providers.openai]
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[providers.deepseek]
model = "deepseek-chat"
base_url = "https://api.deepseek.com/v1"
api_key_env = "DEEPSEEK_API_KEY"
```

---

## 扩展开发

MoreCode Agent 采用零侵入扩展设计，新增组件无需修改已有代码：

| 扩展场景 | 操作步骤 |
|----------|---------|
| **新增 Agent** | ① `agent/src/{name}/mod.rs` 实现 `Agent` trait → ② `registry.rs` 注册 → ③ `prompts/system/{name}.md` 添加系统提示 |
| **新增 LLM Provider** | ① `llm/src/{name}/` 实现 `LlmProvider` trait → ② `Cargo.toml` 添加 feature flag |
| **新增工具** | ① `tool/src/builtin/{name}.rs` 实现 `Tool` trait → ② `catalog/` 分配可见性 → ③ `registry.rs` 注册 |
| **新增沙箱后端** | ① `sandbox/src/os_layer/{name}.rs` → ② `lib.rs` 添加 feature 条件编译 |

---

## 实现路线

```
Phase 1（MVP）:
  LLM Provider → Token 计数 → 上下文压缩(L1+L4) → 权限管理 → 沙箱 → 通信 → AST(基础)

Phase 2（增强）:
  Prompt 缓存(五层) → 流式输出 → 中断取消 → Letta 式记忆 → Hook → MCP → Prompt 模板 → 工具注册

Phase 3（创新）:
  上下文压缩(Focus) → AST(Probe+LSP-MCP) → 遥测 → 配置 → 文件监听 → Checkpoint

Phase 4（前沿）:
  上下文压缩(ACON 蒸馏) → 语义检索(Mem0-G) → WASM 加载 → MCP Server → IDE 集成
```

---

## 许可证

[GNU General Public License v3.0](LICENSE)
