use crate::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RewardConfig {
    pub enabled: bool,
    pub novelty: f32,
    pub badge: f32,
    pub starter: f32,
    pub level_up: f32,
    pub collision: f32,
    pub collision_attempts: u32,
    pub story: f32,
    pub approach: f32,
    pub dialogue: f32,
}
impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            novelty: 0.25,
            badge: 2.0,
            starter: 1.0,
            level_up: 0.5,
            collision: 0.5,
            collision_attempts: 3,
            story: 1.5,
            approach: 0.15,
            dialogue: 0.4,
        }
    }
}
impl RewardConfig {
    pub fn validate(&self) -> bool {
        [
            self.novelty,
            self.badge,
            self.starter,
            self.level_up,
            self.collision,
            self.story,
            self.approach,
            self.dialogue,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= 0.0 && *v <= 5.0)
            && self.collision_attempts > 0
    }
}
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Campaign {
    visited: BTreeSet<String>,
    #[serde(default)]
    story_milestones: BTreeSet<String>,
    #[serde(default)]
    closest: BTreeMap<String, i64>,
    #[serde(default)]
    dialogue_pages: BTreeMap<String, BTreeSet<String>>,
    #[serde(default)]
    assisted: bool,
    #[serde(default)]
    interventions: BTreeMap<String, u64>,
    #[serde(default)]
    decisions: u64,
    #[serde(default)]
    buttons: BTreeMap<String, u64>,
    #[serde(default)]
    history: Vec<serde_json::Value>,
    badges: BTreeSet<String>,
    previous_position: Option<String>,
    previous_screen: String,
    collisions: u32,
    events: u64,
    #[serde(default)]
    starter_seen: bool,
    #[serde(default)]
    party_levels: Vec<u64>,
    #[serde(default)]
    best_party_level_total: u64,
}
#[wasm_bindgen]
impl Brain {
    pub fn mark_intervention(&mut self, reason: &str) -> Result<(), String> {
        if ![
            "manual_appetitive",
            "manual_aversive",
            "human_input",
            "tuning",
            "lab_conditioning",
            "erased_memory",
            "restored_checkpoint",
        ]
        .contains(&reason)
        {
            return Err("Unknown intervention category".into());
        }
        self.campaign.assisted = true;
        *self
            .campaign
            .interventions
            .entry(reason.into())
            .or_default() += 1;
        self.record_event(serde_json::json!({"kind":"intervention","reason":reason,"tick":self.tick,"config":self.config}));
        Ok(())
    }
    pub fn run_record(&self) -> Result<String, String> {
        serde_json::to_string(&serde_json::json!({"model_id":MODEL_ID,"interface_id":INTERFACE_ID,"graph_id":self.graph_id,"config":self.config,
            "assisted":self.campaign.assisted,"interventions":self.campaign.interventions,"decisions":self.campaign.decisions,
            "buttons":self.campaign.buttons,"visited_tiles":self.campaign.visited.len(),"badges":self.campaign.badges,
            "story_milestones":self.campaign.story_milestones,"approach_best_distances":self.campaign.closest,"dialogue_pages":self.campaign.dialogue_pages,
            "operant":self.operant_state.report(),"outcome_events":self.campaign.events,"recent_history":self.campaign.history,
            "history_limit":64,"scope":"Local experimental controller; record is not a tamper-proof certificate or proof of learning"})).map_err(|e|e.to_string())
    }
    /// Evaluator is isolated from the controller; it can only schedule DAN pulses.
    pub fn outcome(&mut self, json: &str, button: &str) -> Result<String, String> {
        let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let screen = v
            .pointer("/status/screen")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let position = format!(
            "{}:{}:{}",
            v.pointer("/map_info/name")
                .unwrap_or(&serde_json::Value::Null),
            v.pointer("/map_info/player/x")
                .unwrap_or(&serde_json::Value::Null),
            v.pointer("/map_info/player/y")
                .unwrap_or(&serde_json::Value::Null)
        );
        let mut events = Vec::new();
        let story = self.campaign.story_feedback(&v, button);
        if !story.arrivals.is_empty() {
            events.push("story_location_first_visit");
        }
        if story.approached {
            events.push("closer_to_story_character");
        }
        if story.dialogue {
            events.push("new_story_dialogue_page");
        }
        if screen == "overworld" && self.campaign.previous_screen == "overworld" {
            if self.campaign.previous_position.as_deref() != Some(&position) {
                self.campaign.collisions = 0;
                if self.campaign.visited.insert(position.clone()) {
                    events.push("first_visit");
                }
            } else if ["up", "down", "left", "right"].contains(&button)
                && v.pointer("/observe/menus")
                    .and_then(|v| v.as_array())
                    .is_some_and(|m| m.is_empty())
                && !v
                    .pointer("/flow_state/animating")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true)
                && v.pointer("/observe/visible_dialogue")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .is_empty()
            {
                self.campaign.collisions += 1;
                if self.campaign.collisions == self.config.rewards.collision_attempts {
                    events.push("repeated_position");
                    self.campaign.collisions = 0;
                }
            } else {
                self.campaign.collisions = 0;
            }
        } else {
            self.campaign.collisions = 0;
        }
        // The game's actual schema stores two ordered boolean badge arrays.
        for region in ["johto", "kanto"] {
            if let Some(badges) = v
                .pointer(&format!("/status/badges/{region}"))
                .and_then(|v| v.as_array())
            {
                for (index, earned) in badges.iter().enumerate() {
                    if earned.as_bool() == Some(true)
                        && self.campaign.badges.insert(format!("{region}:{index}"))
                        && self.campaign.previous_position.is_some()
                    {
                        events.push("new_badge");
                    }
                }
            }
        }
        if let Some(party) = v.pointer("/status/party").and_then(|v| v.as_array()) {
            if !party.is_empty() && !self.campaign.starter_seen {
                if self.campaign.previous_position.is_some() {
                    events.push("first_party_member");
                }
                self.campaign.starter_seen = true;
            }
            let levels = party
                .iter()
                .map(|p| p["level"].as_u64().unwrap_or(0))
                .collect::<Vec<_>>();
            if levels.len() == self.campaign.party_levels.len()
                && levels.iter().sum::<u64>() > self.campaign.best_party_level_total
            {
                events.push("party_level_increase");
            }
            self.campaign.best_party_level_total = self
                .campaign
                .best_party_level_total
                .max(levels.iter().sum());
            self.campaign.party_levels = levels;
        }
        let reward = events
            .iter()
            .map(|e| match *e {
                "story_location_first_visit" => self.config.rewards.story,
                "closer_to_story_character" => self.config.rewards.approach,
                "new_story_dialogue_page" => self.config.rewards.dialogue,
                "first_visit" => self.config.rewards.novelty,
                "new_badge" => self.config.rewards.badge,
                "first_party_member" => self.config.rewards.starter,
                "party_level_increase" => self.config.rewards.level_up,
                _ => 0.0,
            })
            .fold(0.0, f32::max);
        let punishment = if events.contains(&"repeated_position") {
            self.config.rewards.collision
        } else {
            0.0
        };
        if self.config.rewards.enabled {
            if reward > 0.0 {
                let previous = self.reward_ticks;
                self.appetitive_pulse()?;
                self.reward_ticks = previous
                    .max((reward * self.config.reward_pulse_ms / self.config.dt_ms).round() as u64);
            }
            if punishment > 0.0 {
                let previous = self.pulse_ticks;
                self.aversive_pulse()?;
                self.pulse_ticks = previous.max(
                    (punishment * self.config.reward_pulse_ms / self.config.dt_ms).round() as u64,
                );
            }
        }
        self.campaign.decisions += 1;
        *self
            .campaign
            .buttons
            .entry(if button.is_empty() { "no-op" } else { button }.into())
            .or_default() += 1;
        if !events.is_empty() {
            self.record_event(serde_json::json!({"kind":"outcome","tick":self.tick,"events":events,"position":position,
                "story":story,"reward_scale":reward,"aversive_scale":punishment,"pulses_enabled":self.config.rewards.enabled}));
        }
        self.campaign.visited.insert(position.clone());
        self.campaign.previous_position = Some(position);
        self.campaign.previous_screen = screen.into();
        self.campaign.events += events.len() as u64;
        serde_json::to_string(&serde_json::json!({"events":events,"reward_scale":reward,"punishment_scale":punishment,"enabled":self.config.rewards.enabled,"visited":self.campaign.visited.len(),"total_events":self.campaign.events,
            "story":story,"rule":"declared story-location curriculum, observed-character distance high-water and capped dialogue novelty; reward schedules DAN current only"})).map_err(|e|e.to_string())
    }
}

