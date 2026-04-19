# MoreCode Agent

> 基于 Rust 构建的多 Agent 编码助手，围绕 Coordinator、认知分析 Agent、执行 Agent、专项 Agent 进行协作式任务处理。

---

## 功能概览

- 多 Agent 编排：Coordinator、Explorer、Impact Analyzer、Planner、Coder、Reviewer、Tester、Research、DocWriter、Debugger。
- 递归编排：支持 Map-Filter-Reduce 风格的子任务拆分。
- 四级路由：Simple / Medium / Complex / Research。
- 多 LLM Provider：OpenAI 兼容层 + Anthropic + Google + Mock。
- Prompt 缓存、项目记忆、语义缓存、工具注册、Guardian 沙箱、Daemon 模式。
- 终端界面骨架：面板、视图、部件、主题与事件模型已接入。

## 技术栈

- Rust
- Tokio
- Ratatui
- Landlock / Seccomp / WASM
- MCP
- tracing / OpenTelemetry / Langfuse

## 快速开始

### 环境要求

- Rust stable
- 最低 Rust 版本：`1.88.0`

### 获取代码

```bash
git clone https://github.com/im10furry/MoreCode.git
cd MoreCode
```

### 构建

```bash
# 最小构建
cargo build -p cli --no-default-features

# 完整构建
cargo build --all-features
```

### 使用示例

```bash
# 通过当前 CLI 入口运行请求
cargo run -p cli -- run "summarize the current project"

# 查看 daemon 状态
cargo run -p cli -- daemon status

# 环境诊断
cargo run -p cli -- doctor

# 查看项目记忆状态
cargo run -p cli -- memory status
```

## 配置

配置采用多级合并策略：`项目级 > 全局级 > 代码默认值`

全局配置目录：

```text
~/.morecode/
├── config.toml
├── routing.toml
├── daemon.toml
├── providers.toml
└── cost.toml
```

项目级配置目录：

```text
.morecode/
├── config.toml
└── routing.toml
```

### Provider 配置示例

```toml
[providers.openai]
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[providers.deepseek]
model = "deepseek-chat"
base_url = "https://api.deepseek.com/v1"
api_key_env = "DEEPSEEK_API_KEY"
```

## 扩展开发

- 新增 Agent：在 `agent/src/{name}/` 实现并注册。
- 新增 LLM Provider：在 `llm/src/{name}/` 实现并加 feature。
- 新增 Tool：在 `tool/src/builtin/{name}.rs` 实现并注册。
- 新增 Sandbox Backend：在 `sandbox/src/os_layer/{name}.rs` 实现并导出。

## 目录结构

```text
core/            核心类型
coordinator/     编排与路由
agent/           Agent 实现
communication/   四级通信系统
llm/             LLM Provider 抽象与实现
context/         上下文管理
memory/          项目记忆与 Letta 分层记忆
prompt/          Prompt 模板与缓存
tool/            工具系统
config/          配置系统
sandbox/         Guardian 与 OS/WASM 沙箱
recursive/       递归编排
daemon/          Daemon 运行时
tui/             终端界面
cli/             CLI 入口
```

## 路线图

```text
Phase 1:
  LLM Provider -> Token 计数 -> 上下文压缩 -> 权限 -> 沙箱 -> 通信 -> AST

Phase 2:
  Prompt 缓存 -> 流式输出 -> 中断取消 -> Letta 记忆 -> Hook -> MCP -> 工具注册

Phase 3:
  Focus 压缩 -> AST Probe/LSP-MCP -> 遥测 -> 配置 -> 文件监听 -> Checkpoint

Phase 4:
  ACON 蒸馏 -> Mem0-G 检索 -> WASM 加载 -> MCP Server -> IDE 集成
```

## License

[GNU General Public License v3.0](LICENSE)
