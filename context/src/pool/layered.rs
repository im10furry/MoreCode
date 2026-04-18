use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::pool::PlatformInfo;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InjectionStrategy {
    DirectInjection,
    CompressedSummary,
    Structured,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextBudget {
    pub global_tokens: usize,
    pub platform_tokens: usize,
    pub project_tokens: usize,
    pub task_tokens: usize,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            global_tokens: 200_000,
            platform_tokens: 500,
            project_tokens: 8_000,
            task_tokens: 7_000,
        }
    }
}

pub struct ContextPool {
    platform_info: Arc<PlatformInfo>,
    project_knowledge: RwLock<String>,
    task_context: RwLock<String>,
    agent_contexts: RwLock<HashMap<String, String>>,
    pub budget: ContextBudget,
    pub injection_strategy: InjectionStrategy,
}

impl ContextPool {
    pub fn new(platform_info: Arc<PlatformInfo>, global_token_budget: usize) -> Self {
        let budget = ContextBudget {
            global_tokens: global_token_budget,
            ..ContextBudget::default()
        };

        Self {
            platform_info,
            project_knowledge: RwLock::new(String::new()),
            task_context: RwLock::new(String::new()),
            agent_contexts: RwLock::new(HashMap::new()),
            budget,
            injection_strategy: InjectionStrategy::DirectInjection,
        }
    }

    pub fn with_injection_strategy(mut self, injection_strategy: InjectionStrategy) -> Self {
        self.injection_strategy = injection_strategy;
        self
    }

    pub async fn build_context_for_agent(&self, agent_id: &str, agent_type: &str) -> String {
        let platform = truncate_to_budget(
            &self.platform_info.to_context_block(),
            self.budget.platform_tokens,
        );
        let knowledge = truncate_to_budget(
            &self.project_knowledge.read().await,
            self.budget.project_tokens,
        );
        let task = truncate_to_budget(&self.task_context.read().await, self.budget.task_tokens);
        let agent_ctx = self.agent_contexts.read().await.get(agent_id).cloned();

        let mut sections = vec![platform];
        if !knowledge.is_empty() {
            sections.push(format!("## Project Knowledge\n\n{knowledge}"));
        }
        if !task.is_empty() {
            sections.push(format!("## Task Context\n\n{task}"));
        }
        if let Some(agent_ctx) = agent_ctx {
            sections.push(format!("## {agent_type} Specific Context\n\n{agent_ctx}"));
        }

        sections.join("\n\n")
    }

    pub async fn build_recursive_context(&self, task_focus: &str) -> String {
        let platform = truncate_to_budget(
            &self.platform_info.to_context_block(),
            self.budget.platform_tokens,
        );
        let focus = truncate_to_budget(task_focus, self.budget.task_tokens);
        format!("{platform}\n\n## Recursive Task Focus\n\n{focus}")
    }

    pub async fn update_project_knowledge(&self, content: String) {
        *self.project_knowledge.write().await = content;
    }

    pub async fn update_task_context(&self, content: String) {
        *self.task_context.write().await = content;
    }

    pub async fn set_agent_context(&self, agent_id: String, content: String) {
        self.agent_contexts.write().await.insert(agent_id, content);
    }

    pub async fn estimate_tokens(&self) -> usize {
        let platform = self
            .platform_info
            .to_context_block()
            .chars()
            .count()
            .div_ceil(4);
        let project = self
            .project_knowledge
            .read()
            .await
            .chars()
            .count()
            .div_ceil(4);
        let task = self.task_context.read().await.chars().count().div_ceil(4);
        let agent = self
            .agent_contexts
            .read()
            .await
            .values()
            .map(|value| value.chars().count().div_ceil(4))
            .sum::<usize>();

        platform + project + task + agent
    }
}

#[async_trait]
pub trait ContextProvider: Send + Sync {
    async fn build_context(&self, agent_id: &str, agent_type: &str) -> String;
    async fn update_project_knowledge(&self, content: String);
    async fn update_task_context(&self, content: String);
    async fn estimate_tokens(&self) -> usize;
}

#[async_trait]
impl ContextProvider for ContextPool {
    async fn build_context(&self, agent_id: &str, agent_type: &str) -> String {
        self.build_context_for_agent(agent_id, agent_type).await
    }

    async fn update_project_knowledge(&self, content: String) {
        self.update_project_knowledge(content).await;
    }

    async fn update_task_context(&self, content: String) {
        self.update_task_context(content).await;
    }

    async fn estimate_tokens(&self) -> usize {
        self.estimate_tokens().await
    }
}

fn truncate_to_budget(content: &str, budget_tokens: usize) -> String {
    let max_chars = budget_tokens * 4;
    if content.chars().count() <= max_chars {
        content.to_string()
    } else {
        content.chars().take(max_chars).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{ContextPool, InjectionStrategy};
    use crate::pool::PlatformInfo;

    #[tokio::test]
    async fn build_context_stacks_layers_in_order() {
        let pool = ContextPool::new(Arc::new(PlatformInfo::default()), 50_000)
            .with_injection_strategy(InjectionStrategy::DirectInjection);
        pool.update_project_knowledge("Project".into()).await;
        pool.update_task_context("Task".into()).await;
        pool.set_agent_context("coder".into(), "Agent".into()).await;

        let context = pool.build_context_for_agent("coder", "Coder").await;
        let platform = context.find("## Platform Environment").unwrap();
        let project = context.find("## Project Knowledge").unwrap();
        let task = context.find("## Task Context").unwrap();
        let agent = context.find("## Coder Specific Context").unwrap();

        assert!(platform < project && project < task && task < agent);
    }

    #[tokio::test]
    async fn agents_receive_isolated_contexts() {
        let pool = ContextPool::new(Arc::new(PlatformInfo::default()), 50_000);
        pool.set_agent_context("coder".into(), "Coder state".into())
            .await;
        pool.set_agent_context("tester".into(), "Tester state".into())
            .await;

        let coder = pool.build_context_for_agent("coder", "Coder").await;
        let tester = pool.build_context_for_agent("tester", "Tester").await;

        assert!(coder.contains("Coder state"));
        assert!(!coder.contains("Tester state"));
        assert!(tester.contains("Tester state"));
    }

    #[tokio::test]
    async fn recursive_context_only_includes_platform_and_focus() {
        let pool = ContextPool::new(Arc::new(PlatformInfo::default()), 50_000);
        pool.update_project_knowledge("Project".into()).await;
        pool.update_task_context("Task".into()).await;
        pool.set_agent_context("child".into(), "Agent".into()).await;

        let context = pool.build_recursive_context("Only this focus").await;
        assert!(context.contains("## Platform Environment"));
        assert!(context.contains("## Recursive Task Focus"));
        assert!(!context.contains("## Project Knowledge"));
        assert!(!context.contains("## Task Context"));
        assert!(!context.contains("Agent"));
    }

    #[tokio::test]
    async fn token_estimate_includes_all_layers() {
        let pool = ContextPool::new(Arc::new(PlatformInfo::default()), 50_000);
        pool.update_project_knowledge("Project".into()).await;
        pool.update_task_context("Task".into()).await;
        pool.set_agent_context("coder".into(), "Agent".into()).await;
        assert!(pool.estimate_tokens().await > 0);
    }
}
