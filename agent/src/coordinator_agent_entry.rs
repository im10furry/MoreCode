#![forbid(unsafe_code)]

mod config;
mod context;
pub mod coder;
mod error;
mod execution_report;
pub mod explorer;
mod handoff;
pub mod handoff_min;
pub mod impact_analyzer;
pub mod pipeline;
pub mod planner;
pub mod registry_min;
pub mod reviewer;
mod support;
#[cfg(test)]
mod test_support;
pub mod tester;
mod trait_def;
pub mod trait_def_min;

pub use config::{AgentConfig, ExplorerConfig, LlmConfig, PlannerConfig};
pub use context::{AgentContext, SharedResources};
pub use coder::codegen::{CodeChangeDraft, CodeChangeKind, CodeGenerationOutput};
pub use coder::Coder;
pub use error::AgentError;
pub use execution_report::{AgentExecutionMetrics, AgentExecutionReport};
pub use explorer::Explorer;
pub use handoff::AgentHandoff;
pub use impact_analyzer::{ImpactAnalyzer, ImpactChange, ImpactReport, RiskAssessment};
pub use pipeline::{CognitivePipeline, CognitivePipelineResult};
pub use planner::Planner;
pub use reviewer::rule_engine::{ReviewFinding, ReviewReport, ReviewSeverity, ReviewVerdict};
pub use reviewer::Reviewer;
pub use tester::framework::{TestCommand, TestFramework, TestRunSummary};
pub use tester::{Tester, TesterExecutionReport};
pub use trait_def::{Agent, AgentCapabilities};