impl Brain {
    fn record_event(&mut self, event: serde_json::Value) {
        if self.campaign.history.len() >= 64 {
            self.campaign.history.remove(0);
        }
        self.campaign.history.push(event);
    }
}

#[derive(Default, Serialize)]
struct StoryFeedback {
    arrivals: Vec<String>,
    approached: bool,
    dialogue: bool,
    target: Option<String>,
    closest_distance: Option<i64>,
    milestones: Vec<String>,
}
impl Campaign {
    /// Engineered reward curriculum, isolated from sensory encoding and decoding.
    /// Locations certify arrival only, never a conversation or completed quest.
    fn story_feedback(&mut self, v: &serde_json::Value, button: &str) -> StoryFeedback {
        let mut result = StoryFeedback::default();
        if v.pointer("/status/screen").and_then(|v| v.as_str()) != Some("overworld") {
            return result;
        }
        let map = v
            .pointer("/map_info/name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let (label, character) = match map {
            "PlayersHouse1F" => ("Mom's floor", "mom"),
            "NewBarkTown" => ("New Bark Town", ""),
            "ElmsLab" => ("Elm's lab", "elm"),
            "Route29" => ("Route 29", ""),
            "CherrygroveCity" => ("Cherrygrove City", ""),
            "Route30" => ("Route 30", ""),
            "MrPokemonsHouse" => ("Mr. Pokémon's house", "mrpokemon"),
            _ => ("", ""),
        };
        // The first observation establishes a baseline; loading an advanced save
        // does not itself earn a new-arrival pulse.
        if !label.is_empty()
            && self.story_milestones.insert(label.into())
            && self.previous_position.is_some()
        {
            result.arrivals.push(label.into());
        }
        if !character.is_empty() {
            result.target = Some(label.into());
            let px = v.pointer("/map_info/player/x").and_then(|v| v.as_i64());
            let py = v.pointer("/map_info/player/y").and_then(|v| v.as_i64());
            if let (Some(px), Some(py), Some(objects)) = (
                px,
                py,
                v.pointer("/map_info/objects").and_then(|v| v.as_array()),
            ) {
                for object in objects {
                    let name = object["name"].as_str().unwrap_or("");
                    let normalized: String = name
                        .chars()
                        .filter(|c| c.is_ascii_alphanumeric())
                        .flat_map(char::to_lowercase)
                        .collect();
                    if !normalized.contains(character) {
                        continue;
                    }
                    if let (Some(x), Some(y)) = (object["x"].as_i64(), object["y"].as_i64()) {
                        let distance = (px - x).abs() + (py - y).abs();
                        let key = format!("{map}:{name}");
                        // Only player movement, not an NPC walking toward us,
                        // can earn a distance improvement. Strict lifetime minima
                        // prevent pacing and re-entry from farming reward.
                        if let Some(best) = self.closest.get_mut(&key) {
                            if distance < *best {
                                result.approached |= ["up", "down", "left", "right"]
                                    .contains(&button)
                                    && self.previous_position.as_deref()
                                        != Some(&format!(
                                            "{}:{}:{}",
                                            v["map_info"]["name"], px, py
                                        ));
                                *best = distance;
                            }
                        } else {
                            self.closest.insert(key, distance);
                        }
                        result.closest_distance = Some(
                            result
                                .closest_distance
                                .map_or(distance, |d| d.min(distance)),
                        );
                    }
                }
            }
            if let Some(text) = v
                .pointer("/observe/visible_dialogue")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
            {
                // This is dialogue in the target location, not speaker recognition.
                let pages = self.dialogue_pages.entry(map.into()).or_default();
                let page = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if pages.len() < 8 && pages.insert(page) && self.previous_position.is_some() {
                    result.dialogue = true;
                }
            }
        }
        result.milestones = self.story_milestones.iter().cloned().collect();
        result
    }
}
