use crate::*;
#[derive(Clone, Serialize, Deserialize)]
pub struct Binding {
    pub button: String,
    pub kind: String,
    pub side: String,
    pub gain: f32,
}
pub fn default_bindings() -> Vec<Binding> {
    [
        ("up", "DNpe017", "L"),
        ("down", "DNg11", "L"),
        ("left", "DNp20", "L"),
        ("right", "DNp20", "R"),
        ("a", "DNpe017", "R"),
        ("b", "DNg11", "R"),
        ("start", "DNp01", ""),
        ("select", "MDN", ""),
    ]
    .into_iter()
    .map(|(button, kind, side)| Binding {
        button: button.into(),
        kind: kind.into(),
        side: side.into(),
        gain: 1.0,
    })
    .collect()
}

#[wasm_bindgen]
impl Brain {
    pub fn configuration(&self) -> Result<String, String> {
        serde_json::to_string(&self.config).map_err(|e| e.to_string())
    }
    pub fn catalog(&self) -> Result<String, String> {
        serde_json::to_string(&self.metadata).map_err(|e| e.to_string())
    }
    /// Fixed engineered interface. No game state, reward or route enters decoding.
    pub fn action(&self) -> Result<String, String> {
        let mut rates = Vec::new();
        for binding in &self.config.decoder {
            let (button, kind, side) = (&binding.button, &binding.kind, &binding.side);
            let indices = self
                .metadata
                .cells
                .iter()
                .enumerate()
                .filter(|(_, c)| c.kind == *kind && (side.is_empty() || c.side == *side))
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            let spikes = indices.iter().map(|&i| self.counts[i] as f32).sum::<f32>();
            let spikes_per_cell = spikes / indices.len().max(1) as f32;
            let score = spikes_per_cell * binding.gain;
            rates.push(serde_json::json!({"button":button,"cell_type":kind,"side":side,"spikes_per_cell":spikes_per_cell,"gain":binding.gain,"score":score,"indices":indices}));
        }
        // Deterministic sampling from measured neural activity provides exploration.
        // It never selects an action whose bound population had zero spikes.
        let total = rates
            .iter()
            .map(|r| r["score"].as_f64().unwrap())
            .sum::<f64>();
        let mut seed = self.tick.wrapping_add(0x9e3779b97f4a7c15);
        seed = (seed ^ (seed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        seed = (seed ^ (seed >> 27)).wrapping_mul(0x94d049bb133111eb);
        seed ^= seed >> 31;
        let mut choice = ((seed >> 11) as f64 / 9007199254740992.0) * total;
        let mut button = serde_json::Value::Null;
        for r in &rates {
            let score = r["score"].as_f64().unwrap();
            if choice < score {
                button = r["button"].clone();
                break;
            }
            choice -= score;
        }
        serde_json::to_string(&serde_json::json!({"button":button,"frames":8,"readouts":rates,"mapping":"fixed engineered BCI v2; seeded spike-weighted sampling"})).map_err(|e|e.to_string())
    }
    /// Structured sensory prosthesis: stable feature hashes address actual
    /// annotated central-brain sensory neurons. No goal or correct action input.
    pub fn observe(&mut self, json: &str) -> Result<String, String> {
        let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let features = sensory_features(&v);
        let pool = self
            .metadata
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.class == "cb_sensory")
            .map(|(i, _)| i as u32)
            .collect::<Vec<_>>();
        if pool.is_empty() {
            return Err("No annotated cb_sensory population".into());
        }
        let mut indices = Vec::new();
        for feature in &features {
            let mut h = 0xcbf29ce484222325u64;
            for b in feature.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            for k in 0..4u64 {
                indices.push(
                    pool[(h.wrapping_add(k.wrapping_mul(0x9e3779b97f4a7c15)) as usize)
                        % pool.len()],
                );
            }
        }
        indices.sort_unstable();
        indices.dedup();
        self.stimulate(&indices, self.config.stimulus_mv)?;
        serde_json::to_string(&serde_json::json!({"encoding":"structured local sensory prosthesis v3","features":features,"indices":indices,"strength_mv":self.config.stimulus_mv})).map_err(|e|e.to_string())
    }
    /// Anatomical soma projection, with brightness determined by measured spikes.
    pub fn render(&self, width: u32, height: u32, yaw: f32, pitch: f32) -> Result<Vec<u8>, String> {
        self.render_population(width, height, yaw, pitch, "all")
    }
    /// This filters only the display; every retained neuron is still simulated.
    pub fn render_population(
        &self,
        width: u32,
        height: u32,
        yaw: f32,
        pitch: f32,
        group: &str,
    ) -> Result<Vec<u8>, String> {
        self.render_styled(width, height, yaw, pitch, group, "{}")
    }
    pub fn render_styled(
        &self,
        width: u32,
        height: u32,
        yaw: f32,
        pitch: f32,
        group: &str,
        style_json: &str,
    ) -> Result<Vec<u8>, String> {
        let mut view = AnatomyView::new(&self.view_anatomy())?;
        view.update_activity(&self.view_activity())?;
        view.render(width, height, yaw, pitch, 1.0, group, style_json)
    }
}

#[wasm_bindgen]
impl Brain {
    /// Explicit artificial-current channels for conditioning/calibration.
    /// All targets are actual cells; input never assigns their output spikes.
    pub fn inject_currents(&mut self, json: &str) -> Result<(), String> {
        let pairs: Vec<(u32, f32)> = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut values = vec![0.0; self.voltage.len()];
        for (i, mv) in pairs {
            if i as usize >= values.len() || !mv.is_finite() || mv.abs() > 100.0 {
                return Err("Invalid current target or dose".into());
            }
            values[i as usize] += mv;
            if values[i as usize].abs() > 100.0 {
                return Err("Combined current exceeds 100 mV-equivalent".into());
            }
        }
        self.external = values;
        for i in 0..self.voltage.len() {
            if self.external[i] != 0.0 && !self.awake[i] {
                self.active.push(i);
                self.awake[i] = true;
            }
        }
        Ok(())
    }
}

#[wasm_bindgen]
impl Brain {
    pub fn find_cells(&self, query: &str) -> Result<String, String> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Err("Enter a cell type or source ID".into());
        }
        let matches = self
            .metadata
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.id == query || c.kind.to_lowercase().contains(&query))
            .take(32)
            .map(|(i, c)| serde_json::json!({"index":i,"cell":c,"spikes":self.counts[i]}))
            .collect::<Vec<_>>();
        serde_json::to_string(&matches).map_err(|e| e.to_string())
    }
    pub fn pick(
        &self,
        width: u32,
        height: u32,
        x: f32,
        y: f32,
        yaw: f32,
        pitch: f32,
    ) -> Result<String, String> {
        let view = AnatomyView::new(&self.view_anatomy())?;
        let index = view.pick(width, height, x, y, yaw, pitch, 1.0, "all")?;
        if index < 0 {
            return Err("No source soma near that point".into());
        }
        self.inspect(index as u32)
    }
}

