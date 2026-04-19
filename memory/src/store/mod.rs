mod agent_notes;
mod conventions;
mod manager;
mod overview;
mod risk_areas;
mod tech_stack;
mod types;

pub use agent_notes::{list_agent_notes, load_agent_note, write_agent_note, AgentNoteRecord};
pub use conventions::{load_conventions, save_conventions};
pub use manager::{MemoryManager, MemoryManagerTrait};
pub use overview::{load_overview, save_overview};
pub use risk_areas::{append_risk, load_risk_areas, save_risk_areas};
pub use tech_stack::{load_tech_stack, save_tech_stack};
pub use types::{
    ApiEndpoint, ApiEndpoints, DataModel, DataModels, DependencyEdge, DependencyGraph, FieldInfo,
    FileChange, MemoryUpdate, MemoryWriteRequest, MetaJson, ModuleInfo, ModuleMap, ProjectMemory,
    ProjectMemorySnapshot, RiskAreas, RiskInfo, TechStack,
};
