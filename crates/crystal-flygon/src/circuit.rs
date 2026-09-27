//! Sensory -> measured recurrent graph -> descending population -> policy adapter.
//! The adapter is engineered online learning, not an anatomical synapse or a
//! biological claim. Observations and explicit story scents drive sensory cells;
//! no candidate button is injected into the graph.
use crate::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
const BUTTONS: [&str; 8] = ["up", "down", "left", "right", "a", "b", "start", "select"];
const WIDTH: usize = 256;
const WINDOW_MS: f32 = 150.0;

pub(crate) struct Wiring {
    pub(crate) inputs: Vec<usize>,
    pub(crate) outputs: Vec<usize>,
    hops: Vec<u16>,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct CircuitState {
    weights: Vec<f32>,
    value_weights: Vec<f32>,
    context: String,
    heads: BTreeMap<String,(Vec<f32>,Vec<f32>)>,
    rollout: Vec<Transition>,
    last_value: f32,
    last_advantage: f32,
    last_entropy: f32,
    decisions: u64,
    updates: u64,
    pending: Option<Decision>,
    story: crate::story::StoryLedger,
    recent: Vec<Value>,
    breadcrumbs: crate::breadcrumbs::Breadcrumbs,
}
#[derive(Clone, Serialize, Deserialize)]
struct Decision {
    before: Value,
    features: Vec<f32>,
    score_gradient: Vec<f32>,
    action: usize,
    #[serde(default)]
    probabilities: Vec<f32>,
}
#[derive(Serialize, Deserialize)]
struct Transition {
    decision: Decision,
    reward: f32,
}

// Avoid selecting only the highest-degree cells of one type or hemisphere.
fn diverse_population(
    cells: &[Cell],
    candidates: Vec<usize>,
    degree: &[u64],
    limit: usize,
) -> Vec<usize> {
    let mut groups: BTreeMap<(&str, &str), Vec<usize>> = BTreeMap::new();
    for i in candidates {
        groups
            .entry((&cells[i].kind, &cells[i].side))
            .or_default()
            .push(i);
    }
    let mut groups: Vec<VecDeque<usize>> = groups
        .into_values()
        .map(|mut g| {
            g.sort_unstable_by_key(|&i| (std::cmp::Reverse(degree[i]), i));
            VecDeque::from(g)
        })
        .collect();
    groups.sort_by_key(|g| (std::cmp::Reverse(degree[g[0]]), g[0]));
    let mut selected = Vec::new();
    while selected.len() < limit {
        let previous = selected.len();
        for group in &mut groups {
            if let Some(i) = group.pop_front() {
                selected.push(i);
            }
            if selected.len() == limit {
                break;
            }
        }
        if selected.len() == previous {
            break;
        }
    }
    selected
}

// Every reachable descending cell contributes. Round-robin type/side ordering
// spreads large populations across bounded policy features. This compression is
// an engineered readout, not additional anatomical wiring.
fn pool_readout(signals: impl Iterator<Item = f32>) -> Vec<f32> {
    let mut pooled = vec![0.0f32; WIDTH];
    let mut sizes = [0usize; WIDTH];
    for (i, signal) in signals.enumerate() {
        pooled[i % WIDTH] += signal;
        sizes[i % WIDTH] += 1;
    }
    for (value, size) in pooled.iter_mut().zip(sizes) {
        *value /= (size.max(1) as f32).sqrt();
    }
    let norm = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-6 {
        for value in &mut pooled {
            *value /= norm;
        }
    }
    pooled
}

fn sensory_channel(feature: &str) -> usize {
    if feature.starts_with("story-scent:") || feature.starts_with("exploration-scent:") {
        return match feature.rsplit(':').next() {
            Some("north") => 0,
            Some("east") => 1,
            Some("south") => 2,
            _ => 3,
        };
    }
    if feature.starts_with("party:") || feature.starts_with("battle:") {
        return 6;
    }
    if feature.starts_with("menu:")
        || feature.starts_with("text:")
        || feature.starts_with("dialogue:")
    {
        return 5;
    }
    if feature.starts_with("object:") {
        return 4;
    }
    if feature.starts_with("terrain:")
        || feature.starts_with("permission:")
        || feature.starts_with("boundary:")
    {
        let mut fields = feature.split(':').skip(1);
        let x = fields
            .next()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        let y = fields
            .next()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        return if y.abs() >= x.abs() {
            if y < 0 { 0 } else { 2 }
        } else if x > 0 {
            1
        } else {
            3
        };
    }
    7
}
impl CircuitState {
    fn activate_context(
        &mut self,
        context: String,
        config: &crate::operant::OperantConfig,
        learning: bool,
    ) {
        if self.context == context {
            return;
        }
        if learning {
            self.train_rollout(self.last_value, config);
        } else {
            self.rollout.clear();
        }
        if !self.weights.is_empty() {
            let key = if self.context.is_empty() {
                "legacy".into()
            } else {
                self.context.clone()
            };
            self.heads.insert(
                key,
                (
                    std::mem::take(&mut self.weights),
                    std::mem::take(&mut self.value_weights),
                ),
            );
        }
        let (actor,critic)=self.heads.remove(&context).unwrap_or_default();
        self.weights=actor;self.value_weights=critic;self.context=context;self.last_value=0.0;
    }
    pub(crate) fn updates(&self) -> u64 {
        self.updates
    }
    pub(crate) fn valid(&self) -> bool {
        self.heads.len() <= 4096
            && self.heads.iter().all(|(key, (actor, critic))| {
                key.len() <= 512
                    && actor.len() == WIDTH * 8
                    && actor.iter().all(|x| x.is_finite() && x.abs() <= 4.0)
                    && (critic.is_empty() || critic.len() == WIDTH)
                    && critic.iter().all(|x| x.is_finite() && x.abs() <= 20.0)
            })
            && (self.pending.is_none() || self.weights.len() == WIDTH * 8)
            && (self.weights.is_empty() || self.weights.len() == WIDTH * 8)
            && self.weights.iter().all(|x| x.is_finite() && x.abs() <= 4.0)
            && self.pending.as_ref().is_none_or(|p| {
                p.action < 8
                    && p.features.len() == WIDTH
                    && p.score_gradient.len() == 8
                    && p.probabilities.len() == 8
                    && p.probabilities
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                    && p.features.iter().all(|x| x.is_finite() && x.abs() <= 1.0)
                    && p.score_gradient
                        .iter()
                        .all(|x| x.is_finite() && x.abs() <= 1.0)
            })
            && (self.value_weights.is_empty() || self.value_weights.len() == WIDTH)
            && self
                .value_weights
                .iter()
                .all(|v| v.is_finite() && v.abs() <= 20.0)
            && [self.last_value, self.last_advantage, self.last_entropy]
                .iter()
                .all(|v| v.is_finite())
            && self.rollout.len() <= 8
            && self.rollout.iter().all(|t| {
                t.reward.is_finite()
                    && t.reward.abs() <= 5.0
                    && t.decision.features.len() == WIDTH
                    && t.decision.action < 8
                    && t.decision.score_gradient.len() == 8
                    && t.decision.probabilities.len() == 8
                    && t.decision
                        .features
                        .iter()
                        .all(|v| v.is_finite() && v.abs() <= 1.0)
                    && t.decision
                        .score_gradient
                        .iter()
                        .all(|v| v.is_finite() && v.abs() <= 1.0)
                    && t.decision
                        .probabilities
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            })
    }
    pub(crate) fn interrupt(&mut self) {
        self.pending = None;
        self.rollout.clear();
    }
    pub(crate) fn erase(&mut self) {
        self.heads.clear();
        self.weights.fill(0.0);
        self.value_weights.fill(0.0);
        self.last_value = 0.0;
        self.last_advantage = 0.0;
        self.interrupt();
    }
    pub(crate) fn report(&self) -> Value {
        json!({"interface":"sensory-descending-actor-critic-v4-pooled", "decisions":self.decisions,
            "updates":self.updates,"neural_trial_ms":self.decisions * WINDOW_MS as u64,
            "story":self.story.report(),"recent":self.recent,
            "value":self.last_value,"advantage":self.last_advantage,"entropy":self.last_entropy,
            "pending_transitions":self.rollout.len(),
            "learning_context":self.context,"stored_contexts":self.heads.len(),
            "breadcrumb":self.breadcrumbs.current,
            "learning":"event-triggered actor-critic on measured descending activity; paired PAM/PPL feedback"})
    }

