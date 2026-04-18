use chrono::{TimeZone, Utc};
use mc_core::{
    format_duration, generate_id, generate_trace_id, now_utc, AgentExecutionReport,
    AgentExecutionStatus, AgentStatus, AgentType, ArchitectureLayer, ArchitecturePattern,
    ChangeType, CodeConventions, Color, CommitPoint, Complexity, ContextAllocation,
    CoordinatorPhase, DarkTheme, DependencyEdge, DependencyGraph, DependencyType,
    DocumentationConvention, ErrorHandlingPattern, ExecutionPlan, FinishReason, IssueSeverity,
    McError, MemoryCategory, MessageRole, ModelInfo, ModuleInfo, NamedColor, NamingConvention,
    ParallelGroup, PlanMetadata, ProjectContext, ProjectInfo, ProjectStructure, ResultType,
    RiskArea, RiskAreas, RiskCategory, RiskLevel, ScanMetadata, SemanticColor, SubTask,
    TaskDependency, TaskDescription, TaskIntent, TaskResult, TechStack, TestingConvention, Theme,
    TokenUsage, ToolCallStatus, ToolDefinition,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::error::Error as _;

fn fixed_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 4, 18, 10, 20, 30)
        .single()
        .unwrap()
}

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_string(value).unwrap();
    let decoded: T = serde_json::from_str(&encoded).unwrap();
    assert_eq!(*value, decoded);
}

fn sample_sub_task() -> SubTask {
    SubTask {
        id: "subtask-1".to_string(),
        description: "update core types".to_string(),
        target_files: vec!["core/src/task.rs".to_string()],
        expected_output: "new structs and enums".to_string(),
        token_budget: 8_000,
        priority: 0,
        estimated_complexity: Complexity::Complex,
        acceptance_criteria: vec!["all tests pass".to_string()],
        completed: false,
        assigned_agent: AgentType::Coder,
    }
}

fn sample_project_context() -> ProjectContext {
    ProjectContext {
        project_info: ProjectInfo {
            name: "MoreCode".to_string(),
            description: "Multi-agent coding assistant".to_string(),
            version: Some("0.1.0".to_string()),
            language: "Rust".to_string(),
            framework: Some("Tokio".to_string()),
            license: Some("MIT".to_string()),
            repository_url: Some("https://example.com/morecode".to_string()),
        },
        structure: ProjectStructure {
            directory_tree: ".\n  core/".to_string(),
            total_files: 42,
            total_lines: 2_048,
            entry_files: vec!["src/main.rs".to_string()],
            config_files: vec!["Cargo.toml".to_string()],
            modules: vec![ModuleInfo {
                path: "core".to_string(),
                name: "core".to_string(),
                description: "shared types".to_string(),
                exports: vec!["AgentType".to_string()],
                dependencies: vec!["serde".to_string()],
                file_count: 10,
                line_count: 900,
            }],
        },
        tech_stack: TechStack {
            language_version: "Rust 1.88+".to_string(),
            rust_edition: Some("2021".to_string()),
            framework: Some("Tokio".to_string()),
            database: None,
            orm: None,
            auth: None,
            build_tool: Some("cargo".to_string()),
            package_manager: Some("cargo".to_string()),
            dependencies: HashMap::from([("serde".to_string(), "1.0".to_string())]),
            dev_dependencies: HashMap::from([("tempfile".to_string(), "3".to_string())]),
            updated_at: fixed_time(),
        },
        architecture: ArchitecturePattern {
            name: "Workspace".to_string(),
            description: "Cargo workspace with layered crates".to_string(),
            layers: vec![ArchitectureLayer {
                name: "core".to_string(),
                responsibility: "shared type definitions".to_string(),
                modules: vec!["core".to_string()],
                depends_on: vec![],
            }],
            design_decisions: vec!["keep core lightweight".to_string()],
        },
        dependency_graph: DependencyGraph {
            nodes: vec!["core".to_string(), "communication".to_string()],
            edges: vec![DependencyEdge {
                from: "communication".to_string(),
                to: "core".to_string(),
                relation_type: "depends_on".to_string(),
                weight: Some(1.0),
            }],
            circular_dependencies: vec![],
        },
        conventions: CodeConventions {
            naming: NamingConvention {
                function: "snake_case".to_string(),
                struct_: "PascalCase".to_string(),
                constant: "SCREAMING_SNAKE_CASE".to_string(),
                module: "snake_case".to_string(),
                enum_variant: "PascalCase".to_string(),
                type_parameter: "UpperCamelCase".to_string(),
            },
            error_handling: ErrorHandlingPattern {
                custom_error_type: true,
                error_type_name: Some("McError".to_string()),
                unwrap_policy: "allowed_in_tests".to_string(),
                propagation: "question_mark".to_string(),
                error_crate: Some("thiserror".to_string()),
            },
            testing: TestingConvention {
                naming_pattern: "describe_behavior".to_string(),
                test_attribute: "#[test]".to_string(),
                test_file_location: "tests/".to_string(),
                mock_framework: None,
                coverage_threshold: Some(0.8),
            },
            documentation: DocumentationConvention {
                language: "zh-CN".to_string(),
                format: "Markdown".to_string(),
                code_example_style: "Rust".to_string(),
                api_doc_tool: Some("rustdoc".to_string()),
            },
            custom_rules: vec!["no unwrap outside tests".to_string()],
        },
        risk_areas: RiskAreas {
            items: vec![RiskArea {
                file: "core/src/error.rs".to_string(),
                description: "cross-crate error coupling".to_string(),
                level: RiskLevel::High,
                category: RiskCategory::Maintainability,
                historical_issues: vec!["missing source chain".to_string()],
                updated_at: fixed_time(),
            }],
        },
        scan_metadata: ScanMetadata {
            scanned_at: fixed_time(),
            files_scanned: 42,
            total_lines: 2_048,
            scan_duration_ms: 1_500,
            memory_version: 3,
            scanner_version: "1.0.0".to_string(),
            scan_mode: "full".to_string(),
            changed_files: Vec::new(),
        },
        root_path: "C:/repo/MoreCode".to_string(),
    }
}

