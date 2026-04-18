use std::{collections::HashMap, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::MemoryError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    UserPreference,
    ProjectConvention,
    TaskState,
    ErrorPattern,
    Context,
    Custom(String),
}

impl MemoryCategory {
    pub fn default_priority(&self) -> u8 {
        match self {
            Self::UserPreference => 2,
            Self::ProjectConvention => 3,
            Self::TaskState => 1,
            Self::ErrorPattern => 4,
            Self::Context => 5,
            Self::Custom(_) => 6,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::UserPreference => "用户偏好",
            Self::ProjectConvention => "项目约定",
            Self::TaskState => "任务状态",
            Self::ErrorPattern => "错误模式",
            Self::Context => "上下文信息",
            Self::Custom(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryBlock {
    pub id: String,
    pub category: MemoryCategory,
    pub content: String,
    pub priority: u8,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_modified_by: String,
    pub access_count: u64,
}

impl MemoryBlock {
    pub fn new(
        id: impl Into<String>,
        category: MemoryCategory,
        content: impl Into<String>,
        last_modified_by: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            priority: category.default_priority(),
            category,
            content: content.into(),
            version: 0,
            created_at: now,
            updated_at: now,
            last_modified_by: last_modified_by.into(),
            access_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreMemory {
    blocks: HashMap<String, MemoryBlock>,
    token_limit: usize,
    current_tokens: usize,
}

pub trait CoreMemoryManager: Send + Sync {
    fn upsert_block(&mut self, block: MemoryBlock) -> Result<(), MemoryError>;
    fn remove_block(&mut self, id: &str) -> Option<MemoryBlock>;
    fn get_by_category(&self, category: &MemoryCategory) -> Vec<&MemoryBlock>;
    fn compact(&mut self, target_tokens: usize) -> Vec<MemoryBlock>;
    fn to_context_string(&self) -> String;
    fn search(&self, query: &str) -> Vec<MemoryBlock>;
    fn token_budget(&self) -> usize;
}

impl CoreMemory {
    pub fn new(token_limit: usize) -> Self {
        Self {
            blocks: HashMap::new(),
            token_limit,
            current_tokens: 0,
        }
    }

    pub fn max_token_budget(&self) -> usize {
        self.token_limit
    }

    pub fn current_tokens(&self) -> usize {
        self.current_tokens
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn get_block(&mut self, id: &str) -> Option<&MemoryBlock> {
        let block = self.blocks.get_mut(id)?;
        block.access_count += 1;
        Some(block)
    }

    pub async fn save_to_path(&self, path: &Path) -> Result<(), MemoryError> {
        let contents = serde_json::to_vec_pretty(self)?;
        fs::write(path, contents).await?;
        Ok(())
    }

    pub async fn load_from_path(path: &Path) -> Result<Self, MemoryError> {
        let contents = fs::read_to_string(path).await?;
        Ok(serde_json::from_str(&contents)?)
    }

    fn sorted_blocks(&self) -> Vec<&MemoryBlock> {
        let mut blocks: Vec<&MemoryBlock> = self.blocks.values().collect();
        blocks.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| right.access_count.cmp(&left.access_count))
                .then_with(|| right.updated_at.cmp(&left.updated_at))
        });
        blocks
    }
}

impl CoreMemoryManager for CoreMemory {
    fn upsert_block(&mut self, mut block: MemoryBlock) -> Result<(), MemoryError> {
        if block.priority == 0 {
            block.priority = block.category.default_priority();
        }

        let new_tokens = estimate_tokens(&block.content);
        let existing = self.blocks.get(&block.id);
        let existing_tokens = existing
            .map(|existing_block| estimate_tokens(&existing_block.content))
            .unwrap_or(0);
        let next_total = self
            .current_tokens
            .saturating_sub(existing_tokens)
            .saturating_add(new_tokens);

        if next_total > self.token_limit {
            return Err(MemoryError::TokenBudgetExceeded {
                required: new_tokens,
                available: self
                    .token_limit
                    .saturating_sub(self.current_tokens.saturating_sub(existing_tokens)),
            });
        }

        let now = Utc::now();
        block.created_at = existing
            .map(|entry| entry.created_at)
            .unwrap_or(block.created_at);
        block.updated_at = now;
        block.version = existing.map(|entry| entry.version + 1).unwrap_or(1);
        block.access_count = existing
            .map(|entry| entry.access_count)
            .unwrap_or(block.access_count);

        self.current_tokens = next_total;
        self.blocks.insert(block.id.clone(), block);
        Ok(())
    }

    fn remove_block(&mut self, id: &str) -> Option<MemoryBlock> {
        let removed = self.blocks.remove(id)?;
        self.current_tokens = self
            .current_tokens
            .saturating_sub(estimate_tokens(&removed.content));
        Some(removed)
    }

    fn get_by_category(&self, category: &MemoryCategory) -> Vec<&MemoryBlock> {
        self.blocks
            .values()
            .filter(|block| &block.category == category)
            .collect()
    }

    fn compact(&mut self, target_tokens: usize) -> Vec<MemoryBlock> {
        if self.current_tokens <= target_tokens {
            return Vec::new();
        }

        let mut candidates: Vec<MemoryBlock> = self.blocks.values().cloned().collect();
        candidates.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.access_count.cmp(&right.access_count))
                .then_with(|| left.updated_at.cmp(&right.updated_at))
        });

        let mut evicted = Vec::new();
        for candidate in candidates {
            if self.current_tokens <= target_tokens {
                break;
            }

            if candidate.priority <= 1 {
                continue;
            }

            if let Some(removed) = self.remove_block(&candidate.id) {
                evicted.push(removed);
            }
        }

        evicted
    }

    fn to_context_string(&self) -> String {
        let mut output = String::from("## Core Memory\n");
        for block in self.sorted_blocks() {
            output.push_str(&format!(
                "\n### [{}] {} (v{}, p{})\n{}\n",
                block.category.display_name(),
                block.id,
                block.version,
                block.priority,
                block.content
            ));
        }
        output
    }

    fn search(&self, query: &str) -> Vec<MemoryBlock> {
        let query = query.to_lowercase();
        self.blocks
            .values()
            .filter(|block| {
                block.id.to_lowercase().contains(&query)
                    || block.content.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    fn token_budget(&self) -> usize {
        self.token_limit.saturating_sub(self.current_tokens)
    }
}

pub fn estimate_tokens(content: &str) -> usize {
    let characters = content.chars().count();
    if characters == 0 {
        0
    } else {
        characters.div_ceil(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_budget_is_enforced() {
        let mut memory = CoreMemory::new(2);
        let block = MemoryBlock::new("task:1", MemoryCategory::TaskState, "1234567890", "tester");
        let result = memory.upsert_block(block);
        assert!(matches!(
            result,
            Err(MemoryError::TokenBudgetExceeded { .. })
        ));
    }
}
