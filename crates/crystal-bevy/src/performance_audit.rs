//! Serializable inputs for repeatable, disposable renderer audits.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

/// One scenario. Unknown fields are rejected to catch misspelled assertions.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditScenario {
    /// Stable identifier used to compare runs.
    pub name: String,
    /// Authored starting map.
    pub map: String,
    /// Authoritative starting tile.
    pub tile: [i16; 2],
    /// Field, production battle commands, or the checked Route44 Surf shore.
    pub fixture: AuditFixture,
    /// Untimed input steps, for example mounting Surf through A/Yes.
    pub prepare: Vec<AuditStep>,
    /// Timed inputs, each with an authoritative completion assertion.
    pub steps: Vec<AuditStep>,
    /// Minimum measured steady-state duration, excluding preparation and warmup.
    pub seconds: u32,
}

/// Existing disposable developer fixtures; no saved progress is modified.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditFixture {
    /// New game at the specified tile.
    Field,
    /// Existing production battle preview.
    Battle,
    /// Existing checked land-to-water Surf preview, before ordinary A/Yes.
    SurfShore,
}

/// A held direction or pulsed ordinary controller button.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditStep {
    /// right, left, up, down, a, b, start, or wait.
    pub key: String,
    /// Required state before this step can finish.
    pub until: AuditExpectation,
    /// Failure deadline in real seconds, unaffected by FPS.
    pub timeout_seconds: u32,
}

/// Every supplied condition must hold at a visibly settled frame.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditExpectation {
    /// Required map after a warp or connection.
    pub map: Option<String>,
    /// Required authoritative position.
    pub tile: Option<[i16; 2]>,
    /// Debug name of the production mode: Normal, Surf, etc.
    pub mode: Option<String>,
    /// Required production battle presence.
    pub battle: Option<bool>,
    /// Require a move to consume PP relative to the initial battle party.
    #[serde(default)]
    pub pp_consumed: bool,
    /// Minimum real duration for idle or animation samples.
    #[serde(default)]
    pub seconds: u32,
}

/// Installed by the dedicated example, before renderer creation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditRequest {
    /// Validated input recipe.
    pub scenario: AuditScenario,
    /// Empty output directory, created by the caller.
    pub directory: PathBuf,
    /// Whether timestamp queries and render diagnostics are enabled.
    pub gpu: bool,
}

impl AuditScenario {
    /// Reject invalid recipes before loading the renderer.
    ///
    /// # Errors
    /// Returns an error for unbounded durations, invalid keys or empty assertions.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.is_empty() && self.name.len() <= 100,
            "invalid scenario name"
        );
        ensure!(!self.map.is_empty(), "scenario map is required");
        ensure!((1..=120).contains(&self.seconds), "seconds must be 1..120");
        ensure!(
            self.prepare.len() + self.steps.len() <= 256,
            "too many input steps"
        );
        for step in self.prepare.iter().chain(&self.steps) {
            ensure!(
                matches!(
                    step.key.as_str(),
                    "right" | "left" | "up" | "down" | "a" | "b" | "start" | "wait"
                ),
                "invalid key {}",
                step.key
            );
            ensure!(
                (1..=120).contains(&step.timeout_seconds),
                "invalid step timeout"
            );
            let expected = &step.until;
            ensure!(
                expected.seconds < step.timeout_seconds,
                "invalid minimum step duration"
            );
            ensure!(
                expected.map.is_some()
                    || expected.tile.is_some()
                    || expected.mode.is_some()
                    || expected.battle.is_some()
                    || expected.pp_consumed
                    || expected.seconds > 0,
                "step needs an assertion"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AuditScenario;

    #[test]
    fn rejects_typo_and_unbounded_scenarios() {
        let valid = r#"{"name":"idle","map":"NewBarkTown","tile":[13,6],"fixture":"field","prepare":[],"steps":[],"seconds":1}"#;
        let mut scenario: AuditScenario = serde_json::from_str(valid).expect("valid recipe");
        scenario.validate().expect("bounded idle scenario");
        scenario.seconds = 0;
        assert!(scenario.validate().is_err());
        assert!(
            serde_json::from_str::<AuditScenario>(&valid.replace("seconds", "secnds")).is_err()
        );
    }
}
