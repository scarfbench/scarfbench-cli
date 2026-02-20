use bon::Builder;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// This captures the conversion status
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]   // Nice to see
pub enum Status {
    Converted,
    Prepared,
    Validated,
}
/// This captures the conversion status
#[derive(Clone, Debug, Deserialize, Serialize, Display)]
#[serde(rename_all = "snake_case")]   // Nice to see
#[strum(serialize_all = "snake_case")]
pub enum Framework {
    Spring,
    Quarkus,
    Jakarta,
}

/// Captures the expected schema of the metadata JSON file
#[derive(Debug, Clone, Serialize, Builder, Deserialize)]
#[serde(deny_unknown_fields)]
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
}