pub mod compression;
pub mod error;
pub mod pool;
pub mod project;
pub mod session;

pub use compression::{
    AutoCompactOutcome, AutoCompactor, ChatMessage, CompactStats, CompactSummary,
    CompressionCoordinator, CompressionPromptBuilder, CompressionResult, ContextCompressor,
    ExtractorType, L0RetentionPolicy, LlmClient, LlmError, LlmRequest, LlmResponse,
    MemoryCompactResult, MemoryCompactor, MemoryStore, MessageRole, MicroCompactor,
    ReactiveTruncator, ResponseFormat, SimpleTokenCounter, TokenCounter,
};
pub use error::{ContextError, Result};
pub use pool::{
    ContextBudget, ContextPool, ContextProvider, DefaultPlatformDetector, DevToolChain,
    FileEncoding, InjectionStrategy, LineEnding, PackageManager, PathHandler, PathStyle,
    PlatformCache, PlatformDetector, PlatformInfo, ShellConfig, ShellType, TerminalInfo,
};
pub use project::{
    ChangeType, CodeConventions, ErrorHandlingPattern, ImpactChange, ImpactReport,
    NamingConvention, ProjectContext, ProjectInfo, RiskArea, RiskLevel, ScanMetadata, TechStack,
    TestingConvention,
};
pub use session::{RecoveryMode, RollingSummaryPacket, SessionMemoryLayers};
