use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{bail, Result};
use tokio::{sync::Mutex, task::JoinSet, time::timeout};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use crate::{
    aggregator::ResultAggregator,
    budget_allocator::TokenBudgetAllocator,
    config::RecursiveConfig,
    error::RecursiveError,
    filter::{FilterEngine, FilterStrategy, RegexCache},
    limiter::ResourceLimiter,
    result::AggregatedResult,
    stats::RecursiveStats,
    sub_agent::{AgentFactory, SubAgentResult, SubAgentSpec, SubAgentStatus},
};

/// Async recursive orchestrator that implements the Map-Filter-Reduce workflow.
pub struct RecursiveOrchestrator {
    agent_factory: Arc<dyn AgentFactory>,
    config: RecursiveConfig,
    filter_strategy: FilterStrategy,
    budget_allocator: TokenBudgetAllocator,
    resource_limiter: ResourceLimiter,
    regex_cache: RegexCache,
    stats: Arc<Mutex<RecursiveStats>>,
}

impl RecursiveOrchestrator {
    pub fn new(agent_factory: Arc<dyn AgentFactory>, config: RecursiveConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            agent_factory,
            resource_limiter: ResourceLimiter::new(
                config.max_sub_agents,
                config.max_depth,
                config.max_total,
            ),
            config,
            filter_strategy: FilterStrategy::KeepAll,
            budget_allocator: TokenBudgetAllocator::default(),
            regex_cache: RegexCache::new(),
            stats: Arc::new(Mutex::new(RecursiveStats::default())),
        })
    }

    /// Override the output filter strategy.
    pub fn with_filter_strategy(mut self, strategy: FilterStrategy) -> Self {
        self.filter_strategy = strategy;
        self
    }

    /// Override the recursion depth while enforcing the design ceiling of 3 levels.
    pub fn with_max_depth(mut self, max: usize) -> Result<Self> {
        if max > 3 {
            bail!("递归深度不能超过 3 层，请求值: {}", max);
        }
        self.config.max_depth = max;
        self.resource_limiter.max_depth = max;
        Ok(self)
    }

    /// Access the mutable token allocator for model-specific tuning in callers or tests.
    pub fn budget_allocator_mut(&mut self) -> &mut TokenBudgetAllocator {
        &mut self.budget_allocator
    }

    /// Snapshot the current recursive execution stats.
    pub async fn recursive_stats(&self) -> RecursiveStats {
        self.stats.lock().await.clone()
    }

    /// Execute one Map-Filter-Reduce recursion layer.
    pub async fn execute_recursive(
        &self,
        task: &str,
        sub_agent_specs: Vec<SubAgentSpec>,
        parent_cancel: CancellationToken,
        depth: usize,
    ) -> Result<AggregatedResult> {
        if depth >= self.config.max_depth {
            return Err(RecursiveError::DepthLimitExceeded {
                current_depth: depth,
                max_depth: self.config.max_depth,
            }
            .into());
        }

        if sub_agent_specs.is_empty() {
            return Err(RecursiveError::NoSuccessfulResults.into());
        }

        if sub_agent_specs.len() > self.config.max_sub_agents {
            return Err(RecursiveError::TooManySubAgents {
                requested: sub_agent_specs.len(),
                max_sub_agents: self.config.max_sub_agents,
            }
            .into());
        }

        if !self
            .resource_limiter
            .can_spawn(depth, sub_agent_specs.len())
        {
            return Err(RecursiveError::GlobalAgentLimitExceeded {
                requested: sub_agent_specs.len(),
                available: self
                    .config
                    .max_total
                    .saturating_sub(self.resource_limiter.current_count()),
            }
            .into());
        }

        let prepared_specs = self.allocate_budgets(sub_agent_specs, depth);
        let mut sub_results = self
            .execute_sub_agents(task, prepared_specs, parent_cancel.child_token(), depth)
            .await?;

        if sub_results
            .iter()
            .all(|result| result.status != SubAgentStatus::Completed)
        {
            return Err(RecursiveError::NoSuccessfulResults.into());
        }

        let filter_engine = FilterEngine::new(self.regex_cache.clone());
        let mut filtered_results = Vec::with_capacity(sub_results.len());
        for result in &mut sub_results {
            let filtered = filter_engine
                .apply(&self.filter_strategy, &result.raw_output)
                .await?;
            if !filtered.retained.is_empty() {
                result.filtered_output = Some(filtered.retained.join("\n"));
            }
            filtered_results.push(filtered);
        }

        let aggregator = ResultAggregator::new(self.config.max_aggregate_tokens);
        Ok(aggregator.aggregate(task, &filtered_results, &sub_results, depth))
    }

    fn allocate_budgets(&self, mut specs: Vec<SubAgentSpec>, depth: usize) -> Vec<SubAgentSpec> {
        if specs.is_empty() {
            return specs;
        }

        let requested_child_total = specs
            .iter()
            .map(|spec| (spec.token_budget as u64).max(self.config.min_child_budget))
            .sum::<u64>();
        let estimated_parent_budget = ((requested_child_total as f64)
            / self.budget_allocator.child_budget_ratio())
        .ceil() as u64;
        let budgets = self.budget_allocator.calculate_child_budget(
            estimated_parent_budget,
            specs.len(),
            depth,
        );

        for (spec, budget) in specs.iter_mut().zip(budgets.into_iter()) {
            spec.token_budget = budget as usize;
            if spec.timeout_secs == 0 {
                spec.timeout_secs = self.config.sub_agent_timeout_secs;
            }
        }

        specs
    }

    async fn execute_sub_agents(
        &self,
        _parent_task: &str,
        specs: Vec<SubAgentSpec>,
        parent_cancel: CancellationToken,
        depth: usize,
    ) -> Result<Vec<SubAgentResult>> {
        let _permit = self
            .resource_limiter
            .try_acquire(specs.len())
            .ok_or_else(|| RecursiveError::GlobalAgentLimitExceeded {
                requested: specs.len(),
                available: self
                    .config
                    .max_total
                    .saturating_sub(self.resource_limiter.current_count()),
            })?;

        let mut join_set = JoinSet::new();
        for spec in specs {
            let factory = Arc::clone(&self.agent_factory);
            let child_cancel = parent_cancel.child_token();
            let timeout_duration = Duration::from_secs(spec.timeout_secs);
            join_set.spawn(async move {
                let start = Instant::now();
                let mut pending = SubAgentResult::pending(spec.clone());
                pending.mark_running();
                let spec_for_cancel = spec.clone();
                let spec_for_create = spec.clone();
                let spec_for_create_error = spec.clone();

                let mut agent = tokio::select! {
                    _ = child_cancel.cancelled() => {
                        return SubAgentResult::cancelled(
                            spec_for_cancel,
                            pending.token_usage.clone(),
                            start.elapsed(),
                        );
                    }
                    created = factory.create_agent(spec.agent_type, spec_for_create) => {
                        match created {
                            Ok(agent) => agent,
                            Err(error) => {
                                return SubAgentResult::failed(
                                    spec_for_create_error,
                                    pending.token_usage.clone(),
                                    start.elapsed(),
                                    error.to_string(),
                                );
                            }
                        }
                    }
                };

                let token_usage = |agent: &dyn crate::sub_agent::SubAgentExecutor| agent.token_usage();

                tokio::select! {
                    _ = child_cancel.cancelled() => {
                        SubAgentResult::cancelled(spec.clone(), token_usage(agent.as_ref()), start.elapsed())
                    }
                    timed = timeout(timeout_duration, agent.execute(child_cancel.clone())) => {
                        match timed {
                            Ok(Ok(output)) => SubAgentResult::completed(
                                spec.clone(),
                                output,
                                token_usage(agent.as_ref()),
                                start.elapsed(),
                            ),
                            Ok(Err(error)) => {
                                if child_cancel.is_cancelled() {
                                    SubAgentResult::cancelled(
                                        spec.clone(),
                                        token_usage(agent.as_ref()),
                                        start.elapsed(),
                                    )
                                } else {
                                    SubAgentResult::failed(
                                        spec.clone(),
                                        token_usage(agent.as_ref()),
                                        start.elapsed(),
                                        error.to_string(),
                                    )
                                }
                            }
                            Err(_) => SubAgentResult::timed_out(
                                spec,
                                token_usage(agent.as_ref()),
                                start.elapsed(),
                            ),
                        }
                    }
                }
            });
        }

        let mut results = Vec::new();
        loop {
            tokio::select! {
                _ = parent_cancel.cancelled() => {
                    debug!("parent cancellation received; waiting for child agents to stop cooperatively");
                    let graceful = timeout(Duration::from_millis(50), async {
                        while join_set.join_next().await.is_some() {}
                    }).await;
                    if graceful.is_err() {
                        join_set.abort_all();
                        while join_set.join_next().await.is_some() {}
                    }
                    return Err(RecursiveError::Cancelled.into());
                }
                joined = join_set.join_next() => {
                    match joined {
                        Some(Ok(result)) => {
                            self.record_stats(&result, depth + 1).await;
                            results.push(result);
                        }
                        Some(Err(join_error)) if join_error.is_cancelled() => {
                            warn!("child join task was aborted before returning a result");
                        }
                        Some(Err(join_error)) => {
                            warn!(error = %join_error, "child join task panicked");
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(results)
    }

    async fn record_stats(&self, result: &SubAgentResult, depth: usize) {
        let mut stats = self.stats.lock().await;
        stats.record_agent(
            result.spec.agent_type,
            depth,
            result.token_usage.total_tokens as u64,
            result.duration.as_millis() as u64,
            result.status == SubAgentStatus::Completed,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use mc_core::{agent::AgentType, token::TokenUsage};
    use tokio::time::sleep;
    use tokio_util::sync::CancellationToken;

    use super::RecursiveOrchestrator;
    use crate::{
        config::RecursiveConfig,
        filter::{FilterRule, FilterRulePriority, FilterRuleType, FilterStrategy},
        sub_agent::{AgentFactory, SubAgentExecutor, SubAgentSpec},
    };

    #[derive(Clone)]
    struct MockBehavior {
        output: std::result::Result<String, String>,
        delay: Duration,
        token_usage: TokenUsage,
        cancellations: Arc<AtomicUsize>,
        starts: Arc<AtomicUsize>,
    }

    struct MockFactory {
        behaviors: HashMap<String, MockBehavior>,
    }

    impl MockFactory {
        fn new(behaviors: HashMap<String, MockBehavior>) -> Self {
            Self { behaviors }
        }
    }

    struct MockExecutor {
        behavior: MockBehavior,
    }

    #[async_trait]
    impl AgentFactory for MockFactory {
        async fn create_agent(
            &self,
            _agent_type: AgentType,
            spec: SubAgentSpec,
        ) -> Result<Box<dyn SubAgentExecutor>> {
            let behavior = self
                .behaviors
                .get(&spec.id)
                .cloned()
                .ok_or_else(|| anyhow!("missing mock behavior"))?;
            Ok(Box::new(MockExecutor { behavior }))
        }
    }

    #[async_trait]
    impl SubAgentExecutor for MockExecutor {
        async fn execute(&mut self, cancel: CancellationToken) -> Result<String> {
            self.behavior.starts.fetch_add(1, Ordering::SeqCst);
            tokio::select! {
                _ = cancel.cancelled() => {
                    self.behavior.cancellations.fetch_add(1, Ordering::SeqCst);
                    Err(anyhow!("cancelled"))
                }
                _ = sleep(self.behavior.delay) => {
                    match &self.behavior.output {
                        Ok(output) => Ok(output.clone()),
                        Err(error) => Err(anyhow!(error.clone())),
                    }
                }
            }
        }

        fn token_usage(&self) -> TokenUsage {
            self.behavior.token_usage.clone()
        }
    }

    fn spec(id: &str, timeout_secs: u64) -> SubAgentSpec {
        SubAgentSpec::new(
            id,
            AgentType::Explorer,
            format!("focus {}", id),
            vec![PathBuf::from("src/auth.rs")],
            4_000,
            timeout_secs,
            vec!["read_file".to_string(), "search".to_string()],
        )
    }

    #[tokio::test]
    async fn orchestrator_runs_map_filter_reduce() -> Result<()> {
        let cancellations = Arc::new(AtomicUsize::new(0));
        let starts = Arc::new(AtomicUsize::new(0));
        let behaviors = HashMap::from([
            (
                "a".to_string(),
                MockBehavior {
                    output: Ok("noise\n发现: jwt issue".to_string()),
                    delay: Duration::from_millis(5),
                    token_usage: TokenUsage {
                        total_tokens: 10,
                        ..TokenUsage::default()
                    },
                    cancellations: Arc::clone(&cancellations),
                    starts: Arc::clone(&starts),
                },
            ),
            (
                "b".to_string(),
                MockBehavior {
                    output: Ok("风险: password issue".to_string()),
                    delay: Duration::from_millis(5),
                    token_usage: TokenUsage {
                        total_tokens: 20,
                        ..TokenUsage::default()
                    },
                    cancellations: Arc::clone(&cancellations),
                    starts: Arc::clone(&starts),
                },
            ),
            (
                "c".to_string(),
                MockBehavior {
                    output: Ok("auth is safe".to_string()),
                    delay: Duration::from_millis(5),
                    token_usage: TokenUsage {
                        total_tokens: 30,
                        ..TokenUsage::default()
                    },
                    cancellations,
                    starts,
                },
            ),
        ]);
        let factory = Arc::new(MockFactory::new(behaviors));
        let orchestrator = RecursiveOrchestrator::new(factory, RecursiveConfig::default())?
            .with_filter_strategy(FilterStrategy::RuleBased {
                keep_rules: vec![
                    FilterRule {
                        pattern: "发现".to_string(),
                        rule_type: FilterRuleType::Contains("发现".to_string()),
                        priority: FilterRulePriority::Keep,
                    },
                    FilterRule {
                        pattern: "风险".to_string(),
                        rule_type: FilterRuleType::Contains("风险".to_string()),
                        priority: FilterRulePriority::Keep,
                    },
                    FilterRule {
                        pattern: "safe".to_string(),
                        rule_type: FilterRuleType::Contains("safe".to_string()),
                        priority: FilterRulePriority::Keep,
                    },
                ],
                discard_rules: vec![FilterRule {
                    pattern: "noise".to_string(),
                    rule_type: FilterRuleType::StartsWith("noise".to_string()),
                    priority: FilterRulePriority::Discard,
                }],
            });

        let aggregated = orchestrator
            .execute_recursive(
                "analyze auth",
                vec![spec("a", 10), spec("b", 10), spec("c", 10)],
                CancellationToken::new(),
                0,
            )
            .await?;

        assert_eq!(aggregated.sub_agents_used, 3);
        assert_eq!(aggregated.depth_reached, 1);
        assert_eq!(aggregated.total_tokens_used, 60);
        assert!(aggregated
            .findings
            .iter()
            .any(|finding| finding.contains("jwt issue")));
        assert!(!aggregated.findings.iter().any(|finding| finding == "noise"));
        assert!(!aggregated.contradictions.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn orchestrator_propagates_parent_cancellation() -> Result<()> {
        let cancellations = Arc::new(AtomicUsize::new(0));
        let starts = Arc::new(AtomicUsize::new(0));
        let behavior = MockBehavior {
            output: Ok("never reached".to_string()),
            delay: Duration::from_secs(5),
            token_usage: TokenUsage::default(),
            cancellations: Arc::clone(&cancellations),
            starts: Arc::clone(&starts),
        };
        let factory = Arc::new(MockFactory::new(HashMap::from([
            ("a".to_string(), behavior.clone()),
            ("b".to_string(), behavior),
        ])));
        let orchestrator = RecursiveOrchestrator::new(factory, RecursiveConfig::default())?;

        let cancel = CancellationToken::new();
        let child = cancel.clone();
        let handle = tokio::spawn(async move {
            orchestrator
                .execute_recursive("cancel me", vec![spec("a", 10), spec("b", 10)], child, 0)
                .await
        });

        for _ in 0..20 {
            if starts.load(Ordering::SeqCst) == 2 {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
        cancel.cancel();

        let result = handle.await.expect("join orchestrator");
        assert!(result.is_err());
        assert_eq!(cancellations.load(Ordering::SeqCst), 2);
        Ok(())
    }

    #[tokio::test]
    async fn with_max_depth_rejects_depth_above_three() -> Result<()> {
        let factory = Arc::new(MockFactory::new(HashMap::new()));
        let orchestrator = RecursiveOrchestrator::new(factory, RecursiveConfig::default())?;
        assert!(orchestrator.with_max_depth(4).is_err());
        Ok(())
    }
}
