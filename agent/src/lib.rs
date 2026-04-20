#![forbid(unsafe_code)]

pub mod context;
pub mod error;
pub mod execution_report;
pub mod handoff;
pub mod registry;
pub mod stream;
pub mod trait_def;
pub mod lifecycle;

pub use context::*;
pub use error::*;
pub use execution_report::*;
pub use handoff::*;
pub use lifecycle::*;
pub use mc_core::{AgentLayer, AgentType};
pub use mc_core::task::{ExecutionPlan, ProjectContext, TaskDescription};
pub use registry::*;
pub use stream::*;
pub use trait_def::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
