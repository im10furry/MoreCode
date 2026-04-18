#![forbid(unsafe_code)]

#[path = "lifecycle_v2.rs"]
mod lifecycle;
#[path = "config_v2.rs"]
pub mod config;
#[path = "context_v2.rs"]
pub mod context;
pub mod debugger;
pub mod doc_writer;
#[path = "error_v2.rs"]
pub mod error;
#[path = "execution_report_v2.rs"]
pub mod execution_report;
#[path = "handoff_v2.rs"]
pub mod handoff;
#[path = "registry_v2.rs"]
pub mod registry;
pub mod research;
#[path = "trait_def_v2.rs"]
pub mod trait_def;

pub use config::AgentConfig;
pub use context::{AgentContext, SharedResources};
pub use debugger::{
    Debugger, DefaultLogAnalyzer, DefaultStackTraceParser, ErrorAnalysis, ErrorType, FixReport,
    FixSuggestion, LogAnalyzer, LogPattern, ParsedStackTrace, StackFrame, StackTraceParser,
    SuggestedChange,
};
pub use doc_writer::{
    DocWriter, DocumentType, Documentation, GeneratedDocument, SimpleTemplateEngine,
    TemplateEngine,
};
pub use error::AgentError;
pub use execution_report::{build_report, serialize_extra};
pub use handoff::AgentHandoff;
pub use lifecycle::{AgentLifecycle, NoopLifecycle};
pub use registry::{AgentFactory, AgentRegistry};
pub use research::{
    Research, ResearchFinding, ResearchReport, ResearchSource, ResearchSourceKind,
    TechnologyComparison,
};
pub use trait_def::{Agent, AgentCapabilities, AgentEventSink, AgentStreamEvent};
