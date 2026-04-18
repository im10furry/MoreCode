use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub package_managers: Vec<String>,
    pub databases: Vec<String>,
}

impl TechStack {
    pub fn primary_language(&self) -> Option<&str> {
        self.languages.first().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::TechStack;

    #[test]
    fn primary_language_uses_first_language() {
        let stack = TechStack {
            languages: vec!["Rust".into(), "TypeScript".into()],
            frameworks: vec![],
            package_managers: vec![],
            databases: vec![],
        };

        assert_eq!(stack.primary_language(), Some("Rust"));
    }
}
