//! Explicit engineered situation/action cues for the existing KC→MBON01 circuit.
//! No game route, reward table, added edge or button shortcut is used here.
use crate::*;

/// Disjoint anatomical pools prevent duplicate cells across spatial channels.
/// Plain laboratory cues retain their original allocation.
fn spatial_cue_indices(pool: &[usize], context: &str, button: &str, size: usize) -> Vec<usize> {
    let channels = context
        .strip_prefix("spatial-map-v2:")
        .and_then(|json| serde_json::from_str::<Vec<String>>(json).ok())
        .filter(|channels| channels.len() == 8 && size >= 8);
    let channels = channels.unwrap_or_else(|| vec![context.to_owned()]);
    let mut selected = Vec::with_capacity(size);
    for (channel, cue) in channels.iter().enumerate() {
        let count = size / channels.len() + usize::from(channel < size % channels.len());
        let mut seed = 0xcbf29ce484222325u64;
        for b in cue.bytes().chain([0]).chain(button.bytes()) {
            seed = (seed ^ b as u64).wrapping_mul(0x100000001b3);
        }
        let mut candidates: Vec<_> = pool
            .iter()
            .copied()
            .enumerate()
            .filter(|(slot, _)| slot % channels.len() == channel)
            .map(|(_, i)| i)
            .collect();
        candidates.sort_unstable_by_key(|&i| {
            let mut x = seed.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            x ^ (x >> 31)
        });
        selected.extend(candidates.into_iter().take(count));
    }
    selected
}

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
        let pool = spatial_cue_indices(&pool, context, button, size);
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OperantConfig {
    /// Reproducible action sampling; does not alter game state or learned weights.
    pub action_seed: u64,
    /// Anatomical sensory population budget, independent of policy dimensions.
    pub sensory_neurons: usize,
    pub sensory_window_ms: u32,
    pub visible_menu_inputs: bool,
    pub exploration: f64,
    pub frames: u32,
    pub teacher_enabled: bool,
    pub readout_learning_rate: f32,
    pub discount: f32,
    pub value_learning_rate: f32,
    pub entropy_bonus: f32,
    pub appetitive_pulse_ms: f32,
    pub aversive_pulse_ms: f32,
    pub breadcrumb_reward: f32,
    pub bad_action_penalty: f32,
    pub allow_select: bool,
}
impl Default for OperantConfig {
    fn default() -> Self {
        Self {
            action_seed: 0,
            sensory_neurons: 256,
            sensory_window_ms: 150,
            visible_menu_inputs: false,
            exploration: 0.1,
            frames: 16,
            teacher_enabled: true,
            readout_learning_rate: 0.05,
            discount: 0.99,
            value_learning_rate: 0.1,
            entropy_bonus: 0.005,
            appetitive_pulse_ms: 200.0,
            aversive_pulse_ms: 200.0,
            breadcrumb_reward: 0.5,
            bad_action_penalty: 2.0,
            allow_select: false,
        }
    }
}
impl OperantConfig {
    pub fn valid(&self) -> bool {
        [self.appetitive_pulse_ms, self.aversive_pulse_ms]
            .iter()
            .all(|v| v.is_finite() && (10.0..=500.0).contains(v))
            && [self.breadcrumb_reward, self.bad_action_penalty]
                .iter()
                .all(|v| v.is_finite() && (0.0..=5.0).contains(v))
            && (256..=16384).contains(&self.sensory_neurons)
            && (50..=300).contains(&self.sensory_window_ms)
            && self.exploration.is_finite()
            && (0.0..=1.0).contains(&self.exploration)
            && (1..=60).contains(&self.frames)
            && self.readout_learning_rate.is_finite()
            && (0.0..=0.5).contains(&self.readout_learning_rate)
            && self.discount.is_finite()
            && (0.0..=0.999).contains(&self.discount)
            && self.value_learning_rate.is_finite()
            && (0.0..=0.5).contains(&self.value_learning_rate)
            && self.entropy_bonus.is_finite()
            && (0.0..=0.1).contains(&self.entropy_bonus)
    }
}
#[cfg(test)]
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
    let location = format!(
        "{}|{}|{},{}|{h:016x}",
        v["status"]["screen"],
        v["map_info"]["name"],
        v["map_info"]["player"]["x"],
        v["map_info"]["player"]["y"]
    );
    let mut channels = vec![
        vec![location],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    for feature in crate::interface::local_map_features(v) {
        let parts: Vec<_> = feature.split(':').collect();
        let channel = match parts[0] {
            "facing" => 1,
            "object" => 7,
            _ => {
                let dx = parts
                    .get(1)
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                let dy = parts
                    .get(2)
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                if dx.abs().max(dy.abs()) > 2 || (dx == 0 && dy == 0) {
                    6
                } else if dy.abs() >= dx.abs() {
                    if dy < 0 { 2 } else { 4 }
                } else if dx > 0 {
                    3
                } else {
                    5
                }
            }
        };
        channels[channel].push(feature);
    }
    // Each channel has a separate, stable KC allocation. Unchanged regions keep
    // the same neural input even when another region or the location changes.
    let hashes: Vec<String> = channels
        .iter()
        .map(|features| {
            let mut h = 0xcbf29ce484222325u64;
            for feature in features {
                for b in feature.bytes().chain([0]) {
                    h = (h ^ b as u64).wrapping_mul(0x100000001b3);
                }
            }
            format!("{h:016x}")
        })
        .collect();
    format!("spatial-map-v2:{}", serde_json::to_string(&hashes).unwrap())
}
#[cfg(test)]
fn map_of(v: &serde_json::Value) -> &str {
    v.pointer("/map_info/name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
}
#[wasm_bindgen]
impl Brain {
    pub fn operant_decide(&mut self, json: &str) -> Result<String, String> {
        self.circuit_decide(json)
    }
    pub fn operant_feedback(&mut self, json: &str) -> Result<String, String> {
        self.circuit_feedback(json)
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

#[cfg(test)]
mod sensory_tests {
    use super::context_of;
    use crate::interface::local_map_features;
    use serde_json::{Value, json};

    fn observation() -> Value {
        json!({
            "status": {"screen": "overworld"},
            "map_info": {
                "name": "PlayersHouse2F",
                "player": {"x": 1, "y": 0, "facing": "Right"},
                "objects": [{"name": "NPC", "x": 2, "y": 1}],
                "terrain": {"origin_x": 0, "origin_y": 0,
                    "rows": [[null, {"terrain": "Land", "permission": 0},
                        {"terrain": "Wall", "permission": 7}]]}
            },
            "observe": {"menus": [], "visible_dialogue": null}
        })
    }

    #[test]
    fn action_cues_distinguish_local_map_changes_at_the_same_position() {
        let original = observation();
        let context = context_of(&original);
        for (path, value) in [
            ("/map_info/player/facing", json!("Left")),
            ("/map_info/objects/0/x", json!(3)),
            ("/map_info/terrain/rows/0/2/terrain", json!("Water")),
            ("/map_info/terrain/rows/0/1/permission", json!(1)),
            ("/map_info/terrain/rows/0/0", json!({"terrain": "Land"})),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(path).unwrap() = value;
            assert_ne!(context, context_of(&changed), "ignored {path}");
        }
    }

    #[test]
    fn local_map_coordinates_use_the_observed_origin_at_map_edges() {
        let features = local_map_features(&observation());
        for expected in [
            "facing:Right",
            "object:1:1",
            "terrain:-1:0:outside",
            "terrain:0:0:Land",
            "permission:0:0:0",
            "terrain:1:0:Wall",
        ] {
            assert!(features.iter().any(|f| f == expected), "missing {expected}");
        }
    }

    #[test]
    fn sensory_context_ignores_reward_state_and_distant_objects() {
        let original = observation();
        let mut changed = original.clone();
        changed["reward_state"] = json!({"event_flags": ["SECRET_GOAL"], "money": 9999});
        changed["frame"] = json!(12345);
        changed["map_info"]["objects"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "DISTANT", "x": 30, "y": 40}));
        assert_eq!(context_of(&original), context_of(&changed));
    }

    #[test]
    fn object_iteration_order_does_not_change_perception() {
        let mut first = observation();
        first["map_info"]["objects"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "SECOND", "x": 4, "y": 2}));
        let mut second = first.clone();
        second["map_info"]["objects"]
            .as_array_mut()
            .unwrap()
            .reverse();
        assert_eq!(context_of(&first), context_of(&second));
    }

    #[test]
    fn one_region_change_preserves_seven_neural_channels() {
        let original = observation();
        let mut changed = original.clone();
        changed["map_info"]["terrain"]["rows"][0][2]["terrain"] = json!("Water");
        let pool: Vec<_> = (0..4096).collect();
        let before = super::spatial_cue_indices(&pool, &context_of(&original), "right", 8);
        let after = super::spatial_cue_indices(&pool, &context_of(&changed), "right", 8);
        assert_ne!(before[3], after[3], "east terrain must change its KC input");
        for channel in [0, 1, 2, 4, 5, 6, 7] {
            assert_eq!(before[channel], after[channel], "unrelated channel changed");
        }
    }

    #[test]
    fn equivalent_local_geometry_transfers_between_locations() {
        let original = observation();
        let mut moved = original.clone();
        moved["map_info"]["name"] = json!("AnotherMap");
        moved["map_info"]["player"]["x"] = json!(11);
        moved["map_info"]["terrain"]["origin_x"] = json!(10);
        moved["map_info"]["objects"][0]["x"] = json!(12);
        let pool: Vec<_> = (0..4096).collect();
        let before = super::spatial_cue_indices(&pool, &context_of(&original), "a", 8);
        let after = super::spatial_cue_indices(&pool, &context_of(&moved), "a", 8);
        assert_ne!(before[0], after[0]);
        assert_eq!(&before[1..], &after[1..]);
    }

    #[test]
    fn spatial_cues_keep_exact_dose_without_duplicate_cells() {
        let context = context_of(&observation());
        for size in [1, 7, 8, 9, 16, 127, 128] {
            let pool: Vec<_> = (0..size.max(8)).collect();
            let mut cells = super::spatial_cue_indices(&pool, &context, "up", size);
            assert_eq!(cells.len(), size);
            cells.sort_unstable();
            cells.dedup();
            assert_eq!(cells.len(), size);
        }
    }

    #[test]
    fn empty_observation_has_no_invented_map_features() {
        assert!(local_map_features(&json!({})).is_empty());
    }
}

pub(crate) fn default_sensory_window() -> u32 { 150 }
