use crate::agent::AgentType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Intent inferred from the user task description.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaskIntent {
    /// Request adds new product or code functionality.
    FeatureAddition,
    /// Request fixes an existing bug.
    BugFix,
    /// Request restructures code without changing intended behavior.
    Refactoring,
    /// Request improves runtime or resource performance.
    PerformanceOptimization,
    /// Request updates or creates documentation.
    Documentation,
    /// Request performs research or technical comparison.
    Research,
    /// Request initializes a new project or scaffold.
    ProjectInit,
    /// Request does not fit the predefined intent set.
    Other(String),
}

/// Estimated task complexity used for routing and planning.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Complexity {
    /// Simple work that a single agent can usually finish.
    Simple,
    /// Medium work that needs planning across multiple files.
    Medium,
    /// Complex work that likely spans architecture boundaries.
    Complex,
    /// Research-oriented work that depends on external information.
    Research,
}

/// Coordinator pipeline phase.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum CoordinatorPhase {
    /// Phase 1: identify user intent.
    IntentRecognition,
    /// Phase 2: assess complexity.
    ComplexityAssessment,
    /// Phase 3: select suitable agents.
    AgentSelection,
    /// Phase 4: apply memory-aware routing.
    MemoryAwareRouting,
    /// Phase 5: allocate resources and budgets.
    ResourceAllocation,
    /// Phase 6: dispatch executable tasks.
    TaskDispatch,
    /// Phase 7: monitor task execution.
    Monitoring,
    /// Phase 8: aggregate intermediate results.
    ResultAggregation,
    /// Phase 9: deliver the final output.
    Delivery,
}

/// File or dependency change type observed in analysis results.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Add a new file.
    AddFile,
    /// Modify an existing file.
    ModifyFile,
    /// Delete an existing file.
    DeleteFile,
    /// Add a new dependency.
    AddDependency,
    /// Modify configuration.
    ModifyConfig,
    /// Apply a database migration.
    DatabaseMigration,
}

/// Output category produced by an agent task.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResultType {
    /// Output is a code change.
    CodeChange,
    /// Output is an analysis report.
    AnalysisReport,
    /// Output is a research report.
    ResearchReport,
    /// Output is a test result.
    TestResult,
    /// Output is a review result.
    ReviewResult,
    /// Output is a fix report.
    FixReport,
    /// Output is documentation content.
    Documentation,
}

/// Severity level for findings and review issues.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Blocking issue that must be fixed first.
    Blocker,
    /// Critical issue with high urgency.
    Critical,
    /// Warning-level issue that should be addressed.
    Warning,
    /// Suggestion-level issue that may improve quality.
    Suggestion,
    /// Informational note only.
    Info,
}

/// Risk category used by project scanning and planning.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum RiskCategory {
    /// Architecture or design mismatch risk.
    Architecture,
    /// Runtime or resource performance risk.
    Performance,
    /// Security vulnerability or exposure risk.
    Security,
    /// Compatibility or versioning risk.
    Compatibility,
    /// Data corruption or loss risk.
    Data,
    /// External dependency or supply-chain risk.
    Dependency,
    /// Maintainability and readability risk.
    Maintainability,
    /// Test coverage or stability risk.
    Testing,
    /// Operational deployment or observability risk.
    Operations,
}

/// Memory bucket category for future memory systems.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemoryCategory {
    /// User preferences and interaction habits.
    UserPreference,
    /// Project conventions and architectural rules.
    ProjectConvention,
    /// Current task progress and execution state.
    TaskState,
    /// Historical error patterns and solutions.
    ErrorPattern,
    /// Technical knowledge and best practices.
    TechnicalKnowledge,
    /// Recorded design and implementation decisions.
    DecisionRecord,
}

/// Risk level assigned to a file or module.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Safe-to-change low risk area.
    Low,
    /// Area needs caution during modification.
    Medium,
    /// Area requires careful analysis and testing.
    High,
    /// Area is critical and changes may break the system.
    Critical,
}

impl RiskLevel {
    pub const fn score(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    pub fn max(left: Self, right: Self) -> Self {
        if left.score() >= right.score() {
            left
        } else {
            right
        }
    }
}

/// Dependency strength between two subtasks.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Downstream task must wait for the upstream task.
    Strong,
    /// Downstream task can proceed but benefits from the upstream output.
    Weak,
}

