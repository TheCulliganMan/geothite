//! Flygon neural runtime. Game rewards can inject current, never choose actions.
pub const INTERFACE_ID: &str = "flygon-connected-sensory-descending-actor-critic-v4-pooled";
pub const MODEL_ID: &str = "flygon-lif-kc-dan-ltd-v2";
mod breadcrumbs;
mod campaign;
mod checkpoint;
mod circuit;
mod sensory;
mod conditioning;
#[cfg(test)]
mod dynamics_tests;
mod interface;
mod operant;
mod objectives;
mod recovery_objectives;
mod shop_objectives;
mod choice_objectives;
mod pocket_objectives;
mod curriculum;
mod field_objectives;
mod story;
mod view;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub use view::AnatomyView;
use wasm_bindgen::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    pub kind: String,
    pub class: String,
    pub side: String,
    pub transmitter: String,
    pub position: Option<[f32; 3]>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub dataset: String,
    pub cells: Vec<Cell>,
    pub contacts: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub operant: Option<operant::OperantConfig>,
    #[serde(default)]
    pub rewards: campaign::RewardConfig,
    #[serde(default = "interface::default_bindings")]
    pub decoder: Vec<interface::Binding>,
    #[serde(default)]
    pub reset_synaptic_current_on_spike: bool,
    #[serde(default)]
    pub drop_refractory_input: bool,
    /// Integrate exponentially decaying synaptic/adaptation currents over a step.
    /// False preserves the original end-of-step approximation for lab profiles.
    #[serde(default)]
    pub exponential_current_integration: bool,
    pub dt_ms: f32,
    pub tau_membrane_ms: f32,
    pub tau_synapse_ms: f32,
    pub rest_mv: f32,
    pub threshold_mv: f32,
    pub refractory_ms: f32,
    pub delay_ms: f32,
    pub contact_gain_mv: f32,
    pub stimulus_mv: f32,
    pub dopamine_mv: f32,
    pub reward_pulse_ms: f32,
    pub eligibility_ms: f32,
    pub kc_rest_mv: f32,
    pub adaptation_jump_mv: f32,
    pub adaptation_tau_ms: f32,
    pub learning_rate: f32,
    pub minimum_weight_fraction: f32,
    pub maximum_weight_fraction: f32,
    pub learning: bool,
    pub decision_ms: f32,
}

// Solution of dv/dt = (rest - v + current)/tau_m with exponentially
// decaying current. Threshold crossings remain discretized at step boundaries.
fn current_coupling(dt: f32, tau_m: f32, tau_current: f32) -> f32 {
    let (dt, tm, tc) = (dt as f64, tau_m as f64, tau_current as f64);
    if tm == tc {
        ((dt / tm) * (-dt / tm).exp()) as f32
    } else {
        (tc / (tm - tc) * ((-dt / tm).exp() - (-dt / tc).exp())) as f32
    }
}
impl Config {
    /// Validate runtime configuration before allocating neural state.
    pub fn validate(&self) -> Result<(), String> {
        if self.operant.as_ref().is_some_and(|c| !c.valid()) {
            return Err("Invalid action-memory configuration".into());
        }
        if !self.rewards.validate() {
            return Err("Invalid reward settings".into());
        }
        if self.decoder.is_empty()
            || self.decoder.len() > 8
            || self.decoder.iter().any(|b| {
                !["up", "down", "left", "right", "a", "b", "start", "select"]
                    .contains(&b.button.as_str())
                    || b.kind.is_empty()
                    || !b.gain.is_finite()
                    || b.gain < 0.0
            })
        {
            return Err("Invalid decoder bindings".into());
        }
        let values = [
            self.dt_ms,
            self.tau_membrane_ms,
            self.tau_synapse_ms,
            self.rest_mv,
            self.threshold_mv,
            self.refractory_ms,
            self.delay_ms,
            self.contact_gain_mv,
            self.stimulus_mv,
            self.dopamine_mv,
            self.reward_pulse_ms,
            self.eligibility_ms,
            self.kc_rest_mv,
            self.adaptation_jump_mv,
            self.adaptation_tau_ms,
            self.learning_rate,
            self.minimum_weight_fraction,
            self.maximum_weight_fraction,
            self.decision_ms,
        ];
        if values.iter().any(|x| !x.is_finite())
            || self.dt_ms <= 0.0
            || self.dt_ms > 1.0
            || self.tau_membrane_ms < self.dt_ms
            || self.tau_synapse_ms < self.dt_ms
            || self.threshold_mv <= self.rest_mv
            || self.delay_ms < self.dt_ms
            || self.delay_ms / self.dt_ms > 1000.0
            || self.refractory_ms < 0.0
            || self.kc_rest_mv >= self.threshold_mv
            || self.adaptation_jump_mv < 0.0
            || self.adaptation_tau_ms <= 0.0
            || self.eligibility_ms <= 0.0
            || self.learning_rate < 0.0
            || self.minimum_weight_fraction <= 0.0
            || self.minimum_weight_fraction > 1.0
            || self.maximum_weight_fraction < 1.0
            || self.decision_ms <= 0.0
            || self.decision_ms > 1000.0
            || self.contact_gain_mv <= 0.0
            || self.reward_pulse_ms < 0.0
        {
            return Err("Invalid neural parameters".into());
        }
        Ok(())
    }
}

