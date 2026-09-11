//! Explicit engineered situation/action cues for the existing KC→MBON01 circuit.
//! No game route, reward table, added edge or button shortcut is used here.
use crate::*;

impl Brain {
    fn operant_cue(
        &mut self,
        context: &str,
        button: &str,
        size: usize,
        tonic: f32,
    ) -> Result<Vec<usize>, String> {
        if context.len() > 4096
            || !["up", "down", "left", "right", "a", "b", "start", "select"].contains(&button)
            || !(1..=128).contains(&size)
            || !tonic.is_finite()
            || !(0.0..=20.0).contains(&tonic)
        {
            return Err("Invalid action-memory cue".into());
        }
        let outputs: Vec<usize> = self
            .metadata
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == "MBON01")
            .map(|(i, _)| i)
            .collect();
        let mut pool: Vec<usize> = self
            .plastic
            .iter()
            .filter(|&&(_, e)| outputs.contains(&(self.targets[e] as usize)))
            .map(|&(pre, _)| pre)
            .collect();
        pool.sort_unstable();
        pool.dedup();
        if outputs.len() != 2 || pool.len() < size {
            return Err("Action-memory circuit unavailable".into());
        }
        let mut seed = 0xcbf29ce484222325u64;
        for b in context.bytes().chain([0]).chain(button.bytes()) {
            seed = (seed ^ b as u64).wrapping_mul(0x100000001b3);
        }
        pool.sort_unstable_by_key(|&i| {
            let mut x = seed.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            x ^ (x >> 31)
        });
        pool.truncate(size);
        let mut pairs: Vec<(u32, f32)> = self
            .metadata
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == "MBON01" || c.kind == "MBON11")
            .map(|(i, _)| (i as u32, tonic))
            .collect();
        pairs.extend(pool.iter().map(|&i| (i as u32, self.config.stimulus_mv)));
        self.inject_currents(&serde_json::to_string(&pairs).map_err(|e| e.to_string())?)?;
        Ok(pool)
    }
    fn operant_outputs(&self) -> Vec<serde_json::Value> {
        self.metadata.cells.iter().enumerate().filter(|(_,c)|c.kind=="MBON01"||c.kind=="MBON11").map(|(i,c)|serde_json::json!({"kind":c.kind,"index":i,"source_id":c.id,"spikes":self.counts[i],"voltage_mv":self.voltage[i]})).collect()
    }
}
#[wasm_bindgen]
impl Brain {
    /// Probe a candidate cue with dopamine off and plasticity frozen. Resets
    /// transient dynamics between probes, retaining actual learned efficacies.
    pub fn action_memory_probe(
        &mut self,
        context: &str,
        button: &str,
        size: usize,
        tonic: f32,
    ) -> Result<String, String> {
        self.reset_dynamics();
        let indices = self.operant_cue(context, button, size, tonic)?;
        let learning = self.config.learning;
        self.config.learning = false;
        let result = self.advance(500.0);
        self.config.learning = learning;
        result?;
        serde_json::to_string(&serde_json::json!({"context":context,"button":button,"indices":indices,"outputs":self.operant_outputs(),"telemetry":serde_json::from_str::<serde_json::Value>(&self.summary()?).map_err(|e|e.to_string())?,"scope":"artificial situation/action KC cue; actual MBON response; probe plasticity and reward off"})).map_err(|e|e.to_string())
    }
    /// Pair a cue with actual PAM01 current; the existing delayed DAN spikes and
    /// KC eligibility rule alone can modify weights. Frozen config is respected.
    pub fn action_memory_pair(
        &mut self,
        context: &str,
        button: &str,
        size: usize,
        tonic: f32,
        reward: bool,
    ) -> Result<String, String> {
        self.reset_dynamics();
        let indices = self.operant_cue(context, button, size, tonic)?;
        self.advance(100.0)?;
        if reward {
            self.appetitive_pulse()?;
        }
        self.advance(200.0)?;
        serde_json::to_string(&serde_json::json!({"context":context,"button":button,"indices":indices,"reward":reward,"outputs":self.operant_outputs(),"telemetry":serde_json::from_str::<serde_json::Value>(&self.summary()?).map_err(|e|e.to_string())?})).map_err(|e|e.to_string())
    }
}

