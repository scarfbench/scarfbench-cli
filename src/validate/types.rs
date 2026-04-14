use bon::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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

/// Usage data from agent trajectory
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageData {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
}

/// Message from agent trajectory
#[derive(Debug, Clone, Deserialize)]
pub struct TrajectoryMessage {
    #[serde(default)]
    #[allow(dead_code)]
    pub model: Option<String>,
    #[serde(default)]
    pub usage: Option<UsageData>,
}

/// Trajectory entry
#[derive(Debug, Clone, Deserialize)]
pub struct TrajectoryEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(default)]
    pub message: Option<TrajectoryMessage>,
}

/// Model costs loaded from CSV
#[derive(Debug, Clone)]
pub struct ModelCosts {
    costs: HashMap<String, (f64, f64)>, // model -> (input_cost_per_1M, output_cost_per_1M)
}

impl ModelCosts {
    pub fn load_from_csv(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut costs = HashMap::new();
        let content = fs::read_to_string(path)?;
        
        for (i, line) in content.lines().enumerate() {
            if i == 0 {
                continue; // Skip header
            }
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                let model = parts[0].to_string();
                let input_cost: f64 = parts[1].parse()?;
                let output_cost: f64 = parts[2].parse()?;
                costs.insert(model, (input_cost, output_cost));
            }
        }
        
        Ok(Self { costs })
    }

    fn normalize_model_name(model: &str) -> String {
        // Remove provider prefixes (e.g., "aws/")
        let model = if let Some(idx) = model.find('/') {
            &model[idx + 1..]
        } else {
            model
        };
        
        // Remove date suffixes (e.g., -20241022)
        if let Some(idx) = model.rfind('-') {
            if model[idx + 1..].len() == 8 && model[idx + 1..].chars().all(|c| c.is_ascii_digit()) {
                return model[..idx].to_string();
            }
        }
        
        model.to_string()
    }

    pub fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Option<f64> {
        let normalized = Self::normalize_model_name(model);
        
        // Try to find the model in the costs map
        let (input_cost, output_cost) = if let Some(costs) = self.costs.get(&normalized) {
            *costs
        } else {
            // Default fallback pricing for unknown models
            // Using a reasonable mid-range price (similar to claude-sonnet-4-5)
            log::warn!("Model '{}' not found in costs CSV, using default pricing ($2.00 input, $10.00 output per 1M tokens)", model);
            (2.0, 10.0)
        };
        
        Some(
            (input_tokens as f64 / 1_000_000.0) * input_cost +
            (output_tokens as f64 / 1_000_000.0) * output_cost
        )
    }
}

/// Key for grouping conversion results by agent, source, and target framework
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ConversionKey {
    pub agent: String,
    pub source_framework: String,
    pub target_framework: String,
}

/// Aggregated statistics for a group of conversions
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionStats {
    pub compile_pass: usize,
    pub compile_fail: usize,
    pub run_pass: usize,
    pub run_fail: usize,
    pub test_pass: usize,
    pub test_fail: usize,
    pub total: usize,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
}

impl ConversionStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_result(&mut self, metadata: &Metadata, usage: Option<(UsageData, Option<String>)>) {
        self.total += 1;

        // Compile: PASS = True, FAIL = False
        match metadata.compile_ok {
            ValidationOutcome::True => self.compile_pass += 1,
            ValidationOutcome::False => self.compile_fail += 1,
            ValidationOutcome::Unk => {},
        }

        // Run (deploy): PASS = True, FAIL = False
        match metadata.deploy_ok {
            ValidationOutcome::True => self.run_pass += 1,
            ValidationOutcome::False => self.run_fail += 1,
            ValidationOutcome::Unk => {},
        }

        // Tests: only PASS(100%) = pass, everything else = fail
        if metadata.test_pass_percent.contains("100%")
            || metadata.test_pass_percent.ends_with("(100.0%)") {
            self.test_pass += 1;
        } else if metadata.test_pass_percent != "UNK"
            && !metadata.test_pass_percent.is_empty() {
            self.test_fail += 1;
        }

        // Add usage data if available
        if let Some((usage, model)) = usage {
            self.input_tokens += usage.input_tokens;
            self.output_tokens += usage.output_tokens;
            self.cache_read_input_tokens += usage.cache_read_input_tokens;
            self.cache_creation_input_tokens += usage.cache_creation_input_tokens;
            
            // Store the model name (use the first one we see)
            if self.model_name.is_none() {
                self.model_name = model;
            }
        }
    }

    pub fn compile_rate(&self) -> f64 {
        let total = self.compile_pass + self.compile_fail;
        if total == 0 {
            0.0
        } else {
            self.compile_pass as f64 / total as f64
        }
    }

    pub fn run_rate(&self) -> f64 {
        let total = self.run_pass + self.run_fail;
        if total == 0 {
            0.0
        } else {
            self.run_pass as f64 / total as f64
        }
    }

    pub fn test_rate(&self) -> f64 {
        let total = self.test_pass + self.test_fail;
        if total == 0 {
            0.0
        } else {
            self.test_pass as f64 / total as f64
        }
    }
}

/// Aggregator for collecting conversion statistics
#[derive(Debug, Default)]
pub struct ConversionCostCalculator {
    stats: HashMap<ConversionKey, ConversionStats>,
    model_costs: Option<ModelCosts>,
}

