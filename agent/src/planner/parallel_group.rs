use std::collections::{HashMap, VecDeque};

use mc_core::TaskDependency;

pub(crate) fn topological_layers(task_ids: &[String], deps: &[TaskDependency]) -> Vec<Vec<String>> {
    let mut indegree = HashMap::<String, usize>::new();
    let mut adjacency = HashMap::<String, Vec<String>>::new();
    for task_id in task_ids {
        indegree.insert(task_id.clone(), 0);
        adjacency.entry(task_id.clone()).or_default();
    }
    for dep in deps {
        adjacency
            .entry(dep.upstream_task_id.clone())
            .or_default()
            .push(dep.downstream_task_id.clone());
        *indegree.entry(dep.downstream_task_id.clone()).or_default() += 1;
    }

    let mut queue = VecDeque::from(
        indegree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(task_id, _)| task_id.clone())
            .collect::<Vec<_>>(),
    );
    let mut layers = Vec::new();
    while !queue.is_empty() {
        let width = queue.len();
        let mut layer = Vec::new();
        for _ in 0..width {
            let node = queue.pop_front().expect("queue");
            layer.push(node.clone());
            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    if let Some(entry) = indegree.get_mut(neighbor) {
                        *entry -= 1;
                        if *entry == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        layers.push(layer);
    }
    layers
}
