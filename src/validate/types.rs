use bon::Builder;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// This captures the conversion status
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // Nice to see
pub enum Status {
    Converted,
    Completed,
    Prepared,
    Validated,
    Failed,
}
/// This captures the conversion status
#[derive(Clone, Debug, Deserialize, Serialize, Display)]
#[serde(rename_all = "snake_case")] // Nice to see
#[strum(serialize_all = "snake_case")]
pub enum Framework {
    Spring,
    Quarkus,
    Jakarta,
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ValidationOutcome {
    True,
    False,
    #[default]
    Unk,
}
/// Captures the expected schema of the metadata JSON file
#[derive(Debug, Clone, Serialize, Builder, Deserialize)]
pub struct Metadata {
    /// The status of the runs
    pub status: Status,
    /// Name of the agent
    pub agent: String,
    /// Name of the application
    pub app: String,
    /// Application layer
    pub layer: String,
    /// The repeat of pass at k value
    pub repeat: usize,
    /// The source framework
    pub source_framework: Framework,
    /// The target framework
    pub target_framework: Framework,
    /// Number of known test cases (skip if not present while reading)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_smoke_tests: Option<u32>,
    /// Compile status
    #[serde(default)]
    pub compile_ok: ValidationOutcome,
    /// Deploy status
    #[serde(default)]
    pub deploy_ok: ValidationOutcome,
    /// Absolute number of tests that passed (None if unknown/not run)
    #[serde(default)]
    pub tests_passed: Option<u32>,
    /// Failure reason (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    /// Failure category for classification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_category: Option<FailureCategory>,
    /// Whether this run needs to be rerun due to inconclusive results
    #[serde(default, skip_serializing_if = "is_false")]
    pub inconclusive: bool,
    /// Proper agent name (e.g. "claude-code"), distinct from folder-derived `agent`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solution_name: Option<String>,
    /// LLM model identifier (e.g. "claude-opus-4-6")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional variant discriminator (e.g. "with-skills", "with-claude-md")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// This struct holds the metadata.json of the tests we have in the smoke dir
#[derive(Debug, Clone, Serialize, Builder, Deserialize)]
pub struct SmokeTestMetadata {
    pub(crate) num_smoke_tests: u32,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    CompileError,
    BuildConfigError,
    BuildFailure,
    DockerBuildError,
    DockerImageMissing,
    DockerRunError,
    ContainerConflict,
    DeployTimeout,
    DeployError,
    DeployFailure,
    AppStartupFailure,
    BuildOrDeployFailure,
    CompileDependency,
    DeployDependency,
    TestFailure,
    TestFailures,
    TestParseError,
    TestTimeoutOom,
    NoTestOutput,
    ValidationTruncated,
    ProcessTerminated,
    Timeout,
    MissingLog,
    Unknown,
}

/// New types for leaderboard
#[derive(Debug, Serialize, Deserialize)]
pub struct Leaderboard {
    pub solution: LeaderboardSolution,
    pub results: Vec<LeaderboardResults>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardSolution {
    pub agent: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardResults {
    pub from: String,
    pub to: String,
    pub layer: String,
    pub app: String,
    pub repeats: Vec<Repeat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repeat {
    pub compile: bool,
    pub run: bool,
    pub tests_passed: u32,
    pub tests_total: u32,
}
