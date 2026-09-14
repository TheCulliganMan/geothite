//! Engineered reward telemetry. This evaluates consequences; it never selects
//! a button, warps the player, or changes game progression or neural weights.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// A field script may remain suspended throughout a trainer battle. Its
// animation flag does not mean an explicitly visible battle menu is blocked.
fn battle_menu_ready(v: &Value) -> bool {
    v.pointer("/status/screen").and_then(Value::as_str) == Some("battle")
        && v.pointer("/reward_state/battle").is_some_and(Value::is_object)
        && v.pointer("/observe/battle_message").and_then(Value::as_str).is_none_or(|s| s.is_empty())
        && v.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|menus| menus.iter().any(|m| {
            m["kind"].as_str() == Some("battle") && m["entries"].as_array().is_some_and(|entries| !entries.is_empty())
        }))
}
fn unchanged_battle_menu(before: &Value, after: &Value) -> bool {
    battle_menu_ready(before) && battle_menu_ready(after)
        && before.pointer("/frame").and_then(Value::as_u64).zip(after.pointer("/frame").and_then(Value::as_u64)).is_some_and(|(a,b)| b > a)
        && ["/observe/menus", "/reward_state/battle", "/status/party"].iter().all(|p| before.pointer(p) == after.pointer(p))
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct StoryLedger {
    seen: BTreeSet<String>,
    dialogue: BTreeSet<String>,
    battle_min_hp: BTreeMap<usize, u64>,
    battle_faints: BTreeSet<usize>,
    battle_effects: BTreeSet<String>,
    in_battle: bool,
    stagnant_actions: u64,
    switch_screen_actions: u64,
    unproductive_actions: u64,
    explored_tiles: BTreeSet<String>,
    recent_tiles: Vec<String>,
    last_dialogue_object: String,
    // Restoration earns at most once per set of permanent achievements. Menu
    // toggles, HP loss and repeat battles do not replenish this budget.
    restoration_epochs: BTreeSet<String>,
    menu_openings_since_progress: u64,
    menu_reopen_cooldown: u64,
    battle_pack_visits_without_effect: u64,
    battle_pack_reopen_cooldown: u64,
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
// Only inspect visible refusal text, never hidden game state for action choice.
fn refusal_text(text: &str) -> bool {
    let text = text.to_lowercase().replace('’', "'");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    [
        "isn't the time",
        "not the time",
        "can't use",
        "cannot use",
        "can't be used here",
        "nothing to cut",
        "nothing registered",
        "no registered item",
    ]
    .iter()
    .any(|phrase| text.contains(phrase))
}
fn new_refusal(before: &Value, after: &Value) -> bool {
    ["/observe/visible_dialogue", "/recent_events/last_action"]
        .iter()
        .any(|path| {
            let text = after.pointer(path).and_then(Value::as_str).unwrap_or("");
            refusal_text(text) && before.pointer(path) != after.pointer(path)
        })
}

fn milestones(v: &Value) -> BTreeSet<String> {
    let mut marks = BTreeSet::new();
    if v.pointer("/status/screen").and_then(Value::as_str) == Some("overworld") {
        if let Some(map) = v
            .pointer("/map_info/name")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
        {
            marks.insert(format!("place:{map}"));
        }
    }
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
    for item in v.pointer("/reward_state/items").and_then(Value::as_array).into_iter().flatten() {
        if item["quantity"].as_u64().unwrap_or(0)>0 {
            if let Some(id)=item["id"].as_str() { marks.insert(format!("item_discovered:{id}")); }
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
    pub fn menu_reopen_cooldown(&self) -> u64 { self.menu_reopen_cooldown }
    pub fn battle_pack_reopen_cooldown(&self) -> u64 { self.battle_pack_reopen_cooldown }

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
        if !object.is_empty() && !page.is_empty() && page != previous && !refusal_text(page) {
            let stable_page: String = page.chars().map(|c| if c.is_ascii_digit() { '#' } else { c }).collect();
            let key = format!("{}:{object}:{stable_page}", after["map_info"]["name"]);
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
            self.battle_effects.clear();
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
                // Effective setup can contribute before damage occurs. Pay each
                // effect once per opponent/battle, never merely selecting a move.
                let enemy=state["active_enemy"].as_u64().unwrap_or(0);
                let mut effects=Vec::new();
                if !state["enemy_status"].is_null() { effects.push(format!("status:{enemy}:{}",state["enemy_status"])); }
                if state["enemy_wrapped"]==true { effects.push(format!("trapped:{enemy}")); }
                if state["player_substitute_hp"].as_u64().unwrap_or(0)>0 { effects.push("substitute".into()); }
                if state["player_mist_active"]==true { effects.push("mist".into()); }
                for effect in effects {
                    if self.battle_effects.insert(effect.clone()) && pay { reward.push(format!("battle:setup:{effect}")); }
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
        // Match party slots conservatively: swapping/depositing must not look
        // like healing. A blackout is explicitly excluded by the loss penalty.
        if aversion.is_empty() {
            if let (Some(old),Some(new))=(before.pointer("/status/party").and_then(Value::as_array),after.pointer("/status/party").and_then(Value::as_array)) {
                let same_party=old.len()==new.len() && old.iter().zip(new).all(|(b,a)| b["nickname"]==a["nickname"] && b["slot"]==a["slot"] && b["max_hp"]==a["max_hp"]);
                if same_party {
                    let restored=old.iter().zip(new).any(|(b,a)| {
                        let hp=b["hp"].as_u64().zip(a["hp"].as_u64()).is_some_and(|(b,a)| a>b);
                        let cured=!b["status"].is_null() && a["status"].is_null();
                        let pp=b["moves"].as_array().zip(a["moves"].as_array()).is_some_and(|(bm,am)| bm.len()==am.len() && bm.iter().zip(am).any(|(b,a)| b["name"]==a["name"] && b["current_pp"].as_u64().zip(a["current_pp"].as_u64()).is_some_and(|(b,a)| a>b)));
                        hp || cured || pp
                    });
                    let epoch=self.seen.iter().filter(|s| s.starts_with("story:") || s.starts_with("badge:") || s.starts_with("place:")).cloned().collect::<Vec<_>>().join("|");
                    if restored && self.restoration_epochs.insert(epoch) { reward.push("resource:restored_party".into()); }
                }
            }
        }
        self.in_battle = a.is_some();
        // A blackout can heal/warp the party before this observation. The battle
        // result remains authoritative, and a loss never pays for the new place.
        if !aversion.is_empty() {
            reward.clear();
        }
        (reward, aversion)
    }
    pub fn action_feedback(
        &mut self,
        before: &Value,
        after: &Value,
        button: &str,
    ) -> (Vec<String>, Vec<String>) {
        // Browsing and closing menus must not exhaust the field-action gap.
        let field_action=[before,after].iter().all(|v|v.pointer("/status/screen").and_then(Value::as_str)==Some("overworld")
            && v.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|m|m.is_empty())
            && v.pointer("/flow_state/animating")!=Some(&Value::Bool(true)));
        if field_action {self.menu_reopen_cooldown=self.menu_reopen_cooldown.saturating_sub(1);}
        self.battle_pack_reopen_cooldown=self.battle_pack_reopen_cooldown.saturating_sub(1);
        let (mut rewards, mut aversions) = self.feedback(before, after);
        let in_battle=|v:&Value| v.pointer("/status/screen").and_then(Value::as_str)==Some("battle");
        let real_battle_effect=["/reward_state/items","/status/party","/reward_state/battle/player_turns","/reward_state/battle/enemy_turns","/reward_state/battle/enemy_hp","/reward_state/battle/enemy_status"]
            .iter().any(|p| before.pointer(p)!=after.pointer(p));
        if !in_battle(before) || !in_battle(after) || real_battle_effect {
            self.battle_pack_visits_without_effect=0;
            self.battle_pack_reopen_cooldown=0;
        } else if button=="a" {
            let rows=|v:&Value| v.pointer("/observe/menus").and_then(Value::as_array)
                .and_then(|menus|menus.iter().rev().find(|m|m["kind"]=="battle"))
                .and_then(|m|m["entries"].as_array()).cloned().unwrap_or_default();
            let b=rows(before);let a=rows(after);
            let main=|rows:&[Value]| rows.iter().any(|r| r.as_str().is_some_and(|s|s.trim().trim_start_matches('>').trim()=="FIGHT"))
                && rows.iter().any(|r| r.as_str().is_some_and(|s|s.trim().trim_start_matches('>').trim()=="PACK"));
            let selected_pack=b.iter().any(|r|r.as_str().is_some_and(|s|s.trim()==">PACK"));
            if main(&b) && selected_pack && !a.is_empty() && !main(&a) {
                self.battle_pack_visits_without_effect=self.battle_pack_visits_without_effect.saturating_add(1);
                if self.battle_pack_visits_without_effect>=3 {
                    aversions.push("action:battle_pack_loop".into());
                    self.battle_pack_reopen_cooldown=24;
                }
            }
        }

        // A visible rejection is the consequence of the initiating menu A,
        // not useful dialogue and not the later A/B that dismisses the message.
        if button=="a" && in_battle(before) && in_battle(after) && !real_battle_effect {
            let selected=before.pointer("/observe/menus").and_then(Value::as_array).and_then(|ms|ms.last())
                .and_then(|m|m["entries"].as_array()).and_then(|rows|rows.iter().filter_map(Value::as_str).find(|s|s.trim().starts_with('>')))
                .map(|s|s.trim().trim_start_matches('>').trim());
            let message=after.pointer("/observe/battle_message").and_then(Value::as_str).unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
            if selected==Some("SWITCH") && message.ends_with(" is already out.") {
                aversions.push("action:already_active_switch".into());
            }
            if selected==Some("RUN") && before.pointer("/observe/battle").and_then(Value::as_str).is_some_and(|s|s.starts_with("Trainer "))
                && message.contains("no running from a trainer battle") {
                aversions.push("action:trainer_escape_rejected".into());
            }
        }

        if after.pointer("/flygon_guard").and_then(Value::as_str) == Some("profile_blocked") {
            aversions.push("action:profile_blocked".into());
        }
        let before_page = before
            .pointer("/observe/visible_dialogue")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let setup_screen = after.pointer("/status/screen").and_then(Value::as_str).unwrap_or("");
        if matches!(setup_screen,"intro"|"title"|"clock"|"introduction"|"gender"|"naming")
            && button == "a" && before.pointer("/observe/text") != after.pointer("/observe/text") {
            let text=after.pointer("/observe/text").and_then(Value::as_str).unwrap_or("");
            let page:String=text.chars().map(|c|if c.is_ascii_digit(){'#'}else{c}).collect();
            if !page.is_empty() && self.seen.insert(format!("setup-page:{setup_screen}:{page}")) {
                rewards.push(format!("setup:advance:{setup_screen}"));
            }
        }
        let after_page = after
            .pointer("/observe/visible_dialogue")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let talked_object = after
            .pointer("/reward_state/last_talked_object")
            .and_then(Value::as_str)
            .unwrap_or("");
        let opened_dialogue =
            before_page.is_empty() && !after_page.is_empty() && !talked_object.is_empty();
        if !before_page.is_empty() && !refusal_text(before_page) && after_page.is_empty() && !talked_object.is_empty()
            && before.pointer("/map_info/name") == after.pointer("/map_info/name")
            && after.pointer("/status/screen").and_then(Value::as_str) == Some("overworld")
            && after.pointer("/observe/menus").and_then(Value::as_array).is_none_or(|m|m.is_empty())
            && after.pointer("/flow_state/animating").and_then(Value::as_bool) != Some(true)
            && matches!(button,"a"|"b")
            && self.seen.insert(format!("conversation-complete:{}:{talked_object}",after["map_info"]["name"])) {
            rewards.push(format!("dialogue:complete:{talked_object}"));
        }

        if opened_dialogue {
            let novel_dialogue = rewards.iter().any(|event| event.starts_with("dialogue:"));
            if talked_object == self.last_dialogue_object && !novel_dialogue {
                aversions.push(format!("action:repeat_npc:{talked_object}"));
            }
            self.last_dialogue_object = talked_object.into();
        }
        if button == "select" {
            aversions.push("action:select".into());
        }
        // Event-local detection: old status text must not punish dismissal or
        // suppress a later, unrelated NPC page. Select itself always costs.
        if new_refusal(before, after) {
            aversions.push("action:refused".into());
        }
        let animating = before
            .pointer("/flow_state/animating")
            .and_then(Value::as_bool)
            == Some(true)
            || after
                .pointer("/flow_state/animating")
                .and_then(Value::as_bool)
                == Some(true);
        let animating = animating && !(battle_menu_ready(before) && battle_menu_ready(after));
        if unchanged_battle_menu(before, after) {
            aversions.push("action:unchanged_battle_menu".into());
        }
        if !aversions.is_empty() {
            rewards.clear();
        }
        let tile = |v: &Value| -> Option<String> {
            let map = v.pointer("/map_info/name")?.as_str()?;
            let x = v.pointer("/map_info/player/x")?.as_i64()?;
            let y = v.pointer("/map_info/player/y")?.as_i64()?;
            Some(format!("{map}:{x}:{y}"))
        };
        if let Some(tile) = tile(before) {
            self.explored_tiles.insert(tile);
        }
        let moved = tile(before).is_some() && tile(before) != tile(after);
        let overworld =
            after.pointer("/status/screen").and_then(Value::as_str) == Some("overworld");
        let menu_open = [before, after].iter().any(|v| {
            v.pointer("/observe/menus")
                .and_then(Value::as_array)
                .is_some_and(|m| !m.is_empty())
        });
        if !animating
            && overworld
            && before_page.is_empty()
            && after_page.is_empty()
            && !menu_open
            && matches!(button, "up" | "down" | "left" | "right")
            && !moved
            && before.pointer("/map_info/player/facing") == after.pointer("/map_info/player/facing")
        {
            aversions.push("action:blocked_movement".into());
        }
        if !animating && aversions.is_empty() && before.pointer("/reward_state/battle").filter(|v|v.is_object()).is_none() && after.pointer("/reward_state/battle").filter(|v|v.is_object()).is_none() && overworld && before.pointer("/status/screen").and_then(Value::as_str)==Some("overworld") && rewards.is_empty() && before_page.is_empty() && after_page.is_empty() && !menu_open
            && !moved && matches!(button,"a"|"b") {
            aversions.push("action:idle_input".into());
        }
        let opened_start = overworld && button == "start"
            && before.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|m| m.is_empty())
            && after.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|m| m.iter().any(|m| m["kind"] == "start"));
        if opened_start {
            self.menu_reopen_cooldown=32;
            self.menu_openings_since_progress = self.menu_openings_since_progress.saturating_add(1);
            if self.menu_openings_since_progress > 1 {
                aversions.push("action:repeated_menu_open".into());
            }
        }
        let no_party=before.pointer("/status/party").and_then(Value::as_array).is_some_and(|p|p.is_empty());
        if no_party && overworld && button=="start" && before.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|m|m.is_empty()) && menu_open {
            aversions.push("action:unneeded_menu".into());
        }
        // Narrative choices and Pokemon pictures are required starter interactions.
        // Only optional utility menus count as distractions before receiving a party.
        if no_party && overworld && before.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|menus| menus.iter().any(|m| matches!(m["kind"].as_str(), Some("start" | "party" | "pack" | "pokedex" | "pokegear" | "options" | "trainer_card" | "save")))) {
            if after.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|m|m.is_empty()) {
                rewards.push("interface:closed_unneeded_menu".into());
            } else { aversions.push("inaction:unneeded_menu".into()); }
        }
        if moved
            && rewards.is_empty()
            && tile(after).is_some_and(|t| {
                self.recent_tiles
                    .iter()
                    .rev()
                    .take(12)
                    .filter(|old| **old == t)
                    .count()
                    >= 3
            })
        {
            aversions.push("action:short_loop".into());
        }
        // Pay real setup-stage transitions once, so the cold brain can leave
        // clock/name setup without a scripted boot sequence.
        if before.pointer("/status/screen") != after.pointer("/status/screen") {
            let stage = after
                .pointer("/status/screen")
                .and_then(Value::as_str)
                .unwrap_or("");
            if matches!(stage, "title" | "clock" | "intro" | "naming" | "overworld")
                && self.seen.insert(format!("setup:{stage}"))
            {
                rewards.push(format!("setup:{stage}"));
            }
        }
        if moved {
            if let Some(tile) = tile(after) {
                if self.recent_tiles.len() >= 64 {
                    self.recent_tiles.remove(0);
                }
                self.recent_tiles.push(tile);
            }
        }
        let varied_travel = moved && self.recent_tiles.iter().collect::<BTreeSet<_>>().len() >= 16;
        let battle_turn = ["player_turns", "enemy_turns"].iter().any(|key| {
            let path = format!("/reward_state/battle/{key}");
            match (
                before.pointer(&path).and_then(Value::as_u64),
                after.pointer(&path).and_then(Value::as_u64),
            ) {
                (Some(b), Some(a)) => a > b,
                _ => false,
            }
        });
        let exploring = after.pointer("/status/screen").and_then(Value::as_str)
            == Some("overworld")
            && tile(after).is_some_and(|tile| self.explored_tiles.insert(tile));
        // Cursor movement and refusal text are not progress toward leaving the
        // switch selector. Allow enough inputs to reach any party slot first.
        let switch_open = |v: &Value| {
            v.pointer("/observe/pokemon_switch_open").and_then(Value::as_bool) == Some(true)
        };
        if !switch_open(before) || !switch_open(after) || battle_turn {
            self.switch_screen_actions = 0;
        } else if !animating {
            self.switch_screen_actions = self.switch_screen_actions.saturating_add(1);
            if self.switch_screen_actions >= 12 {
                rewards.clear();
                aversions.push("inaction:pokemon_switch".into());
            }
        }
        if !rewards.is_empty() || (exploring && aversions.is_empty()) {
            self.stagnant_actions = 0;
            self.unproductive_actions = 0;
            self.menu_openings_since_progress = 0;
        } else if battle_turn && aversions.is_empty() {
            // Status moves, healing and switching are legitimate battle turns.
            // They buy time but do not generate positive conditioning.
            self.stagnant_actions = 0;
            self.unproductive_actions = self.unproductive_actions.saturating_sub(16);
        } else if !animating {
            self.unproductive_actions = self.unproductive_actions.saturating_add(1);
            let changed = [
                "/map_info/name",
                "/map_info/player/x",
                "/map_info/player/y",
                "/observe/visible_dialogue",
                "/observe/text",
                "/observe/menus",
                "/observe/battle_message",
                "/reward_state/battle/player_turns",
                "/reward_state/battle/enemy_turns",
            ]
            .iter()
            .any(|path| before.pointer(path) != after.pointer(path));
            self.stagnant_actions = if changed {
                0
            } else {
                self.stagnant_actions.saturating_add(1)
            };
            if self.stagnant_actions >= 12 {
                aversions.push("inaction:stalled".into());
            }
            // Long varied return trips stay neutral. Repeated small loops and
            // menus eventually cost; no distance-to-goal heuristic is involved.
            if self.unproductive_actions >= 256 && !varied_travel {
                aversions.push("inaction:no_progress".into());
            }
        }
        (rewards, aversions)
    }

    pub fn report(&self) -> Value {
        serde_json::json!({"milestones":self.seen,"teacher":"story-events-v1",
            "switch_screen_actions":self.switch_screen_actions,
            "battle_pack_visits_without_effect":self.battle_pack_visits_without_effect,"battle_pack_reopen_cooldown":self.battle_pack_reopen_cooldown,"stagnant_actions":self.stagnant_actions,"unproductive_actions":self.unproductive_actions,
            "explored_tiles":self.explored_tiles.len(),"recent_unique_tiles":self.recent_tiles.iter().collect::<BTreeSet<_>>().len()})
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn starter_picture_and_choice_are_not_idle_or_unneeded_menus() {
        let field = serde_json::json!({"status":{"screen":"overworld","party":[]},"observe":{"menus":[]},"map_info":{"name":"ElmsLab","player":{"x":8,"y":4,"facing":"Up"}}});
        let mut picture=field.clone(); picture["observe"]["menus"]=serde_json::json!([{"kind":"pokemon_picture","species":"CHIKORITA"}]);
        let mut choice=field.clone(); choice["observe"]["menus"]=serde_json::json!([{"kind":"yes_no","selected":0}]);
        for (before,after) in [(&field,&picture),(&picture,&picture),(&picture,&choice),(&choice,&choice)] {
            let (_,aversions)=super::StoryLedger::default().action_feedback(before,after,"a");
            assert!(!aversions.iter().any(|a| matches!(a.as_str(),"action:idle_input"|"inaction:unneeded_menu")), "{aversions:?}");
        }
        let mut utility=field.clone();utility["observe"]["menus"]=serde_json::json!([{"kind":"pokedex"}]);
        assert!(super::StoryLedger::default().action_feedback(&utility,&utility,"a").1.contains(&"inaction:unneeded_menu".into()));
    }
    #[test]
    fn battle_setup_pays_for_effects_not_repeating_moves_or_cures() {
        let mut ledger=super::StoryLedger::default();
        let before=serde_json::json!({"reward_state":{"battle":{"enemy_hp":20,"active_enemy":0,"enemy_status":null}}});
        let mut after=before.clone();after["reward_state"]["battle"]["enemy_status"]=serde_json::json!("Poison");
        assert!(ledger.feedback(&before,&after).0.iter().any(|s|s.starts_with("battle:setup:status:")));
        assert!(!ledger.feedback(&after,&after).0.iter().any(|s|s.starts_with("battle:setup:")));
        ledger.feedback(&after,&before);
        assert!(!ledger.feedback(&before,&after).0.iter().any(|s|s.starts_with("battle:setup:")));
        let mut loaded=super::StoryLedger::default();
        assert!(!loaded.feedback(&after,&after).0.iter().any(|s|s.starts_with("battle:setup:")));
    }
    #[test]
    fn item_discovery_does_not_reward_buy_sell_cycles() {
        let mut ledger=super::StoryLedger::default();
        let before=serde_json::json!({"reward_state":{"items":[]}});
        let after=serde_json::json!({"reward_state":{"items":[{"id":"POTION","quantity":1}]}});
        assert!(ledger.feedback(&before,&after).0.contains(&"item_discovered:POTION".into()));
        ledger.feedback(&after,&before);
        assert!(!ledger.feedback(&before,&after).0.contains(&"item_discovered:POTION".into()));
        assert!(super::StoryLedger::default().feedback(&after,&after).0.is_empty());
    }

    #[test]
    fn repeated_start_opening_costs_without_paying_for_closure() {
        let mut ledger=super::StoryLedger::default();
        let closed=serde_json::json!({"status":{"screen":"overworld","party":[{"hp":20}]},"map_info":{"name":"Route30","player":{"x":7,"y":50}},"observe":{"menus":[],"visible_dialogue":null}});
        let mut open=closed.clone();open["observe"]["menus"]=serde_json::json!([{"kind":"start","entries":[">PACK"]}]);
        assert!(!ledger.action_feedback(&closed,&open,"start").1.contains(&"action:repeated_menu_open".into()));
        assert_eq!(ledger.menu_reopen_cooldown(),32);
        let restored:super::StoryLedger=serde_json::from_str(&serde_json::to_string(&ledger).unwrap()).unwrap();
        assert_eq!(restored.menu_reopen_cooldown(),32);
        let mut browsing=open.clone();browsing["observe"]["menus"]=serde_json::json!([{"kind":"pokedex","entries":[">152 -----"]}]);
        for _ in 0..64 {ledger.action_feedback(&browsing,&browsing,"down");}
        assert_eq!(ledger.menu_reopen_cooldown(),32);
        assert!(ledger.action_feedback(&open,&closed,"b").0.is_empty());
        assert_eq!(ledger.menu_reopen_cooldown(),32);
        assert!(ledger.action_feedback(&closed,&open,"start").1.contains(&"action:repeated_menu_open".into()));
        for _ in 0..32 { ledger.action_feedback(&closed,&closed,"b"); }
        assert_eq!(ledger.menu_reopen_cooldown(),0);
    }

    #[test]
    fn restoration_is_useful_but_damage_heal_cycles_do_not_pay_again() {
        let mut ledger=super::StoryLedger::default();
        let before=serde_json::json!({"status":{"screen":"overworld","party":[{"slot":0,"nickname":"CINDER","hp":4,"max_hp":20,"status":null,"moves":[]}]},"map_info":{"name":"Route30"}});
        let mut healed=before.clone();healed["status"]["party"][0]["hp"]=serde_json::json!(20);
        assert!(ledger.feedback(&before,&healed).0.contains(&"resource:restored_party".into()));
        ledger.feedback(&healed,&before);
        assert!(!ledger.feedback(&before,&healed).0.contains(&"resource:restored_party".into()));
        let mut restored:super::StoryLedger=serde_json::from_str(&serde_json::to_string(&ledger).unwrap()).unwrap();
        assert!(!restored.feedback(&before,&healed).0.contains(&"resource:restored_party".into()));
        let mut swapped=healed.clone();swapped["status"]["party"][0]["nickname"]=serde_json::json!("OTHER");
        assert!(!super::StoryLedger::default().feedback(&before,&swapped).0.contains(&"resource:restored_party".into()));
        let mut battle=before.clone();battle["reward_state"]=serde_json::json!({"battle":{"enemy_hp":20}});
        healed["reward_state"]=serde_json::json!({"battle":null,"battle_result":1});
        assert!(super::StoryLedger::default().feedback(&battle,&healed).0.is_empty());
    }

    use super::*;
    use serde_json::json;
    fn observation() -> Value {
        json!({"status":{"screen":"overworld","party":[],"badges":{"johto":vec![false;8],"kanto":vec![false;8]}},
            "map_info":{"name":"ElmsLab"},"observe":{"visible_dialogue":""},
            "reward_state":{"version":1,"event_flags":[],"key_items":[],"machines":[],"caught_species":[],"battle":null}})
    }
    #[test]
    fn battle_menu_idle_feedback_requires_a_completed_ineffective_input() {
        let before = json!({"frame":100,"status":{"screen":"battle","party":[{"hp":19,"moves":[{"current_pp":35}]}]},
            "flow_state":{"animating":true},"observe":{"battle_message":null,"menus":[{"kind":"battle","entries":[">FIGHT"," PACK"]}]},
            "reward_state":{"battle":{"player_turns":0,"enemy_turns":0,"enemy_hp":21}}});
        let mut after = before.clone();
        assert!(!unchanged_battle_menu(&before,&after), "no game time elapsed");
        after["frame"]=json!(116);
        assert!(unchanged_battle_menu(&before,&after), "suspended field script must not excuse idle menu input");
        let mut ledger=StoryLedger::default();
        assert!(ledger.action_feedback(&before,&after,"start").1.contains(&"action:unchanged_battle_menu".into()));
        after["observe"]["menus"][0]["entries"]=json!([" FIGHT",">PACK"]);
        assert!(!unchanged_battle_menu(&before,&after), "cursor movement is useful feedback");
        after["observe"]["menus"]=json!([]);
        assert!(!unchanged_battle_menu(&before,&after), "command animation may be running");
        after=before.clone();after["frame"]=json!(116);after["reward_state"]["battle"]["player_turns"]=json!(1);
        assert!(!unchanged_battle_menu(&before,&after), "a status move counts as a real turn");
        after=before.clone();after["frame"]=json!(116);after["observe"]["battle_message"]=json!("TACKLE missed!");
        assert!(!unchanged_battle_menu(&before,&after), "text progression is not an idle menu");
    }
    #[test]
    fn invalid_battle_requests_penalize_initiator_not_message_dismissal() {
        let mut ledger=StoryLedger::default();
        let before=json!({"status":{"screen":"battle","party":[{"hp":19}]},"observe":{"battle":"Trainer ABE","menus":[{"kind":"battle","entries":[">SWITCH","STATS","CANCEL"]}]},"reward_state":{"battle":{"player_turns":0,"enemy_hp":27}}});
        let mut rejected=before.clone();rejected["observe"]["menus"]=json!([]);rejected["observe"]["battle_message"]=json!("CYNDAQUIL\nis already out.");
        assert!(ledger.action_feedback(&before,&rejected,"a").1.contains(&"action:already_active_switch".into()));
        assert!(!ledger.action_feedback(&rejected,&before,"a").1.contains(&"action:already_active_switch".into()));
        let mut run=before.clone();run["observe"]["menus"][0]["entries"]=json!(["FIGHT","<PKMN>","PACK",">RUN"]);
        rejected["observe"]["battle_message"]=json!("No! There's no\nrunning from a\ntrainer battle!");
        assert!(ledger.action_feedback(&run,&rejected,"a").1.contains(&"action:trainer_escape_rejected".into()));
        run["observe"]["battle"]=json!("Wild NORMAL");assert!(!ledger.action_feedback(&run,&rejected,"a").1.contains(&"action:trainer_escape_rejected".into()));
        rejected["observe"]["battle_message"]=json!("CYNDAQUIL\nis already out.");
        rejected["reward_state"]["battle"]["player_turns"]=json!(1);assert!(!ledger.action_feedback(&before,&rejected,"a").1.contains(&"action:already_active_switch".into()));
    }
    #[test]
    fn third_unproductive_battle_pack_visit_penalizes_only_the_opener() {
        let mut ledger=StoryLedger::default();
        let main=json!({"status":{"screen":"battle","party":[{"hp":20}]},"observe":{"menus":[{"kind":"battle","entries":["FIGHT","<PKMN>",">PACK","RUN"]}]},"reward_state":{"items":[{"id":"POKE_BALL","quantity":5}],"battle":{"player_turns":0,"enemy_turns":0,"enemy_hp":10}}});
        let mut pack=main.clone();pack["observe"]["menus"][0]["entries"]=json!([">POTION x1","CANCEL"]);
        for visit in 1..=4 {
            let (_,bad)=ledger.action_feedback(&main,&pack,"a");
            assert_eq!(bad.contains(&"action:battle_pack_loop".into()),visit>=3);
            assert_eq!(ledger.battle_pack_reopen_cooldown()>0,visit>=3);
            assert!(!ledger.action_feedback(&pack,&main,"b").1.contains(&"action:battle_pack_loop".into()));
        }
        let mut resumed:StoryLedger=serde_json::from_str(&serde_json::to_string(&ledger).unwrap()).unwrap();
        assert_eq!(resumed.battle_pack_reopen_cooldown(),ledger.battle_pack_reopen_cooldown());
        for _ in 0..24 {resumed.action_feedback(&main,&main,"up");}
        assert_eq!(resumed.battle_pack_reopen_cooldown(),0);
        let mut used=main.clone();used["reward_state"]["battle"]["player_turns"]=json!(1);
        used["reward_state"]["items"][0]["quantity"]=json!(4);
        ledger.action_feedback(&pack,&used,"a");
        assert_eq!(ledger.battle_pack_reopen_cooldown(),0);
        let mut next_pack=used.clone();next_pack["observe"]["menus"]=pack["observe"]["menus"].clone();
        assert!(!ledger.action_feedback(&used,&next_pack,"a").1.contains(&"action:battle_pack_loop".into()));
        assert_eq!(ledger.battle_pack_visits_without_effect,1);
    }
    #[test]
    fn conversation_completion_pays_once_and_does_not_farm_reopening() {
        let mut ledger=StoryLedger::default();
        let mut before=observation();
        before["observe"]["visible_dialogue"]=json!("Have a good trip!");
        before["reward_state"]["last_talked_object"]=json!("MOM");
        let mut after=before.clone();after["observe"]["visible_dialogue"]=Value::Null;
        assert!(ledger.action_feedback(&before,&after,"a").0.contains(&"dialogue:complete:MOM".into()));
        assert!(!ledger.action_feedback(&before,&after,"a").0.contains(&"dialogue:complete:MOM".into()));
    }
    #[test]
    fn npc_choice_is_not_completion_and_changing_numbers_do_not_farm_pages() {
        let mut ledger=StoryLedger::default();let mut before=observation();
        before["observe"]["visible_dialogue"]=json!("You have 12 points.");
        before["reward_state"]["last_talked_object"]=json!("TOWNSPERSON");
        let mut choice=before.clone();choice["observe"]["visible_dialogue"]=Value::Null;
        choice["observe"]["menus"]=json!([{"kind":"name_choices","options":["YES","NO"]}]);
        assert!(!ledger.action_feedback(&before,&choice,"a").0.iter().any(|s|s.starts_with("dialogue:complete:")));
        let mut closed=choice.clone();closed["observe"]["menus"]=json!([]);
        assert!(ledger.action_feedback(&before,&closed,"a").0.contains(&"dialogue:complete:TOWNSPERSON".into()));
        let mut fresh=StoryLedger::default();let mut page=before.clone();
        assert!(fresh.feedback(&closed,&page).0.contains(&"dialogue:TOWNSPERSON".into()));
        page["observe"]["visible_dialogue"]=json!("You have 13 points.");
        assert!(!fresh.feedback(&closed,&page).0.iter().any(|s|s.starts_with("dialogue:")));
    }
    #[test]
    fn stale_warning_does_not_poison_dialogue_or_dismissal() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        before["recent_events"] = json!({"last_action":"Can't use that here."});
        before["observe"]["visible_dialogue"] = json!("Can't use that here.");
        let mut after = before.clone();
        after["observe"]["visible_dialogue"] = json!("");
        assert_eq!(
            ledger.action_feedback(&before, &after, "b"),
            (vec![], vec![])
        );
        before = after.clone();
        after["observe"]["visible_dialogue"] = json!("Please deliver this egg.");
        after["reward_state"]["last_talked_object"] = json!("Elm");
        assert_eq!(
            ledger.action_feedback(&before, &after, "a"),
            (vec!["dialogue:Elm".into()], vec![])
        );
    }
    #[test]
    fn long_return_trip_stays_neutral_after_timeout() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        before["map_info"]["player"] = json!({"x":0,"y":0});
        for x in (1..=600).chain((0..600).rev()) {
            let mut after = before.clone();
            after["map_info"]["player"]["x"] = json!(x);
            assert_eq!(
                ledger.action_feedback(&before, &after, "left"),
                (vec![], vec![])
            );
            before = after;
        }
        assert!(ledger.unproductive_actions >= 256);
        assert!(
            ledger
                .action_feedback(&before, &before, "b")
                .1
                .contains(&"inaction:no_progress".into())
        );
    }
    #[test]
    fn nondamaging_battle_turns_buy_time_without_rewards() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        before["reward_state"]["battle"] = json!({"enemy_hp":20,"player_turns":0});
        for turn in 1..=300 {
            for _ in 0..4 {
                assert!(ledger.action_feedback(&before, &before, "a").1.is_empty());
            }
            let mut after = before.clone();
            after["reward_state"]["battle"]["player_turns"] = json!(turn);
            assert_eq!(
                ledger.action_feedback(&before, &after, "a"),
                (vec![], vec![])
            );
            before = after;
        }
    }
    #[test]
    fn select_spam_and_oak_warning_never_pay_even_with_stale_npc() {
        let mut ledger = StoryLedger::default();
        let before = observation();
        let mut after = before.clone();
        after["reward_state"]["last_talked_object"] = json!("Elm");
        after["observe"]["visible_dialogue"] = json!("OAK: This isn't the time to use that!");
        let (reward, aversion) = ledger.action_feedback(&before, &after, "select");
        assert!(reward.is_empty());
        assert!(aversion.contains(&"action:refused".into()));
        for _ in 0..20 {
            let (reward, aversion) = ledger.action_feedback(&after, &after, "select");
            assert!(reward.is_empty());
            assert!(aversion.contains(&"action:select".into()));
        }
        let mut dismissed = after.clone();
        dismissed["observe"]["visible_dialogue"] = json!("");
        assert!(
            !ledger
                .action_feedback(&after, &dismissed, "a")
                .1
                .contains(&"action:refused".into())
        );
    }
    #[test]
    fn exploration_and_backtracking_have_no_distance_cost_but_loops_expire() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        before["map_info"]["player"] = json!({"x":0,"y":0});
        for x in 1..=300 {
            let mut after = before.clone();
            after["map_info"]["player"]["x"] = json!(x);
            assert_eq!(
                ledger.action_feedback(&before, &after, "right"),
                (vec![], vec![])
            );
            before = after;
        }
        for x in (100..300).rev() {
            let mut after = before.clone();
            after["map_info"]["player"]["x"] = json!(x);
            assert!(ledger.action_feedback(&before, &after, "left").1.is_empty());
            before = after;
        }
        for n in 0..80 {
            let mut after = before.clone();
            after["map_info"]["player"]["x"] = json!(101 + n % 2);
            let penalties = ledger.action_feedback(&before, &after, "right").1;
            assert_eq!(
                penalties.contains(&"inaction:no_progress".into()),
                n >= 55 && ledger.recent_tiles.iter().collect::<BTreeSet<_>>().len() < 16
            );
            before = after;
        }
        let encoded = serde_json::to_string(&ledger).unwrap();
        let mut restored: StoryLedger = serde_json::from_str(&encoded).unwrap();
        assert!(
            restored
                .action_feedback(&before, &before, "b")
                .1
                .contains(&"inaction:no_progress".into())
        );
        let mut progress = before.clone();
        progress["reward_state"]["event_flags"] = json!(["EVENT_GOT_MYSTERY_EGG"]);
        assert!(
            restored
                .action_feedback(&before, &progress, "a")
                .1
                .is_empty()
        );
        assert_eq!(
            restored.action_feedback(&progress, &progress, "a").1,
            vec!["action:idle_input"]
        );
    }
    #[test]
    fn idle_cost_has_grace_and_ignores_animation_frames() {
        let mut ledger = StoryLedger::default();
        let mut state = observation();
        state["flow_state"] = json!({"animating":true});
        for _ in 0..300 {
            assert!(ledger.action_feedback(&state, &state, "a").1.is_empty());
        }
        state["flow_state"]["animating"] = json!(false);
        for n in 1..=13 {
            let (reward, penalties) = ledger.action_feedback(&state, &state, "left");
            assert!(reward.is_empty());
            assert_eq!(penalties.contains(&"inaction:stalled".into()), n >= 12);
        }
    }
    #[test]
    fn switch_screen_churn_costs_after_grace_and_resets_on_exit_or_turn() {
        let mut ledger: StoryLedger = serde_json::from_str("{}").unwrap();
        let mut state = observation();
        state["status"]["screen"] = json!("battle");
        state["observe"]["pokemon_switch_open"] = json!(true);
        state["reward_state"]["battle"] = json!({"player_turns":0});
        for n in 1..=14 {
            let mut next = state.clone();
            next["observe"]["menus"] = json!([{"kind":"battle", "selected":n % 2}]);
            next["observe"]["text"] = json!(format!("Switch choice {}", n % 2));
            let penalties = ledger.action_feedback(&state, &next, "down").1;
            assert_eq!(penalties.contains(&"inaction:pokemon_switch".into()), n >= 12);
            state = next;
        }
        let mut restored: StoryLedger =
            serde_json::from_str(&serde_json::to_string(&ledger).unwrap()).unwrap();
        assert!(restored.action_feedback(&state, &state, "a").1
            .contains(&"inaction:pokemon_switch".into()));
        state["flow_state"] = json!({"animating":true});
        let count = restored.switch_screen_actions;
        assert!(!restored.action_feedback(&state, &state, "a").1
            .contains(&"inaction:pokemon_switch".into()));
        assert_eq!(restored.switch_screen_actions, count);
        state["flow_state"]["animating"] = json!(false);
        let mut next = state.clone();
        next["reward_state"]["battle"]["player_turns"] = json!(1);
        assert!(!restored.action_feedback(&state, &next, "a").1
            .contains(&"inaction:pokemon_switch".into()));
        assert_eq!(restored.switch_screen_actions, 0);
        restored.action_feedback(&next, &next, "down");
        let mut closed = next.clone();
        closed["observe"]["pokemon_switch_open"] = json!(false);
        restored.action_feedback(&next, &closed, "b");
        assert_eq!(restored.switch_screen_actions, 0);
        restored.action_feedback(&closed, &next, "a");
        assert_eq!(restored.switch_screen_actions, 0);
    }
    #[test]
    fn healed_blackout_does_not_reward_respawn_location() {
        let mut before = observation();
        before["reward_state"]["battle"] = json!({"enemy_hp":10});
        let mut after = observation();
        after["map_info"]["name"] = json!("NewPokecenter");
        after["status"]["party"] = json!([{"hp":20,"level":5}]);
        after["reward_state"]["battle_result"] = json!(1);
        let (rewards, penalties) = StoryLedger::default().action_feedback(&before, &after, "a");
        assert!(rewards.is_empty());
        assert_eq!(penalties, vec!["battle:defeat"]);
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
    fn new_places_pay_once_including_interiors_and_survive_restore() {
        let mut ledger = StoryLedger::default();
        let mut before = observation();
        for map in [
            "NewBarkTown",
            "Route29",
            "CherrygroveCity",
            "CherrygrovePokecenter1F",
            "PewterCity",
            "MtSilverRoom3",
        ] {
            let mut after = before.clone();
            after["map_info"]["name"] = json!(map);
            assert_eq!(
                ledger.feedback(&before, &after).0,
                vec![format!("place:{map}")]
            );
            let serialized = serde_json::to_string(&ledger).unwrap();
            ledger = serde_json::from_str(&serialized).unwrap();
            assert!(ledger.feedback(&before, &after).0.is_empty());
            assert!(StoryLedger::default().feedback(&after, &after).0.is_empty());
            before = after;
        }
        let mut revisit = before.clone();
        revisit["map_info"]["name"] = json!("ElmsLab");
        assert!(ledger.feedback(&before, &revisit).0.is_empty());
    }
    #[test]
    fn walking_menu_toggling_and_repeat_dialogue_do_not_pay() {
        let mut ledger = StoryLedger::default();
        let before = observation();
        let mut after = before.clone();
        after["map_info"]["player"] = json!({"x":4,"y":5});
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
    #[test]
    fn repeat_npc_and_profile_guard_are_aversive() {
        let mut ledger = StoryLedger::default();
        let empty = observation();
        let mut talking = empty.clone();
        talking["reward_state"]["last_talked_object"] = json!("ElmsLab_Elm");
        talking["observe"]["visible_dialogue"] = json!("Take a Pokemon.");
        assert!(ledger.action_feedback(&empty, &talking, "a").1.is_empty());
        let dismissed = {
            let mut value = talking.clone();
            value["observe"]["visible_dialogue"] = json!("");
            value
        };
        let penalties = ledger.action_feedback(&dismissed, &talking, "a").1;
        assert!(
            penalties
                .iter()
                .any(|event| event == "action:repeat_npc:ElmsLab_Elm")
        );

        let mut guarded = empty.clone();
        guarded["flygon_guard"] = json!("profile_blocked");
        assert!(
            ledger
                .action_feedback(&empty, &guarded, "a")
                .1
                .contains(&"action:profile_blocked".into())
        );
    }
}