use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OperantConfig {
    pub cue_size: usize,
    pub tonic_mv: f32,
    pub pairings: u32,
    pub new_place_pairings: u32,
    pub exploration: f64,
    pub inverse_temperature: f64,
    pub frames: u32,
    pub teacher_enabled: bool,
}
impl Default for OperantConfig {
    fn default() -> Self {
        Self {
            cue_size: 8,
            tonic_mv: 6.9,
            pairings: 6,
            new_place_pairings: 12,
            exploration: 0.1,
            inverse_temperature: 6.0,
            frames: 16,
            teacher_enabled: true,
        }
    }
}
impl OperantConfig {
    pub fn valid(&self) -> bool {
        (1..=128).contains(&self.cue_size)
            && (1..=12).contains(&self.pairings)
            && (self.pairings..=24).contains(&self.new_place_pairings)
            && self.tonic_mv.is_finite()
            && (0.0..=20.0).contains(&self.tonic_mv)
            && self.exploration.is_finite()
            && (0.0..=1.0).contains(&self.exploration)
            && self.inverse_temperature.is_finite()
            && (0.0..=20.0).contains(&self.inverse_temperature)
            && (1..=60).contains(&self.frames)
    }
}
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct OperantState {
    baselines: BTreeMap<String, [u64; 2]>,
    calibration_config: String,
    decisions: u64,
    rewarded: u64,
    neural_trial_ms: u64,
    dialogue: BTreeSet<String>,
    locations: BTreeSet<String>,
    recent: Vec<serde_json::Value>,
    pending: Option<Pending>,
    #[serde(default)]
    story: crate::story::StoryLedger,
}
#[derive(Serialize, Deserialize)]
struct Pending {
    context: String,
    button: String,
    before: serde_json::Value,
    score: f64,
    spikes: [u64; 2],
}
impl OperantState {
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({"interface":"artificial action-cue dual-MBON BCI v3","decisions":self.decisions,"conditioned_actions":self.rewarded,"calibration_entries":self.baselines.len(),"neural_trial_ms":self.neural_trial_ms,"locations":self.locations,"story":self.story.report(),"recent":self.recent,"history_limit":64,"teacher":"story-events-v1; evaluator only; no button choice"})
    }
}
fn context_of(v: &serde_json::Value) -> String {
    let mut h = 0xcbf29ce484222325u64;
    for b in format!(
        "{}|{}",
        v.pointer("/observe/visible_dialogue")
            .unwrap_or(&serde_json::Value::Null),
        v.pointer("/observe/menus")
            .unwrap_or(&serde_json::Value::Null)
    )
    .bytes()
    {
        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
    }
    // Some overlays (e.g. trainer card) are only exposed as rendered text.
    // Preserve the ordinary-world key while distinguishing those visible screens.
    if let Some(text) = v
        .pointer("/observe/text")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
    {
        for b in text.bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
    }
    let dialog = v
        .pointer("/observe/visible_dialogue")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty());
    let menus = v
        .pointer("/observe/menus")
        .and_then(|v| v.as_array())
        .is_some_and(|m| !m.is_empty());
    if !dialog && !menus {
        if let Some(lines) = v
            .pointer("/observe/rendered_text")
            .and_then(|v| v.as_array())
        {
            for line in lines {
                if let Some(text) = line.as_str() {
                    for b in text.trim().bytes() {
                        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
                    }
                }
            }
        }
    }
    format!(
        "{}|{}|{},{}|{h:016x}",
        v["status"]["screen"],
        v["map_info"]["name"],
        v["map_info"]["player"]["x"],
        v["map_info"]["player"]["y"]
    )
}
fn map_of(v: &serde_json::Value) -> &str {
    v.pointer("/map_info/name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
}
impl Brain {
    fn measured_cue(
        &mut self,
        context: &str,
        button: &str,
        c: &OperantConfig,
    ) -> Result<[u64; 2], String> {
        let r: serde_json::Value = serde_json::from_str(
            &self.action_memory_probe(context, button, c.cue_size, c.tonic_mv)?,
        )
        .map_err(|e| e.to_string())?;
        self.operant_state.neural_trial_ms += 500;
        let mut counts = [0, 0];
        for output in r["outputs"].as_array().ok_or("Missing MBON outputs")? {
            counts[if output["kind"] == "MBON01" { 0 } else { 1 }] +=
                output["spikes"].as_u64().unwrap_or(0);
        }
        Ok(counts)
    }
    fn baseline_cue(
        &mut self,
        context: &str,
        button: &str,
        c: &OperantConfig,
    ) -> Result<[u64; 2], String> {
        let key = format!("{context}|{button}");
        if let Some(&n) = self.operant_state.baselines.get(&key) {
            return Ok(n);
        }
        // A fixed naive calibration, not a learned action-value table. Temporarily
        // probe baseline efficacies, then restore every saved plastic weight.
        let saved: Vec<f32> = self.plastic.iter().map(|&(_, e)| self.weights[e]).collect();
        for &(_, e) in &self.plastic {
            self.weights[e] = self.contacts[e] as f32 * self.config.contact_gain_mv;
        }
        let result = self.measured_cue(context, button, c);
        for (&(_, e), w) in self.plastic.iter().zip(saved) {
            self.weights[e] = w;
        }
        let n = result?;
        if self.operant_state.baselines.len() < 100_000 {
            self.operant_state.baselines.insert(key, n);
        }
        Ok(n)
    }
}
#[wasm_bindgen]
impl Brain {
    pub fn operant_decide(&mut self, json: &str) -> Result<String, String> {
        let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let c = self
            .config
            .operant
            .clone()
            .ok_or("Action-memory mode is not configured")?;
        let mut calibration = self.config.clone();
        calibration.learning = false;
        let identity = serde_json::to_string(&calibration).map_err(|e| e.to_string())?;
        if self.operant_state.calibration_config != identity {
            self.operant_state.baselines.clear();
            self.operant_state.calibration_config = identity;
        }
        let context = context_of(&v);
        let mut readouts = Vec::new();
        let mut scores = Vec::new();
        let buttons = ["up", "down", "left", "right", "a", "b", "start", "select"];
        for button in buttons {
            let baseline = self.baseline_cue(&context, button, &c)?;
            let spikes = self.measured_cue(&context, button, &c)?;
            let appetitive = (baseline[0] as f64 - spikes[0] as f64) / baseline[0].max(1) as f64;
            let aversive = (baseline[1] as f64 - spikes[1] as f64) / baseline[1].max(1) as f64;
            let value = appetitive - aversive;
            let score = (c.inverse_temperature * value.clamp(-1.0, 1.0)).exp();
            scores.push(score);
            readouts.push(serde_json::json!({"button":button,"naive_mbon_spikes":baseline,"mbon_spikes":spikes,"learned_response":value,"score":score}));
        }
        let total: f64 = scores.iter().sum();
        let probabilities: Vec<f64> = scores
            .iter()
            .map(|s| (1.0 - c.exploration) * s / total + c.exploration / 8.0)
            .collect();
        let mut seed = self
            .operant_state
            .decisions
            .wrapping_add(0x9e3779b97f4a7c15);
        seed = (seed ^ (seed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        seed = (seed ^ (seed >> 27)).wrapping_mul(0x94d049bb133111eb);
        seed ^= seed >> 31;
        let mut sample = (seed >> 11) as f64 / 9007199254740992.0;
        let mut selected = 7;
        for (i, p) in probabilities.iter().enumerate() {
            readouts[i]["probability"] = serde_json::json!(p);
            if sample < *p {
                selected = i;
                break;
            }
            sample -= p;
        }
        for (i, p) in probabilities.iter().enumerate() {
            readouts[i]["probability"] = serde_json::json!(p);
        }
        let button = buttons[selected];
        // Leave the measured selected cue in the renderer, not an unrelated probe.
        self.measured_cue(&context, button, &c)?;
        self.operant_state.pending = Some(Pending {
            context: context.clone(),
            button: button.into(),
            before: v,
            score: readouts[selected]["learned_response"].as_f64().unwrap(),
            spikes: serde_json::from_value(readouts[selected]["mbon_spikes"].clone())
                .map_err(|e| e.to_string())?,
        });
        self.operant_state.decisions += 1;
        serde_json::to_string(&serde_json::json!({"action":{"button":button,"frames":c.frames,"readouts":readouts,"mapping":"actual MBON suppression relative to naive calibration; fixed softmax plus declared exploration"},"context":context,"telemetry":serde_json::from_str::<serde_json::Value>(&self.summary()?).map_err(|e|e.to_string())?,"neural_trial_ms":self.operant_state.neural_trial_ms})).map_err(|e|e.to_string())
    }
    pub fn operant_feedback(&mut self, json: &str) -> Result<String, String> {
        let after: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let pending = self
            .operant_state
            .pending
            .take()
            .ok_or("No neural action awaiting outcome")?;
        let c = self
            .config
            .operant
            .clone()
            .ok_or("Action-memory mode is not configured")?;
        let before = &pending.before;
        let before_map = map_of(before);
        let after_map = map_of(&after);
        self.operant_state
            .story
            .baseline_places(self.operant_state.locations.iter());
        let (rewards, aversions) = self.operant_state.story.feedback(before, &after);
        if after
            .pointer("/status/screen")
            .and_then(serde_json::Value::as_str)
            == Some("overworld")
        {
            self.operant_state.locations.insert(after_map.into());
        }
        let reason = (!rewards.is_empty()).then(|| rewards.join("; "));
        let aversion = (!aversions.is_empty()).then(|| aversions.join("; "));
        let goal = false;
        let enabled = c.teacher_enabled && self.config.rewards.enabled;
        let pairings = if rewards.iter().any(|event| event.starts_with("place:")) {
            c.new_place_pairings
        } else {
            c.pairings
        };
        let mut training = None;
        if (reason.is_some() || aversion.is_some()) && enabled {
            for _ in 0..pairings {
                let json = if aversion.is_some() {
                    self.action_memory_punish(
                        &pending.context,
                        &pending.button,
                        c.cue_size,
                        c.tonic_mv,
                    )?
                } else {
                    self.action_memory_pair(
                        &pending.context,
                        &pending.button,
                        c.cue_size,
                        c.tonic_mv,
                        true,
                    )?
                };
                training = Some(
                    serde_json::from_str::<serde_json::Value>(&json).map_err(|e| e.to_string())?,
                );
                self.operant_state.neural_trial_ms += 300;
            }
            self.operant_state.rewarded += 1;
        }
        let event = serde_json::json!({"decision":self.operant_state.decisions,"context":pending.context,"button":pending.button,"pre_learning_response":pending.score,"pre_mbon_spikes":pending.spikes,"before_map":before_map,"after_map":after_map,"teacher_id":"story-events-v1","conditioning_pairings":if training.is_some(){pairings}else{0},"reward":reason,"aversion":aversion,"enabled":enabled,"learning":self.config.learning,"goal_reached":goal});
        if self.operant_state.recent.len() >= 64 {
            self.operant_state.recent.remove(0);
        }
        self.operant_state.recent.push(event.clone());
        serde_json::to_string(&serde_json::json!({"events":reason.into_iter().chain(aversion).collect::<Vec<_>>(),"enabled":enabled,"goal_reached":goal,"event":event,"story":self.operant_state.story.report(),"training":training,"neural_trial_ms":self.operant_state.neural_trial_ms})).map_err(|e|e.to_string())
    }
}

#[wasm_bindgen]
impl Brain {
    pub fn action_memory_punish(
        &mut self,
        context: &str,
        button: &str,
        size: usize,
        tonic: f32,
    ) -> Result<String, String> {
        self.reset_dynamics();
        let indices = self.operant_cue(context, button, size, tonic)?;
        self.advance(100.0)?;
        self.aversive_pulse()?;
        self.advance(200.0)?;
        serde_json::to_string(&serde_json::json!({"context":context,"button":button,"indices":indices,"valence":"aversive PPL101","outputs":self.operant_outputs(),"telemetry":serde_json::from_str::<serde_json::Value>(&self.summary()?).map_err(|e|e.to_string())?})).map_err(|e|e.to_string())
    }
}
