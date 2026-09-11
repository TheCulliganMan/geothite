//! Engineered reward telemetry. This evaluates consequences; it never selects
//! a button, warps the player, or changes game progression or neural weights.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct StoryLedger {
    seen: BTreeSet<String>,
    dialogue: BTreeSet<String>,
    battle_min_hp: BTreeMap<usize, u64>,
    battle_faints: BTreeSet<usize>,
    in_battle: bool,
}

fn strings(v: &Value, path: &str) -> Vec<String> {
    v.pointer(path)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn milestones(v: &Value) -> BTreeSet<String> {
    let mut marks = BTreeSet::new();
    for flag in strings(v, "/reward_state/event_flags") {
        // Completion markers only. Object visibility and temporary script flags
        // are not achievements; re-toggling any marker never pays twice.
        if [
            "EVENT_GOT_",
            "EVENT_BEAT_",
            "EVENT_GAVE_",
            "EVENT_RETURNED_",
            "EVENT_CLEARED_",
            "EVENT_RESCUED_",
            "EVENT_DELIVERED_",
            "EVENT_LEARNED_",
            "EVENT_RECEIVED_",
            "EVENT_HELPED_",
            "EVENT_TALKED_TO_",
            "EVENT_MET_",
            "EVENT_OPENED_",
            "EVENT_USED_",
            "EVENT_WALL_OPENED_",
        ]
        .iter()
        .any(|prefix| flag.starts_with(prefix))
        {
            marks.insert(format!("story:{flag}"));
        }
    }
    for (path, prefix) in [
        ("key_items", "key_item"),
        ("machines", "machine"),
        ("caught_species", "caught"),
        ("field_actions", "field_move_used"),
    ] {
        for value in strings(v, &format!("/reward_state/{path}")) {
            marks.insert(format!("{prefix}:{value}"));
        }
    }
    for flag in strings(v, "/reward_state/engine_flags") {
        if [
            "ENGINE_STRENGTH_ACTIVE",
            "ENGINE_FLASH",
            "ENGINE_POKEDEX",
            "ENGINE_POKEGEAR",
            "ENGINE_RADIO_CARD",
            "ENGINE_EXPN_CARD",
        ]
        .contains(&flag.as_str())
        {
            marks.insert(format!("capability:{flag}"));
        }
    }
    if let Some(scenes) = v.pointer("/reward_state/scenes").and_then(Value::as_object) {
        for (map, scene) in scenes {
            if let Some(scene) = scene.as_str() {
                marks.insert(format!("scene:{map}:{scene}"));
            }
        }
    }
    for region in ["johto", "kanto"] {
        if let Some(badges) = v
            .pointer(&format!("/status/badges/{region}"))
            .and_then(Value::as_array)
        {
            for (i, badge) in badges.iter().enumerate() {
                if badge.as_bool() == Some(true) {
                    marks.insert(format!("badge:{region}:{}", i + 1));
                }
            }
        }
    }
    if v.pointer("/reward_state/hall_of_fame")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        > 0
    {
        marks.insert("story:hall_of_fame".into());
    }
    if let Some(party) = v.pointer("/status/party").and_then(Value::as_array) {
        if party.iter().any(|p| p["is_egg"] != true) {
            marks.insert("story:received_starter".into());
        }
        for p in party {
            // One global level high-water prevents depositing/retrieving or
            // swapping party slots from manufacturing new achievements.
            for level in 1..=p["level"].as_u64().unwrap_or(0).min(100) {
                marks.insert(format!("level:{level}"));
            }
            for m in p["moves"].as_array().into_iter().flatten() {
                if let Some(name) = m["name"].as_str() {
                    if [
                        "CUT",
                        "FLY",
                        "SURF",
                        "STRENGTH",
                        "FLASH",
                        "WHIRLPOOL",
                        "WATERFALL",
                    ]
                    .contains(&name.to_ascii_uppercase().as_str())
                    {
                        marks.insert(format!("field_move_learned:{name}"));
                    }
                }
            }
        }
    }
    marks
}
impl StoryLedger {
    pub fn feedback(&mut self, before: &Value, after: &Value) -> (Vec<String>, Vec<String>) {
        // Baseline every action, including the first after loading a save or
        // handing control back. Human-earned progress is never credited to a cue.
        self.seen.extend(milestones(before));
        let mut reward = Vec::new();
        for mark in milestones(after) {
            if self.seen.insert(mark.clone()) {
                reward.push(mark);
            }
        }
        let object = after
            .pointer("/reward_state/last_talked_object")
            .and_then(Value::as_str)
            .unwrap_or("");
        let page = after
            .pointer("/observe/visible_dialogue")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let previous = before
            .pointer("/observe/visible_dialogue")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        // Require an actual script object, not help text or opening a menu.
        // Use page + object + map, not coordinates or a route ranking.
        if !object.is_empty() && !page.is_empty() && page != previous {
            let key = format!("{}:{object}:{page}", after["map_info"]["name"]);
            if self.dialogue.len() < 16384 && self.dialogue.insert(key) {
                reward.push(format!("dialogue:{object}"));
            }
        }
        let b = before
            .pointer("/reward_state/battle")
            .filter(|v| v.is_object());
        let a = after
            .pointer("/reward_state/battle")
            .filter(|v| v.is_object());
        if !self.in_battle || b.is_none() {
            self.battle_min_hp.clear();
            self.battle_faints.clear();
        }
        // Seed enemy HP from BEFORE the action, then reward only new lows.
        for (state, pay) in [(b, false), (a, true)] {
            if let Some(state) = state {
                let mut health = BTreeMap::new();
                for (i, enemy) in state["enemy_party"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    if let Some(hp) = enemy["hp"].as_u64() {
                        health.insert(i, hp);
                    }
                }
                // The active opponent is authoritative even when the party
                // snapshot still holds its switch-in value (or a wild party is empty).
                if let Some(hp) = state["enemy_hp"].as_u64() {
                    health.insert(state["active_enemy"].as_u64().unwrap_or(0) as usize, hp);
                }
                for (i, hp) in health {
                    let lowest = self.battle_min_hp.entry(i).or_insert(hp);
                    if hp < *lowest {
                        if pay {
                            reward.push(format!("battle:damage:{i}"));
                        }
                        *lowest = hp;
                    }
                }
                for i in state["rewarded_enemies"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_u64)
                {
                    if self.battle_faints.insert(i as usize) && pay {
                        reward.push(format!("battle:defeated:{i}"));
                    }
                }
            }
        }
        let mut aversion = Vec::new();
        if b.is_some() && a.is_none() {
            let all_fainted = after
                .pointer("/status/party")
                .and_then(Value::as_array)
                .is_some_and(|p| !p.is_empty() && p.iter().all(|p| p["hp"].as_u64() == Some(0)));
            if all_fainted {
                aversion.push("battle:party_fainted".into());
            }
            if let Some(result) = after
                .pointer("/reward_state/battle_result")
                .and_then(Value::as_u64)
            {
                match result & 0x3f {
                    0 => reward.push(
                        if result & 0x40 != 0 {
                            "battle:capture"
                        } else {
                            "battle:victory"
                        }
                        .into(),
                    ),
                    1 => {
                        if aversion.is_empty() {
                            aversion.push("battle:defeat".into());
                        }
                    }
                    _ => {} // Running away is not victory.
                }
            }
        }
        self.in_battle = a.is_some();
        (reward, aversion)
    }
    pub fn report(&self) -> Value {
        serde_json::json!({"milestones":self.seen,"teacher":"story-events-v1"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn observation() -> Value {
        json!({"status":{"party":[],"badges":{"johto":vec![false;8],"kanto":vec![false;8]}},
            "map_info":{"name":"ElmsLab"},"observe":{"visible_dialogue":""},
            "reward_state":{"version":1,"event_flags":[],"key_items":[],"machines":[],"caught_species":[],"battle":null}})
    }
    #[test]
    fn story_hms_and_all_badges_pay_once_and_never_on_load() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        for flag in [
            "EVENT_GOT_A_POKEMON_FROM_ELM",
            "EVENT_GOT_MYSTERY_EGG_FROM_MR_POKEMON",
            "EVENT_GAVE_MYSTERY_EGG_TO_ELM",
            "EVENT_BEAT_ROCKET_EXECUTIVEM_3",
            "EVENT_RETURNED_MACHINE_PART",
            "EVENT_BEAT_RED",
        ] {
            let mut after = before.clone();
            after["reward_state"]["event_flags"]
                .as_array_mut()
                .unwrap()
                .push(json!(flag));
            assert!(
                ledger
                    .feedback(&before, &after)
                    .0
                    .contains(&format!("story:{flag}"))
            );
            assert!(ledger.feedback(&before, &after).0.is_empty());
            assert!(StoryLedger::default().feedback(&after, &after).0.is_empty());
            before = after;
        }
        for i in 1..=7 {
            let mut after = before.clone();
            after["reward_state"]["machines"]
                .as_array_mut()
                .unwrap()
                .push(json!(format!("HM0{i}")));
            assert!(
                ledger
                    .feedback(&before, &after)
                    .0
                    .contains(&format!("machine:HM0{i}"))
            );
            before = after;
        }
        for region in ["johto", "kanto"] {
            for i in 0..8 {
                let mut after = before.clone();
                after["status"]["badges"][region][i] = json!(true);
                assert!(
                    ledger
                        .feedback(&before, &after)
                        .0
                        .contains(&format!("badge:{region}:{}", i + 1))
                );
                before = after;
            }
        }
        let mut after = before.clone();
        after["reward_state"]["hall_of_fame"] = json!(1);
        assert!(
            ledger
                .feedback(&before, &after)
                .0
                .contains(&"story:hall_of_fame".into())
        );
    }
    #[test]
    fn field_moves_and_quest_scenes_pay_on_real_transitions() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        before["status"]["party"] = json!([{"level":5,"hp":20,"is_egg":false,"moves":[]}]);
        before["reward_state"]["field_actions"] = json!([]);
        for name in [
            "CUT",
            "FLY",
            "SURF",
            "STRENGTH",
            "FLASH",
            "WHIRLPOOL",
            "WATERFALL",
        ] {
            let mut after = before.clone();
            after["status"]["party"][0]["moves"]
                .as_array_mut()
                .unwrap()
                .push(json!({"name":name}));
            assert!(
                ledger
                    .feedback(&before, &after)
                    .0
                    .contains(&format!("field_move_learned:{name}"))
            );
            before = after;
            let mut after = before.clone();
            if name == "STRENGTH" {
                after["reward_state"]["engine_flags"] = json!(["ENGINE_STRENGTH_ACTIVE"]);
                assert!(
                    ledger
                        .feedback(&before, &after)
                        .0
                        .contains(&"capability:ENGINE_STRENGTH_ACTIVE".into())
                );
            } else {
                after["reward_state"]["field_actions"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(name.to_lowercase()));
                assert!(
                    ledger
                        .feedback(&before, &after)
                        .0
                        .contains(&format!("field_move_used:{}", name.to_lowercase()))
                );
            }
            before = after;
        }
        let mut after = before.clone();
        after["reward_state"]["scenes"] = json!({"ElmsLab":"SCENE_ELMSLAB_NOTHING"});
        assert!(
            ledger
                .feedback(&before, &after)
                .0
                .contains(&"scene:ElmsLab:SCENE_ELMSLAB_NOTHING".into())
        );
        assert!(ledger.feedback(&before, &after).0.is_empty());
    }
    #[test]
    fn battle_damage_cannot_be_farmed_by_healing_and_fleeing_does_not_pay() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        before["reward_state"]["battle"] =
            json!({"enemy_party":[{"hp":100}],"rewarded_enemies":[]});
        let mut after = before.clone();
        after["reward_state"]["battle"]["enemy_party"][0]["hp"] = json!(60);
        assert_eq!(ledger.feedback(&before, &after).0, vec!["battle:damage:0"]);
        assert!(ledger.feedback(&after, &before).0.is_empty());
        assert!(ledger.feedback(&before, &after).0.is_empty());
        let mut exit = after.clone();
        exit["reward_state"]["battle"] = Value::Null;
        for (result, expected) in [
            (2, None),
            (0, Some("battle:victory")),
            (64, Some("battle:capture")),
        ] {
            exit["reward_state"]["battle_result"] = json!(result);
            let rewards = ledger.feedback(&after, &exit).0;
            assert_eq!(
                rewards,
                expected.into_iter().map(str::to_owned).collect::<Vec<_>>()
            );
        }
        exit["reward_state"]["battle_result"] = json!(1);
        assert_eq!(ledger.feedback(&after, &exit).1, vec!["battle:defeat"]);
    }
    #[test]
    fn active_wild_opponent_hp_is_authoritative() {
        let mut before = observation();
        before["reward_state"]["battle"] =
            json!({"enemy_party":[],"active_enemy":null,"enemy_hp":20});
        let mut after = before.clone();
        after["reward_state"]["battle"]["enemy_hp"] = json!(5);
        assert_eq!(
            StoryLedger::default().feedback(&before, &after).0,
            vec!["battle:damage:0"]
        );
    }
    #[test]
    fn walking_menu_toggling_and_repeat_dialogue_do_not_pay() {
        let mut ledger = StoryLedger::default();
        let before = observation();
        let mut after = before.clone();
        after["map_info"]["name"] = json!("Route30");
        after["observe"]["menus"] = json!([{"kind":"party"}]);
        assert!(ledger.feedback(&before, &after).0.is_empty());
        after["reward_state"]["last_talked_object"] = json!("ElmsLab_Elm");
        after["observe"]["visible_dialogue"] = json!("Take a Pokemon.");
        assert_eq!(
            ledger.feedback(&before, &after).0,
            vec!["dialogue:ElmsLab_Elm"]
        );
        assert!(ledger.feedback(&before, &after).0.is_empty());
        let encoded = serde_json::to_string(&ledger).unwrap();
        let mut restored: StoryLedger = serde_json::from_str(&encoded).unwrap();
        assert!(restored.feedback(&before, &after).0.is_empty());
    }
}