#[wasm_bindgen]
pub struct Brain {
    circuit: circuit::CircuitState,
    wiring: Option<circuit::Wiring>,
    graph_id: String,
    metadata: Metadata,
    config: Config,
    campaign: campaign::Campaign,
    offsets: Vec<u32>,
    targets: Vec<u32>,
    contacts: Vec<u32>,
    weights: Vec<f32>,
    voltage: Vec<f32>,
    current: Vec<f32>,
    adaptation: Vec<f32>,
    refractory: Vec<u32>,
    external: Vec<f32>,
    trace: Vec<f32>,
    last_spike: Vec<u64>,
    counts: Vec<u32>,
    queue: Vec<Vec<u32>>,
    tick: u64,
    last_window_steps: u32,
    kc: Vec<bool>,
    dan: Vec<usize>,
    mbon: Vec<bool>,
    plastic: Vec<(usize, usize)>,
    gains: Vec<Vec<f32>>,
    gate_index: Vec<usize>,
    gate_gains: Vec<Vec<f32>>,
    dan_index: Vec<usize>,
    pulse_ticks: u64,
    reward_ticks: u64,
    delivered_dan_spikes: u64,
    active: Vec<usize>,
    awake: Vec<bool>,
}

fn decode_graph(data: &[u8]) -> Result<(Vec<u32>, Vec<u32>, Vec<u32>), String> {
    if data.len() < 16 || &data[..8] != b"FLYGON01" {
        return Err("Invalid Flygon graph header".into());
    }
    let word = |i: usize| u32::from_le_bytes(data[i..i + 4].try_into().unwrap());
    let n = word(8) as usize;
    let e = word(12) as usize;
    let expected = 16usize
        .checked_add(
            n.checked_add(1)
                .and_then(|x| x.checked_mul(4))
                .ok_or("Graph overflow")?,
        )
        .and_then(|x| x.checked_add(e.checked_mul(8)?))
        .ok_or("Graph overflow")?;
    if expected != data.len() || n == 0 {
        return Err("Graph length mismatch".into());
    }
    let offsets = (0..=n).map(|i| word(16 + i * 4)).collect::<Vec<_>>();
    let base = 16 + (n + 1) * 4;
    let targets = (0..e).map(|i| word(base + i * 4)).collect::<Vec<_>>();
    let contacts = (0..e)
        .map(|i| word(base + e * 4 + i * 4))
        .collect::<Vec<_>>();
    if offsets[0] != 0
        || offsets[n] as usize != e
        || offsets.windows(2).any(|x| x[0] > x[1])
        || targets.iter().any(|x| *x as usize >= n)
        || contacts.contains(&0)
    {
        return Err("Invalid CSR graph".into());
    }
    Ok((offsets, targets, contacts))
}

