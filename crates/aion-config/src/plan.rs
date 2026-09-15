use serde::{Deserialize, Serialize};

/// Configuration for Plan Mode.
///
/// Plan Mode restricts the agent to read-only tools while it builds
/// an implementation plan.  After the user approves the plan the agent
/// exits plan mode and regains full tool access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanConfig {
    /// Whether Plan Mode tools (EnterPlanMode / ExitPlanMode) are registered.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Directory for plan files, relative to the project root.
    #[serde(default = "default_plan_directory")]
    pub plan_directory: String,

    /// Optional override for the plan mode system prompt text.
    ///
    /// When `Some`, the embedded text replaces the built-in instructions
    /// returned by `aion_agent::plan::prompt::plan_mode_instructions()`.
    /// When `None`, the built-in default is used.
    #[serde(default)]
    pub prompt: Option<String>,
}

impl Default for PlanConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            plan_directory: default_plan_directory(),
            prompt: None,
        }
    }
}

// --- Default value functions ---

fn default_true() -> bool {
    true
}

fn default_plan_directory() -> String {
    ".aionrs/plans".to_string()
}

#[cfg(test)]
#[path = "plan_test.rs"]
mod plan_test;