impl Brain {
    pub(crate) fn circuit_activity(&self) -> serde_json::Value {
        if self.config.operant.is_some() {
            return serde_json::json!(
                self.wiring
                    .as_ref()
                    .map(|w| {
                        [
                            ("Sensory inputs", &w.inputs),
                            ("Descending readout", &w.outputs),
                        ]
                        .iter()
                        .map(|(label, indices)| {
                            serde_json::json!({"population":label,"neurons":indices.len(),
                        "active":indices.iter().filter(|&&i|self.counts[i] > 0).count(),
                        "spikes":indices.iter().map(|&i|self.counts[i] as u64).sum::<u64>()})
                        })
                        .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            );
        }
        let labels = ["KC", "PAM01", "PPL101", "MBON01", "MBON11"];
        let mut populations = [0u32; 5];
        let mut active = [0u32; 5];
        let mut spikes = [0u64; 5];
        for (i, c) in self.metadata.cells.iter().enumerate() {
            let group = if self.kc[i] {
                Some(0)
            } else {
                labels.iter().position(|&label| c.kind == label)
            };
            if let Some(g) = group {
                populations[g] += 1;
                spikes[g] += self.counts[i] as u64;
                if self.counts[i] > 0 {
                    active[g] += 1;
                }
            }
        }
        serde_json::json!(labels.iter().enumerate().map(|(g,label)|serde_json::json!({"population":label,"neurons":populations[g],"active":active[g],"spikes":spikes[g]})).collect::<Vec<_>>())
    }
}

/// Shared local map input for both sensory and action-memory modes. Only
/// observation fields enter here; evaluator state never becomes a sensory cue.
pub(crate) fn local_map_features(v: &serde_json::Value) -> Vec<String> {
    let mut features = Vec::new();
    if let Some(s) = v
        .pointer("/map_info/player/facing")
        .and_then(|v| v.as_str())
    {
        features.push(format!("facing:{s}"));
    }
    let px = v
        .pointer("/map_info/player/x")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let py = v
        .pointer("/map_info/player/y")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let ox = v
        .pointer("/map_info/terrain/origin_x")
        .and_then(|v| v.as_i64())
        .unwrap_or(px - 6);
    let oy = v
        .pointer("/map_info/terrain/origin_y")
        .and_then(|v| v.as_i64())
        .unwrap_or(py - 6);
    if let Some(objects) = v.pointer("/map_info/objects").and_then(|v| v.as_array()) {
        for object in objects {
            if let (Some(x), Some(y)) = (object["x"].as_i64(), object["y"].as_i64()) {
                let dx = x - px;
                let dy = y - py;
                if dx.abs() <= 6 && dy.abs() <= 6 {
                    features.push(format!("object:{dx}:{dy}"));
                }
            }
        }
    }
    if let Some(rows) = v
        .pointer("/map_info/terrain/rows")
        .and_then(|v| v.as_array())
    {
        for (y, row) in rows.iter().take(13).enumerate() {
            if let Some(row) = row.as_array() {
                for (x, tile) in row.iter().take(13).enumerate() {
                    if tile.is_null() {
                        let dx = ox + x as i64 - px;
                        let dy = oy + y as i64 - py;
                        if dx.abs() <= 6 && dy.abs() <= 6 {
                            features.push(format!("terrain:{dx}:{dy}:outside"));
                        }
                    }
                    if let Some(terrain) = tile.get("terrain").and_then(|v| v.as_str()) {
                        let dx = ox + x as i64 - px;
                        let dy = oy + y as i64 - py;
                        if dx.abs() <= 6 && dy.abs() <= 6 {
                            features.push(format!("terrain:{dx}:{dy}:{terrain}"));
                            if let Some(permission) =
                                tile.get("permission").and_then(|v| v.as_u64())
                            {
                                features.push(format!("permission:{dx}:{dy}:{permission}"));
                            }
                        }
                    }
                }
            }
        }
    }
    features.sort_unstable();
    features.dedup();
    features
}

/// Only current, player-visible observations enter the neural encoder.
pub(crate) fn sensory_features(v: &serde_json::Value) -> Vec<String> {
    sensory_features_with_menus(v, false)
}
pub(crate) fn sensory_features_with_menus(v: &serde_json::Value, visible_menus: bool) -> Vec<String> {
    let mut features = Vec::<String>::new();
    if let Some(map) = v.pointer("/map_info/name").and_then(|v| v.as_str()) {
        features.push(format!("map:{map}"));
    }
    if let Some(s) = v.pointer("/status/screen").and_then(|v| v.as_str()) {
        features.push(format!("screen:{s}"));
    }
    for (field, prefix) in [
        ("text", "text"),
        ("visible_dialogue", "dialogue"),
        ("battle", "battle"),
    ] {
        if visible_menus && field == "battle" { continue; }
        if let Some(s) = v
            .pointer(&format!("/observe/{field}"))
            .and_then(|v| v.as_str())
        {
            if !s.trim().is_empty() {
                features.push(format!("{prefix}:present"));
            }
            for token in s.split_whitespace().take(96) {
                features.push(format!("{prefix}:{token}"));
            }
        }
    }
    if let Some(menus) = v.pointer("/observe/menus").and_then(|v| v.as_array()) {
        for menu in menus.iter().take(8) {
            for key in ["kind", "surface", "selected", "cursor_column", "cursor_row", "move_info"] {
                if let Some(value) = menu.get(key) {
                    features.push(format!("menu:{key}:{value}"));
                }
            }
        }
    }
    if visible_menus {
        // The cursor is embedded in most visible entry strings, not `selected`.
        // Encode labels and positions without deciding which option is desirable.
        if let Some(menus) = v.pointer("/observe/menus").and_then(|v| v.as_array()) {
            for (surface, menu) in menus.iter().take(8).enumerate() {
                if let Some(entries) = menu.get("entries").or_else(|| menu.get("options")).and_then(|v| v.as_array()) {
                    features.push(format!("menu:{surface}:rows:{}", entries.len()));
                    for (row, entry) in entries.iter().take(24).enumerate() {
                        if let Some(text) = entry.as_str() {
                            let text = text.trim();
                            if text.starts_with('>') { features.push(format!("menu:{surface}:cursor:{row}")); }
                            let label = text.trim_start_matches('>').trim();
                            for token in label.split_whitespace().take(12) {
                                features.push(format!("menu:{surface}:row:{row}:{token}"));
                            }
                        }
                    }
                }
            }
        }
        let prefix = if v.pointer("/status/screen").and_then(|v| v.as_str()) == Some("battle") { "battle" } else { "text" };
        if let Some(lines) = v.pointer("/observe/rendered_text").and_then(|v| v.as_array()) {
            for line in lines.iter().take(24).filter_map(|v| v.as_str()) {
                for token in line.split_whitespace().take(24) { features.push(format!("{prefix}:{token}")); }
            }
        }
        if let Some(text) = v.pointer("/observe/battle_message").and_then(|v| v.as_str()) {
            for token in text.split_whitespace().take(64) { features.push(format!("battle:message:{token}")); }
        }
    }
    // Visible HP and encounter kind must survive detailed-menu mode. The old
    // mode dropped the entire battle overlay, including this essential feedback.
    if let Some(battle)=v.pointer("/reward_state/battle").filter(|b| b.is_object()) {
        if let (Some(hp),Some(max))=(battle["enemy_hp"].as_u64(),battle["enemy_max_hp"].as_u64()) {
            features.push(format!("battle:enemy_health:{}",hp.saturating_mul(4)/max.max(1)));
        }
        if let Some(kind)=v.pointer("/observe/battle").and_then(|v|v.as_str()).and_then(|s|s.split_whitespace().next()) {
            features.push(format!("battle:kind:{kind}"));
        }
        features.push(format!("battle:enemy_status:{}",battle["enemy_status"]));
    }
    features.extend(local_map_features(&v));
    if let Some(party) = v.pointer("/status/party").and_then(|v| v.as_array()) {
        for (i, p) in party.iter().enumerate().take(6) {
            if let (Some(hp), Some(max)) = (p["hp"].as_u64(), p["max_hp"].as_u64()) {
                features.push(format!(
                    "party:{i}:health:{}",
                    hp.saturating_mul(4) / max.max(1)
                ));
            }
            for key in ["status", "moves"] {
                if let Some(value) = p.get(key) {
                    features.push(format!("party:{i}:{key}:{value}"));
                }
            }
        }
    }
    features.sort();
    features.dedup();
    features
}

#[cfg(test)]
mod menu_input_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn visible_entries_distinguish_cursor_and_battle_submenu() {
        let fight = json!({"status":{"screen":"battle"},"observe":{"battle":"ai_move_flags: 979 source_script: secret", "menus":[{"kind":"battle","entries":[">FIGHT"," PACK"]}]}});
        let mut pack = fight.clone();
        pack["observe"]["menus"][0]["entries"] = json!([" FIGHT",">PACK"]);
        let mut moves = fight.clone();
        moves["observe"]["menus"][0]["entries"] = json!([">TACKLE"," LEER"," CANCEL"]);
        let encoded = sensory_features_with_menus(&fight, true);
        assert!(encoded.contains(&"menu:0:cursor:0".into()));
        assert!(encoded.contains(&"menu:0:row:0:FIGHT".into()));
        assert!(sensory_features_with_menus(&pack, true).contains(&"menu:0:cursor:1".into()));
        assert_ne!(encoded, sensory_features_with_menus(&moves, true));
        assert!(!encoded.iter().any(|f| f.contains("979") || f.contains("secret")));
        assert_eq!(sensory_features(&fight), sensory_features_with_menus(&fight, false));
    }
    #[test]
    fn selected_move_panel_reaches_the_brain() {
        let mut v=json!({"observe":{"menus":[{"kind":"battle","surface":"moves","move_info":{"name":"TACKLE","current_pp":0,"max_pp":35,"type":"NORMAL","disabled":false}}]}});
        let available=sensory_features_with_menus(&v,true);
        assert!(available.iter().any(|s|s.starts_with("menu:move_info:")&&s.contains("TACKLE")&&s.contains("current_pp")));
        v["observe"]["menus"][0]["move_info"]=json!({"name":"TACKLE","disabled":true});
        assert_ne!(available,sensory_features_with_menus(&v,true));
    }
    #[test]
    fn visible_battle_messages_reach_the_encoder() {
        let v = json!({"status":{"screen":"battle"},"observe":{"rendered_text":["TOTODILE","18/21"],"battle_message":"TACKLE missed!","menus":[]}});
        let features = sensory_features_with_menus(&v, true);
        assert!(features.contains(&"battle:TOTODILE".into()));
        assert!(features.contains(&"battle:message:missed!".into()));
    }
}