#[wasm_bindgen]
impl Brain {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8], metadata_json: &str, config_json: &str) -> Result<Brain, String> {
        let mut digest = Sha256::new();
        digest.update(data);
        digest.update(metadata_json.as_bytes());
        let graph_id = format!("{:x}", digest.finalize());
        let metadata: Metadata = serde_json::from_str(metadata_json).map_err(|e| e.to_string())?;
        let config: Config = serde_json::from_str(config_json).map_err(|e| e.to_string())?;
        config.validate()?;
        let (offsets, targets, contacts) = decode_graph(data)?;
        let n = metadata.cells.len();
        if n + 1 != offsets.len() {
            return Err("Metadata does not match graph".into());
        }
        let mut weights = vec![0.0; targets.len()];
        for i in 0..n {
            let sign = match metadata.cells[i].transmitter.as_str() {
                "acetylcholine" | "ach" => 1.0,
                "gaba" | "glutamate" | "histamine" => -1.0,
                _ => 0.0,
            };
            for e in offsets[i] as usize..offsets[i + 1] as usize {
                weights[e] = contacts[e] as f32 * config.contact_gain_mv * sign;
            }
        }
        let kc = metadata
            .cells
            .iter()
            .map(|c| c.kind.starts_with("KC"))
            .collect::<Vec<_>>();
        let dan = metadata
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == "PPL101" || c.kind == "PAM01")
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let mbon = metadata
            .cells
            .iter()
            .map(|c| c.kind == "MBON11" || c.kind == "MBON01")
            .collect::<Vec<_>>();
        let mut plastic = Vec::new();
        for i in 0..n {
            if kc[i] {
                for e in offsets[i] as usize..offsets[i + 1] as usize {
                    if mbon[targets[e] as usize] && weights[e] > 0.0 {
                        plastic.push((i, e));
                    }
                }
            }
        }
        // Compartment participation is an anatomical contact-pooling proxy,
        // not a measured dopamine diffusion or receptor map.
        let mut gains = vec![vec![0.0; dan.len()]; plastic.len()];
        for (p, &(_, e)) in plastic.iter().enumerate() {
            let target = targets[e] as usize;
            let wanted = if metadata.cells[target].kind == "MBON11" {
                "PPL101"
            } else {
                "PAM01"
            };
            for (d, &cell) in dan.iter().enumerate() {
                if metadata.cells[cell].kind != wanted {
                    continue;
                }
                for edge in offsets[cell] as usize..offsets[cell + 1] as usize {
                    if targets[edge] as usize == target {
                        gains[p][d] += contacts[edge] as f32;
                    }
                }
            }
            let total = gains[p].iter().sum::<f32>();
            if total > 0.0 {
                for g in &mut gains[p] {
                    *g /= total;
                }
            }
        }
        // Edges terminating on one MBON share the same dopamine pooling vector.
        // Compute that dot product once per target, retaining its arithmetic order.
        let mut gate_targets = Vec::new();
        let mut gate_gains = Vec::new();
        let gate_index = plastic
            .iter()
            .enumerate()
            .map(|(p, &(_, e))| {
                let target = targets[e];
                if let Some(index) = gate_targets.iter().position(|&t| t == target) {
                    index
                } else {
                    gate_targets.push(target);
                    gate_gains.push(gains[p].clone());
                    gate_targets.len() - 1
                }
            })
            .collect();
        let mut dan_index = vec![usize::MAX; n];
        for (d, &i) in dan.iter().enumerate() {
            dan_index[i] = d;
        }
        let slots = (config.delay_ms / config.dt_ms).round() as usize + 1;
        let voltage = kc
            .iter()
            .map(|&is_kc| {
                if is_kc {
                    config.kc_rest_mv
                } else {
                    config.rest_mv
                }
            })
            .collect();
        Ok(Self {
            circuit: circuit::CircuitState::default(),
            wiring: None,
            graph_id,
            metadata,
            config,
            campaign: campaign::Campaign::default(),
            offsets,
            targets,
            contacts,
            weights,
            voltage,
            current: vec![0.0; n],
            adaptation: vec![0.0; n],
            refractory: vec![0; n],
            external: vec![0.0; n],
            trace: vec![0.0; n],
            last_spike: vec![0; n],
            counts: vec![0; n],
            queue: vec![Vec::new(); slots],
            tick: 0,
            last_window_steps: 0,
            kc,
            dan,
            mbon,
            plastic,
            gains,
            gate_index,
            gate_gains,
            dan_index,
            pulse_ticks: 0,
            reward_ticks: 0,
            delivered_dan_spikes: 0,
            active: Vec::new(),
            awake: vec![false; n],
        })
    }
    pub fn configure(&mut self, json: &str) -> Result<(), String> {
        let c: Config = serde_json::from_str(json).map_err(|e| e.to_string())?;
        c.validate()?;
        if c.dt_ms != self.config.dt_ms
            || c.delay_ms != self.config.delay_ms
            || c.contact_gain_mv != self.config.contact_gain_mv
            || c.operant.as_ref().map(|v| (v.sensory_neurons, v.sensory_window_ms, v.visible_menu_inputs)) != self.config.operant.as_ref().map(|v| (v.sensory_neurons, v.sensory_window_ms, v.visible_menu_inputs))
            || c.exponential_current_integration != self.config.exponential_current_integration
        {
            return Err("Interface or integration changes require an explicit brain reset".into());
        }
        if self.plastic.iter().any(|&(_, e)| {
            let base = self.contacts[e] as f32 * c.contact_gain_mv;
            self.weights[e] < base * c.minimum_weight_fraction - 1e-5
                || self.weights[e] > base * c.maximum_weight_fraction + 1e-5
        }) {
            return Err("New efficacy bounds exclude existing memory; erase memory or use compatible bounds".into());
        }
        if c.rest_mv != self.config.rest_mv || c.kc_rest_mv != self.config.kc_rest_mv {
            // Previously resting cells must integrate toward the new resting potential.
            for i in 0..self.voltage.len() {
                if !self.awake[i] {
                    self.active.push(i);
                    self.awake[i] = true;
                }
            }
        }
        self.config = c;
        Ok(())
    }
    pub fn stimulate(&mut self, indices: &[u32], strength: f32) -> Result<(), String> {
        if !strength.is_finite()
            || strength.abs() > 100.0
            || indices.iter().any(|i| *i as usize >= self.voltage.len())
        {
            return Err("Invalid stimulus".into());
        }
        self.external.fill(0.0);
        for &i in indices {
            let i = i as usize;
            self.external[i] = strength;
            if strength != 0.0 && !self.awake[i] {
                self.active.push(i);
                self.awake[i] = true;
            }
        }
        Ok(())
    }
    /// Experimental aversive pulse. Reward polarity is not a global weight sign.
    pub fn aversive_pulse(&mut self) -> Result<(), String> {
        if self.dan.is_empty() || self.plastic.is_empty() {
            return Err("PPL101/MBON11 memory circuit unavailable".into());
        }
        self.pulse_ticks = (self.config.reward_pulse_ms / self.config.dt_ms).round() as u64;
        for &i in &self.dan {
            if self.metadata.cells[i].kind == "PPL101" && !self.awake[i] {
                self.active.push(i);
                self.awake[i] = true;
            }
        }
        Ok(())
    }
    pub fn appetitive_pulse(&mut self) -> Result<(), String> {
        if !self
            .dan
            .iter()
            .any(|&i| self.metadata.cells[i].kind == "PAM01")
        {
            return Err("PAM01 reward population unavailable".into());
        }
        self.reward_ticks = (self.config.reward_pulse_ms / self.config.dt_ms).round() as u64;
        for &i in &self.dan {
            if self.metadata.cells[i].kind == "PAM01" && !self.awake[i] {
                self.active.push(i);
                self.awake[i] = true;
            }
        }
        Ok(())
    }
    pub fn step(&mut self, duration_ms: f32) -> Result<String, String> {
        self.advance(duration_ms)?;
        self.telemetry()
    }
    /// Integrate without serializing the full spike vector for the browser UI.
    pub fn advance(&mut self, duration_ms: f32) -> Result<(), String> {
        if !duration_ms.is_finite() || duration_ms <= 0.0 || duration_ms > 1000.0 {
            return Err("Invalid step duration".into());
        }
        let steps = (duration_ms / self.config.dt_ms).round() as u32;
        if steps == 0 {
            return Err("Interval smaller than neural step".into());
        }
        self.counts.fill(0);
        self.last_window_steps = steps;
        let leak = (-self.config.dt_ms / self.config.tau_membrane_ms).exp();
        let syn = (-self.config.dt_ms / self.config.tau_synapse_ms).exp();
        let delay = self.queue.len() - 1;
        let adapt_decay = (-self.config.dt_ms / self.config.adaptation_tau_ms).exp();
        let syn_coupling = current_coupling(
            self.config.dt_ms,
            self.config.tau_membrane_ms,
            self.config.tau_synapse_ms,
        );
        let adapt_coupling = current_coupling(
            self.config.dt_ms,
            self.config.tau_membrane_ms,
            self.config.adaptation_tau_ms,
        );
        let mut dan_fired = vec![0.0; self.dan.len()];
        let mut modulation_by_gate = vec![0.0; self.gate_gains.len()];
        for _ in 0..steps {
            let slot = self.tick as usize % self.queue.len();
            let due = std::mem::take(&mut self.queue[slot]);
            dan_fired.fill(0.0);
            for &source in &due {
                let i = source as usize;
                if self.dan_index[i] != usize::MAX {
                    dan_fired[self.dan_index[i]] += 1.0;
                }
                for e in self.offsets[i] as usize..self.offsets[i + 1] as usize {
                    let target = self.targets[e] as usize;
                    let weight = self.weights[e];
                    if self.config.drop_refractory_input && self.refractory[target] > 0 {
                        continue;
                    }
                    self.current[target] += weight;
                    if weight != 0.0 && !self.awake[target] {
                        self.active.push(target);
                        self.awake[target] = true;
                    }
                }
            }
            self.queue[slot] = due;
            self.queue[slot].clear();
            let pulse = self.pulse_ticks > 0;
            for &i in &self.active {
                let previous_current = self.current[i];
                let previous_adaptation = self.adaptation[i];
                self.current[i] *= syn;
                self.adaptation[i] *= adapt_decay;
                if self.refractory[i] > 0 {
                    self.refractory[i] -= 1;
                    continue;
                }
                let stimulus = self.external[i]
                    + if self.dan_index[i] != usize::MAX
                        && ((pulse && self.metadata.cells[i].kind == "PPL101")
                            || (self.reward_ticks > 0 && self.metadata.cells[i].kind == "PAM01"))
                    {
                        self.config.dopamine_mv
                    } else {
                        0.0
                    };
                let rest = if self.kc[i] {
                    self.config.kc_rest_mv
                } else {
                    self.config.rest_mv
                };
                self.voltage[i] = if self.config.exponential_current_integration {
                    rest + (self.voltage[i] - rest) * leak
                        + previous_current * syn_coupling
                        + stimulus * (1.0 - leak)
                        - previous_adaptation * adapt_coupling
                } else {
                    rest + (self.voltage[i] - rest) * leak
                        + (self.current[i] + stimulus - self.adaptation[i]) * (1.0 - leak)
                };
                if self.voltage[i] >= self.config.threshold_mv {
                    self.voltage[i] = rest;
                    if self.config.reset_synaptic_current_on_spike {
                        self.current[i] = 0.0;
                    }
                    self.refractory[i] =
                        (self.config.refractory_ms / self.config.dt_ms).round() as u32;
                    self.counts[i] += 1;
                    if self.kc[i] {
                        self.adaptation[i] += self.config.adaptation_jump_mv;
                        self.trace[i] *= (-((self.tick - self.last_spike[i]) as f32)
                            * self.config.dt_ms
                            / self.config.eligibility_ms)
                            .exp();
                        self.trace[i] += 1.0;
                    }
                    self.last_spike[i] = self.tick;
                    if self.dan_index[i] != usize::MAX {
                        self.delivered_dan_spikes += 1;
                    }
                    let arrival = (self.tick as usize + delay) % self.queue.len();
                    self.queue[arrival].push(i as u32);
                }
            }
            // Initial, explicitly experimental aversive LTD. Local KC eligibility
            // gates changes only when an identified DAN actually spikes.
            if dan_fired.iter().any(|v| *v > 0.0) && self.config.learning {
                for (gate, gains) in self.gate_gains.iter().enumerate() {
                    modulation_by_gate[gate] = gains
                        .iter()
                        .zip(&dan_fired)
                        .map(|(g, s)| g * s)
                        .sum::<f32>();
                }
                for (p, &(pre, e)) in self.plastic.iter().enumerate() {
                    let modulation = modulation_by_gate[self.gate_index[p]];
                    if modulation == 0.0 {
                        continue;
                    }
                    let eligibility = self.trace[pre]
                        * (-((self.tick - self.last_spike[pre]) as f32) * self.config.dt_ms
                            / self.config.eligibility_ms)
                            .exp();
                    let base = self.contacts[e] as f32 * self.config.contact_gain_mv;
                    self.weights[e] = (self.weights[e]
                        * (-self.config.learning_rate * eligibility * modulation).exp())
                    .clamp(
                        base * self.config.minimum_weight_fraction,
                        base * self.config.maximum_weight_fraction,
                    );
                }
            }
            self.pulse_ticks = self.pulse_ticks.saturating_sub(1);
            self.reward_ticks = self.reward_ticks.saturating_sub(1);
            self.tick += 1;
        }
        Ok(())
    }
    pub fn telemetry(&self) -> Result<String, String> {
        self.snapshot(true)
    }
    pub fn summary(&self) -> Result<String, String> {
        self.snapshot(false)
    }
    pub fn inspect(&self, index: u32) -> Result<String, String> {
        let i = index as usize;
        if i >= self.voltage.len() {
            return Err("Unknown neuron".into());
        }
        let mut outgoing =
            (self.offsets[i] as usize..self.offsets[i + 1] as usize).collect::<Vec<_>>();
        outgoing.sort_unstable_by_key(|&e| std::cmp::Reverse(self.contacts[e]));
        let connections=outgoing.iter().take(64).map(|&e|{let post=self.targets[e] as usize;serde_json::json!({"post":post,"source_id":self.metadata.cells[post].id,"cell_type":self.metadata.cells[post].kind,"contacts":self.contacts[e],"weight":self.weights[e],"spikes":self.counts[post]})}).collect::<Vec<_>>();
        serde_json::to_string(&serde_json::json!({"outgoing_connections":connections,"outgoing_total":outgoing.len(),"outgoing_display_limit":64,"index":index,"cell":self.metadata.cells[i],"voltage_mv":self.voltage[i],"synaptic_current":self.current[i],"spikes":self.counts[i],"last_spike_tick":self.last_spike[i],"kc":self.kc[i],"mbon":self.mbon[i],"plastic_connections":self.plastic.iter().filter(|&&(pre,_)|pre==i).map(|&(_,e)|serde_json::json!({"post":self.targets[e],"source_id":self.metadata.cells[self.targets[e] as usize].id,"weight":self.weights[e],"baseline":self.contacts[e] as f32*self.config.contact_gain_mv})).collect::<Vec<_>>()})).map_err(|e|e.to_string())
    }
}

