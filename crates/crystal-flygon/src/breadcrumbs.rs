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
    // Keep legacy land memory readable; water is learned from fresh telemetry.
    water: BTreeMap<String, BTreeSet<String>>,
    blocked: BTreeSet<String>,
    blocked_at: BTreeMap<String, u64>,
    observations: u64,
    best: BTreeMap<String, usize>,
    explored: BTreeSet<String>,
    oriented: BTreeSet<String>,
    pub current: Value,
    rewarded_edges: BTreeSet<String>,
    /// Signed distance feedback for the last completed movement, not a novelty bonus.
    #[serde(skip)]
    pub progress: f32,
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
        let water = self.water.entry(map(v).into()).or_default();
        for (y, row) in v
            .pointer("/map_info/terrain/rows")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            for (x, tile) in row.as_array().into_iter().flatten().enumerate() {
                if let Some(t) = tile["terrain"].as_str() {
                    let key = format!("{},{}", ox + x as i64, oy + y as i64);
                    // Ordinary WATER only. Currents, waterfalls and whirlpools
                    // require their own directional or field-move semantics.
                    if t == "Water" && tile["permission"].as_u64() == Some(0x29) {
                        water.insert(key.clone());
                    } else { water.remove(&key); }
                    cells.insert(key, t == "Land");
                }
            }
        }
    }
    fn target(v: &Value) -> Option<(String, Vec<(i64, i64)>)> {
        if let Some(target)=crate::curriculum::target(v) {return Some(target);}
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
            // Route 30 connects north to Route 31. Keep all boundary candidates;
            // observed terrain and failed edges determine the traversable exit.
            "Route30" if delivered => ("Travel north to Route 31", (0..v.pointer("/map_info/dimensions/0")?.as_i64()?).map(|x| (x, 0)).collect()),
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
    fn interaction(v: &Value) -> Option<(String, &'static str, bool)> {
        if crate::field_objectives::teaching_target(v).is_some(){return None;}
        let (label, _) = Self::target(v)?;
        if matches!(crate::curriculum::goal(v),Some(crate::curriculum::Goal::Pc|crate::curriculum::Goal::CutTree)) {
            let (x,y)=pos(v)?;
            let cut=crate::curriculum::goal(v)==Some(crate::curriculum::Goal::CutTree);
            let tiles=if cut{crate::curriculum::cut_tiles(v)}else{crate::curriculum::pc_tiles(v)};
            for (px,py) in tiles {
                let (direction,facing)=match (px-x,py-y) {(0,-1)=>("north","Up"),(1,0)=>("east","Right"),(0,1)=>("south","Down"),(-1,0)=>("west","Left"),_=>continue};
                return Some((format!("{}:{}:{px},{py}",map(v),if cut{"CUT"}else{"PC"}),direction,v.pointer("/map_info/player/facing").and_then(Value::as_str)==Some(facing)));
            }
            return None;
        }
        let needle = if let Some(crate::curriculum::Goal::Talk(needle,_))=crate::curriculum::goal(v) { needle } else { match label.as_str() {
            "Speak to Mom" => "MOM",
            "Speak to Professor Elm" => "ELMSLAB_ELM",
            "Choose a starter Pokemon" => "POKE_BALL",
            "Speak to Mr. Pokemon" => "MR_POKEMON",
            _ => return None,
        }};
        let (x, y) = pos(v)?;
        for object in v.pointer("/map_info/objects")?.as_array()? {
            let name = object["name"].as_str()?;
            if !crate::curriculum::matches_object(object,needle) || (label=="Speak to Professor Elm" && name.contains("AIDE")) {
                continue;
            }
            if needle=="FARFETCHD" && map(v)=="IlexForest"
                && crate::curriculum::farfetchd_approaches(object).is_some_and(|points|!points.contains(&(x,y))){continue;}
            let delta = (object["x"].as_i64()? - x, object["y"].as_i64()? - y);
            let (direction, facing) = match delta {
                (0, -2) if crate::curriculum::counter_at(v,x,y-1) => ("north", "Up"),
                (2, 0) if crate::curriculum::counter_at(v,x+1,y) => ("east", "Right"),
                (0, 2) if crate::curriculum::counter_at(v,x,y+1) => ("south", "Down"),
                (-2, 0) if crate::curriculum::counter_at(v,x-1,y) => ("west", "Left"),
                (0, -1) => ("north", "Up"),
                (1, 0) => ("east", "Right"),
                (0, 1) => ("south", "Down"),
                (-1, 0) => ("west", "Left"),
                _ => continue,
            };
            return Some((
                format!("{}:{label}:{name}", map(v)),
                direction,
                v.pointer("/map_info/player/facing").and_then(Value::as_str) == Some(facing),
            ));
        }
        None
    }
    fn surf_active(v: &Value) -> bool {
        matches!(v.pointer("/reward_state/movement_mode").and_then(Value::as_str), Some("Surf" | "SurfPika"))
    }
    fn is_water(&self, v: &Value, key: &str) -> bool {
        self.water.get(map(v)).is_some_and(|cells| cells.contains(key))
    }
    fn traversable(&self, v: &Value, key: &str) -> bool {
        self.terrain.get(map(v)).and_then(|cells| cells.get(key)) == Some(&true)
            || (Self::surf_active(v) && self.is_water(v, key))
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
        if !self.traversable(v, &format!("{},{}", start.0, start.1)) {
            return None;
        }
        let occupied: BTreeSet<_> = v
            .pointer("/map_info/objects")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|o| Some((o["x"].as_i64()?, o["y"].as_i64()?)))
            .collect();
        // cues() also probes hypothetical neighboring player positions. Do not
        // seed Dijkstra inside an NPC tile and infer a shortcut through them.
        if occupied.contains(&start) { return None; }
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
                let next_key = format!("{nx},{ny}");
                // Landing ends Surf. Do not plan a later water re-entry as an
                // ordinary step; activating Surf is a separate interaction.
                if self.is_water(v, &next_key) && !self.is_water(v, &format!("{x},{y}")) { continue; }
                // Never route through unseen cells: missing connectivity means
                // explore a known frontier, not guess a shortcut through a maze.
                if !self.traversable(v, &next_key) {
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
        let frontier: Vec<_> = cells
            .iter()
            .filter(|(key, _)| self.traversable(v, key))
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
            .collect();
        if !frontier.is_empty() {
            return frontier;
        }
        // A small room may be fully visible before any of it has been visited.
        // Continue seeking unvisited ground instead of losing all curiosity.
        cells
            .iter()
            .filter(|(key, _)| self.traversable(v, key))
            .filter_map(|(key, _)| {
                if self.explored.contains(&format!("{}:{key}", map(v))) {
                    return None;
                }
                let (x, y) = key.split_once(',')?;
                Some((x.parse::<i64>().ok()?, y.parse::<i64>().ok()?))
            })
            .collect()
    }
    pub fn cues(&mut self, v: &Value) -> Vec<String> {
        self.observe(v);
        if let Some(name)=crate::field_objectives::teaching_target(v){
            let label=format!("Teach {name} to a compatible party member");
            self.current=json!({"target":label,"routing":"complete HM teaching before continuing navigation"});
            return vec![format!("story-target:{label}")];
        }
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
        // Opening errands predate the later curriculum, but their edge doors
        // also require a step beyond the target tile. Only an observed warp to
        // the declared destination supplies the outward cue.
        let opening_exit = match label.as_str() {
            "Leave home for Elm's lab" | "Leave the lab and travel west" => Some("NEW_BARK_TOWN"),
            "Return the Mystery Egg to Elm" => Some("ROUTE_30"),
            _ => None,
        };
        if let (Some(destination), Some((x,y))) = (opening_exit, pos(v)) {
            let width=v.pointer("/map_info/dimensions/0").and_then(Value::as_i64).unwrap_or(0);
            let height=v.pointer("/map_info/dimensions/1").and_then(Value::as_i64).unwrap_or(0);
            let authored=v.pointer("/map_info/curriculum_exits").and_then(Value::as_array).is_some_and(|exits|exits.iter().any(|e|
                e["kind"]=="warp" && e["target"]==destination && e["x"].as_i64()==Some(x) && e["y"].as_i64()==Some(y)));
            if authored && goals.contains(&(x,y)) && x>=0 && y>=0 && x<width && y<height {
                let direction=if y==height-1{Some("south")}else if y==0{Some("north")}else if x==0{Some("west")}else if x==width-1{Some("east")}else{None};
                if let Some(direction)=direction{cues.push(format!("story-scent:{direction}"));return cues;}
            }
        }
        if let Some(direction) = crate::curriculum::outward(v) {
            cues.push(format!("story-scent:{direction}"));
            return cues;
        }
        // Reaching the boundary tile is not the same as entering the next
        // map. Keep the outward cue until the observed transition completes.
        if let Some((x, y)) = pos(v) {
            let width = v.pointer("/map_info/dimensions/0").and_then(Value::as_i64).unwrap_or(0);
            let height = v.pointer("/map_info/dimensions/1").and_then(Value::as_i64).unwrap_or(0);
            let outward = match label.as_str() {
                "Travel north to Route 31" | "Go north to Route 30" if y == 0 => Some("north"),
                "Return south with the Mystery Egg" if y == height - 1 => Some("south"),
                "Travel west to Cherrygrove" | "Leave town west for Route 29" if x == 0 => Some("west"),
                "Return east to Elm" | "Return east to New Bark Town" if x == width - 1 => Some("east"),
                _ => None,
            };
            if let Some(direction) = outward {
                cues.push(format!("story-scent:{direction}"));
                return cues;
            }
        }
        if let Some((_, direction, aligned)) = Self::interaction(v) {
            cues.push(format!(
                "story-interaction:{}",
                if aligned { "ready" } else { "face" }
            ));
            if !aligned {
                cues.push(format!("story-scent:{direction}"));
            }
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
        self.progress = 0.0;
        self.observations = self.observations.saturating_add(1);
        // A failed movement is not permanent terrain: scripts, transient
        // occupancy and older feedback timing can all make a legal edge fail.
        // Legacy saves lack timestamps and get a bounded grace period too.
        let now=self.observations;
        self.blocked.retain(|edge| now.saturating_sub(*self.blocked_at.get(edge).unwrap_or(&0)) < 64);
        self.blocked_at.retain(|edge,_| self.blocked.contains(edge));
        self.observe(before);
        let known_before = self.terrain.get(map(after)).map_or(0, BTreeMap::len);
        self.observe(after);
        if map(before) != map(after) || [before,after].iter().any(|v|crate::field_objectives::teaching_target(v).is_some()) {
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
        if let (Some((old, _, false)), Some((new, _, true))) =
            (Self::interaction(before), Self::interaction(after))
        {
            if old == new && self.oriented.insert(new.clone()) {
                return Some(format!("breadcrumb:face:{new}"));
            }
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
        let exploration = (b != a && novel).then(|| {
            if revealed >= 3 {
                format!("exploration:revealed_{revealed}_tiles")
            } else {
                "exploration:new_ground".into()
            }
        });
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
                let edge=format!("{}:{},{}:{},{}",map(before),b.0,b.1,b.0+delta.0,b.1+delta.1);
                self.blocked.insert(edge.clone());
                self.blocked_at.insert(edge,self.observations);
            }
        }
        // Compare positions against the same observed graph and occupants.
        // A moving NPC must not create apparent progress while we stand still.
        let mut moved = before.clone();
        moved["map_info"]["player"] = after["map_info"]["player"].clone();
        let target = Self::target(before);
        if target.as_ref().map(|t| &t.0) != Self::target(after).as_ref().map(|t| &t.0) {
            return exploration;
        }
        let (label, old, new) = if let Some((label, goals)) = target.as_ref().filter(|(_,goals)| self.distance(before,goals).is_some()) {
            (label.clone(), self.distance(before, goals), self.distance(&moved, goals))
        } else {
            // Reward approaching a reachable frontier even when these stepping
            // stones were visited on a previous trip. Freeze the frontier set
            // across this action; newly exposed terrain cannot move the goalpost.
            let goals = self.frontiers(before);
            ("Explore reachable frontier".into(), self.distance(before,&goals), self.distance(&moved,&goals))
        };
        let (Some(old), Some(new)) = (old,new) else { return exploration; };
        if b != a {
            // Repeatable, signed shaping: recovery earns back the cost of a
            // detour. A closed walk sums to a negative movement cost, so it
            // cannot farm the same approach edge. Novelty remains separate.
            self.progress = 0.3 * (old as f32 - new as f32) - 0.01;
        }
        let best = self
            .best
            .entry(format!("{}:{label}", map(before)))
            .or_insert(old);
        let edge=format!("{}:{label}:{},{}:{},{}",map(before),b.0,b.1,a.0,a.1);
        // A backward/forward loop must not repeatedly earn approach rewards.
        // Changed objectives have their own ledger, allowing genuine return trips.
        if b != a && new < old && !self.rewarded_edges.insert(edge) { return exploration; }
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

#[cfg(test)]
mod tests {
    #[test]
    fn opening_door_keeps_outward_cue_until_the_map_changes() {
        let mut memory=Breadcrumbs::default();
        let mut o=json!({"status":{"screen":"overworld","party":[{}]},"reward_state":{"event_flags":[],"scenes":{"ElmsLab":"SCENE_ELMSLAB_NOOP"}},"observe":{"menus":[]},"map_info":{"name":"ElmsLab","dimensions":[10,12],"player":{"x":5,"y":11,"facing":"Down"},"curriculum_exits":[{"kind":"warp","target":"NEW_BARK_TOWN","x":5,"y":11}]}});
        assert!(memory.cues(&o).contains(&"story-scent:south".into()));
        o["observe"]["menus"]=json!([{"kind":"yes_no"}]);
        assert!(!memory.cues(&o).contains(&"story-scent:south".into()));
        o["observe"]["menus"]=json!([]);
        o["map_info"]["curriculum_exits"][0]["target"]=json!("OTHER_ROOM");
        assert!(!memory.cues(&o).contains(&"story-scent:south".into()));
        o["map_info"]["name"]=json!("NewBarkTown");
        o["map_info"]["dimensions"]=json!([20,18]);
        assert!(!memory.cues(&o).contains(&"story-scent:south".into()));
    }
    #[test]
    fn hm_teaching_suspends_walking_cues_and_route_rewards_until_learned(){
        let mut memory=Breadcrumbs::default();
        let mut v=json!({"status":{"screen":"overworld","party":[{"moves":[{"name":"TACKLE"}]}]},"reward_state":{"machines":["HM_CUT"],"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM","EVENT_HERDED_FARFETCHD","EVENT_GOT_HM01_CUT"]},"map_info":{"name":"IlexForest","dimensions":[30,38],"player":{"x":6,"y":29,"facing":"Up"},"objects":[],"curriculum_exits":[]},"observe":{"menus":[]}});
        let cues=memory.cues(&v);assert_eq!(cues,vec!["story-target:Teach CUT to a compatible party member"]);
        let mut moved=v.clone();moved["map_info"]["player"]["y"]=json!(28);
        assert!(memory.feedback(&v,&moved,"up").is_none());assert_eq!(memory.progress,0.0);
        v["status"]["party"][0]["moves"]=json!([{"name":"CUT"}]);
        assert!(!memory.cues(&v).iter().any(|s|s.contains("Teach CUT")));
    }
    #[test]
    fn farfetchd_is_ready_only_from_a_forward_chase_side(){
        let mut o=json!({"status":{"screen":"overworld"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]},"map_info":{"name":"IlexForest","player":{"x":29,"y":23,"facing":"Up"},"objects":[{"name":"ILEXFOREST_FARFETCHD","x":29,"y":22}]}});
        assert!(Breadcrumbs::interaction(&o).is_none());
        o["map_info"]["player"]=json!({"x":29,"y":21,"facing":"Down"});
        assert!(Breadcrumbs::interaction(&o).is_some_and(|(_,direction,ready)|direction=="south"&&ready));
    }
    #[test]
    fn surf_routes_use_observed_water_and_stop_at_landing() {
        let mut v=json!({"map_info":{"name":"shore","dimensions":[5,1],"player":{"x":0,"y":0},"objects":[],"terrain":{"origin_x":0,"origin_y":0,"rows":[[
            {"terrain":"Water","permission":41},{"terrain":"Water","permission":41},
            {"terrain":"Land","permission":0},{"terrain":"Water","permission":41},
            {"terrain":"Water","permission":36}
        ]]}},"reward_state":{"movement_mode":"Normal"}});
        let mut memory=Breadcrumbs::default();memory.observe(&v);
        assert_eq!(memory.distance(&v,&[(1,0)]),None);
        v["reward_state"]["movement_mode"]=json!("Surf");
        assert_eq!(memory.distance(&v,&[(1,0)]),Some(1));
        assert_eq!(memory.distance(&v,&[(2,0)]),Some(2));
        assert_eq!(memory.distance(&v,&[(3,0)]),None);
        v["map_info"]["player"]["x"]=json!(3);
        assert_eq!(memory.distance(&v,&[(4,0)]),None);
        let restored:Breadcrumbs=serde_json::from_value(serde_json::to_value(&memory).unwrap()).unwrap();
        v["map_info"]["player"]["x"]=json!(0);
        v["reward_state"]["movement_mode"]=json!("Normal");
        assert_eq!(restored.distance(&v,&[(1,0)]),None);
        let mut legacy=serde_json::to_value(&memory).unwrap();legacy.as_object_mut().unwrap().remove("water");
        assert!(serde_json::from_value::<Breadcrumbs>(legacy).is_ok());
    }
    #[test]
    fn stale_failed_edges_expire_but_observed_walls_do_not() {
        let mut memory=Breadcrumbs::default();
        let o=json!({"status":{"screen":"overworld"},"map_info":{"name":"room","dimensions":[3,1],"player":{"x":0,"y":0,"facing":"Right"},"objects":[],"terrain":{"origin_x":0,"origin_y":0,"rows":[[{"terrain":"Land"},{"terrain":"Land"},{"terrain":"Wall"}]]}},"observe":{"menus":[]}});
        memory.observe(&o);memory.blocked.insert("room:0,0:1,0".into());
        assert_eq!(memory.distance(&o,&[(1,0)]),None);
        for _ in 0..64 {memory.feedback(&o,&o,"b");}
        assert_eq!(memory.distance(&o,&[(1,0)]),Some(1));
        assert_eq!(memory.distance(&o,&[(2,0)]),None);
    }

    #[test]
    fn unknown_destination_rewards_frontier_approach_only_once() {
        let mut memory=Breadcrumbs::default();
        let before=json!({"status":{"screen":"overworld"},"map_info":{"name":"unknown","dimensions":[5,1],"player":{"x":0,"y":0,"facing":"Right"},"objects":[],"terrain":{"origin_x":0,"origin_y":0,"rows":[[{"terrain":"Land"},{"terrain":"Land"},{"terrain":"Land"}]]}},"observe":{"menus":[]}});
        let mut after=before.clone();after["map_info"]["player"]["x"]=json!(1);
        memory.explored.insert("unknown:1,0".into());
        assert_eq!(memory.feedback(&before,&after,"right"),Some("breadcrumb:Explore reachable frontier".into()));
        assert!(memory.feedback(&before,&after,"right").is_none());
    }

    #[test]
    fn exit_goal_keeps_cue_until_actual_map_transition() {
        let mut memory = Breadcrumbs::default();
        let mut o = json!({"status":{"screen":"overworld"},
            "reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]},
            "map_info":{"name":"Route30","dimensions":[20,54],"player":{"x":5,"y":0}},
            "observe":{"menus":[]}});
        assert!(memory.cues(&o).contains(&"story-scent:north".into()));
        o["observe"]["menus"] = json!([{"kind":"start"}]);
        assert!(!memory.cues(&o).contains(&"story-scent:north".into()));
        o["observe"]["menus"] = json!([]);
        o["map_info"]["name"] = json!("Route31");
        assert!(!memory.cues(&o).contains(&"story-scent:north".into()));
    }

    #[test]
    fn familiar_recovery_is_rewarded_but_closed_walks_lose() {
        let mut memory = Breadcrumbs::default();
        let far = json!({"status":{"screen":"overworld"},
            "reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]},
            "map_info":{"name":"Route30","dimensions":[1,3],
                "player":{"x":0,"y":2,"facing":"Up"},"objects":[],
                "terrain":{"origin_x":0,"origin_y":0,"rows":[
                    [{"terrain":"Land"}],[{"terrain":"Land"}],[{"terrain":"Land"}]]}},
            "observe":{"menus":[]}});
        let mut near = far.clone(); near["map_info"]["player"]["y"] = json!(1);
        memory.feedback(&far, &near, "up");
        assert!((memory.progress - 0.29).abs() < 0.0001);
        memory.feedback(&near, &far, "down");
        let retreat = memory.progress;
        let mut restored: Breadcrumbs = serde_json::from_str(&serde_json::to_string(&memory).unwrap()).unwrap();
        assert!(restored.feedback(&far, &near, "up").is_none());
        assert!(restored.progress > 0.0, "used edges still teach recovery after resume");
        assert!(retreat + restored.progress < 0.0, "round trips cannot earn progress");
        restored.feedback(&near, &near, "b");
        assert_eq!(restored.progress, 0.0, "idle inputs do not inherit movement credit");
        let mut menu = near.clone(); menu["observe"]["menus"] = json!([{"kind":"start"}]);
        restored.feedback(&far, &menu, "up");
        assert_eq!(restored.progress, 0.0, "menu controls are not navigation");
    }

    #[test]
    fn delivered_egg_keeps_a_northbound_route_objective() {
        let o=json!({"map_info":{"name":"Route30","dimensions":[20,54]},
            "reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"],"key_items":[]}});
        let (label,goals)=Breadcrumbs::target(&o).unwrap();
        assert_eq!(label,"Travel north to Route 31");
        assert_eq!(goals.len(),20);
        assert!(goals.iter().all(|p| p.1==0));
        let mut returning=o.clone();returning["reward_state"]["key_items"]=json!(["MYSTERY_EGG"]);
        assert_eq!(Breadcrumbs::target(&returning).unwrap().0,"Return south with the Mystery Egg");
    }

    use super::*;
    #[test]
    fn directional_cues_do_not_route_through_an_occupied_neighbor() {
        let mut memory=Breadcrumbs::default();
        let o=json!({"map_info":{"name":"room","dimensions":[3,3],
            "player":{"x":1,"y":0},"objects":[{"x":1,"y":1}],
            "terrain":{"origin_x":0,"origin_y":0,"rows":[
                [{"terrain":"Land"},{"terrain":"Land"},{"terrain":"Land"}],
                [{"terrain":"Land"},{"terrain":"Land"},{"terrain":"Land"}],
                [{"terrain":"Land"},{"terrain":"Land"},{"terrain":"Land"}]]}}});
        memory.observe(&o);
        assert_eq!(memory.distance(&o,&[(1,2)]),Some(4));
        let mut hypothetical=o.clone();hypothetical["map_info"]["player"]["y"]=json!(1);
        assert_eq!(memory.distance(&hypothetical,&[(1,2)]),None);
    }
    #[test]
    fn nurse_counter_is_a_reachable_interaction_without_talking_through_walls() {
        let mut v=json!({"status":{"screen":"overworld","party":[{"hp":15,"max_hp":43}]},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]},"map_info":{"name":"VioletPokecenter1F","player":{"x":3,"y":3,"facing":"Up"},"objects":[{"name":"NURSE","x":3,"y":1}],"terrain":{"origin_x":3,"origin_y":2,"rows":[[{"permission":144,"terrain":"Wall"}]]}}});
        assert!(crate::curriculum::target(&v).unwrap().1.contains(&(3,3)));
        assert!(matches!(Breadcrumbs::interaction(&v),Some((_,"north",true))));
        v["map_info"]["player"]["facing"]=json!("Left");assert!(matches!(Breadcrumbs::interaction(&v),Some((_,"north",false))));
        for permission in [0,7] {v["map_info"]["terrain"]["rows"][0][0]["permission"]=json!(permission);assert!(Breadcrumbs::interaction(&v).is_none());assert!(!crate::curriculum::target(&v).unwrap().1.contains(&(3,3)));}
        v["map_info"]["terrain"]["rows"][0][0]["permission"]=json!(152);assert!(Breadcrumbs::interaction(&v).is_some());
        v["map_info"]["terrain"]["origin_x"]=json!(4);assert!(Breadcrumbs::interaction(&v).is_none());
    }
    #[test]
    fn correct_object_facing_pays_only_once() {
        let mut memory=Breadcrumbs::default();
        let before=json!({"status":{"screen":"overworld","party":[]},
            "map_info":{"name":"PlayersHouse1F","player":{"x":2,"y":2,"facing":"Down"},
            "objects":[{"name":"PLAYERSHOUSE1F_MOM","x":2,"y":1}]},
            "observe":{"menus":[],"visible_dialogue":null},"flow_state":{"animating":false}});
        let mut after=before.clone();after["map_info"]["player"]["facing"]=json!("Up");
        assert!(memory.feedback(&before,&after,"up").unwrap().starts_with("breadcrumb:face:"));
        assert!(memory.feedback(&before,&after,"up").is_none());
        let mut restored:Breadcrumbs=serde_json::from_str(&serde_json::to_string(&memory).unwrap()).unwrap();
        assert!(restored.feedback(&before,&after,"up").is_none());
    }
}