/// Structured task description used by the coordinator pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDescription {
    /// Task identifier generated as UUID v4 text.
    pub id: String,
    /// Raw user input that originated the task.
    pub user_input: String,
    /// Inferred task intent.
    pub intent: TaskIntent,
    /// Estimated task complexity.
    pub complexity: Complexity,
    /// Files expected to be affected.
    pub affected_files: Vec<String>,
    /// Whether the task needs adding a dependency.
    pub requires_new_dependency: bool,
    /// Whether the task includes architecture-level change.
    pub involves_architecture_change: bool,
    /// Whether the task needs external research.
    pub needs_external_research: bool,
    /// Whether the task requires verification through tests.
    pub requires_testing: bool,
    /// User-forced agent choices, if any.
    pub forced_agents: Option<Vec<AgentType>>,
    /// Additional task constraints.
    pub constraints: Vec<String>,
    /// Optional extra task details.
    pub details: Option<String>,
    /// Optional project root path for file-system aware agents.
    pub project_root: Option<String>,
    /// Creation timestamp in UTC.
    pub created_at: DateTime<Utc>,
}

impl TaskDescription {
    pub fn simple(user_input: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_input: user_input.into(),
            intent: TaskIntent::Other("unspecified".to_string()),
            complexity: Complexity::Simple,
            affected_files: Vec::new(),
            requires_new_dependency: false,
            involves_architecture_change: false,
            needs_external_research: false,
            requires_testing: false,
            forced_agents: None,
            constraints: Vec::new(),
            details: None,
            project_root: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_root(
        user_input: impl Into<String>,
        project_root: impl AsRef<std::path::Path>,
    ) -> Self {
        let mut task = Self::simple(user_input);
        task.project_root = Some(project_root.as_ref().to_string_lossy().into_owned());
        task
    }

    pub fn project_root_path(&self) -> Option<std::path::PathBuf> {
        self.project_root.as_ref().map(std::path::PathBuf::from)
    }
}

/// Explorer-produced structured project context snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectContext {
    /// High-level project metadata.
    pub project_info: ProjectInfo,
    /// Project filesystem and module structure.
    pub structure: ProjectStructure,
    /// Technology stack details.
    pub tech_stack: TechStack,
    /// Identified architecture pattern.
    pub architecture: ArchitecturePattern,
    /// Dependency graph across modules or files.
    pub dependency_graph: DependencyGraph,
    /// Code style and engineering conventions.
    pub conventions: CodeConventions,
    /// Risk areas identified during scanning.
    pub risk_areas: RiskAreas,
    /// Scan execution metadata.
    pub scan_metadata: ScanMetadata,
    /// Root path of the scanned project.
    pub root_path: String,
}

/// Basic project information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name.
    pub name: String,
    /// Project description.
    pub description: String,
    /// Optional project version.
    pub version: Option<String>,
    /// Primary programming language.
    pub language: String,
    /// Primary framework, if any.
    pub framework: Option<String>,
    /// Project license identifier.
    pub license: Option<String>,
    /// Repository URL, if known.
    pub repository_url: Option<String>,
}

/// Filesystem and module-level structure summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectStructure {
    /// Human-readable directory tree.
    pub directory_tree: String,
    /// Total file count.
    pub total_files: usize,
    /// Total line count.
    pub total_lines: usize,
    /// Entry-point files.
    pub entry_files: Vec<String>,
    /// Configuration files.
    pub config_files: Vec<String>,
    /// Major module summaries.
    pub modules: Vec<ModuleInfo>,
}

/// Summary of an individual project module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Module path.
    pub path: String,
    /// Module name.
    pub name: String,
    /// Responsibility description.
    pub description: String,
    /// Exported public interfaces.
    pub exports: Vec<String>,
    /// Dependent module names or paths.
    pub dependencies: Vec<String>,
    /// Number of files in the module.
    pub file_count: usize,
    /// Number of code lines in the module.
    pub line_count: usize,
}

/// Structured technology stack information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechStack {
    /// Primary language version string.
    pub language_version: String,
    /// Rust edition when applicable.
    pub rust_edition: Option<String>,
    /// Primary framework, if any.
    pub framework: Option<String>,
    /// Database technology, if any.
    pub database: Option<String>,
    /// ORM technology, if any.
    pub orm: Option<String>,
    /// Authentication mechanism, if any.
    pub auth: Option<String>,
    /// Build tool, if any.
    pub build_tool: Option<String>,
    /// Package manager, if any.
    pub package_manager: Option<String>,
    /// Production dependencies keyed by package name.
    pub dependencies: HashMap<String, String>,
    /// Development dependencies keyed by package name.
    pub dev_dependencies: HashMap<String, String>,
    /// Last update timestamp for the stack snapshot.
    pub updated_at: DateTime<Utc>,
}

