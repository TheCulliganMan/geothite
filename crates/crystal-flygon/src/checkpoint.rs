use crate::*;
#[derive(Serialize, Deserialize)]
struct State {
    #[serde(default)]
    circuit: circuit::CircuitState,
    version: u32,
    model_id: String,
    interface_id: String,
    graph_id: String,
    config: Config,
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
    #[serde(default)]
    last_window_steps: u32,
    weights: Vec<f32>,
    pulse_ticks: u64,
    reward_ticks: u64,
    delivered_dan_spikes: u64,
    active: Vec<usize>,
    campaign: campaign::Campaign,
}
#[wasm_bindgen]
impl Brain {
    pub fn checkpoint(&self) -> Result<String, String> {
        serde_json::to_string(&serde_json::json!({"version":1,"model_id":MODEL_ID,"interface_id":INTERFACE_ID,"graph_id":self.graph_id,"config":self.config,
            "voltage":self.voltage,"current":self.current,"adaptation":self.adaptation,"refractory":self.refractory,
            "external":self.external,"trace":self.trace,"last_spike":self.last_spike,"counts":self.counts,
            "queue":self.queue,"tick":self.tick,"last_window_steps":self.last_window_steps,"weights":self.plastic.iter().map(|&(_,e)|self.weights[e]).collect::<Vec<_>>(),
            "pulse_ticks":self.pulse_ticks,"reward_ticks":self.reward_ticks,"delivered_dan_spikes":self.delivered_dan_spikes,
            "active":self.active,"campaign":self.campaign,"circuit":self.circuit})).map_err(|e|e.to_string())
    }
    /// Explicit migration only: new pools cannot reinterpret old actor weights.
    /// Preserve synaptic learning, campaign/story/navigation ledgers and seed.
    pub fn upgrade_readout_checkpoint(&mut self, json: &str) -> Result<String, String> {
        let mut state: State = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if state.interface_id != "flygon-connected-sensory-descending-actor-critic-v4"
            || !state.circuit.valid()
        {
            return Err("Readout upgrade requires a valid legacy v4 checkpoint".into());
        }
        let source_interface = state.interface_id.clone();
        state.interface_id = INTERFACE_ID.into();
        state.config.exponential_current_integration = self.config.exponential_current_integration;
        if let (Some(old), Some(target)) = (&mut state.config.operant, &self.config.operant) {
            old.visible_menu_inputs = target.visible_menu_inputs;
        }
        state.circuit.upgrade_readout();
        // Normal restore still validates graph, topology, bounds and settings
        // before changing this brain. Only the declared layout changes above
        // are permitted; ordinary restore retains its strict identity checks.
        self.restore(&serde_json::to_string(&state).map_err(|e| e.to_string())?)?;
        self.reset_dynamics();
        Ok(serde_json::json!({"source_interface":source_interface,"target_interface":INTERFACE_ID,
            "synaptic_weights_preserved":true,"policy_reset":true,"ledgers_preserved":true,
            "reason":"Pooled descending features require new actor/critic weights; legacy checkpoint remains the rollback source"}).to_string())
    }

