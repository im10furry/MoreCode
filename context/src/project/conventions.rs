use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NamingConvention {
    SnakeCase,
    CamelCase,
    PascalCase,
    KebabCase,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestingConvention {
    UnitFirst,
    IntegrationHeavy,
    SnapshotFriendly,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorHandlingPattern {
    ThisErrorAndAnyhow,
    DomainResults,
    PanicFree,
    Unknown(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeConventions {
    pub naming: Vec<NamingConvention>,
    pub testing: Vec<TestingConvention>,
    pub error_handling: Vec<ErrorHandlingPattern>,
    pub notes: Vec<String>,
}
