use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use mc_context::ProjectContext;
use mc_core::Complexity;
use serde::{Deserialize, Serialize};

use crate::error::CoordinatorError;
use crate::intent::{TaskType, UserIntent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteThresholds {
    pub simple_max: i32,
    pub medium_max: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityConfig {
    pub llm_complexity_weights: HashMap<Complexity, i32>,
    pub file_count_tiers: Vec<(usize, usize, i32)>,
    pub task_type_weights: HashMap<String, i32>,
    pub needs_context_bonus: i32,
    pub domain_bonus: i32,
    pub project_size_tiers: Vec<(usize, usize, i32)>,
    pub route_thresholds: RouteThresholds,
    pub llm_weight_multiplier: f32,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            llm_complexity_weights: HashMap::from([
                (Complexity::Simple, 5),
                (Complexity::Medium, 15),
                (Complexity::Complex, 30),
                (Complexity::Research, 45),
            ]),
            file_count_tiers: vec![(0, 2, 5), (3, 8, 15), (9, usize::MAX, 30)],
            task_type_weights: HashMap::from([
                (TaskType::FeatureDevelopment.as_key(), 15),
                (TaskType::BugFix.as_key(), 8),
                (TaskType::Refactoring.as_key(), 12),
                (TaskType::Documentation.as_key(), 4),
                (TaskType::Testing.as_key(), 6),
                (TaskType::Configuration.as_key(), 8),
                (TaskType::CodeReview.as_key(), 10),
                (TaskType::Debugging.as_key(), 18),
            ]),
            needs_context_bonus: 10,
            domain_bonus: 5,
            project_size_tiers: vec![(0, 200, 0), (201, 2_000, 5), (2_001, usize::MAX, 10)],
            route_thresholds: RouteThresholds {
                simple_max: 15,
                medium_max: 35,
            },
            llm_weight_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplexityFactors {
    pub file_count: usize,
    pub task_type: String,
    pub needs_context: bool,
    pub domain_count: usize,
    pub project_size: usize,
    pub llm_complexity: Complexity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalibrationRecord {
    pub factors: ComplexityFactors,
    pub evaluated_score: i32,
    pub evaluated_route: RouteLevel,
    pub actual_needed_more_agents: bool,
    pub actual_tokens_used: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RouteLevel {
    Simple,
    Medium,
    Complex,
    Research,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityEvaluation {
    pub route_level: RouteLevel,
    pub score: i32,
    pub factors: ComplexityFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityEvaluator {
    pub config: ComplexityConfig,
    calibration_history: Vec<CalibrationRecord>,
}

impl ComplexityEvaluator {
    pub fn new(config: ComplexityConfig) -> Self {
        Self {
            config,
            calibration_history: Vec::new(),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, CoordinatorError> {
        let content = std::fs::read_to_string(path).map_err(|err| {
            CoordinatorError::ConfigLoadFailed(path.to_path_buf(), err.to_string())
        })?;

        let config = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&content).map_err(|err| {
                CoordinatorError::ConfigParseFailed(path.to_path_buf(), err.to_string())
            })?,
            _ => serde_json::from_str(&content).map_err(|err| {
                CoordinatorError::ConfigParseFailed(path.to_path_buf(), err.to_string())
            })?,
        };

        Ok(Self::new(config))
    }

    pub fn evaluate(
        &self,
        intent: &UserIntent,
        project_ctx: Option<&ProjectContext>,
    ) -> RouteLevel {
        self.evaluate_with_details(intent, project_ctx).route_level
    }

    pub fn evaluate_with_details(
        &self,
        intent: &UserIntent,
        project_ctx: Option<&ProjectContext>,
    ) -> ComplexityEvaluation {
        if intent.needs_research || matches!(intent.estimated_complexity, Complexity::Research) {
            return ComplexityEvaluation {
                route_level: RouteLevel::Research,
                score: *self
                    .config
                    .llm_complexity_weights
                    .get(&Complexity::Research)
                    .unwrap_or(&45),
                factors: ComplexityFactors {
                    file_count: intent.target_files.len(),
                    task_type: intent.task_type.as_key(),
                    needs_context: intent.needs_project_context,
                    domain_count: intent.domains.len(),
                    project_size: project_ctx.map(project_size_hint).unwrap_or(0),
                    llm_complexity: intent.estimated_complexity,
                },
            };
        }

        let mut score = 0;
        if let Some(base) = self
            .config
            .llm_complexity_weights
            .get(&intent.estimated_complexity)
        {
            score += (*base as f32 * self.config.llm_weight_multiplier) as i32;
        }

        let file_count = intent.target_files.len();
        for (min, max, tier_score) in &self.config.file_count_tiers {
            if file_count >= *min && file_count <= *max {
                score += *tier_score;
                break;
            }
        }

        score += self
            .config
            .task_type_weights
            .get(&intent.task_type.as_key())
            .copied()
            .unwrap_or(10);

        if intent.needs_project_context {
            score += self.config.needs_context_bonus;
        }

        if intent.domains.len() > 1 {
            score += (intent.domains.len() as i32 - 1) * self.config.domain_bonus;
        }

        let project_size = project_ctx.map(project_size_hint).unwrap_or(0);
        for (min, max, tier_score) in &self.config.project_size_tiers {
            if project_size >= *min && project_size <= *max {
                score += *tier_score;
                break;
            }
        }

        let route_level = if score <= self.config.route_thresholds.simple_max {
            RouteLevel::Simple
        } else if score <= self.config.route_thresholds.medium_max {
            RouteLevel::Medium
        } else {
            RouteLevel::Complex
        };

        ComplexityEvaluation {
            route_level,
            score,
            factors: ComplexityFactors {
                file_count,
                task_type: intent.task_type.as_key(),
                needs_context: intent.needs_project_context,
                domain_count: intent.domains.len(),
                project_size,
                llm_complexity: intent.estimated_complexity,
            },
        }
    }

    pub fn record_calibration(&mut self, record: CalibrationRecord) {
        self.calibration_history.push(record);
        if self.calibration_history.len() > 500 {
            let drain = self.calibration_history.len() - 500;
            self.calibration_history.drain(..drain);
        }
    }

    pub fn calibrate_from_history(&mut self) -> Option<ComplexityConfig> {
        if self.calibration_history.len() < 20 {
            return None;
        }

        let mut adjusted = self.config.clone();
        let mut type_adjustments: HashMap<String, i32> = HashMap::new();

        for record in &self.calibration_history {
            if record.actual_needed_more_agents {
                let delta = 3i32.min(50 - record.evaluated_score);
                *type_adjustments
                    .entry(record.factors.task_type.clone())
                    .or_insert(0) += delta;
            } else if record.evaluated_score > 30 && record.actual_tokens_used < 5_000 {
                let delta = -3i32.max(-(record.evaluated_score - 10));
                *type_adjustments
                    .entry(record.factors.task_type.clone())
                    .or_insert(0) += delta;
            }
        }

        for (task_type, delta) in type_adjustments {
            let clamped = delta.clamp(-10, 10);
            let entry = adjusted.task_type_weights.entry(task_type).or_insert(10);
            *entry = (*entry + clamped).max(1);
        }

        let mut scores = self
            .calibration_history
            .iter()
            .map(|record| record.evaluated_score)
            .collect::<Vec<_>>();
        scores.sort_unstable();
        adjusted.route_thresholds.simple_max = scores[scores.len() / 3];
        adjusted.route_thresholds.medium_max = scores[scores.len() * 2 / 3];

        self.config = adjusted.clone();
        Some(adjusted)
    }
}

fn project_size_hint(project_ctx: &ProjectContext) -> usize {
    let root_dir = &project_ctx.info.root_dir;
    if root_dir.as_os_str().is_empty() || !root_dir.exists() {
        return project_ctx.notes.len() + project_ctx.risk_areas.len();
    }

    WalkBuilder::new(root_dir)
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
        })
        .take(5_000)
        .count()
}
