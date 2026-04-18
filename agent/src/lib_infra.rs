#[path = "infra_context.rs"]
pub mod context;
#[path = "infra_error.rs"]
pub mod error;
#[path = "infra_execution_report.rs"]
pub mod execution_report;
#[path = "handoff.rs"]
pub mod handoff;
#[path = "infra_registry.rs"]
pub mod registry;
#[path = "stream.rs"]
pub mod stream;
#[path = "infra_trait_def.rs"]
pub mod trait_def;
#[path = "infra_lifecycle.rs"]
pub mod lifecycle;

pub use context::{
    AgentContext, AgentEvent, AgentEventKind, EventBus, GlobalConfig, ImpactReport,
    ImpactRiskLevel, InMemoryEventBus, LlmClient, LlmRequest, LlmResponse, Logger, MemoryManager,
    SharedResources, ToolRegistry, TracingLogger,
};
pub use error::{AgentError, ErrorCategory};
pub use execution_report::{
    AgentExecutionReport, ExecutionStatus, FileChange, FileChangeType, LlmCallStats,
    ToolCallRecord,
};
pub use handoff::AgentHandoff;
pub use lifecycle::{
    execute_streaming_with_context, execute_with_context, AgentLifecycle, LifecycleHandler,
    LoggingLifecycle, NoopLifecycle,
};
pub use mc_core::{AgentLayer, AgentType};
pub use mc_core::task::{ExecutionPlan, ProjectContext, TaskDescription};
pub use registry::{AgentFactory, AgentRegistry, SharedAgentHandle};
pub use stream::{StdoutStreamForwarder, StreamEvent, StreamEventType, StreamForwarder};
pub use trait_def::{
    Agent, AgentCapabilities, AgentCapability, AgentConfig, AgentConfigPatch, LlmAgentConfig,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
