use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use mc_llm::{estimate_text_tokens, MessageRole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PromptLayer {
    Global,
    Organization,
    Project,
    Session,
    Turn,
}

impl PromptLayer {
    pub fn all() -> [PromptLayer; 5] {
        [
            PromptLayer::Global,
            PromptLayer::Organization,
            PromptLayer::Project,
            PromptLayer::Session,
            PromptLayer::Turn,
        ]
    }

    pub fn depth(self) -> u8 {
        match self {
            PromptLayer::Global => 0,
            PromptLayer::Organization => 1,
            PromptLayer::Project => 2,
            PromptLayer::Session => 3,
            PromptLayer::Turn => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PromptLayer::Global => "global",
            PromptLayer::Organization => "organization",
            PromptLayer::Project => "project",
            PromptLayer::Session => "session",
            PromptLayer::Turn => "turn",
        }
    }

    pub fn default_ttl_secs(self) -> Option<u64> {
        match self {
            PromptLayer::Global => Some(3600),
            PromptLayer::Organization => Some(1800),
            PromptLayer::Project => Some(600),
            PromptLayer::Session | PromptLayer::Turn => None,
        }
    }

    pub fn should_cache(self) -> bool {
        matches!(
            self,
            PromptLayer::Global | PromptLayer::Organization | PromptLayer::Project
        )
    }

    pub fn breakpoint_name(self) -> Option<&'static str> {
        match self {
            PromptLayer::Global => Some("after_global"),
            PromptLayer::Organization => Some("after_organization"),
            PromptLayer::Project => Some("after_project"),
            PromptLayer::Session | PromptLayer::Turn => None,
        }
    }
}

impl std::fmt::Display for PromptLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptLayerContent {
    layer: PromptLayer,
    system_prompt: String,
    variables: HashMap<String, String>,
    version: u64,
    content_hash: u64,
    updated_at: DateTime<Utc>,
}

impl PromptLayerContent {
    pub fn new(layer: PromptLayer, system_prompt: impl Into<String>) -> Self {
        Self::from_parts(layer, system_prompt.into(), HashMap::new(), 0)
    }

    pub(crate) fn from_parts(
        layer: PromptLayer,
        system_prompt: String,
        variables: HashMap<String, String>,
        version: u64,
    ) -> Self {
        let content_hash = hash_prompt_content(&system_prompt, &variables);
        Self {
            layer,
            system_prompt,
            variables,
            version,
            content_hash,
            updated_at: Utc::now(),
        }
    }

    pub fn layer(&self) -> PromptLayer {
        self.layer
    }

    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn content_hash(&self) -> u64 {
        self.content_hash
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, Default)]
pub struct PromptLayers {
    layers: BTreeMap<PromptLayer, PromptLayerContent>,
    turn_history: Vec<TurnMessage>,
}

impl PromptLayers {
    pub fn new() -> Self {
        let mut layers = BTreeMap::new();
        layers.insert(
            PromptLayer::Global,
            PromptLayerContent::new(PromptLayer::Global, ""),
        );
        Self {
            layers,
            turn_history: Vec::new(),
        }
    }

    pub fn get(&self, layer: PromptLayer) -> Option<&PromptLayerContent> {
        self.layers.get(&layer)
    }

    pub(crate) fn set(&mut self, content: PromptLayerContent) {
        self.layers.insert(content.layer(), content);
    }

    pub fn sorted_layers(&self) -> Vec<&PromptLayerContent> {
        PromptLayer::all()
            .into_iter()
            .filter_map(|layer| self.layers.get(&layer))
            .collect()
    }

    pub fn merge_variables(&self) -> HashMap<String, String> {
        let mut merged = HashMap::new();
        for content in self.sorted_layers() {
            for (key, value) in content.variables() {
                merged.insert(key.clone(), value.clone());
            }
        }
        merged
    }

    pub fn turn_history(&self) -> &[TurnMessage] {
        &self.turn_history
    }

    pub(crate) fn append_turn_message(&mut self, message: TurnMessage) {
        self.turn_history.push(message);
    }

    pub(crate) fn replace_turn_history(&mut self, messages: Vec<TurnMessage>) {
        self.turn_history = messages;
    }
}

impl Default for PromptLayerContent {
    fn default() -> Self {
        Self::new(PromptLayer::Global, "")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: usize,
}

impl TurnMessage {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        let content = content.into();
        Self {
            role,
            token_count: estimate_text_tokens(&content),
            timestamp: Utc::now(),
            content,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

pub(crate) fn hash_prompt_content(system_prompt: &str, variables: &HashMap<String, String>) -> u64 {
    let mut ordered = variables.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.0.cmp(right.0));

    let mut normalized = String::with_capacity(system_prompt.len() + ordered.len() * 16);
    normalized.push_str(system_prompt);
    for (key, value) in ordered {
        normalized.push('\u{0}');
        normalized.push_str(key);
        normalized.push('=');
        normalized.push_str(value);
    }

    seahash::hash(normalized.as_bytes())
}
