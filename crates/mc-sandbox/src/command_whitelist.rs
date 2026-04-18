use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::command::ParsedCommand;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRule {
    #[serde(default)]
    pub allow_any_arguments: bool,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub exact_subcommands: HashSet<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prefix_subcommands: Vec<String>,
}

impl CommandRule {
    pub fn any() -> Self {
        Self {
            allow_any_arguments: true,
            exact_subcommands: HashSet::new(),
            prefix_subcommands: Vec::new(),
        }
    }

    pub fn exact(subcommands: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            allow_any_arguments: false,
            exact_subcommands: subcommands.into_iter().map(Into::into).collect(),
            prefix_subcommands: Vec::new(),
        }
    }

    pub fn matches(&self, command: &ParsedCommand) -> bool {
        if self.allow_any_arguments {
            return true;
        }

        let Some(subcommand) = command.args().first() else {
            return false;
        };

        self.exact_subcommands.contains(subcommand)
            || self
                .prefix_subcommands
                .iter()
                .any(|prefix| subcommand.starts_with(prefix))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandWhitelist {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    rules: HashMap<String, CommandRule>,
}

impl CommandWhitelist {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn allow_program(&mut self, executable: impl Into<String>) {
        self.rules.insert(executable.into(), CommandRule::any());
    }

    pub fn allow_exact_subcommand(
        &mut self,
        executable: impl Into<String>,
        subcommand: impl Into<String>,
    ) {
        let entry = self
            .rules
            .entry(executable.into())
            .or_insert_with(|| CommandRule::exact(Vec::<String>::new()));
        entry.allow_any_arguments = false;
        entry.exact_subcommands.insert(subcommand.into());
    }

    pub fn allow_subcommand_prefix(
        &mut self,
        executable: impl Into<String>,
        prefix: impl Into<String>,
    ) {
        let entry = self
            .rules
            .entry(executable.into())
            .or_insert_with(|| CommandRule::exact(Vec::<String>::new()));
        entry.allow_any_arguments = false;
        entry.prefix_subcommands.push(prefix.into());
    }

    pub fn is_safe(&self, command: &ParsedCommand) -> bool {
        self.rules
            .get(&command.executable_name)
            .map(|rule| rule.matches(command))
            .unwrap_or(false)
    }
}

impl Default for CommandWhitelist {
    fn default() -> Self {
        let mut whitelist = Self::new();
        for executable in ["ls", "pwd", "cat", "echo", "rg"] {
            whitelist.allow_program(executable);
        }
        for subcommand in ["status", "log", "diff", "show"] {
            whitelist.allow_exact_subcommand("git", subcommand);
        }
        whitelist
    }
}
