//! Sensory -> measured recurrent graph -> descending population -> policy adapter.
//! The adapter is engineered online learning, not an anatomical synapse or a
//! biological claim. Observations and explicit story scents drive sensory cells;
//! no candidate button is injected into the graph.
use crate::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
const BUTTONS: [&str; 8] = ["up", "down", "left", "right", "a", "b", "start", "select"];
const WIDTH: usize = 256;

// Actual navigation progress supersedes accumulated inactivity, but never a
// collision, loop, refusal or battle penalty from the action being evaluated.
fn merge_navigation_feedback(rewards: &mut Vec<String>, aversions: &mut Vec<String>, crumb: String) {
    aversions.retain(|event| !matches!(event.as_str(), "inaction:no_progress" | "inaction:stalled"));
    if aversions.is_empty() {
        rewards.push(crumb);
    }
}

// A signed distance change already charges a detour and every movement.
// Penalizing the recovery leg again as a short loop teaches the wrong direction.
fn merge_distance_feedback(aversions: &mut Vec<String>, progress: f32) {
    if progress != 0.0 {
        aversions.retain(|e| e != "action:short_loop");
    }
    if progress > 0.0 {
        aversions.retain(|e| !matches!(e.as_str(), "inaction:no_progress" | "inaction:stalled"));
    }
}

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
    heads: BTreeMap<String, (Vec<f32>, Vec<f32>)>,
    rollout: Vec<Transition>,
    last_value: f32,
    last_advantage: f32,
    last_entropy: f32,
    decisions: u64,
    #[serde(default = "crate::operant::default_sensory_window")]
    decision_window_ms: u32,
    updates: u64,
    pending: Option<Decision>,
    story: crate::story::StoryLedger,
    recent: Vec<Value>,
    breadcrumbs: crate::breadcrumbs::Breadcrumbs,
    sensory_history: crate::sensory::History,
    exploration_pressure: f32,
}
#[derive(Clone, Serialize, Deserialize)]
struct Decision {
    before: Value,
    features: Vec<f32>,
    score_gradient: Vec<f32>,
    action: usize,
    #[serde(default)]
    probabilities: Vec<f32>,
    #[serde(default = "unit_temperature")]
    temperature: f32,
}
#[derive(Serialize, Deserialize)]
struct Transition {
    decision: Decision,
    reward: f32,
}

// Repeated observed states temporarily soften an entrenched actor. This changes
// sampling only: measured logits and saved learned weights remain intact, and
// novel states immediately restore the original distribution.
fn unit_temperature() -> f32 { 1.0 }

fn loop_temperature(pressure: f32) -> f32 {
    1.0 + 4.0 * pressure.clamp(0.0, 1.0)
}

// Avoid selecting only the highest-degree cells of one type or hemisphere.
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

fn diverse_population(cells: &[Cell], candidates: Vec<usize>, degree: &[u64], limit: usize) -> Vec<usize> {
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

fn has_visible_message(observation: &Value) -> bool {
    ["/observe/visible_dialogue", "/observe/battle_message"].iter().any(|path| {
        observation.pointer(path).and_then(Value::as_str).is_some_and(|s| !s.trim().is_empty())
    })
}

fn start_cooldown_blocks(v:&Value,cooldown:u64)->bool {
    (cooldown>0 || (matches!(crate::curriculum::goal(v),Some(crate::curriculum::Goal::Exit(_) | crate::curriculum::Goal::Talk(_,_) | crate::curriculum::Goal::Visit(_,_))) && crate::objectives::idle_field_pack(v)))
        && v.pointer("/status/screen").and_then(Value::as_str)==Some("overworld")
        && v.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|m|m.is_empty())
        && crate::field_objectives::teaching_target(v).is_none()
}
fn action_mask(observation: &Value, allow_select: bool) -> Vec<bool> {
    let screen = observation
        .pointer("/status/screen")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    // The opening errands require dialogue and walking, not the Start menu.
    // Leave setup controls intact and restore field menu access after Elm
    // receives the egg, before later field-move and party-management tasks.
    let opening_errand = screen == "overworld"
        && !observation
            .pointer("/reward_state/event_flags")
            .and_then(Value::as_array)
            .is_some_and(|flags| {
                flags
                    .iter()
                    .any(|f| f.as_str() == Some("EVENT_GAVE_MYSTERY_EGG_TO_ELM"))
            });
    let dialogue_page = matches!(screen, "overworld" | "battle")
        && has_visible_message(observation)
        && observation
            .pointer("/observe/menus")
            .and_then(Value::as_array)
            .is_some_and(|menus| menus.is_empty());
    let picture = observation.pointer("/observe/menus").and_then(Value::as_array)
        .is_some_and(|menus| menus.iter().any(|m| m["kind"] == "pokemon_picture"));
    let allowed: Vec<bool> = (0..8)
        .map(|i| {
            !(i == 7 && !allow_select
                || i == 6 && (opening_errand || screen == "battle")
                || (dialogue_page || picture) && i != 4 && i != 5
                || i == 4 && !has_visible_message(observation) && crate::objectives::trainer_ball_selected(observation))
        })
        .collect();

    allowed
}
// Transfer only an explicit relative menu objective, never a guessed answer,
// an ordinary combat move, navigation, or a story question's text.
fn relative_menu_objective(context: &str) -> Option<Vec<&'static str>> {
    if !["menu:", "objective:hm:", "objective:pc:"].iter().any(|p| context.starts_with(p)) {
        return None;
    }
    let directions = ["aligned", "north", "south", "west", "east"].into_iter().filter(|direction| {
        ["objective:menu:", "objective:hm:cursor:", "objective:pc:cursor:"].iter().any(|prefix| {
            let token = format!("{prefix}{direction}");
            context.match_indices(&token).any(|(i, _)| context[i + token.len()..].chars().next()
                .is_none_or(|c| c == '|' || c == ':'))
        }) || matches!(*direction, "west" | "east") && context.split("objective:pocket:").skip(1).any(|suffix| {
            // A tab already being aligned does not mean its selected item
            // should be confirmed. Transfer only actual horizontal tab motion.
            suffix.split(':').nth(1).is_some_and(|part| part.split('|').next() == Some(*direction))
        })
    }).collect::<Vec<_>>();
    (!directions.is_empty()).then_some(directions)
}