fn sample_execution_plan() -> ExecutionPlan {
    let sub_task = sample_sub_task();
    ExecutionPlan {
        plan_id: "plan-1".to_string(),
        task_description: "Implement core".to_string(),
        summary: "Implement core task structures and validation".to_string(),
        parallel_groups: vec![ParallelGroup {
            id: "group-1".to_string(),
            name: "core".to_string(),
            sub_tasks: vec![sub_task.clone()],
            can_parallel: false,
            depends_on: vec![],
            agent_type: AgentType::Coder,
        }],
        group_dependencies: HashMap::new(),
        sub_tasks: vec![sub_task.clone()],
        dependencies: vec![TaskDependency {
            upstream_task_id: "scan".to_string(),
            downstream_task_id: "code".to_string(),
            dependency_type: DependencyType::Strong,
            description: "planning depends on scan".to_string(),
        }],
        commit_points: vec![CommitPoint {
            id: "cp-1".to_string(),
            name: "core-ready".to_string(),
            waiting_tasks: vec!["code".to_string()],
            target_branch: "main".to_string(),
            completed: false,
            merged_at: None,
        }],
        context_allocations: vec![ContextAllocation {
            sub_task_id: sub_task.id.clone(),
            agent_type: AgentType::Coder,
            token_budget: 8_000,
            required_files: vec!["core/src/task.rs".to_string()],
            project_knowledge_subset: vec!["core types".to_string()],
            context_window_limit: 16_000,
        }],
        total_estimated_tokens: 8_000,
        total_estimated_duration_ms: 15_000,
        plan_metadata: PlanMetadata {
            generated_by: AgentType::Planner,
            generated_at: fixed_time(),
            model_used: "gpt-5.4".to_string(),
            generation_duration_ms: 400,
            tokens_used: 640,
            version: 1,
        },
        created_at: fixed_time(),
    }
}