    fn value(&self, features: &[f32]) -> f32 {
        self.value_weights
            .iter()
            .zip(features)
            .map(|(w, x)| w * x)
            .sum::<f32>()
    }

    fn train_rollout(&mut self, bootstrap: f32, config: &crate::operant::OperantConfig) {
        if self.rollout.is_empty() {
            return;
        }
        if self.value_weights.is_empty() {
            self.value_weights = vec![0.0; WIDTH];
        }
        let mut actor = vec![0.0; WIDTH * 8];
        let mut critic = vec![0.0; WIDTH];
        let mut returns = bootstrap;
        // Batch gradients use the same policy/value parameters for every sample.
        // Bootstrapping lets delayed progress train earlier neutral transitions.
        for transition in self.rollout.iter().rev() {
            returns = transition.reward + config.discount * returns;
            let d = &transition.decision;
            let mut advantage = (returns - self.value(&d.features)).clamp(-5.0, 5.0);
            // Select is always an unwanted action in this game interface.
            // A pessimistic critic must not reinterpret its penalty as success.
            if d.action == 7 && transition.reward < 0.0 { advantage = advantage.min(transition.reward); }
            self.last_advantage = advantage;
            let norm = d.features.iter().map(|x| x * x).sum::<f32>().max(1.0);
            let entropy = -d
                .probabilities
                .iter()
                .map(|p| p * p.max(1e-8).ln())
                .sum::<f32>();
            for (j, &x) in d.features.iter().enumerate() {
                critic[j] += advantage * x / norm;
                for a in 0..8 {
                    let entropy_gradient =
                        -d.probabilities[a] * (d.probabilities[a].max(1e-8).ln() + entropy);
                    actor[a * WIDTH + j] += (-d.score_gradient[a] * advantage
                        + config.entropy_bonus * entropy_gradient)
                        * x
                        / norm;
                }
            }
        }
        let samples = self.rollout.len() as f32;
        for (w, g) in self.weights.iter_mut().zip(actor) {
            *w = (*w + config.readout_learning_rate * (g / samples).clamp(-1.0, 1.0))
                .clamp(-4.0, 4.0);
        }
        for (w, g) in self.value_weights.iter_mut().zip(critic) {
            *w = (*w + config.value_learning_rate * (g / samples).clamp(-1.0, 1.0))
                .clamp(-20.0, 20.0);
        }
        self.updates += 1;
        self.rollout.clear();
    }
}

impl Brain {
    pub(crate) fn ensure_wiring(&mut self) -> Result<(), String> {
        if self.wiring.is_some() {
            return Ok(());
        }
        let n = self.metadata.cells.len();
        let mut degree = vec![0u64; n];
        for pre in 0..n {
            for e in self.offsets[pre] as usize..self.offsets[pre + 1] as usize {
                if self.weights[e] != 0.0 {
                    degree[pre] += self.contacts[e] as u64;
                    degree[self.targets[e] as usize] += self.contacts[e] as u64;
                }
            }
        }
        let mut inputs: Vec<usize> = (0..n)
            .filter(|&i| {
                matches!(
                    self.metadata.cells[i].class.as_str(),
                    "visual_projection" | "cb_sensory"
                )
            })
            .filter(|&i| degree[i] > 0)
            .collect();
        inputs = diverse_population(&self.metadata.cells, inputs, &degree, WIDTH);
        // Explicit odor-like memory prosthesis: reserve a sparse sensory pool
        // in the model's actual plastic KC->MBON circuit. The prior sensory
        // selection could bypass these KCs entirely, leaving DAN pulses with
        // no eligible synapses. These cells encode observations, never buttons.
        inputs.truncate(WIDTH - 64);
        let mut memory: Vec<usize> = self.plastic.iter().map(|&(pre, _)| pre).collect();
        memory.sort_unstable();
        memory.dedup();
        let memory = diverse_population(&self.metadata.cells, memory, &degree, WIDTH);
        inputs.extend(memory.into_iter().take(64));
        // Direction is explicitly pre -> post. Zero-conductance transmitter
        // edges cannot establish a functioning route in this LIF model.
        let mut hops = vec![u16::MAX; n];
        let mut queue = VecDeque::new();
        for &i in &inputs {
            hops[i] = 0;
            queue.push_back(i);
        }
        while let Some(pre) = queue.pop_front() {
            if hops[pre] >= 12 {
                continue;
            }
            for e in self.offsets[pre] as usize..self.offsets[pre + 1] as usize {
                let post = self.targets[e] as usize;
                if self.weights[e] != 0.0 && hops[post] == u16::MAX {
                    hops[post] = hops[pre] + 1;
                    queue.push_back(post);
                }
            }
        }
        let mut outputs: Vec<usize> = (0..n)
            .filter(|&i| {
                self.metadata.cells[i].class == "descending_neuron"
                    && hops[i] != u16::MAX
                    && hops[i] > 0
            })
            .collect();
        outputs = diverse_population(&self.metadata.cells, outputs, &degree, n);
        if inputs.is_empty() || outputs.is_empty() {
            return Err("No conducting sensory-to-descending paths in this graph".into());
        }
        // Reverse reachability prunes inputs that cannot reach a selected output.
        // This temporary reverse index is discarded; simulation still uses only
        // the original forward CSR and retains all neurons and measured edges.
        let mut counts = vec![0usize; n + 1];
        for (e, &post) in self.targets.iter().enumerate() {
            if self.weights[e] != 0.0 {
                counts[post as usize + 1] += 1;
            }
        }
        for i in 1..=n {
            counts[i] += counts[i - 1];
        }
        let mut cursor = counts.clone();
        let mut sources = vec![0u32; counts[n]];
        for pre in 0..n {
            for e in self.offsets[pre] as usize..self.offsets[pre + 1] as usize {
                if self.weights[e] != 0.0 {
                    let post = self.targets[e] as usize;
                    sources[cursor[post]] = pre as u32;
                    cursor[post] += 1;
                }
            }
        }
        let mut reverse_hops = vec![u16::MAX; n];
        for &i in &outputs {
            reverse_hops[i] = 0;
            queue.push_back(i);
        }
        while let Some(post) = queue.pop_front() {
            if reverse_hops[post] >= 12 {
                continue;
            }
            for &pre in &sources[counts[post]..counts[post + 1]] {
                let pre = pre as usize;
                if reverse_hops[pre] == u16::MAX {
                    reverse_hops[pre] = reverse_hops[post] + 1;
                    queue.push_back(pre);
                }
            }
        }
        inputs.retain(|&i| reverse_hops[i] != u16::MAX);
        self.wiring = Some(Wiring {
            inputs,
            hops: outputs.iter().map(|&i| hops[i]).collect(),
            outputs,
        });
        Ok(())
    }

