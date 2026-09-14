//! Engineered story curriculum. Supplies sensory cues and rewards, never inputs.
//! Routes use observed terrain and failed movement edges, not straight-line distance.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Breadcrumbs {
    terrain: BTreeMap<String, BTreeMap<String, bool>>,
    blocked: BTreeSet<String>,
    best: BTreeMap<String, usize>,
    explored: BTreeSet<String>,
    pub current: Value,
}
fn pos(v: &Value) -> Option<(i64, i64)> {
    Some((
        v.pointer("/map_info/player/x")?.as_i64()?,
        v.pointer("/map_info/player/y")?.as_i64()?,
    ))
}
fn map(v: &Value) -> &str {
    v.pointer("/map_info/name")
        .and_then(Value::as_str)
        .unwrap_or("")
}
fn has(v: &Value, path: &str, flag: &str) -> bool {
    v.pointer(path)
        .and_then(Value::as_array)
        .is_some_and(|a| a.iter().any(|f| f.as_str() == Some(flag)))
}
impl Breadcrumbs {
    pub fn observe(&mut self, v: &Value) {
        let Some(ox) = v
            .pointer("/map_info/terrain/origin_x")
            .and_then(Value::as_i64)
        else {
            return;
        };
        let Some(oy) = v
            .pointer("/map_info/terrain/origin_y")
            .and_then(Value::as_i64)
        else {
            return;
        };
        let cells = self.terrain.entry(map(v).into()).or_default();
        for (y, row) in v
            .pointer("/map_info/terrain/rows")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            for (x, tile) in row.as_array().into_iter().flatten().enumerate() {
                if let Some(t) = tile["terrain"].as_str() {
                    cells.insert(format!("{},{}", ox + x as i64, oy + y as i64), t == "Land");
                }
            }
        }
    }
    fn target(v: &Value) -> Option<(String, Vec<(i64, i64)>)> {
        let party = v
            .pointer("/status/party")
            .and_then(Value::as_array)
            .is_some_and(|p| !p.is_empty());
        let egg = has(v, "/reward_state/key_items", "MYSTERY_EGG");
        let delivered = has(
            v,
            "/reward_state/event_flags",
            "EVENT_GAVE_MYSTERY_EGG_TO_ELM",
        );
        let scene = v
            .pointer("/reward_state/scenes/ElmsLab")
            .and_then(Value::as_str)
            .unwrap_or("");
        let object = |needle: &str| -> Vec<(i64, i64)> {
            v.pointer("/map_info/objects")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|o| o["name"].as_str().is_some_and(|n| n.contains(needle)))
                .filter_map(|o| Some((o["x"].as_i64()?, o["y"].as_i64()?)))
                .flat_map(|(x, y)| [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)])
                .collect()
        };
        let (label, goals) = match map(v) {
            "PlayersHouse2F" => ("Go downstairs to Mom", vec![(7, 0)]),
            "PlayersHouse1F" if !has(v, "/reward_state/engine_flags", "ENGINE_POKEGEAR") => {
                ("Speak to Mom", object("MOM"))
            }
            "PlayersHouse1F" => ("Leave home for Elm's lab", vec![(6, 7), (7, 7)]),
            "NewBarkTown" if !party || egg => ("Enter Elm's lab", vec![(6, 3)]),
            "NewBarkTown" => ("Leave town west for Route 29", vec![(0, 8), (0, 9)]),
            "ElmsLab" if !party && scene.contains("CANT_LEAVE") => {
                ("Choose a starter Pokemon", object("POKE_BALL"))
            }
            "ElmsLab" if !party || egg || scene.contains("MEET_OFFICER") => (
                "Speak to Professor Elm",
                object("ELMSLAB_ELM")
                    .into_iter()
                    .filter(|p| p.1 < 8)
                    .collect(),
            ),
            "ElmsLab" => ("Leave the lab and travel west", vec![(4, 11), (5, 11)]),
            "Route29" if egg => ("Return east to Elm", vec![(59, 8), (59, 9)]),
            "Route29" => ("Travel west to Cherrygrove", vec![(0, 6), (0, 7)]),
            "CherrygroveCity" if egg => ("Return east to New Bark Town", vec![(39, 6), (39, 7)]),
            "CherrygroveCity" => ("Go north to Route 30", vec![(17, 0), (18, 0)]),
            "Route30" if !egg && !delivered => ("Visit Mr. Pokemon", vec![(17, 5)]),
            "Route30" if egg => ("Return south with the Mystery Egg", vec![(7, 53), (8, 53)]),
            "MrPokemonsHouse" if !egg && !delivered => {
                ("Speak to Mr. Pokemon", object("MR_POKEMON"))
            }
            "MrPokemonsHouse" => ("Return the Mystery Egg to Elm", vec![(2, 7), (3, 7)]),
            _ => return None,
        };
        if goals.is_empty() {
            None
        } else {
            Some((label.into(), goals))
        }
    }
    fn distance(&self, v: &Value, goals: &[(i64, i64)]) -> Option<usize> {
        let start = pos(v)?;
        let dims = v.pointer("/map_info/dimensions")?.as_array()?;
        let (width, height) = (dims.first()?.as_i64()?, dims.get(1)?.as_i64()?);
        if width * height > 20000
            || width <= 0
            || height <= 0
            || start.0 < 0
            || start.1 < 0
            || start.0 >= width
            || start.1 >= height
        {
            return None;
        }
        let cells = self.terrain.get(map(v));
        if cells.and_then(|c| c.get(&format!("{},{}", start.0, start.1))) == Some(&false) {
            return None;
        }
        let occupied: BTreeSet<_> = v
            .pointer("/map_info/objects")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|o| Some((o["x"].as_i64()?, o["y"].as_i64()?)))
            .collect();
        let mut queue = BinaryHeap::from([Reverse((0usize, start))]);
        let mut costs = BTreeMap::from([(start, 0usize)]);
        while let Some(Reverse((cost, (x, y)))) = queue.pop() {
            if costs.get(&(x, y)) != Some(&cost) {
                continue;
            }
            if goals.contains(&(x, y)) {
                return Some(cost);
            }
            for next in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                let (nx, ny) = next;
                if nx < 0 || ny < 0 || nx >= width || ny >= height || occupied.contains(&next) {
                    continue;
                }
                if self
                    .blocked
                    .contains(&format!("{}:{x},{y}:{nx},{ny}", map(v)))
                {
                    continue;
                }
                let known = cells.and_then(|c| c.get(&format!("{nx},{ny}")));
                // Never route through unseen cells: missing connectivity means
                // explore a known frontier, not guess a shortcut through a maze.
                if known != Some(&true) {
                    continue;
                }
                let score = cost + 1;
                if costs.get(&next).is_none_or(|old| score < *old) {
                    costs.insert(next, score);
                    queue.push(Reverse((score, next)));
                }
            }
        }
        None
    }
    fn frontiers(&self, v: &Value) -> Vec<(i64, i64)> {
        let Some(cells) = self.terrain.get(map(v)) else {
            return vec![];
        };
        let width = v
            .pointer("/map_info/dimensions/0")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let height = v
            .pointer("/map_info/dimensions/1")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        cells
            .iter()
            .filter(|(_, land)| **land)
            .filter_map(|(key, _)| {
                let (x, y) = key.split_once(',')?;
                let (x, y) = (x.parse::<i64>().ok()?, y.parse::<i64>().ok()?);
                [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                    .iter()
                    .any(|(nx, ny)| {
                        *nx >= 0
                            && *ny >= 0
                            && *nx < width
                            && *ny < height
                            && !cells.contains_key(&format!("{nx},{ny}"))
                    })
                    .then_some((x, y))
            })
            .collect()
    }
    pub fn cues(&mut self, v: &Value) -> Vec<String> {
        self.observe(v);
        let frontiers = self.frontiers(v);
        let curiosity_distance = self.distance(v, &frontiers);
        let (label, goals) = Self::target(v)
            .unwrap_or_else(|| ("Explore new terrain and places".into(), frontiers.clone()));
        let distance = self.distance(v, &goals);
        self.current = json!({"target":label,"remaining_route_cost":distance,"frontiers":frontiers.len(),"routing":"known walkable connectivity only; explore frontiers when no route is known"});
        let mut cues = vec![format!("story-target:{label}")];
        if !v
            .pointer("/observe/visible_dialogue")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
            || v.pointer("/observe/menus")
                .and_then(Value::as_array)
                .is_some_and(|m| !m.is_empty())
        {
            return cues;
        }
        if let Some((x, y)) = pos(v) {
            // A directional scent is an explicit task aid, still processed by
            // the sensory circuit; it never overrides the sampled button.
            for (name, dx, dy) in [
                ("west", -1, 0),
                ("east", 1, 0),
                ("north", 0, -1),
                ("south", 0, 1),
            ] {
                if self
                    .blocked
                    .contains(&format!("{}:{x},{y}:{},{}", map(v), x + dx, y + dy))
                {
                    continue;
                }
                let mut neighbour = v.clone();
                neighbour["map_info"]["player"]["x"] = json!(x + dx);
                neighbour["map_info"]["player"]["y"] = json!(y + dy);
                if let (Some(here), Some(there)) = (distance, self.distance(&neighbour, &goals)) {
                    if there < here {
                        cues.push(format!("story-scent:{name}"));
                    }
                }
                if let (Some(here), Some(there)) =
                    (curiosity_distance, self.distance(&neighbour, &frontiers))
                {
                    if there < here {
                        cues.push(format!("exploration-scent:{name}"));
                    }
                }
            }
        }
        cues
    }
    pub fn feedback(&mut self, before: &Value, after: &Value, button: &str) -> Option<String> {
        self.observe(before);
        let known_before = self.terrain.get(map(after)).map_or(0, BTreeMap::len);
        self.observe(after);
        if map(before) != map(after) {
            return None;
        }
        let (b, a) = (pos(before)?, pos(after)?);
        let overworld =
            after.pointer("/status/screen").and_then(Value::as_str) == Some("overworld");
        let dialogue = after
            .pointer("/observe/visible_dialogue")
            .and_then(Value::as_str)
            .unwrap_or("");
        let menu_open = [before, after].iter().any(|v| {
            v.pointer("/observe/menus")
                .and_then(Value::as_array)
                .is_some_and(|m| !m.is_empty())
        });
        if !overworld || !dialogue.is_empty() || menu_open {
            return None;
        }
        self.explored
            .insert(format!("{}:{},{}", map(before), b.0, b.1));
        let novel = self
            .explored
            .insert(format!("{}:{},{}", map(after), a.0, a.1));
        let revealed = self
            .terrain
            .get(map(after))
            .map_or(0, BTreeMap::len)
            .saturating_sub(known_before);
        let exploration = (b != a && novel).then(|| if revealed >= 3 {
            format!("exploration:revealed_{revealed}_tiles")
        } else { "exploration:new_ground".into() });
        if b != a {
            self.blocked
                .remove(&format!("{}:{},{}:{},{}", map(before), b.0, b.1, a.0, a.1));
        }
        if b == a
            && before.pointer("/map_info/player/facing") == after.pointer("/map_info/player/facing")
            && before
                .pointer("/flow_state/animating")
                .and_then(Value::as_bool)
                != Some(true)
            && after
                .pointer("/flow_state/animating")
                .and_then(Value::as_bool)
                != Some(true)
        {
            let delta = match button {
                "up" => (0, -1),
                "down" => (0, 1),
                "left" => (-1, 0),
                "right" => (1, 0),
                _ => (0, 0),
            };
            let occupied = before
                .pointer("/map_info/objects")
                .and_then(Value::as_array)
                .is_some_and(|objects| {
                    objects.iter().any(|o| {
                        o["x"].as_i64() == Some(b.0 + delta.0)
                            && o["y"].as_i64() == Some(b.1 + delta.1)
                    })
                });
            if delta != (0, 0) && !occupied {
                self.blocked.insert(format!(
                    "{}:{},{}:{},{}",
                    map(before),
                    b.0,
                    b.1,
                    b.0 + delta.0,
                    b.1 + delta.1
                ));
            }
        }
        let Some((label, goals)) = Self::target(before) else {
            return exploration;
        };
        if Self::target(after).map(|t| t.0) != Some(label.clone()) {
            return exploration;
        }
        let (Some(old), Some(new)) = (self.distance(before, &goals), self.distance(after, &goals))
        else {
            return exploration;
        };
        let best = self
            .best
            .entry(format!("{}:{label}", map(before)))
            .or_insert(old);
        if b != a && new < old && new < *best {
            *best = new;
            Some(format!("breadcrumb:{label}"))
        } else if b != a && new < old {
            Some(format!("breadcrumb:recover:{label}"))
        } else {
            exploration
        }
    }
}