#[test]
fn enum_serde_roundtrip_works() {
    for value in AgentType::ALL {
        assert_roundtrip(&value);
    }

    for value in [
        AgentExecutionStatus::Pending,
        AgentExecutionStatus::Running,
        AgentExecutionStatus::Completed,
        AgentExecutionStatus::Failed,
        AgentExecutionStatus::Cancelled,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        ToolCallStatus::Started,
        ToolCallStatus::Completed,
        ToolCallStatus::Failed,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        Complexity::Simple,
        Complexity::Medium,
        Complexity::Complex,
        Complexity::Research,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        CoordinatorPhase::IntentRecognition,
        CoordinatorPhase::ComplexityAssessment,
        CoordinatorPhase::AgentSelection,
        CoordinatorPhase::MemoryAwareRouting,
        CoordinatorPhase::ResourceAllocation,
        CoordinatorPhase::TaskDispatch,
        CoordinatorPhase::Monitoring,
        CoordinatorPhase::ResultAggregation,
        CoordinatorPhase::Delivery,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        ChangeType::AddFile,
        ChangeType::ModifyFile,
        ChangeType::DeleteFile,
        ChangeType::AddDependency,
        ChangeType::ModifyConfig,
        ChangeType::DatabaseMigration,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        ResultType::CodeChange,
        ResultType::AnalysisReport,
        ResultType::ResearchReport,
        ResultType::TestResult,
        ResultType::ReviewResult,
        ResultType::FixReport,
        ResultType::Documentation,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        IssueSeverity::Blocker,
        IssueSeverity::Critical,
        IssueSeverity::Warning,
        IssueSeverity::Suggestion,
        IssueSeverity::Info,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        RiskCategory::Architecture,
        RiskCategory::Performance,
        RiskCategory::Security,
        RiskCategory::Compatibility,
        RiskCategory::Data,
        RiskCategory::Dependency,
        RiskCategory::Maintainability,
        RiskCategory::Testing,
        RiskCategory::Operations,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        MemoryCategory::UserPreference,
        MemoryCategory::ProjectConvention,
        MemoryCategory::TaskState,
        MemoryCategory::ErrorPattern,
        MemoryCategory::TechnicalKnowledge,
        MemoryCategory::DecisionRecord,
    ] {
        assert_roundtrip(&value);
    }

    for value in [
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ] {
        assert_roundtrip(&value);
    }

    for value in [DependencyType::Strong, DependencyType::Weak] {
        assert_roundtrip(&value);
    }

    for value in MessageRole::ALL {
        assert_roundtrip(&value);
    }

    for value in [
        NamedColor::Black,
        NamedColor::Red,
        NamedColor::Green,
        NamedColor::Yellow,
        NamedColor::Blue,
        NamedColor::Magenta,
        NamedColor::Cyan,
        NamedColor::White,
        NamedColor::DarkGray,
        NamedColor::LightRed,
        NamedColor::LightGreen,
        NamedColor::LightYellow,
        NamedColor::LightBlue,
        NamedColor::LightMagenta,
        NamedColor::LightCyan,
        NamedColor::LightGray,
    ] {
        assert_roundtrip(&value);
    }

    for value in SemanticColor::ALL {
        assert_roundtrip(&value);
    }

    for value in [
        Color::Rgb(1, 2, 3),
        Color::Indexed(99),
        Color::Named(NamedColor::Magenta),
    ] {
        assert_roundtrip(&value);
    }

    assert_roundtrip(&TaskIntent::FeatureAddition);
    assert_roundtrip(&TaskIntent::Other("custom".to_string()));
    assert_roundtrip(&FinishReason::Stop);
    assert_roundtrip(&FinishReason::Unknown("provider-specific".to_string()));
    assert_roundtrip(&McError::InternalError {
        message: "boom".to_string(),
    });
}

#[test]
fn agent_type_display_matches_expected() {
    let expected = [
        (AgentType::Coordinator, "Coordinator"),
        (AgentType::Explorer, "Explorer"),
        (AgentType::ImpactAnalyzer, "ImpactAnalyzer"),
        (AgentType::Planner, "Planner"),
        (AgentType::Coder, "Coder"),
        (AgentType::Reviewer, "Reviewer"),
        (AgentType::Tester, "Tester"),
        (AgentType::Debugger, "Debugger"),
        (AgentType::Research, "Research"),
        (AgentType::DocWriter, "DocWriter"),
    ];

    for (agent, display) in expected {
        assert_eq!(agent.to_string(), display);
    }
}

