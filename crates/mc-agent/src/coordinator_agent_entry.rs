#![forbid(unsafe_code)]

mod config;
mod context;
mod error;
mod execution_report;
pub mod explorer;
mod handoff;
pub mod handoff_min;
pub mod impact_analyzer;
pub mod pipeline;
pub mod planner;
pub mod registry_min;
mod support;
#[cfg(test)]
mod test_support;
mod trait_def;
pub mod trait_def_min;

pub use config::{AgentConfig, ExplorerConfig, LlmConfig, PlannerConfig};
pub use context::{AgentContext, SharedResources};
pub use error::AgentError;
pub use execution_report::{AgentExecutionMetrics, AgentExecutionReport};
pub use explorer::Explorer;
pub use handoff::AgentHandoff;
pub use impact_analyzer::{ImpactAnalyzer, ImpactChange, ImpactReport, RiskAssessment};
pub use pipeline::{CognitivePipeline, CognitivePipelineResult};
pub use planner::Planner;
pub use trait_def::{Agent, AgentCapabilities};
