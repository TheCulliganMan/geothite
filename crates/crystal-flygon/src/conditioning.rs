//! Explicit artificial-current assay; never a hidden Pokémon controller.
use crate::*;
#[wasm_bindgen]
impl Brain {
    pub fn conditioning_cue(&mut self, label: &str) -> Result<String, String> {
        if !["A", "B", "off"].contains(&label) {
            return Err("Cue must be A, B or off".into());
        }
        let outputs = self
            .metadata
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == "MBON01")
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let mut pres = self
            .plastic
            .iter()
            .filter(|&&(_, e)| outputs.contains(&(self.targets[e] as usize)))
            .map(|&(pre, _)| pre)
            .collect::<Vec<_>>();
        pres.sort_unstable();
        pres.dedup();
        if pres.len() < 128 || outputs.len() != 2 {
            return Err("Expected identified PAM01/MBON01 circuit".into());
        }
        let input = match label {
            "A" => pres[..64].to_vec(),
            "B" => pres[64..128].to_vec(),
            _ => Vec::new(),
        };
        let mut pairs = outputs
            .iter()
            .map(|&i| (i as u32, 6.5f32))
            .collect::<Vec<_>>();
        pairs.extend(input.iter().map(|&i| (i as u32, self.config.stimulus_mv)));
        self.inject_currents(&serde_json::to_string(&pairs).map_err(|e| e.to_string())?)?;
        serde_json::to_string(&serde_json::json!({"encoding":"artificial KC conditioning; not game vision","cue":label,"indices":input,"output_indices":outputs,"kc_current_mv":self.config.stimulus_mv,"mbon_tonic_mv":6.5})).map_err(|e|e.to_string())
    }
    pub fn conditioning_trial(&mut self, label: &str, paired: bool) -> Result<String, String> {
        self.reset_dynamics();
        let sensory: serde_json::Value =
            serde_json::from_str(&self.conditioning_cue(label)?).map_err(|e| e.to_string())?;
        self.step(100.0)?;
        if paired {
            self.appetitive_pulse()?;
        }
        let telemetry: serde_json::Value =
            serde_json::from_str(&self.step(200.0)?).map_err(|e| e.to_string())?;
        serde_json::to_string(&serde_json::json!({"sensory":sensory,"telemetry":telemetry,"paired":paired,"learning":self.config.learning})).map_err(|e|e.to_string())
    }
    pub fn conditioning_probe(&mut self, label: &str) -> Result<String, String> {
        self.reset_dynamics();
        let sensory: serde_json::Value =
            serde_json::from_str(&self.conditioning_cue(label)?).map_err(|e| e.to_string())?;
        let learning = self.config.learning;
        self.config.learning = false;
        let result = self.step(500.0);
        self.config.learning = learning;
        let telemetry: serde_json::Value =
            serde_json::from_str(&result?).map_err(|e| e.to_string())?;
        let outputs = self
            .metadata
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == "MBON01")
            .map(|(i, c)| serde_json::json!({"index":i,"source_id":c.id,"spikes":self.counts[i]}))
            .collect::<Vec<_>>();
        serde_json::to_string(&serde_json::json!({"sensory":sensory,"telemetry":telemetry,"outputs":outputs,"game_decoder":serde_json::from_str::<serde_json::Value>(&self.action()?).map_err(|e|e.to_string())?,"probe_ms":500,"reward_during_probe":false,"plasticity_during_probe":false})).map_err(|e|e.to_string())
    }
}