impl Brain {
    fn snapshot(&self, include_spikes: bool) -> Result<String, String> {
        let changed = self
            .plastic
            .iter()
            .filter(|&&(_, e)| {
                (self.weights[e] - self.contacts[e] as f32 * self.config.contact_gain_mv).abs()
                    > 1e-7
            })
            .count();
        let spiking_neurons = self.counts.iter().filter(|&&n| n > 0).count();
        let spikes = if include_spikes {
            self.counts
                .iter()
                .enumerate()
                .filter(|(_, n)| **n > 0)
                .map(|(i, n)| (i, *n))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        serde_json::to_string(&serde_json::json!({"tick":self.tick,"neural_ms":self.tick as f64*self.config.dt_ms as f64,
            "model_id":MODEL_ID,"interface_id":INTERFACE_ID,"activity_window_ms":self.last_window_steps as f64*self.config.dt_ms as f64,"circuit_activity":self.circuit_activity(),"anatomical_neurons":self.metadata.cells.iter().filter(|c|c.position.is_some()).count(),"neurons":self.voltage.len(),"edges":self.targets.len(),"graph_id":self.graph_id,"spiking_neurons":spiking_neurons,"total_spikes":self.counts.iter().map(|&n|n as u64).sum::<u64>(),"spikes":if include_spikes {serde_json::json!(spikes)} else {serde_json::Value::Null},
            "policy_updates":self.circuit.updates(),"plastic_edges":self.plastic.len(),"changed_edges":changed,"dan_spikes":self.delivered_dan_spikes,
            "pulse_ticks":self.pulse_ticks,"reward_ticks":self.reward_ticks,"integrated_neurons":self.active.len(),"learning":self.config.learning,"status":"experimental-unvalidated"})).map_err(|e|e.to_string())
    }
}