impl ConversionCostCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_costs(model_costs: ModelCosts) -> Self {
        Self {
            stats: HashMap::new(),
            model_costs: Some(model_costs),
        }
    }

    pub fn add_conversion(&mut self, metadata: &Metadata, agent_out_path: &Path) {
        let key = ConversionKey {
            agent: metadata.agent.clone(),
            source_framework: metadata.source_framework.to_string(),
            target_framework: metadata.target_framework.to_string(),
        };

        // Parse agent output for usage data
        let usage = Self::parse_agent_output(agent_out_path);

        self.stats
            .entry(key)
            .or_insert_with(ConversionStats::new)
            .add_result(metadata, usage);
    }

    fn parse_agent_output(path: &Path) -> Option<(UsageData, Option<String>)> {
        let content = fs::read_to_string(path).ok()?;
        let mut total_usage = UsageData::default();
        let mut model_name: Option<String> = None;
        
        for line in content.lines() {
            // Skip lines with line numbers
            let line = if let Some(idx) = line.find('|') {
                &line[idx + 1..].trim()
            } else {
                line
            };
            
            if let Ok(entry) = serde_json::from_str::<TrajectoryEntry>(line) {
                if entry.entry_type == "assistant" {
                    if let Some(message) = entry.message {
                        // Capture the model name from the first message
                        if model_name.is_none() {
                            model_name = message.model.clone();
                        }
                        
                        if let Some(usage) = message.usage {
                            total_usage.input_tokens += usage.input_tokens;
                            total_usage.output_tokens += usage.output_tokens;
                            total_usage.cache_read_input_tokens += usage.cache_read_input_tokens;
                            total_usage.cache_creation_input_tokens += usage.cache_creation_input_tokens;
                        }
                    }
                }
            }
        }
        
        if total_usage.input_tokens > 0 || total_usage.output_tokens > 0 {
            Some((total_usage, model_name))
        } else {
            None
        }
    }

    pub fn finalize_costs(&mut self) {
        if let Some(ref costs) = self.model_costs {
            for stats in self.stats.values_mut() {
                let total_input = stats.input_tokens +
                                 stats.cache_creation_input_tokens +
                                 stats.cache_read_input_tokens;
                
                // Use the actual model name from the trajectory, or fall back to a default
                let model_to_use = stats.model_name.as_deref().unwrap_or("unknown");
                if let Some(cost) = costs.calculate_cost(model_to_use, total_input, stats.output_tokens) {
                    stats.cost_usd = Some(cost);
                }
            }
        }
    }

    pub fn print_summary(&self) {
        if self.stats.is_empty() {
            println!("\nNo conversion statistics available.");
            return;
        }

        println!("\n{}", "=".repeat(80));
        println!("CONVERSION COST SUMMARY");
        println!("{}", "=".repeat(80));
        println!(
            "{:<15} {:<12} -> {:<12} {:>8} {:>8} {:>8}",
            "Agent", "From", "To", "Compile", "Run", "Tests"
        );
        println!("{}", "-".repeat(80));

        let mut keys: Vec<_> = self.stats.keys().collect();
        keys.sort_by(|a, b| {
            a.agent
                .cmp(&b.agent)
                .then(a.source_framework.cmp(&b.source_framework))
                .then(a.target_framework.cmp(&b.target_framework))
        });

        for key in keys {
            if let Some(stats) = self.stats.get(key) {
                println!(
                    "{:<15} {:<12} -> {:<12} {:>7.2}% {:>7.2}% {:>7.2}%",
                    key.agent,
                    key.source_framework,
                    key.target_framework,
                    stats.compile_rate() * 100.0,
                    stats.run_rate() * 100.0,
                    stats.test_rate() * 100.0
                );
            }
        }

        println!("{}", "=".repeat(80));
    }

    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct JsonOutput {
            summary: HashMap<String, ConversionSummary>,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ConversionSummary {
            agent: String,
            source_framework: String,
            target_framework: String,
            compile_rate: f64,
            run_rate: f64,
            test_rate: f64,
            total_conversions: usize,
            input_tokens: u64,
            output_tokens: u64,
            cache_read_input_tokens: u64,
            cache_creation_input_tokens: u64,
            #[serde(skip_serializing_if = "Option::is_none")]
            cost_usd: Option<f64>,
        }

        let mut summary = HashMap::new();
        
        for (key, stats) in &self.stats {
            let key_str = format!("{}__{}__{}",
                key.agent,
                key.source_framework,
                key.target_framework
            );
            
            summary.insert(key_str, ConversionSummary {
                agent: key.agent.clone(),
                source_framework: key.source_framework.clone(),
                target_framework: key.target_framework.clone(),
                compile_rate: stats.compile_rate(),
                run_rate: stats.run_rate(),
                test_rate: stats.test_rate(),
                total_conversions: stats.total,
                input_tokens: stats.input_tokens,
                output_tokens: stats.output_tokens,
                cache_read_input_tokens: stats.cache_read_input_tokens,
                cache_creation_input_tokens: stats.cache_creation_input_tokens,
                cost_usd: stats.cost_usd,
            });
        }

        serde_json::to_string_pretty(&JsonOutput { summary }).unwrap_or_else(|_| "{}".to_string())
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