/// Identified high-level architecture pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitecturePattern {
    /// Pattern name such as MVC or monolith.
    pub name: String,
    /// Pattern description.
    pub description: String,
    /// Architecture layers.
    pub layers: Vec<ArchitectureLayer>,
    /// Key design decisions associated with the architecture.
    pub design_decisions: Vec<String>,
}

/// Single architecture layer description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureLayer {
    /// Layer name.
    pub name: String,
    /// Layer responsibility.
    pub responsibility: String,
    /// Modules within the layer.
    pub modules: Vec<String>,
    /// Other layers this layer depends on.
    pub depends_on: Vec<String>,
}

/// Dependency graph across the codebase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Graph nodes representing files or modules.
    pub nodes: Vec<String>,
    /// Directed dependency edges between nodes.
    pub edges: Vec<DependencyEdge>,
    /// Circular dependencies discovered during scanning.
    pub circular_dependencies: Vec<Vec<String>>,
}

/// Directed dependency edge in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Source node identifier.
    pub from: String,
    /// Target node identifier.
    pub to: String,
    /// Dependency relation type such as imports or calls.
    pub relation_type: String,
    /// Optional edge weight such as call frequency.
    pub weight: Option<f32>,
}

/// Coding conventions discovered or configured for the project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeConventions {
    /// Naming conventions.
    pub naming: NamingConvention,
    /// Error handling conventions.
    pub error_handling: ErrorHandlingPattern,
    /// Testing conventions.
    pub testing: TestingConvention,
    /// Documentation conventions.
    pub documentation: DocumentationConvention,
    /// Additional custom rules.
    pub custom_rules: Vec<String>,
}

/// Naming style conventions used in the codebase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamingConvention {
    /// Function naming style.
    pub function: String,
    /// Struct naming style.
    pub struct_: String,
    /// Constant naming style.
    pub constant: String,
    /// Module naming style.
    pub module: String,
    /// Enum variant naming style.
    pub enum_variant: String,
    /// Type parameter naming style.
    pub type_parameter: String,
}

/// Error handling conventions used in the codebase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorHandlingPattern {
    /// Whether the codebase uses a custom error type.
    pub custom_error_type: bool,
    /// Name of the custom error type, if any.
    pub error_type_name: Option<String>,
    /// Policy for `unwrap` usage.
    pub unwrap_policy: String,
    /// Preferred propagation mechanism.
    pub propagation: String,
    /// Primary error crate or strategy.
    pub error_crate: Option<String>,
}

/// Testing-related engineering conventions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestingConvention {
    /// Test naming pattern.
    pub naming_pattern: String,
    /// Test attribute style.
    pub test_attribute: String,
    /// Location of test files.
    pub test_file_location: String,
    /// Mocking framework, if any.
    pub mock_framework: Option<String>,
    /// Required coverage threshold, if any.
    pub coverage_threshold: Option<f32>,
}

/// Documentation-related engineering conventions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentationConvention {
    /// Primary documentation language.
    pub language: String,
    /// Documentation format.
    pub format: String,
    /// Style used for code examples.
    pub code_example_style: String,
    /// API documentation generation tool.
    pub api_doc_tool: Option<String>,
}

/// Collection wrapper for project risk areas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskAreas {
    /// Individual risk area entries.
    pub items: Vec<RiskArea>,
}

/// Risk area identified during project scanning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskArea {
    /// File or module path.
    pub file: String,
    /// Human-readable risk description.
    pub description: String,
    /// Risk level.
    pub level: RiskLevel,
    /// Risk category.
    pub category: RiskCategory,
    /// Historical issues associated with this area.
    pub historical_issues: Vec<String>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Metadata about a project scan operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanMetadata {
    /// Scan timestamp.
    pub scanned_at: DateTime<Utc>,
    /// Number of scanned files.
    pub files_scanned: usize,
    /// Total scanned line count.
    pub total_lines: usize,
    /// Scan duration in milliseconds.
    pub scan_duration_ms: u64,
    /// Memory snapshot version.
    pub memory_version: u32,
    /// Scanner implementation version.
    pub scanner_version: String,
    /// Human-readable scan mode.
    pub scan_mode: String,
    /// Changed files detected for incremental scans.
    pub changed_files: Vec<String>,
}

