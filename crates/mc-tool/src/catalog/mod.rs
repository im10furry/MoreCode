mod core;
mod deferred;
mod extended;

pub use core::register_core_tools;
pub use deferred::register_deferred_tools;
pub use extended::register_extended_tools;

use crate::registry::ToolRegistry;

pub async fn register_all_tools(registry: &ToolRegistry) {
    register_core_tools(registry).await;
    register_extended_tools(registry).await;
    register_deferred_tools(registry).await;
}
