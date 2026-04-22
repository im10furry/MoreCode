//! `core` defines shared core types for the MoreCode workspace.

pub mod agent;
pub mod color;
pub mod constants;
pub mod error;
pub mod id;
pub mod line_ending;
pub mod message;
pub mod result;
pub mod run;
pub mod task;
pub mod time;
pub mod token;

pub use agent::{AgentExecutionStatus, AgentStatus, AgentType, ToolCallStatus};
pub use color::{Color, DarkTheme, NamedColor, SemanticColor, TerminalColor, Theme};
pub use constants::*;
pub use error::{McError, McResult};
pub use id::{generate_id, generate_trace_id};
pub use line_ending::{
    detect_line_endings, is_probably_binary, is_safe_newline_rewrite, normalize_line_endings,
    EolStats, LineEnding,
};
pub use message::{FinishReason, MessageRole};
pub use result::{AgentExecutionReport, TaskResult};
pub use run::*;
pub use task::*;
pub use time::{format_duration, now_utc};
pub use token::{ModelInfo, TokenUsage, ToolDefinition};