#[test]
fn generate_id_is_unique_and_compact() {
    let mut ids = HashSet::new();

    for _ in 0..1_000 {
        let id = generate_id();
        assert_eq!(id.len(), 32);
        assert!(ids.insert(id));
    }
}

#[test]
fn generate_trace_id_includes_prefix() {
    let trace_id = generate_trace_id("task");
    assert!(trace_id.starts_with("task-"));
    assert_eq!(trace_id.len(), 37);
}

#[test]
fn format_duration_covers_all_branches() {
    assert_eq!(format_duration(999), "999ms");
    assert_eq!(format_duration(1_500), "1.5s");
    assert_eq!(format_duration(61_000), "1m 1s");
    assert_eq!(format_duration(3_661_000), "1h 1m 1s");
}

#[test]
fn semantic_color_count_and_theme_mapping_are_complete() {
    let theme = DarkTheme;

    let semantic_color_count = SemanticColor::COUNT;
    assert!(
        semantic_color_count >= 50,
        "expected at least 50 semantic colors, found {semantic_color_count}"
    );
    assert_eq!(SemanticColor::COUNT, SemanticColor::ALL.len());

    for semantic in SemanticColor::ALL {
        let color = theme.color(semantic);
        match color {
            Color::Rgb(_, _, _) | Color::Indexed(_) | Color::Named(_) => {}
        }
    }
}

#[test]
fn mc_error_display_and_source_behave_as_expected() {
    let cases = vec![
        (
            McError::ConfigLoadFailed {
                path: "a.toml".to_string(),
                reason: "missing".to_string(),
            },
            "配置加载失败",
        ),
        (
            McError::ConfigParseFailed {
                path: "a.toml".to_string(),
                reason: "invalid".to_string(),
            },
            "配置解析失败",
        ),
        (
            McError::ConfigValidationFailed {
                field: "app.name".to_string(),
                reason: "required".to_string(),
            },
            "配置验证失败",
        ),
        (
            McError::ChannelClosed {
                channel: "state".to_string(),
            },
            "通道已关闭",
        ),
        (
            McError::SendTimeout {
                channel: "control".to_string(),
                timeout_ms: 30_000,
            },
            "发送超时",
        ),
        (
            McError::BroadcastLagged {
                subscriber: "ui".to_string(),
                skipped: 3,
            },
            "广播订阅者落后",
        ),
        (
            McError::AgentNotRegistered {
                agent_type: "Planner".to_string(),
            },
            "Agent 未注册",
        ),
        (
            McError::AgentTimeout {
                agent_type: "Coder".to_string(),
                timeout_secs: 120,
            },
            "Agent 执行超时",
        ),
        (
            McError::AgentExecutionFailed {
                agent_type: "Tester".to_string(),
                reason: "panic".to_string(),
            },
            "Agent 执行失败",
        ),
        (
            McError::TaskNotFound {
                task_id: "task-1".to_string(),
            },
            "任务未找到",
        ),
        (
            McError::TokenBudgetExceeded {
                used: 10,
                budget: 5,
            },
            "Token 预算超限",
        ),
        (
            McError::FileOperationFailed {
                path: "foo.rs".to_string(),
                reason: "denied".to_string(),
            },
            "文件操作失败",
        ),
        (
            McError::SerializationFailed {
                reason: "bad json".to_string(),
            },
            "JSON 处理失败",
        ),
        (
            McError::LlmError {
                provider: "openai".to_string(),
                reason: "rate limited".to_string(),
            },
            "LLM 调用失败",
        ),
        (
            McError::InternalError {
                message: "unreachable".to_string(),
            },
            "内部错误",
        ),
    ];

    for (error, expected) in cases {
        assert!(error.to_string().contains(expected));
        assert!(error.source().is_none());
    }
}

