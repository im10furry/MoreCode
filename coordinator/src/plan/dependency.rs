use std::collections::{HashMap, VecDeque};

use mc_core::{ParallelGroup, TaskDependency};

use crate::CoordinatorError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDependencyGraph {
    pub topological_layers: Vec<Vec<String>>,
    pub group_dependencies: HashMap<String, Vec<String>>,
}

pub fn analyze_dependencies(
    task_ids: &[String],
    dependencies: &[TaskDependency],
) -> Result<PlanDependencyGraph, CoordinatorError> {
    validate_dependencies(task_ids, dependencies)?;
    let topological_layers = topological_layers(task_ids, dependencies)?;
    Ok(PlanDependencyGraph {
        topological_layers,
        group_dependencies: HashMap::new(),
    })
}

pub fn validate_dependencies(
    task_ids: &[String],
    dependencies: &[TaskDependency],
) -> Result<(), CoordinatorError> {
    let known = task_ids
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>();

    for dependency in dependencies {
        if dependency.upstream_task_id == dependency.downstream_task_id {
            return Err(CoordinatorError::Internal(format!(
                "task '{}' cannot depend on itself",
                dependency.upstream_task_id
            )));
        }
        if !known.contains(&dependency.upstream_task_id) {
            return Err(CoordinatorError::Internal(format!(
                "dependency references unknown upstream task '{}'",
                dependency.upstream_task_id
            )));
        }
        if !known.contains(&dependency.downstream_task_id) {
            return Err(CoordinatorError::Internal(format!(
                "dependency references unknown downstream task '{}'",
                dependency.downstream_task_id
            )));
        }
    }

    let _ = topological_layers(task_ids, dependencies)?;
    Ok(())
}

pub fn topological_layers(
    task_ids: &[String],
    dependencies: &[TaskDependency],
) -> Result<Vec<Vec<String>>, CoordinatorError> {
    let mut indegree = HashMap::<String, usize>::new();
    let mut adjacency = HashMap::<String, Vec<String>>::new();
    for task_id in task_ids {
        indegree.insert(task_id.clone(), 0);
        adjacency.entry(task_id.clone()).or_default();
    }

    for dependency in dependencies {
        adjacency
            .entry(dependency.upstream_task_id.clone())
            .or_default()
            .push(dependency.downstream_task_id.clone());
        *indegree
            .entry(dependency.downstream_task_id.clone())
            .or_default() += 1;
    }

    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(task_id, _)| task_id.clone())
        .collect::<Vec<_>>();
    ready.sort();
    let mut queue = VecDeque::from(ready);
    let mut visited = 0usize;
    let mut layers = Vec::new();

    while !queue.is_empty() {
        let width = queue.len();
        let mut layer = Vec::with_capacity(width);
        let mut newly_ready = Vec::new();
        for _ in 0..width {
            let node = queue
                .pop_front()
                .ok_or_else(|| CoordinatorError::Internal("dependency queue underflow".into()))?;
            visited += 1;
            layer.push(node.clone());

            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    let degree = indegree.get_mut(neighbor).ok_or_else(|| {
                        CoordinatorError::Internal(format!(
                            "dependency graph missing node '{}'",
                            neighbor
                        ))
                    })?;
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        newly_ready.push(neighbor.clone());
                    }
                }
            }
        }

        newly_ready.sort();
        for task_id in newly_ready {
            queue.push_back(task_id);
        }
        layers.push(layer);
    }

    if visited != task_ids.len() {
        return Err(CoordinatorError::Internal(
            "execution plan contains a dependency cycle".into(),
        ));
    }

    Ok(layers)
}

pub fn build_group_dependencies(
    parallel_groups: &[ParallelGroup],
    dependencies: &[TaskDependency],
) -> HashMap<String, Vec<String>> {
    let mut task_to_group = HashMap::<String, String>::new();
    for group in parallel_groups {
        for task in &group.sub_tasks {
            task_to_group.insert(task.id.clone(), group.id.clone());
        }
    }

    let mut graph = HashMap::<String, Vec<String>>::new();
    for dependency in dependencies {
        let Some(from_group) = task_to_group.get(&dependency.upstream_task_id) else {
            continue;
        };
        let Some(to_group) = task_to_group.get(&dependency.downstream_task_id) else {
            continue;
        };
        if from_group == to_group {
            continue;
        }
        graph
            .entry(to_group.clone())
            .or_default()
            .push(from_group.clone());
    }

    for values in graph.values_mut() {
        values.sort();
        values.dedup();
    }

    graph
}

#[cfg(test)]
mod tests {
    use mc_core::{AgentType, DependencyType, ParallelGroup, SubTask, TaskDependency};

    use super::{build_group_dependencies, topological_layers, validate_dependencies};

    fn subtask(id: &str) -> SubTask {
        SubTask {
            id: id.into(),
            description: format!("task-{id}"),
            target_files: vec!["src/lib.rs".into()],
            expected_output: "done".into(),
            token_budget: 1000,
            priority: 0,
            estimated_complexity: mc_core::Complexity::Simple,
            acceptance_criteria: vec!["ok".into()],
            completed: false,
            assigned_agent: AgentType::Coder,
        }
    }

    #[test]
    fn topological_sort_returns_expected_layers() {
        let layers = topological_layers(
            &["T1".into(), "T2".into(), "T3".into()],
            &[
                TaskDependency {
                    upstream_task_id: "T1".into(),
                    downstream_task_id: "T3".into(),
                    dependency_type: DependencyType::Strong,
                    description: "T3 waits for T1".into(),
                },
                TaskDependency {
                    upstream_task_id: "T2".into(),
                    downstream_task_id: "T3".into(),
                    dependency_type: DependencyType::Weak,
                    description: "T3 benefits from T2".into(),
                },
            ],
        )
        .unwrap();

        assert_eq!(
            layers,
            vec![
                vec![String::from("T1"), String::from("T2")],
                vec![String::from("T3")]
            ]
        );
    }

    #[test]
    fn dependency_validation_rejects_cycles() {
        let error = validate_dependencies(
            &["T1".into(), "T2".into()],
            &[
                TaskDependency {
                    upstream_task_id: "T1".into(),
                    downstream_task_id: "T2".into(),
                    dependency_type: DependencyType::Strong,
                    description: "a".into(),
                },
                TaskDependency {
                    upstream_task_id: "T2".into(),
                    downstream_task_id: "T1".into(),
                    dependency_type: DependencyType::Strong,
                    description: "b".into(),
                },
            ],
        )
        .unwrap_err();

        assert!(error.to_string().contains("dependency cycle"));
    }

    #[test]
    fn group_dependency_map_collapses_task_level_edges() {
        let groups = vec![
            ParallelGroup {
                id: "G1".into(),
                name: "layer 1".into(),
                sub_tasks: vec![subtask("T1"), subtask("T2")],
                can_parallel: true,
                depends_on: Vec::new(),
                agent_type: AgentType::Coder,
            },
            ParallelGroup {
                id: "G2".into(),
                name: "layer 2".into(),
                sub_tasks: vec![subtask("T3")],
                can_parallel: false,
                depends_on: Vec::new(),
                agent_type: AgentType::Reviewer,
            },
        ];

        let deps = build_group_dependencies(
            &groups,
            &[TaskDependency {
                upstream_task_id: "T1".into(),
                downstream_task_id: "T3".into(),
                dependency_type: DependencyType::Strong,
                description: "review after coding".into(),
            }],
        );

        assert_eq!(
            deps.get("G2").cloned().unwrap_or_default(),
            vec!["G1".to_string()]
        );
    }
}
