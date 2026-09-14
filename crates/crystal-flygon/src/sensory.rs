//! Engineered sensory prosthesis, not anatomical retinal calibration.
//! Separate task cues from observations; encode positive/negative projections
//! on distinct cells so rectification cannot silently discard half the signal.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

pub(crate) const CHANNELS: usize = 16;
fn direction(s: &str) -> usize {
    match s {
        "north" | "up" => 0,
        "east" | "right" => 1,
        "south" | "down" => 2,
        _ => 3,
    }
}
pub(crate) fn channel(feature: &str) -> usize {
    if feature.starts_with("story-scent:") {
        return 8 + direction(feature.rsplit(':').next().unwrap_or(""));
    }
    if feature.starts_with("exploration-scent:") {
        return 12 + direction(feature.rsplit(':').next().unwrap_or(""));
    }
    if feature.starts_with("party:") || feature.starts_with("battle:") {
        return 6;
    }
    if ["menu:", "text:", "dialogue:"]
        .iter()
        .any(|p| feature.starts_with(p))
    {
        return 5;
    }
    if feature.starts_with("object:") {
        return 4;
    }
    if ["terrain:", "permission:", "boundary:"]
        .iter()
        .any(|p| feature.starts_with(p))
    {
        let mut fields = feature.split(':').skip(1);
        let x = fields
            .next()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let y = fields
            .next()
            .and_then(|s| s.parse::<i64>().ok())
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
fn hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    })
}
pub(crate) fn encode(features: &[String], size: usize) -> Vec<f32> {
    // Repeat the sensory code across independently hashed anatomical banks.
    // Merely enlarging the hash table leaves almost every added cell undriven.
    // Bank zero preserves the released 256-cell encoding exactly.
    let mut drive = Vec::with_capacity(size);
    while drive.len() < size {
        let bank = drive.len() / 256;
        drive.extend(encode_bank(features, (size - drive.len()).min(256), bank as u64));
    }
    drive
}
fn encode_bank(features: &[String], size: usize, bank: u64) -> Vec<f32> {
    let channels = size.min(CHANNELS);
    if channels == 0 {
        return Vec::new();
    }
    let mut projected = vec![0.0f32; size];
    let mut density = vec![0.0f32; size];
    for feature in features {
        let c = channel(feature) % channels;
        let slots = (size - 1 - c) / channels + 1;
        // Nearby geometry carries more useful collision/interaction information
        // than the large repeated background in a 13x13 terrain window.
        let gain = if feature.starts_with("terrain:") || feature.starts_with("permission:") {
            let mut fields = feature.split(':').skip(1);
            let x = fields
                .next()
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0);
            let y = fields
                .next()
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0);
            1.0 / x.abs().max(y.abs()).max(1.0)
        } else {
            1.0
        };
        for k in 0..2u64 {
            let mut h = hash(feature).wrapping_add(k.wrapping_mul(0x9e3779b97f4a7c15)).wrapping_add(bank.wrapping_mul(0x517cc1b727220a95));
            h = (h ^ (h >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            h ^= h >> 27;
            let pairs = (slots / 2).max(1);
            let pair = if pairs >= 2 {
                (h as usize % (pairs / 2)) * 2 + k as usize
            } else {
                0
            };
            let slot = c + pair * 2 * channels;
            projected[slot] += if h >> 63 == 0 { gain } else { -gain };
            density[slot] += gain * gain;
        }
    }
    let mut drive = vec![0.0; size];
    for slot in 0..size {
        if density[slot] == 0.0 {
            continue;
        }
        let signal = (projected[slot] / density[slot].max(1.0).sqrt()).tanh();
        drive[slot] = signal.max(0.0);
        if slot + channels < size {
            drive[slot + channels] = (-signal).max(0.0);
        }
    }
    drive
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct History {
    places: VecDeque<String>,
    last: String,
    repeated: u32,
    movement: String,
}
fn state_key(v: &Value) -> String {
    // Visible state only. No reward ledger, story flags or target action.
    // Scrolling species/search rows is still the same browsing task. Keep full
    // labels in sensory encoding; only loop detection groups these utility pages.
    if v.pointer("/status/screen").and_then(Value::as_str)==Some("overworld")
        && v.pointer("/observe/menus").and_then(Value::as_array).and_then(|m|m.last()).is_some_and(|m|m["kind"]=="pokedex")
        && !v.pointer("/observe/visible_dialogue").and_then(Value::as_str).is_some_and(|s|!s.trim().is_empty()) {
        return serde_json::json!(["overworld",v.pointer("/map_info/name"),"pokedex-browsing"]).to_string();
    }
    serde_json::json!([
        v.pointer("/status/screen"),
        v.pointer("/map_info/name"),
        v.pointer("/map_info/player/x"),
        v.pointer("/map_info/player/y"),
        v.pointer("/observe/menus"),
        v.pointer("/observe/visible_dialogue"),
        v.pointer("/observe/text"),
        v.pointer("/observe/battle"),
        v.pointer("/status/party")
    ])
    .to_string()
}
impl History {
    pub(crate) fn valid(&self) -> bool {
        self.places.len() <= 64
            && self.places.iter().all(|s| s.len() <= 64)
            && self.last.len() <= 64
            && self.movement.len() <= 64
            && self.repeated <= 64
    }
    pub(crate) fn cues(&mut self, v: &Value) -> (Vec<String>, f32) {
        let key = format!("{:016x}", hash(&state_key(v)));
        self.repeated = if key == self.last {
            (self.repeated + 1).min(64)
        } else {
            0
        };
        let visits = self.places.iter().filter(|s| **s == key).count();
        self.last = key.clone();
        if self.places.len() == 64 {
            self.places.pop_front();
        }
        self.places.push_back(key);
        let pressure = ((self.repeated as f32 - 3.0).max(0.0) / 16.0)
            .max((visits as f32 - 3.0).max(0.0) / 16.0)
            .min(1.0);
        let mut cues = vec![
            format!("history:revisits:{}", visits.min(15) / 3),
            format!("history:unchanged:{}", self.repeated.min(15) / 3),
        ];
        if !self.movement.is_empty() {
            cues.push(self.movement.clone());
        }
        (cues, pressure)
    }
    pub(crate) fn feedback(&mut self, before: &Value, after: &Value, button: &str) {
        self.movement.clear();
        if !["up", "down", "left", "right"].contains(&button)
            || before.pointer("/status/screen").and_then(Value::as_str) != Some("overworld")
            || before.pointer("/map_info/player/x").is_none()
        {
            return;
        }
        let changed = ["/map_info/name", "/map_info/player/x", "/map_info/player/y"]
            .iter()
            .any(|path| before.pointer(path) != after.pointer(path));
        let turned =
            before.pointer("/map_info/player/facing") != after.pointer("/map_info/player/facing");
        // Failure to move is not proof of a wall (menus/animation can consume it).
        self.movement = format!(
            "motion:{button}:{}",
            if changed {
                "moved"
            } else if turned {
                "turned"
            } else {
                "no-displacement"
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn pokedex_scrolling_is_a_loop_without_hiding_cursor_inputs(){
        let mut history=History::default();let mut menu_history=History::default();
        let mut pressure=0.0;
        for row in 0..32 {
            let v=json!({"status":{"screen":"overworld"},"map_info":{"name":"VioletPokecenter1F"},"observe":{"menus":[{"kind":"pokedex","entries":[format!(">SPECIES{row}")]}],"text":format!("SPECIES{row}")}});
            pressure=history.cues(&v).1;
            let features=crate::interface::sensory_features_with_menus(&v,true);
            assert!(features.iter().any(|f|f.contains(&format!("SPECIES{row}"))));
            let mut purposeful=v.clone();purposeful["observe"]["menus"][0]["kind"]=json!("pack");
            assert_eq!(menu_history.cues(&purposeful).1,0.0);
        }
        assert_eq!(pressure,1.0);assert!(history.valid());
        let restored:History=serde_json::from_str(&serde_json::to_string(&history).unwrap()).unwrap();
        assert!(restored.valid());assert_eq!(restored.repeated,history.repeated);
    }
    #[test]
    fn story_cue_cannot_suppress_observation_channels() {
        let features = vec![
            "terrain:0:-1:Land".into(),
            "menu:selected:1".into(),
            "battle:enemy".into(),
        ];
        let baseline = encode(&features, 256);
        let mut cued = features;
        cued.push("story-scent:north".into());
        let current = encode(&cued, 256);
        for i in 0..256 {
            if i % 16 < 8 {
                assert_eq!(baseline[i], current[i]);
            }
        }
        assert!(
            current
                .iter()
                .enumerate()
                .any(|(i, &x)| i % 16 == 8 && x > 0.0)
        );
        assert!(current.iter().all(|&x| (0.0..=1.0).contains(&x)));
    }
    #[test]
    fn added_banks_carry_cues_without_tonic_drive_or_cross_channel_leakage() {
        let features = vec!["story-scent:north".into(), "dialogue:hello".into()];
        let expanded = encode(&features, 14080);
        assert_eq!(&expanded[..256], encode(&features, 256));
        for bank in expanded.chunks(256) {
            assert!(bank.iter().any(|&x| x > 0.0));
            assert!(bank.iter().enumerate().all(|(i, &x)| x == 0.0 || [5, 8].contains(&(i % CHANNELS))));
            assert!(bank.iter().all(|&x| (0.0..=1.0).contains(&x)));
        }
        assert!(encode(&[], 14080).iter().all(|&x| x == 0.0));
    }
    #[test]
    fn sparse_cues_survive_both_projection_signs() {
        for n in 0..100 {
            let encoded = encode(&[format!("dialogue:token-{n}")], 256);
            assert!(encoded.iter().any(|&x| x > 0.0));
        }
        assert!(encode(&[], 256).iter().all(|&x| x == 0.0));
        assert!(encode(&["x".into()], 0).is_empty());
    }
    #[test]
    fn repetition_and_cycles_raise_pressure_and_novel_scenes_release_it() {
        let mut h = History::default();
        for i in 0..40 {
            h.cues(&json!({"observe":{"text":format!("tile {}",i%2)}}));
        }
        assert_eq!(h.cues(&json!({"observe":{"text":"tile 0"}})).1, 1.0);
        assert_eq!(h.cues(&json!({"observe":{"text":"new scene"}})).1, 0.0);
        assert!(h.valid());
        let restored: History = serde_json::from_str(&serde_json::to_string(&h).unwrap()).unwrap();
        assert!(restored.valid());
    }
    #[test]
    fn movement_feedback_distinguishes_turning_from_failed_displacement() {
        let before = json!({"status":{"screen":"overworld"},"map_info":{"player":{"x":2,"y":3,"facing":"up"}}});
        let mut h = History::default();
        let mut after = before.clone();
        after["map_info"]["player"]["facing"] = json!("left");
        h.feedback(&before, &after, "left");
        assert_eq!(h.movement, "motion:left:turned");
        h.feedback(&after, &after, "left");
        assert_eq!(h.movement, "motion:left:no-displacement");
        after["map_info"]["player"]["x"] = json!(1);
        h.feedback(&before, &after, "left");
        assert_eq!(h.movement, "motion:left:moved");
    }
}