#[test]
fn channel_capacity_constants_are_correct() {
    assert_eq!(mc_core::CONTROL_CHANNEL_CAPACITY, 32);
    assert_eq!(mc_core::STATE_CHANNEL_CAPACITY, 64);
    assert_eq!(mc_core::DATA_LINK_CHANNEL_CAPACITY, 128);
    assert_eq!(mc_core::BROADCAST_CHANNEL_CAPACITY, 64);
    assert_eq!(mc_core::APPROVAL_CHANNEL_CAPACITY, 10);
}

#[test]
fn token_usage_default_is_zeroed() {
    let usage = TokenUsage::default();
    assert_eq!(usage.prompt_tokens, 0);
    assert_eq!(usage.completion_tokens, 0);
    assert_eq!(usage.total_tokens, 0);
    assert_eq!(usage.cached_tokens, 0);
    assert_eq!(usage.estimated_cost_usd, 0.0);
    assert_eq!(usage.total(), 0);
}

#[test]
fn serde_json_compatibility_for_core_structs() {
    let task = TaskDescription {
        id: "task-1".to_string(),
        user_input: "implement core".to_string(),
        intent: TaskIntent::FeatureAddition,
        complexity: Complexity::Complex,
        affected_files: vec!["core/src/lib.rs".to_string()],
        requires_new_dependency: false,
        involves_architecture_change: false,
        needs_external_research: false,
        requires_testing: true,
        forced_agents: Some(vec![AgentType::Planner, AgentType::Coder]),
        constraints: vec!["no unwrap".to_string()],
        details: Some("Verify serde compatibility for core task models".to_string()),
        project_root: Some("C:/repo/MoreCode".to_string()),
        created_at: fixed_time(),
    };

    let result = TaskResult {
        result_type: ResultType::CodeChange,
        success: true,
        data: json!({ "files": 3 }),
        changed_files: vec!["core/src/task.rs".to_string()],
        generated_content: Some("pub struct Example;".to_string()),
        error_message: None,
    };

    let report = AgentExecutionReport {
        title: "Coder handoff".to_string(),
        key_findings: vec!["task models completed".to_string()],
        relevant_files: vec!["core/src/task.rs".to_string()],
        recommendations: vec!["run cargo test".to_string()],
        warnings: vec!["workspace still in flux".to_string()],
        token_used: 2_000,
        timestamp: fixed_time(),
        extra: Some(json!({ "phase": "coding" })),
    };

    let agent_status = AgentStatus {
        agent_type: AgentType::Coder,
        status: AgentExecutionStatus::Running,
        task_id: "task-1".to_string(),
        token_used: 512,
        started_at: fixed_time(),
        recursion_depth: 0,
    };

    let model = ModelInfo {
        model_id: "gpt-5.4".to_string(),
        display_name: "GPT-5.4".to_string(),
        provider_name: "OpenAI".to_string(),
        max_context_tokens: 128_000,
        max_output_tokens: 16_384,
        input_price_per_million: 5.0,
        output_price_per_million: 15.0,
        supports_streaming: true,
        supports_function_calling: true,
        supports_json_mode: true,
        supports_prompt_caching: true,
    };

    let tool = ToolDefinition {
        name: "search".to_string(),
        description: "Search files".to_string(),
        parameters: json!({
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"]
        }),
        required: false,
    };

    assert_roundtrip(&task);
    assert_roundtrip(&sample_project_context());
    assert_roundtrip(&sample_sub_task());
    assert_roundtrip(&sample_execution_plan());
    assert_roundtrip(&result);
    assert_roundtrip(&report);
    assert_roundtrip(&agent_status);
    assert_roundtrip(&TokenUsage {
        prompt_tokens: 100,
        completion_tokens: 20,
        total_tokens: 120,
        cached_tokens: 10,
        estimated_cost_usd: 0.12,
    });
    assert_roundtrip(&model);
    assert_roundtrip(&tool);

    let theme_json = serde_json::to_string(&DarkTheme).unwrap();
    let _: DarkTheme = serde_json::from_str(&theme_json).unwrap();
}

#[test]
fn now_utc_returns_recent_timestamp() {
    let before = Utc::now();
    let now = now_utc();
    let after = Utc::now();
    assert!(now >= before);
    assert!(now <= after);
}