impl CircuitState {
    fn menu_actor_prior(&self, context: &str) -> Vec<f32> {
        let Some(objective) = relative_menu_objective(context) else { return Vec::new(); };
        let mut prior = vec![0.0; WIDTH * 8];
        let mut donors = 0usize;
        for (key, (actor, _)) in &self.heads {
            if relative_menu_objective(key).as_ref() != Some(&objective)
                || actor.len() != prior.len() || !actor.iter().any(|w| *w != 0.0) { continue; }
            donors += 1;
            // An online mean stays within the existing actor bounds.
            for (mean, weight) in prior.iter_mut().zip(actor) {
                *mean += (*weight - *mean) / donors as f32;
            }
        }
        if donors == 0 { Vec::new() } else { prior }
    }

    fn activate_context(
        &mut self,
        context: String,
        config: &crate::operant::OperantConfig,
        learning: bool,
        next_features: &[f32],
    ) {
        if self.context == context {
            return;
        }
        if learning {
            // Credit belongs to the outgoing action, but its bootstrap is the
            // incoming state. Reusing V(old) rewards/penalizes menu transitions
            // against the wrong continuation. Unseen critics start at zero.
            let bootstrap = self.heads.get(&context).map(|(_, critic)| critic.iter()
                .zip(next_features).map(|(w, x)| w * x).sum()).unwrap_or(0.0);
            self.train_rollout(bootstrap, config);
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
        // Existing heads always win. A new interaction may borrow learned actor
        // associations, but its expected return must be learned independently.
        let (actor, critic) = self.heads.remove(&context)
            .unwrap_or_else(|| (self.menu_actor_prior(&context), Vec::new()));
        self.weights = actor;
        self.value_weights = critic;
        self.context = context;
        self.last_value = 0.0;
    }
    pub(crate) fn updates(&self) -> u64 {
        self.updates
    }
    pub(crate) fn valid(&self) -> bool {
        self.sensory_history.valid()
            && ((self.decisions == 0 && self.decision_window_ms == 0) || (50..=300).contains(&self.decision_window_ms))
            && self.exploration_pressure.is_finite()
            && (0.0..=1.0).contains(&self.exploration_pressure)
            && self.heads.len() <= 4096
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
                    && p.temperature.is_finite() && (1.0..=5.0).contains(&p.temperature)
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
                    && t.decision.temperature.is_finite() && (1.0..=5.0).contains(&t.decision.temperature)
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
        self.sensory_history = Default::default();
        self.exploration_pressure = 0.0;
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
    pub(crate) fn upgrade_readout(&mut self) {
        self.erase();
        self.context.clear();
        self.last_entropy = 0.0;
    }
    pub(crate) fn report(&self) -> Value {
        json!({"interface":INTERFACE_ID, "decisions":self.decisions,
            "updates":self.updates,"neural_trial_ms":self.decisions * self.decision_window_ms as u64,
            "story":self.story.report(),"recent":self.recent,
            "value":self.last_value,"advantage":self.last_advantage,"entropy":self.last_entropy,
            "pending_transitions":self.rollout.len(),
            "learning_context":self.context,"stored_contexts":self.heads.len(),
            "breadcrumb":self.breadcrumbs.current,"exploration_pressure":self.exploration_pressure,
            "learning":"event-triggered actor-critic on measured descending activity; paired PAM/PPL feedback"})
    }

    fn value(&self, features: &[f32]) -> f32 {
        self.value_weights
            .iter()
            .zip(features)
            .map(|(w, x)| w * x)
            .sum::<f32>()
    }

    fn record_outcome(&mut self, decision: Decision, reward: f32, terminal: bool,
        config: &crate::operant::OperantConfig) {
        // Earlier neutral actions end at this decision's state; isolate an
        // immediate error so its penalty cannot propagate into that prefix.
        if reward < 0.0 && !self.rollout.is_empty() {
            self.train_rollout(self.last_value, config);
        }
        self.rollout.push(Transition { decision, reward });
        // V(after) arrives with the next decision's sensory readout. Using
        // last_value here would substitute V(before) for every menu reward.
        if terminal { self.train_rollout(0.0, config); }
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
        for (offset, transition) in self.rollout.iter().rev().enumerate() {
            returns = transition.reward + config.discount * returns;
            let d = &transition.decision;
            let td_error = (returns - self.value(&d.features)).clamp(-5.0, 5.0);
            let mut advantage = td_error;
            // Explicit aversion must reduce the sampled action, even when a
            // pessimistic critic regards another collision as better than usual.
            if transition.reward < 0.0 {
                advantage = advantage.min(transition.reward);
            }
            if offset == 0 {
                self.last_advantage = advantage;
            }
            let norm = d.features.iter().map(|x| x * x).sum::<f32>().max(1.0);
            let entropy = -d
                .probabilities
                .iter()
                .map(|p| p * p.max(1e-8).ln())
                .sum::<f32>();
            for (j, &x) in d.features.iter().enumerate() {
                // The critic estimates return, not the conservative action penalty.
                // Forcing negative TD errors here makes pessimism self-perpetuating.
                critic[j] += td_error * x / norm;
                for a in 0..8 {
                    let entropy_gradient =
                        -d.probabilities[a] * (d.probabilities[a].max(1e-8).ln() + entropy);
                    actor[a * WIDTH + j] += (-d.score_gradient[a] * advantage
                        + config.entropy_bonus * entropy_gradient / d.temperature)
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
        let input_budget = self.config.operant.as_ref().map_or(256, |c| c.sensory_neurons);
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
        inputs = diverse_population(&self.metadata.cells, inputs, &degree, input_budget);
        // Explicit odor-like memory prosthesis: reserve a sparse sensory pool
        // in the model's actual plastic KC->MBON circuit. The prior sensory
        // selection could bypass these KCs entirely, leaving DAN pulses with
        // no eligible synapses. These cells encode observations, never buttons.
        inputs.truncate(input_budget - 64);
        let mut memory: Vec<usize> = self.plastic.iter().map(|&(pre, _)| pre).collect();
        memory.sort_unstable();
        memory.dedup();
        let memory = diverse_population(&self.metadata.cells, memory, &degree, 64);
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
        let mut features = crate::interface::sensory_features_with_menus(&observation, config.visible_menu_inputs);
        features.extend(self.circuit.breadcrumbs.cues(&observation));
        features.extend(crate::objectives::cues(&observation));
        features.extend(crate::shop_objectives::cues(&observation));
        features.extend(crate::choice_objectives::cues(&observation));
        features.extend(crate::field_objectives::cues(&observation));
        let (history_cues, pressure) = self.circuit.sensory_history.cues(&observation);
        features.extend(history_cues);
        self.circuit.exploration_pressure = pressure;
        let wiring = self.wiring.as_ref().unwrap();
        let drive = crate::sensory::encode(&features, wiring.inputs.len());
        let currents: Vec<(u32, f32)> = wiring
            .inputs
            .iter()
            .zip(&drive)
            .map(|(&i, &signal)| (i as u32, self.config.stimulus_mv * signal))
            .collect();
        let indices: Vec<u32> = currents
            .iter()
            .filter(|(_, mv)| *mv > 0.0)
            .map(|(i, _)| *i)
            .collect();
        let wiring_report = json!({"input_indices":wiring.inputs,"readout_indices":wiring.outputs,
            "memory_sensory_neurons":wiring.inputs.iter().filter(|&&i|self.kc[i]).count(),
            "readout_min_hops":wiring.hops,"propagation":"pre_to_post","input_readout_disjoint":true,"readout_features":WIDTH,"readout_pooling":"type-side-round-robin-sqrt-normalized-v1"});
        // One shared sensory pass; no candidate button cues, output tonic drive,
        // dopamine pulses or plasticity during inference. Membrane/synaptic
        // state persists between decisions for temporal sensory integration;
        // explicit interventions reset it and clear pending training transitions.
        self.pulse_ticks = 0;
        self.reward_ticks = 0;
        self.inject_currents(&serde_json::to_string(&currents).map_err(|e| e.to_string())?)?;
        let learning = self.config.learning;
        self.config.learning = false;
        let result = self.advance(config.sensory_window_ms as f32);
        self.config.learning = learning;
        result?;
        let wiring = self.wiring.as_ref().unwrap();
        let mut signals = Vec::with_capacity(wiring.outputs.len());
        let mut activity = Vec::new();
        for (slot, &i) in wiring.outputs.iter().enumerate() {
            let voltage = (self.voltage[i] - self.config.rest_mv)
                / (self.config.threshold_mv - self.config.rest_mv);
            let signal = 0.5 * (self.counts[i] as f32 * 150.0 / config.sensory_window_ms as f32 / 4.0).tanh() + 0.5 * voltage.tanh();
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
        let screen = observation
            .pointer("/status/screen")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let context = if let Some(hm)=crate::field_objectives::context(&observation).filter(|_| crate::field_objectives::teaching_target(&observation).is_some() || observation.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|m|!m.is_empty())) {
            hm
        } else if let Some(shop) = crate::shop_objectives::context(&observation) {
            shop
        } else if let Some(choice) = crate::choice_objectives::context(&observation) {
            choice
        } else if let Some(menu) = crate::objectives::menu_context(&observation) {
            menu
        } else if screen != "overworld" {
            screen.to_owned()
        } else if let Some(kind) = observation
            .pointer("/observe/menus/0/kind")
            .and_then(Value::as_str)
        {
            format!("menu:{kind}")
        } else if has_visible_message(&observation) {
            "dialogue".into()
        } else {
            // Explicit sensory-context gating prevents incompatible routes
            // and changed story objectives from training the same habit.
            // Every head starts untrained; no direction-to-button mapping.
            let cue_prefix = if features.iter().any(|f| f.starts_with("story-scent:")) {
                "story-scent:"
            } else {
                "exploration-scent:"
            };
            let cues = features
                .iter()
                .filter(|f| f.starts_with("story-interaction:") || f.starts_with(cue_prefix))
                .map(|f| f.replace(cue_prefix, "scent:"))
                .collect::<Vec<_>>()
                .join("|");
            // Reuse acquired sensory/motor associations in a new map. The
            // story changes the scent, not the meaning of a learned direction.
            format!("navigation:{cues}")
        };
        self.circuit
            .activate_context(context, &config, learning_enabled, &readout);
        if self.circuit.weights.is_empty() {
            self.circuit.weights = vec![0.0; WIDTH * 8];
        }
        if learning_enabled && (self.circuit.rollout.len() >= 8
            || self.circuit.rollout.iter().any(|t| t.reward != 0.0)) {
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
        let mut allowed = action_mask(&observation, config.allow_select);
        if self.circuit.story.battle_pack_reopen_cooldown()>0 && crate::objectives::trainer_pack_selected(&observation) {
            allowed[4]=false;
        }
        // Give walking and completed interactions time to produce consequences
        // between menu openings. Menus stay usable, and B/Start can close one.
        if start_cooldown_blocks(&observation,self.circuit.story.menu_reopen_cooldown()) {
            allowed[6]=false;
        }
        let count = allowed.iter().filter(|a| **a).count() as f32;
        let temperature = loop_temperature(pressure);
        let max = logits
            .iter()
            .enumerate()
            .filter(|(i, _)| allowed[*i])
            .map(|(_, x)| *x)
            .fold(f32::NEG_INFINITY, f32::max);
        let mut policy: Vec<f32> = logits
            .iter()
            .enumerate()
            .map(|(i, x)| if !allowed[i] { 0.0 } else { ((x - max) / temperature).exp() })
            .collect();
        let total: f32 = policy.iter().sum();
        for p in &mut policy {
            *p /= total;
        }
        let exploration = (config.exploration as f32).max(0.35 * pressure);
        let probabilities: Vec<f32> = policy
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if !allowed[i] {
                    0.0
                } else {
                    (1.0 - exploration) * p + exploration / count
                }
            })
            .collect();
        self.circuit.last_entropy = -policy.iter().map(|p| p * p.max(1e-8).ln()).sum::<f32>();
        let policy_probabilities = policy.clone();
        let mut seed = (self.circuit.decisions ^ config.action_seed).wrapping_add(0x9e3779b97f4a7c15);
        seed = (seed ^ (seed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        seed = (seed ^ (seed >> 27)).wrapping_mul(0x94d049bb133111eb);
        seed ^= seed >> 31;
        let mut sample = (seed >> 11) as f64 / 9007199254740992.0;
        let mut selected = allowed.iter().rposition(|a| *a).unwrap();
        for (i, &p) in probabilities.iter().enumerate() {
            sample -= p as f64;
            if sample < 0.0 {
                selected = i;
                break;
            }
        }
        // Correct the score gradient for the explicit exploration mixture.
        let correction = (1.0 - exploration) * policy[selected]
            / (probabilities[selected].max(1e-8) * temperature);
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
            temperature,
        });
        self.circuit.decision_window_ms = config.sensory_window_ms;
        self.circuit.decisions += 1;
        let readouts: Vec<Value> = BUTTONS.iter().enumerate().map(|(i, button)|
            json!({"button":button,"probability":probabilities[i],"score":logits[i],"learned_response":logits[i]})).collect();
        Ok(json!({"action":{"button":BUTTONS[selected],"frames":config.frames,"readouts":readouts,
                "mapping":"measured sensory-to-descending activity; learned linear policy adapter"},
            "value":self.circuit.last_value,"entropy":self.circuit.last_entropy,"exploration":exploration,"exploration_pressure":pressure,"policy_temperature":temperature,
            "sensory":{"encoding":"opponent-multimodal-sensory-v4","features":features,"indices":indices,
                "currents":currents,"wiring":wiring_report,"readout_activity":activity,"probe_learning":false},
            "telemetry":serde_json::from_str::<Value>(&self.summary()?).map_err(|e|e.to_string())?,
            "neural_trial_ms":self.circuit.decisions * self.circuit.decision_window_ms as u64}).to_string())
    }

    pub(crate) fn circuit_feedback(&mut self, json: &str) -> Result<String, String> {
        let after: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let pending = self
            .circuit
            .pending
            .take()
            .ok_or("No neural action awaiting outcome")?;
        self.circuit
            .sensory_history
            .feedback(&pending.before, &after, BUTTONS[pending.action]);
        let (mut rewards, mut aversions) =
            self.circuit
                .story
                .action_feedback(&pending.before, &after, BUTTONS[pending.action]);
        if let Some(crumb) =
            self.circuit
                .breadcrumbs
                .feedback(&pending.before, &after, BUTTONS[pending.action])
        {
            merge_navigation_feedback(&mut rewards, &mut aversions, crumb);
        }
        let feedback_config = self
            .config
            .operant
            .clone()
            .ok_or("Controller is not configured")?;
        let field_progress = crate::field_objectives::feedback(&pending.before, &after);
        if field_progress > 0.0 { aversions.retain(|e| e != "inaction:unneeded_menu"); }
        let interaction_progress = crate::shop_objectives::feedback(&pending.before, &after, BUTTONS[pending.action]) + crate::curriculum::chase_progress(&pending.before, &after) + crate::objectives::feedback(&pending.before, &after, &mut rewards, &mut aversions) + field_progress + crate::choice_objectives::feedback(&pending.before, &after, BUTTONS[pending.action]);
        if crate::objectives::abandoned_capture(&pending.before,&after,BUTTONS[pending.action]) {
            aversions.push("battle:abandoned_capture".into());
        }
        let navigation_progress = self.circuit.breadcrumbs.progress;
        merge_distance_feedback(&mut aversions, navigation_progress);
        let outcome: f32 = if aversions.iter().any(|e| e.starts_with("battle:")) {
            -2.0
        } else if aversions.iter().any(|e| e.starts_with("action:")) {
            -feedback_config.bad_action_penalty
        } else if aversions.iter().any(|e| e == "inaction:unneeded_menu") {
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
        } else if rewards.iter().any(|e| e.starts_with("breadcrumb:recover:")) {
            0.3
        } else if rewards.iter().any(|e| e.starts_with("breadcrumb:")) {
            feedback_config.breadcrumb_reward
        } else if rewards.iter().all(|e| e.starts_with("exploration:")) && !rewards.is_empty() {
            0.05
        } else if rewards.iter().any(|e| e.starts_with("dialogue:complete:")) {
            1.0
        } else if rewards.iter().any(|e| e.starts_with("dialogue:")) {
            0.25
        } else if !rewards.is_empty() {
            0.5
        } else {
            0.0
        };
        // Keep genuine event rewards and error penalties; add signed progress
        // even when a previously visited approach edge has exhausted novelty.
        let outcome = outcome + navigation_progress + interaction_progress;
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
            self.circuit.record_outcome(pending, outcome, terminal, config);
        } else {
            self.circuit.rollout.clear();
        }
        let event = json!({"decision":self.circuit.decisions,"button":BUTTONS[action],
            "reward":if rewards.is_empty(){Value::Null}else{json!(rewards.join("; "))},
            "aversion":if aversions.is_empty(){Value::Null}else{json!(aversions.join("; "))},
            "outcome":outcome,"navigation_progress":navigation_progress,"interaction_progress":interaction_progress,"enabled":enabled,"teacher_id":"story-events-v1",
            "conditioning_pairings":if enabled && outcome != 0.0 {1} else {0},"learning":"actor-critic + paired DAN stimulation","terminal":terminal,
            "value":self.circuit.last_value,"advantage":if enabled && (terminal || outcome != 0.0) { json!(self.circuit.last_advantage) } else { Value::Null },"goal_reached":false});
        if self.circuit.recent.len() >= 64 {
            self.circuit.recent.remove(0);
        }
        self.circuit.recent.push(event.clone());
        Ok(
            json!({"event":event,"events":rewards.into_iter().chain(aversions).collect::<Vec<_>>(),
            "enabled":enabled,"goal_reached":false,"story":self.circuit.story.report(),
            "breadcrumb":self.circuit.breadcrumbs.current,"training":training,
            "neural_trial_ms":self.circuit.decisions * self.circuit.decision_window_ms as u64})
            .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn observed_loop_softens_an_entrenched_action_without_rewriting_logits() {
        // Representative live stall: Down dominates while East-facing guidance
        // requires discovering Right. No target button enters the sampler.
        let logits = [0.0_f32, 5.5, -0.5, -2.5, -2.0, -2.0];
        let probability = |pressure, button: usize| {
            let weights = logits.map(|x| (x / super::loop_temperature(pressure)).exp());
            weights[button] / weights.iter().sum::<f32>()
        };
        assert!(probability(0.0, 1) > 0.98);
        assert!(probability(1.0, 1) < 0.45);
        assert!(probability(1.0, 3) > 0.07);
        assert_eq!(super::loop_temperature(0.0), 1.0);
        assert_eq!(logits[1], 5.5);
    }

    #[test]
    fn pokemon_picture_accepts_dismissal_without_field_motion() {
        let observation = serde_json::json!({"status":{"screen":"overworld"},"observe":{"menus":[{"kind":"pokemon_picture","species":"CHIKORITA"}]}});
        assert_eq!(super::action_mask(&observation, true), vec![false,false,false,false,true,true,false,false]);
    }
    #[test]
    fn start_gap_keeps_menu_closure_and_required_hm_teaching_available(){
        let mut v=serde_json::json!({"status":{"screen":"overworld","party":[{"moves":[{"name":"TACKLE"}]}]},"observe":{"menus":[]},"reward_state":{"machines":[]}});
        assert!(super::start_cooldown_blocks(&v,32));assert!(!super::start_cooldown_blocks(&v,0));
        v["observe"]["menus"]=serde_json::json!([{"kind":"start","entries":[">PACK"]}]);assert!(!super::start_cooldown_blocks(&v,32));
        v["observe"]["menus"]=serde_json::json!([]);v["reward_state"]["machines"]=serde_json::json!(["HM_CUT"]);
        assert!(!super::start_cooldown_blocks(&v,32));
        v["status"]["party"][0]["moves"]=serde_json::json!([{"name":"CUT"}]);assert!(super::start_cooldown_blocks(&v,32));
    }
    #[test]
    fn context_transition_bootstraps_from_the_incoming_critic_and_observation() {
        let config = crate::operant::OperantConfig::default();
        for (incoming, reward) in [(Some(6.0), 0.0), (Some(6.0), 0.3), (None, 0.3)] {
            let mut old_critic = vec![0.0; super::WIDTH]; old_critic[0] = 2.0;
            let mut state = super::CircuitState {
                context: "menu:old".into(), weights: vec![0.0; super::WIDTH * 8],
                value_weights: old_critic, last_value: 1.0,
                rollout: vec![transition(4, reward)], ..Default::default()
            };
            if let Some(weight) = incoming {
                let mut critic = vec![0.0; super::WIDTH]; critic[0] = weight;
                state.heads.insert("menu:new".into(), (vec![0.1; super::WIDTH * 8], critic));
            }
            let mut observed = vec![0.0; super::WIDTH]; observed[0] = 0.25;
            state.activate_context("menu:new".into(), &config, true, &observed);
            let expected = reward + config.discount * incoming.unwrap_or(0.0) * 0.25 - 1.0;
            assert!((state.last_advantage - expected).abs() < 1e-6);
            assert_eq!(state.updates, 1);
            assert!(state.rollout.is_empty());
            if let Some(weight) = incoming { assert_eq!(state.value_weights[0], weight); }
            else { assert!(state.value_weights.is_empty()); }
        }
    }
    #[test]
    fn rewarded_transition_waits_for_next_state_but_terminal_does_not() {
        let config = crate::operant::OperantConfig::default();
        let mut state = super::CircuitState {
            context: "menu:old".into(), weights: vec![0.0; super::WIDTH * 8],
            last_value: 9.0, ..Default::default()
        };
        state.record_outcome(transition(4, 0.0).decision, 0.3, false, &config);
        assert_eq!(state.updates, 0, "V(before) must not train a menu confirmation");
        assert_eq!(state.rollout.len(), 1);
        state.activate_context("menu:new".into(), &config, true, &vec![0.5; super::WIDTH]);
        assert!((state.last_advantage - 0.3).abs() < 1e-6, "Unseen next state has value zero");
        state.weights = vec![0.0; super::WIDTH * 8];
        state.record_outcome(transition(4, 0.0).decision, 2.0, true, &config);
        assert_eq!(state.updates, 2);
        assert!(state.rollout.is_empty());
    }
    #[test]
    fn deferred_error_still_isolates_earlier_neutral_actions() {
        let config = crate::operant::OperantConfig::default();
        let mut state = super::CircuitState {
            weights: vec![0.0; super::WIDTH * 8],
            rollout: vec![transition(0, 0.0)], ..Default::default()
        };
        state.record_outcome(transition(4, 0.0).decision, -2.0, false, &config);
        assert_eq!(state.updates, 1);
        assert_eq!(state.last_advantage, 0.0);
        assert_eq!(state.rollout.len(), 1);
        assert_eq!(state.rollout[0].decision.action, 4);
        assert_eq!(state.rollout[0].reward, -2.0);
    }
    #[test]
    fn pocket_transfer_uses_horizontal_motion_without_inventing_confirmation() {
        assert_eq!(super::relative_menu_objective("menu:battle:heal:objective:pocket:Items:west"), Some(vec!["west"]));
        assert_eq!(super::relative_menu_objective("menu:battle:capture:objective:pocket:Balls:east"), Some(vec!["east"]));
        assert_eq!(super::relative_menu_objective("menu:battle:heal:objective:pocket:Items:aligned"), None);
        assert_eq!(super::relative_menu_objective("menu:battle:objective:menu:south:objective:pocket:Items:aligned"), Some(vec!["south"]));
    }
    #[test]
    fn trainer_ball_mask_preserves_healing_cancellation_and_wild_capture() {
        let mut v = serde_json::json!({"status":{"screen":"battle"},"observe":{"battle":"Trainer FALKNER", "menus":[{"kind":"battle","surface":"balls","entries":["># BALL x2"," CANCEL"]}]}});
        let mask = super::action_mask(&v, false);
        assert!(!mask[4]);
        assert!(mask[..4].iter().all(|allowed| *allowed));
        assert!(mask[5]);
        v["observe"]["battle"] = serde_json::json!("Wild BATTLETYPE_NORMAL");
        assert!(super::action_mask(&v, false)[4]);
        v["observe"]["battle"] = serde_json::json!("Trainer FALKNER");
        v["observe"]["menus"][0]["entries"] = serde_json::json!(["# BALL x2",">CANCEL"]);
        assert!(super::action_mask(&v, false)[4]);
        v["observe"]["menus"][0]["surface"] = serde_json::json!("items");
        v["observe"]["menus"][0]["entries"] = serde_json::json!([">POTION x1","CANCEL"]);
        assert!(super::action_mask(&v, false)[4]);
        v["observe"]["menus"][0]["surface"] = serde_json::json!("balls");
        v["observe"]["menus"][0]["entries"] = serde_json::json!(["># BALL x2","CANCEL"]);
        v["observe"]["battle_message"] = serde_json::json!("Don't be a thief!");
        assert!(super::action_mask(&v, false)[4]);
    }
    #[test]
    fn menu_transfer_preserves_existing_heads_and_keeps_critics_local() {
        let mut state = super::CircuitState::default();
        let donor = "menu:battle:items:objective:menu:south".to_string();
        let opposite = "menu:battle:items:objective:menu:north".to_string();
        state.heads.insert(donor.clone(), (vec![0.4; super::WIDTH * 8], vec![0.9; super::WIDTH]));
        state.heads.insert(opposite.clone(), (vec![-0.7; super::WIDTH * 8], vec![0.2; super::WIDTH]));
        let target = "objective:hm:CUT|objective:hm:cursor:south".to_string();
        state.activate_context(target.clone(), &crate::operant::OperantConfig::default(), false, &[]);
        assert_eq!(state.weights, vec![0.4; super::WIDTH * 8]);
        assert!(state.value_weights.is_empty());
        assert_eq!(state.heads[&donor].0, vec![0.4; super::WIDTH * 8]);
        assert_eq!(state.heads[&opposite].0, vec![-0.7; super::WIDTH * 8]);
        state.weights.fill(0.6);
        state.activate_context("battle".into(), &crate::operant::OperantConfig::default(), false, &[]);
        assert!(state.weights.is_empty());
        state.activate_context(target, &crate::operant::OperantConfig::default(), false, &[]);
        assert_eq!(state.weights, vec![0.6; super::WIDTH * 8]);
    }

    #[test]
    fn menu_transfer_requires_the_same_explicit_relative_goal() {
        let mut state = super::CircuitState::default();
        state.heads.insert("menu:battle:balls:objective:menu:aligned".into(), (vec![0.2; super::WIDTH * 8], vec![]));
        state.heads.insert("objective:pc:withdraw|objective:pc:cursor:aligned".into(), (vec![0.6; super::WIDTH * 8], vec![]));
        let prior = state.menu_actor_prior("objective:hm:teach|objective:hm:cursor:aligned");
        assert!(prior.iter().all(|w| (*w - 0.4).abs() < 1e-6));
        for context in ["choice:Do you want to trade?", "navigation:objective:menu:aligned", "menu:battle:moves:ordinary", "menu:battle:objective:menu:north", "menu:battle:objective:menu:aligned_unknown"] {
            assert!(state.menu_actor_prior(context).is_empty(), "{context}");
        }
    }
    #[test]
    fn navigation_progress_overrides_stale_inactivity_but_not_bad_actions() {
        let mut rewards = Vec::new();
        let mut penalties = vec!["inaction:no_progress".into(), "inaction:stalled".into()];
        super::merge_navigation_feedback(&mut rewards, &mut penalties, "breadcrumb:recover:route".into());
        assert!(penalties.is_empty());
        assert_eq!(rewards, vec!["breadcrumb:recover:route"]);
        for penalty in ["action:short_loop", "action:blocked_movement", "battle:defeat"] {
            let mut rewards = Vec::new();
            let mut penalties = vec![penalty.into(), "inaction:no_progress".into()];
            super::merge_navigation_feedback(&mut rewards, &mut penalties, "exploration:new_ground".into());
            assert!(rewards.is_empty());
            assert_eq!(penalties, vec![penalty]);
        }
    }

    use super::*;
    #[test]
    fn sensory_selection_can_expand_without_expanding_policy() {
        let cells: Vec<Cell> = (0..1200).map(|i| Cell {
            id: i.to_string(), kind: format!("type{}", i % 10),
            class: "cb_sensory".into(), side: if i % 2 == 0 { "left" } else { "right" }.into(),
            transmitter: "acetylcholine".into(), position: None,
        }).collect();
        let degree = vec![1; cells.len()];
        let selected = diverse_population(&cells, (0..cells.len()).collect(), &degree, 1024);
        assert_eq!(selected.len(), 1024);
        assert_eq!(selected.iter().collect::<std::collections::BTreeSet<_>>().len(), 1024);
        assert_eq!(diverse_population(&cells, (0..cells.len()).collect(), &degree, WIDTH).len(), 256);
        assert_eq!(diverse_population(&cells, vec![0, 1], &degree, 1024), vec![0, 1]);
        let mut config = crate::operant::OperantConfig::default();
        assert_eq!(config.sensory_neurons, 256);
        config.sensory_neurons = 16384;
        assert!(config.valid());
        assert_eq!(config.sensory_window_ms, 150);
        config.sensory_window_ms = 50;
        assert!(config.valid());
        config.sensory_window_ms = 0;
        assert!(!config.valid());
        config.sensory_window_ms = 150;
        config.sensory_neurons = usize::MAX;
        assert!(!config.valid());
    }
    #[test]
    fn idle_inventory_does_not_interrupt_clerk_interaction() {
        let mut v=json!({"status":{"screen":"overworld","money":1000,"party":[{"hp":40,"max_hp":40,"moves":[]}]},"map_info":{"name":"AzaleaMart"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"],"items":[],"key_items":[],"machines":[]},"observe":{"menus":[]}});
        assert!(matches!(crate::curriculum::goal(&v),Some(crate::curriculum::Goal::Talk("CLERK",_))));
        assert!(start_cooldown_blocks(&v,0));
        v["observe"]["menus"]=json!([{"kind":"shop","surface":"top"}]);assert!(!start_cooldown_blocks(&v,0));
        v["observe"]["menus"]=json!([]);v["reward_state"]["items"]=json!([{"id":"POTION","quantity":1}]);assert!(!start_cooldown_blocks(&v,0));
    }
    #[test]
    fn healthy_ball_only_travel_does_not_reopen_start_after_cooldown() {
        let mut v=json!({"status":{"screen":"overworld","party":[{"hp":46,"max_hp":46,"moves":[]}]},"map_info":{"name":"Route32"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"],"items":[{"id":"POKE_BALL","quantity":2}],"key_items":[]},"observe":{"menus":[]}});
        assert!(start_cooldown_blocks(&v,0));
        for (key,value) in [("key_items",json!(["SQUIRTBOTTLE"])),("items",json!([{"id":"POTION","quantity":1}])),("items",Value::Null)] {
            let mut task=v.clone();task["reward_state"][key]=value;assert!(!start_cooldown_blocks(&task,0));
        }
        v["observe"]["menus"]=json!([{"kind":"start"}]);assert!(!start_cooldown_blocks(&v,0));
        v["observe"]["menus"]=json!([]);v["status"]["party"][0]["hp"]=json!(10);assert!(!start_cooldown_blocks(&v,0));
        v["status"]["party"][0]["hp"]=json!(46);v["reward_state"]["machines"]=json!(["HM_CUT"]);v["map_info"]["hm_compatibility"]=json!({"HM_CUT":[{"slot":0,"able":true}]});assert!(!start_cooldown_blocks(&v,0));
        v["status"]["screen"]=json!("battle");assert!(!start_cooldown_blocks(&v,0));
    }
    #[test]
    fn battle_menus_use_directions_and_confirmation_without_start() {
        for surface in ["commands","moves","party","party_actions","items","balls","machines","faint_prompt"] {
            let v=json!({"status":{"screen":"battle"},"observe":{"menus":[{"surface":surface}]}});
            assert_eq!(action_mask(&v,false),vec![true,true,true,true,true,true,false,false]);
        }
        let v=json!({"status":{"screen":"overworld"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]},"observe":{"menus":[]}});
        assert!(action_mask(&v,false)[6],"field HM and party menus remain accessible");
    }
    #[test]
    fn menu_masks_keep_choices_and_never_explore_forbidden_buttons() {
        let mut o = json!({"status":{"screen":"overworld"},"observe":{"menus":[],"visible_dialogue":"Hello"}});
        assert_eq!(
            action_mask(&o, false),
            vec![false, false, false, false, true, true, false, false]
        );
        o["observe"]["menus"] = json!([{"kind":"yes_no"}]);
        assert_eq!(
            action_mask(&o, false),
            vec![true, true, true, true, true, true, false, false]
        );
        o["reward_state"] = json!({"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]});
        assert!(action_mask(&o, false)[6]);
        o["observe"]["menus"] = json!([]);
        assert!(
            !action_mask(&o, false)[6],
            "plain dialogue remains A/B after errands"
        );
        o["observe"]["visible_dialogue"] = Value::Null;
        o["observe"]["battle_message"] = json!("CYNDAQUIL fainted!");
        for screen in ["battle", "overworld"] {
            o["status"]["screen"] = json!(screen);
            assert_eq!(action_mask(&o, false), vec![false, false, false, false, true, true, false, false]);
        }
        o["observe"]["battle_message"] = Value::Null;
        o["status"]["screen"] = json!("title");
        assert!(action_mask(&o, false)[6], "title Start remains available");
    }
    #[test]
    fn distance_feedback_does_not_punish_the_recovery_leg_twice() {
        let mut penalties = vec!["action:short_loop".into(), "inaction:no_progress".into()];
        merge_distance_feedback(&mut penalties, 0.29);
        assert!(penalties.is_empty());
        let mut collision = vec!["action:blocked_movement".into(), "action:short_loop".into()];
        merge_distance_feedback(&mut collision, 0.29);
        assert_eq!(collision, vec!["action:blocked_movement"]);
        let mut idle_loop = vec!["action:short_loop".into()];
        merge_distance_feedback(&mut idle_loop, 0.0);
        assert_eq!(idle_loop, vec!["action:short_loop"]);
    }

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
                temperature: 1.0,
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
    fn pessimistic_critic_recovers_while_negative_actions_are_still_discouraged() {
        let config = crate::operant::OperantConfig {
            discount: 0.9, value_learning_rate: 0.2, entropy_bonus: 0.0,
            ..Default::default()
        };
        let mut features=vec![0.0;WIDTH];features[0]=1.0;
        let mut critic=vec![0.0;WIDTH];critic[0]=-10.0;
        let mut state=CircuitState { weights:vec![0.0;WIDTH*8],value_weights:critic,..Default::default() };
        for step in 0..500 {
            let before=state.value(&features);
            let mut sample=transition(0,-0.1);sample.decision.features=features.clone();
            state.rollout.push(sample);state.train_rollout(before,&config);
            if step==0 {
                assert!(state.value(&features)>before,"A less-bad return must repair an overly pessimistic critic");
                assert!(state.weights[0]<state.weights[WIDTH],"The negative action must still lose preference");
            }
        }
        // A continuing -0.1 reward at discount 0.9 has value -1.
        assert!((state.value(&features)+1.0).abs()<0.01,"Constant costs must converge rather than drive the critic to its bound");
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