    pub fn restore(&mut self, json: &str) -> Result<(), String> {
        let s: State = serde_json::from_str(json).map_err(|e| e.to_string())?;
        s.config.validate()?;
        if !s.circuit.valid() {
            return Err("Invalid circuit policy checkpoint".into());
        }
        let n = self.voltage.len();
        if s.version != 1
            || s.model_id != MODEL_ID
            || s.interface_id != INTERFACE_ID
            || s.graph_id != self.graph_id
            || s.config.dt_ms != self.config.dt_ms
            || s.config.delay_ms != self.config.delay_ms
            || s.config.contact_gain_mv != self.config.contact_gain_mv
            || s.config.operant.as_ref().map(|v| (v.sensory_neurons, v.sensory_window_ms, v.visible_menu_inputs)) != self.config.operant.as_ref().map(|v| (v.sensory_neurons, v.sensory_window_ms, v.visible_menu_inputs))
            || s.config.exponential_current_integration != self.config.exponential_current_integration
        {
            return Err("Checkpoint graph or integration identity mismatch".into());
        }
        for values in [&s.voltage, &s.current, &s.adaptation, &s.external, &s.trace] {
            if values.len() != n || values.iter().any(|v| !v.is_finite()) {
                return Err("Invalid checkpoint neural arrays".into());
            }
        }
        if s.adaptation.iter().any(|&v| v < 0.0)
            || s.trace.iter().any(|&v| v < 0.0)
            || s.external.iter().any(|v| v.abs() > 100.0)
        {
            return Err("Invalid checkpoint current or eligibility state".into());
        }
        if s.refractory.len() != n
            || s.last_spike.len() != n
            || s.counts.len() != n
            || s.queue.len() != self.queue.len()
            || s.weights.len() != self.plastic.len()
            || s.last_spike.iter().any(|&t| t > s.tick)
            || s.queue.iter().flatten().any(|&i| i as usize >= n)
            || s.active.iter().any(|&i| i >= n)
        {
            return Err("Invalid checkpoint topology".into());
        }
        let mut awake = vec![false; n];
        for &i in &s.active {
            if awake[i] {
                return Err("Duplicate active neuron".into());
            }
            awake[i] = true;
        }
        for (i, &(_, edge)) in self.plastic.iter().enumerate() {
            let base = self.contacts[edge] as f32 * s.config.contact_gain_mv;
            let w = s.weights[i];
            if !w.is_finite()
                || w < base * s.config.minimum_weight_fraction - 1e-5
                || w > base * s.config.maximum_weight_fraction + 1e-5
            {
                return Err("Invalid checkpoint weight".into());
            }
        }
        for (i, &(_, edge)) in self.plastic.iter().enumerate() {
            self.weights[edge] = s.weights[i];
        }
        self.config = s.config;
        self.voltage = s.voltage;
        self.current = s.current;
        self.adaptation = s.adaptation;
        self.refractory = s.refractory;
        self.external = s.external;
        self.trace = s.trace;
        self.last_spike = s.last_spike;
        self.counts = s.counts;
        self.queue = s.queue;
        self.tick = s.tick;
        self.last_window_steps = s.last_window_steps;
        self.pulse_ticks = s.pulse_ticks;
        self.reward_ticks = s.reward_ticks;
        self.delivered_dan_spikes = s.delivered_dan_spikes;
        self.active = s.active;
        self.awake = awake;
        self.campaign = s.campaign;
        self.circuit = s.circuit;
        Ok(())
    }
    /// Experimental control: erase learned efficacies; preserve current dynamics.
    pub fn erase_memory(&mut self) {
        self.circuit.erase();
        for &(_, e) in &self.plastic {
            self.weights[e] = self.contacts[e] as f32 * self.config.contact_gain_mv;
        }
    }
    pub fn memory(&self) -> Result<String, String> {
        let edges=self.plastic.iter().enumerate().map(|(p,&(pre,e))|serde_json::json!({"edge":e,"pre":pre,"post":self.targets[e],"baseline":self.contacts[e] as f32*self.config.contact_gain_mv,"weight":self.weights[e],"dan_gain":self.gains[p]})).collect::<Vec<_>>();
        serde_json::to_string(
            &serde_json::json!({"graph_id":self.graph_id,"edges":edges,"dan":self.dan}),
        )
        .map_err(|e| e.to_string())
    }
}

#[wasm_bindgen]
impl Brain {
    /// Conditioning control. Keeps efficacy memory and campaign ledger, but
    /// resets all transient dynamics and pending stimulation explicitly.
    pub fn reset_dynamics(&mut self) {
        for i in 0..self.voltage.len() {
            self.voltage[i] = if self.kc[i] {
                self.config.kc_rest_mv
            } else {
                self.config.rest_mv
            };
        }
        self.current.fill(0.0);
        self.adaptation.fill(0.0);
        self.refractory.fill(0);
        self.external.fill(0.0);
        self.trace.fill(0.0);
        self.last_spike.fill(0);
        self.counts.fill(0);
        for q in &mut self.queue {
            q.clear();
        }
        self.tick = 0;
        self.last_window_steps = 0;
        self.pulse_ticks = 0;
        self.reward_ticks = 0;
        self.delivered_dan_spikes = 0;
        self.active.clear();
        self.awake.fill(false);
    }
}