/// Smallest planner-produced execution unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubTask {
    /// Subtask identifier generated as UUID v4 text.
    pub id: String,
    /// Subtask description.
    pub description: String,
    /// Target file list.
    pub target_files: Vec<String>,
    /// Expected output description.
    pub expected_output: String,
    /// Token budget for the subtask.
    pub token_budget: u32,
    /// Priority where `0` is the highest priority.
    pub priority: u8,
    /// Estimated complexity for the subtask.
    pub estimated_complexity: Complexity,
    /// Acceptance criteria.
    pub acceptance_criteria: Vec<String>,
    /// Whether the subtask has completed.
    pub completed: bool,
    /// Agent assigned to execute the subtask.
    pub assigned_agent: AgentType,
}

/// Full planner-produced execution plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Plan identifier generated as UUID v4 text.
    pub plan_id: String,
    /// Associated task description summary.
    pub task_description: String,
    /// Planner-generated human-readable summary.
    pub summary: String,
    /// Parallel execution groups.
    pub parallel_groups: Vec<ParallelGroup>,
    /// Group dependency graph keyed by group identifier.
    pub group_dependencies: HashMap<String, Vec<String>>,
    /// Flattened subtask list.
    pub sub_tasks: Vec<SubTask>,
    /// Inter-subtask dependencies.
    pub dependencies: Vec<TaskDependency>,
    /// Git merge checkpoints.
    pub commit_points: Vec<CommitPoint>,
    /// Context allocation records.
    pub context_allocations: Vec<ContextAllocation>,
    /// Estimated total token consumption.
    pub total_estimated_tokens: usize,
    /// Estimated total duration in milliseconds.
    pub total_estimated_duration_ms: u64,
    /// Planner metadata.
    pub plan_metadata: PlanMetadata,
    /// Plan creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Group of subtasks that can be scheduled as a unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParallelGroup {
    /// Group identifier.
    pub id: String,
    /// Human-readable group name.
    pub name: String,
    /// Subtasks executed serially within the group.
    pub sub_tasks: Vec<SubTask>,
    /// Whether the group can run in parallel with others.
    pub can_parallel: bool,
    /// Dependent group identifiers.
    pub depends_on: Vec<String>,
    /// Agent type allocated to this group.
    pub agent_type: AgentType,
}

/// Explicit dependency between two subtasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDependency {
    /// Upstream task identifier.
    pub upstream_task_id: String,
    /// Downstream task identifier.
    pub downstream_task_id: String,
    /// Dependency strength.
    pub dependency_type: DependencyType,
    /// Human-readable dependency description.
    pub description: String,
}

/// Merge checkpoint used by concurrent Git workflows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitPoint {
    /// Commit point identifier.
    pub id: String,
    /// Commit point name.
    pub name: String,
    /// Tasks that must complete before merge.
    pub waiting_tasks: Vec<String>,
    /// Target branch for merge.
    pub target_branch: String,
    /// Whether the checkpoint has been completed.
    pub completed: bool,
    /// Merge timestamp if already merged.
    pub merged_at: Option<DateTime<Utc>>,
}

/// Context and budget allocation for a subtask.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextAllocation {
    /// Target subtask identifier.
    pub sub_task_id: String,
    /// Assigned agent type.
    pub agent_type: AgentType,
    /// Token budget assigned to the agent.
    pub token_budget: u32,
    /// Files injected into the context window.
    pub required_files: Vec<String>,
    /// Project-knowledge slices injected into the context window.
    pub project_knowledge_subset: Vec<String>,
    /// Context window size limit.
    pub context_window_limit: usize,
}

/// Metadata generated along with an execution plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanMetadata {
    /// Agent type that generated the plan.
    pub generated_by: AgentType,
    /// Timestamp when the plan was generated.
    pub generated_at: DateTime<Utc>,
    /// LLM model used for plan generation.
    pub model_used: String,
    /// Generation duration in milliseconds.
    pub generation_duration_ms: u64,
    /// Tokens consumed during plan generation.
    pub tokens_used: u32,
    /// Monotonic plan version.
    pub version: u32,
}