    pub(crate) fn circuit_decide(&mut self, json: &str) -> Result<String, String> {
        if self.circuit.pending.is_some() {
            // Preview-only decisions and failed game submissions have no reward.
            // Discard them rather than attributing a later outcome to them.
            self.circuit.interrupt();
        }
        let observation: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let config = self
            .config
            .operant
            .clone()
            .ok_or("Controller is not configured")?;
        self.ensure_wiring()?;
        let mut features = crate::interface::sensory_features(&observation);
        features.extend(self.circuit.breadcrumbs.cues(&observation));
        let wiring = self.wiring.as_ref().unwrap();
        let story_channels: Vec<usize> = features.iter().filter(|f|f.starts_with("story-scent:")).map(|f|sensory_channel(f)).collect();
        // Signed feature projection preserves information when many features
        // address the same sensory population. A binary union would eventually
        // light every input identically and erase distinctions between scenes.
        let mut projection = vec![0.0f32; wiring.inputs.len()];
        let mut density = vec![0.0f32; wiring.inputs.len()];
        for feature in &features {
            let mut hash = 0xcbf29ce484222325u64;
            for byte in feature.bytes() {
                hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
            }
            for k in 0..4u64 {
                let mut mixed = hash.wrapping_add(k.wrapping_mul(0x9e3779b97f4a7c15));
                mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                mixed ^= mixed >> 27;
                let channels = wiring.inputs.len().min(8);
                let channel = sensory_channel(feature) % channels;
                let slots = (wiring.inputs.len() - 1 - channel) / channels + 1;
                let slot = channel + (mixed as usize % slots) * channels;
                projection[slot] += if mixed & (1 << 63) == 0 { 1.0 } else { -1.0 };
                density[slot] += 1.0;
            }
        }
        let currents: Vec<(u32, f32)> = wiring
            .inputs
            .iter()
            .enumerate()
            .map(|(slot, &i)| {
                let signal = (projection[slot] / density[slot].max(1.0).sqrt()).tanh();
                let drive = if story_channels.is_empty() { signal.max(0.0) } else {
                    0.25 * signal.max(0.0) + if story_channels.contains(&(slot % 8)) {1.5} else {0.0}
                };
                (i as u32, self.config.stimulus_mv * drive)
            })
            .collect();
        let indices: Vec<u32> = currents
            .iter()
            .filter(|(_, mv)| *mv > 0.0)
            .map(|(i, _)| *i)
            .collect();
        let wiring_report = json!({"input_indices":wiring.inputs,"readout_indices":wiring.outputs,
            "memory_sensory_neurons":wiring.inputs.iter().filter(|&&i|self.kc[i]).count(),
            "readout_min_hops":wiring.hops,"propagation":"pre_to_post","input_readout_disjoint":true,
            "readout_features":WIDTH,"readout_pooling":"type-side-round-robin-sqrt-normalized-v1"});
        // One shared sensory pass; no candidate button cues, output tonic drive,
        // dopamine pulses or plasticity during inference. Membrane/synaptic
        // state persists between decisions for temporal sensory integration;
        // explicit interventions reset it and clear pending training transitions.
        self.pulse_ticks = 0;
        self.reward_ticks = 0;
        self.inject_currents(&serde_json::to_string(&currents).map_err(|e| e.to_string())?)?;
        let learning = self.config.learning;
        self.config.learning = false;
        let result = self.advance(WINDOW_MS);
        self.config.learning = learning;
        result?;
        let wiring = self.wiring.as_ref().unwrap();
        let mut signals = Vec::with_capacity(wiring.outputs.len());
        let mut activity = Vec::new();
        for (slot, &i) in wiring.outputs.iter().enumerate() {
            let voltage = (self.voltage[i] - self.config.rest_mv)
                / (self.config.threshold_mv - self.config.rest_mv);
            let signal = 0.5 * (self.counts[i] as f32 / 4.0).tanh() + 0.5 * voltage.tanh();
            signals.push(signal);
            activity.push(json!({"index":i,"source_id":self.metadata.cells[i].id,
                "spikes":self.counts[i],"voltage_mv":self.voltage[i],"signal":signal,"pool":slot % WIDTH}));
        }
        // Preserve the direction of measured activity while making learning
        // independent of whether downstream responses are subthreshold.
        // Exactly silent readouts remain silent; no bias or fabricated spikes.
        let readout = pool_readout(signals.into_iter());
        let learning_enabled =
            self.config.learning && self.config.rewards.enabled && config.teacher_enabled;
        // Context is an explicit engineered adapter input. Each head still
        // learns all button scores exclusively from measured neural activity.
        // Menu/intro rewards must not train walking into an A-spamming habit.
        let screen=observation.pointer("/status/screen").and_then(Value::as_str).unwrap_or("unknown");
        let context=if screen!="overworld" {screen.to_owned()}
            else if let Some(kind)=observation.pointer("/observe/menus/0/kind").and_then(Value::as_str) {format!("menu:{kind}")}
            else if observation.pointer("/observe/visible_dialogue").and_then(Value::as_str).is_some_and(|s|!s.is_empty()) {"dialogue".into()}
            else {format!("walk:{}",observation.pointer("/map_info/name").and_then(Value::as_str).unwrap_or("unknown"))};
        self.circuit.activate_context(context,&config,learning_enabled);
        if self.circuit.weights.is_empty() {
            self.circuit.weights = vec![0.0; WIDTH * 8];
        }
        if learning_enabled && self.circuit.rollout.len() >= 8 {
            self.circuit
                .train_rollout(self.circuit.value(&readout), &config);
        } else if !learning_enabled {
            self.circuit.rollout.clear();
        }
        self.circuit.last_value = self.circuit.value(&readout);
        let logits: Vec<f32> = self
            .circuit
            .weights
            .chunks_exact(WIDTH)
            .map(|row| row.iter().zip(&readout).map(|(w, x)| w * x).sum::<f32>())
            .collect();
        let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut policy: Vec<f32> = logits
            .iter()
            .enumerate()
            .map(|(i, x)| {
                if i == 7 && !config.allow_select {
                    0.0
                } else {
                    (x - max).exp()
                }
            })
            .collect();
        let total: f32 = policy.iter().sum();
        for p in &mut policy {
            *p /= total;
        }
        let probabilities: Vec<f32> = policy
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if i == 7 && !config.allow_select {
                    0.0
                } else {
                    (1.0 - config.exploration as f32) * p
                        + config.exploration as f32 / if config.allow_select { 8.0 } else { 7.0 }
                }
            })
            .collect();
        self.circuit.last_entropy = -policy.iter().map(|p| p * p.max(1e-8).ln()).sum::<f32>();
        let policy_probabilities = policy.clone();
        let mut seed = self.circuit.decisions.wrapping_add(0x9e3779b97f4a7c15);
        seed = (seed ^ (seed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        seed = (seed ^ (seed >> 27)).wrapping_mul(0x94d049bb133111eb);
        seed ^= seed >> 31;
        let mut sample = (seed >> 11) as f64 / 9007199254740992.0;
        let mut selected = if config.allow_select {7} else {6};
        for (i, &p) in probabilities.iter().enumerate() {
            sample -= p as f64;
            if sample < 0.0 {
                selected = i;
                break;
            }
        }
        // Correct the score gradient for the explicit exploration mixture.
        let correction = (1.0 - config.exploration as f32) * policy[selected]
            / probabilities[selected].max(1e-8);
        for p in &mut policy {
            *p *= correction;
        }
        policy[selected] -= correction;
        self.circuit.pending = Some(Decision {
            before: observation,
            features: readout,
            score_gradient: policy,
            action: selected,
            probabilities: policy_probabilities,
        });
        self.circuit.decisions += 1;
        let readouts: Vec<Value> = BUTTONS.iter().enumerate().map(|(i, button)|
            json!({"button":button,"probability":probabilities[i],"score":logits[i],"learned_response":logits[i]})).collect();
        Ok(json!({"action":{"button":BUTTONS[selected],"frames":config.frames,"readouts":readouts,
                "mapping":"measured sensory-to-descending activity; learned linear policy adapter"},
            "value":self.circuit.last_value,"entropy":self.circuit.last_entropy,
            "sensory":{"encoding":"partitioned-connected-sensory-v2","features":features,"indices":indices,
                "currents":currents,"wiring":wiring_report,"readout_activity":activity,"probe_learning":false},
            "telemetry":serde_json::from_str::<Value>(&self.summary()?).map_err(|e|e.to_string())?,
            "neural_trial_ms":self.circuit.decisions * WINDOW_MS as u64}).to_string())
    }

    pub(crate) fn circuit_feedback(&mut self, json: &str) -> Result<String, String> {
        let after: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let pending = self
            .circuit
            .pending
            .take()
            .ok_or("No neural action awaiting outcome")?;
        let (mut rewards, aversions) =
            self.circuit
                .story
                .action_feedback(&pending.before, &after, BUTTONS[pending.action]);
        if let Some(crumb) =
            self.circuit
                .breadcrumbs
                .feedback(&pending.before, &after, BUTTONS[pending.action])
        {
            if aversions.is_empty() {
                rewards.push(crumb);
            }
        }
        let feedback_config = self
            .config
            .operant
            .clone()
            .ok_or("Controller is not configured")?;
        let outcome: f32 = if aversions.iter().any(|e| e.starts_with("battle:")) {
            -2.0
        } else if aversions.iter().any(|e| e.starts_with("action:")) {
            -feedback_config.bad_action_penalty
        } else if aversions.iter().any(|e|e=="inaction:unneeded_menu") {
            -0.25
        } else if !aversions.is_empty() {
            -0.05
        } else if rewards
            .iter()
            .any(|e| e.starts_with("story:") || e.starts_with("badge:"))
        {
            2.0
        } else if rewards.iter().any(|e| e.starts_with("place:")) {
            2.5
        } else if rewards.iter().any(|e|e.starts_with("breadcrumb:recover:")) {
            0.3
        } else if rewards.iter().any(|e| e.starts_with("breadcrumb:")) {
            feedback_config.breadcrumb_reward
        } else if rewards.iter().all(|e| e.starts_with("exploration:")) && !rewards.is_empty() {
            0.05
        } else if !rewards.is_empty() {
            0.5
        } else {
            0.0
        };
        let enabled = self.config.learning
            && self.config.rewards.enabled
            && self
                .config
                .operant
                .as_ref()
                .is_some_and(|c| c.teacher_enabled);
        let terminal = aversions.iter().any(|e| e.starts_with("battle:"))
            || rewards.iter().any(|e| {
                matches!(
                    e.as_str(),
                    "battle:victory" | "battle:capture" | "story:EVENT_BEAT_RED"
                )
            });
        let action = pending.action;
        // Pair the still-active sensory trace with real compartment-specific
        // DAN stimulation. Polarity is circuit identity, never a weight sign.
        let mut training = Value::Null;
        let pulse_ms = if outcome > 0.0 {
            feedback_config.appetitive_pulse_ms
        } else {
            feedback_config.aversive_pulse_ms
        } * if outcome.abs() >= 2.0 { 1.0 } else { 0.25 };
        if enabled && outcome != 0.0 {
            if outcome > 0.0 {
                self.appetitive_pulse()?;
                self.reward_ticks = (pulse_ms / self.config.dt_ms).round() as u64;
            } else {
                self.aversive_pulse()?;
                self.pulse_ticks = (pulse_ms / self.config.dt_ms).round() as u64;
            }
            self.advance(pulse_ms)?;
            training = json!({"population":if outcome > 0.0 {"PAM01"} else {"PPL101"},
                "pulse_ms":pulse_ms,"telemetry":serde_json::from_str::<Value>(&self.summary()?).map_err(|e|e.to_string())?});
        }
        if enabled {
            let config = self.config.operant.as_ref().unwrap();
            self.circuit.rollout.push(Transition {
                decision: pending,
                reward: outcome,
            });
            if terminal || outcome != 0.0 {
                let bootstrap = if terminal {
                    0.0
                } else {
                    self.circuit.last_value
                };
                self.circuit.train_rollout(bootstrap, config);
            }
        } else {
            self.circuit.rollout.clear();
        }
        let event = json!({"decision":self.circuit.decisions,"button":BUTTONS[action],
            "reward":if rewards.is_empty(){Value::Null}else{json!(rewards.join("; "))},
            "aversion":if aversions.is_empty(){Value::Null}else{json!(aversions.join("; "))},
            "outcome":outcome,"enabled":enabled,"teacher_id":"story-events-v1",
            "conditioning_pairings":if enabled && outcome != 0.0 {1} else {0},"learning":"actor-critic + paired DAN stimulation","terminal":terminal,
            "value":self.circuit.last_value,"advantage":self.circuit.last_advantage,"goal_reached":false});
        if self.circuit.recent.len() >= 64 {
            self.circuit.recent.remove(0);
        }
        self.circuit.recent.push(event.clone());
        Ok(
            json!({"event":event,"events":rewards.into_iter().chain(aversions).collect::<Vec<_>>(),
            "enabled":enabled,"goal_reached":false,"story":self.circuit.story.report(),
            "breadcrumb":self.circuit.breadcrumbs.current,"training":training,
            "neural_trial_ms":self.circuit.decisions * WINDOW_MS as u64})
            .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires external MaleCNS data: FLYGON_TEST_DATA"]
    fn full_graph_pooled_coverage_and_timed_activity() {
        let root = std::path::PathBuf::from(std::env::var("FLYGON_TEST_DATA").unwrap());
        let graph = std::fs::read(root.join("graph.bin")).unwrap();
        let metadata = std::fs::read_to_string(root.join("metadata.json")).unwrap();
        let mut brain = Brain::new(
            &graph,
            &metadata,
            include_str!("../../../modpacks/flygon/operant.json"),
        )
        .unwrap();
        drop(graph);
        drop(metadata);
        brain.ensure_wiring().unwrap();
        let w = brain.wiring.as_ref().unwrap();
        assert!(w.outputs.len() > WIDTH);
        let unique: std::collections::BTreeSet<_> = w.outputs.iter().copied().collect();
        assert_eq!(unique.len(), w.outputs.len());
        assert!(
            w.outputs
                .iter()
                .all(|&i| brain.metadata.cells[i].class == "descending_neuron")
        );
        assert!(w.inputs.iter().all(|i| !unique.contains(i)));
        assert!(w.hops.iter().all(|&hop| hop > 0 && hop <= 12));
        let output_count = w.outputs.len();
        // Controlled broad sensory stimulus, not a gameplay success claim.
        let inputs: Vec<u32> = w.inputs.iter().map(|&i| i as u32).collect();
        brain.stimulate(&inputs, 30.0).unwrap();
        brain.advance(150.0).unwrap();
        let before = brain.checkpoint().unwrap();
        let samples = brain.view_timed_spikes();
        let counts = brain.view_spikes();
        assert!(!samples.is_empty());
        assert_eq!(samples.len() / 3, counts.len() / 2);
        for (timed, count) in samples.chunks_exact(3).zip(counts.chunks_exact(2)) {
            assert_eq!(&timed[..2], count);
            assert!(timed[2] <= 1_000_000);
        }
        let mut view = crate::view::AnatomyView::new(&brain.view_anatomy()).unwrap();
        view.update_timed_spikes(&samples).unwrap();
        view.update_activity_connections(&brain.view_activity_connections().unwrap())
            .unwrap();
        view.animate(0.0, true).unwrap();
        let early = view.render(320, 240, 0.0, 0.0, 1.0, "all", "{}").unwrap();
        view.animate(0.8, true).unwrap();
        assert_ne!(
            early,
            view.render(320, 240, 0.0, 0.0, 1.0, "all", "{}").unwrap()
        );
        assert_eq!(
            before,
            brain.checkpoint().unwrap(),
            "rendering cannot change neural state"
        );
        brain.advance(50.0).unwrap();
        let continuation = brain.checkpoint().unwrap();
        brain.restore(&before).unwrap();
        brain.advance(50.0).unwrap();
        assert_eq!(continuation, brain.checkpoint().unwrap());
        println!(
            "reachable descending readouts={output_count}; pooled features={WIDTH}; spiking neurons={}; full-graph rendering and exact continuation passed",
            samples.len() / 3
        );
        // Eight disjoint sensory channels, followed by a no-transmission control.
        // All start from rest with learning off; no rewards or policy bypass.
        brain.config.learning = false;
        let readout = |b: &Brain| {
            pool_readout(b.wiring.as_ref().unwrap().outputs.iter().map(|&i| {
                0.5 * (b.counts[i] as f32 / 4.0).tanh()
                    + 0.5
                        * ((b.voltage[i] - b.config.rest_mv)
                            / (b.config.threshold_mv - b.config.rest_mv))
                            .tanh()
            }))
        };
        brain.reset_dynamics();
        brain.advance(150.0).unwrap();
        assert!(readout(&brain).iter().all(|&x| x == 0.0));
        let mut responses = Vec::new();
        for channel in 0..8 {
            brain.reset_dynamics();
            let cue: Vec<u32> = inputs.iter().skip(channel).step_by(8).copied().collect();
            brain.stimulate(&cue, 30.0).unwrap();
            brain.advance(150.0).unwrap();
            let response = readout(&brain);
            assert!(response.iter().any(|x| x.abs() > 0.01));
            responses.push(response);
        }
        let max_similarity = (0..8)
            .flat_map(|a| (a + 1..8).map(move |b| (a, b)))
            .map(|(a, b)| {
                responses[a]
                    .iter()
                    .zip(&responses[b])
                    .map(|(x, y)| x * y)
                    .sum::<f32>()
            })
            .fold(-1.0f32, f32::max);
        assert!(
            max_similarity < 0.99,
            "sensory channels must not collapse: {max_similarity}"
        );
        brain.reset_dynamics();
        brain.weights.fill(0.0); // test-only lesion; external graph is never written
        brain.stimulate(&inputs, 30.0).unwrap();
        brain.advance(150.0).unwrap();
        assert!(
            brain.counts.iter().any(|&n| n > 0),
            "sensory stimulus still fires"
        );
        assert!(
            readout(&brain).iter().all(|&x| x == 0.0),
            "readout requires graph transmission"
        );
        println!(
            "eight sensory channels: maximum pairwise cosine={max_similarity}; quiet and connection-ablation controls passed"
        );
    }
    #[test]
    fn pooled_readout_uses_cells_beyond_the_old_cutoff() {
        for source in [0, WIDTH - 1, WIDTH, WIDTH * 3 + 7] {
            let mut signals = vec![0.0; WIDTH * 4];
            signals[source] = 0.4;
            let readout = pool_readout(signals.into_iter());
            assert_eq!(readout[source % WIDTH], 1.0);
            assert_eq!(readout.iter().filter(|&&x| x != 0.0).count(), 1);
        }
        assert_eq!(
            pool_readout(std::iter::repeat_n(0.0, WIDTH * 4)),
            vec![0.0; WIDTH]
        );
        let readout = pool_readout((0..WIDTH * 3).map(|i| (i as f32).sin()));
        assert!((readout.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-5);
        assert!(readout.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
    }
    fn transition(action: usize, reward: f32) -> Transition {
        let mut gradient = vec![0.125; 8];
        gradient[action] -= 1.0;
        Transition {
            decision: Decision {
                before: json!({}),
                features: vec![0.5; WIDTH],
                score_gradient: gradient,
                action,
                probabilities: vec![0.125; 8],
            },
            reward,
        }
    }
    #[test]
    fn actor_critic_learns_outcome_sign_and_delayed_credit() {
        let config = crate::operant::OperantConfig::default();
        for reward in [-2.0, 2.0] {
            let mut state = CircuitState {
                weights: vec![0.0; WIDTH * 8],
                rollout: vec![transition(7, 0.0), transition(7, reward)],
                ..Default::default()
            };
            state.train_rollout(0.0, &config);
            let logits: Vec<f32> = state
                .weights
                .chunks_exact(WIDTH)
                .map(|row| row.iter().sum::<f32>() * 0.5)
                .collect();
            assert!((logits[7] - logits[0]) * reward > 0.0);
            assert!(state.value(&vec![0.5; WIDTH]) * reward > 0.0);
            assert!(
                state.last_advantage * reward > 0.0,
                "earlier neutral transition receives credit"
            );
            assert!(state.rollout.is_empty() && state.valid());
        }
    }
    #[test]
    fn interruption_discards_pending_credit_without_erasing_learning() {
        let mut state = CircuitState {
            weights: vec![0.2; WIDTH * 8],
            rollout: vec![transition(0, 2.0)],
            ..Default::default()
        };
        state.interrupt();
        state.train_rollout(0.0, &crate::operant::OperantConfig::default());
        assert_eq!(state.updates, 0);
        assert!(state.weights.iter().all(|w| *w == 0.2));
    }
}
