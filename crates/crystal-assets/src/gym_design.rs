//! Authored gym rooms and their native progression contracts.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GymDesign {
    pub schema_version: u8,
    pub status: String,
    pub gyms: Vec<Gym>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Gym {
    pub slot: u8,
    pub id: String,
    pub constant: String,
    pub city: String,
    pub leader: String,
    pub trainer_class: String,
    pub leader_sprite: String,
    pub leader_at: [u16; 2],
    pub badge_flag: String,
    pub requires: Vec<String>,
    pub reward: String,
    pub party: Vec<Member>,
    pub rows: Vec<String>,
    pub art: BTreeMap<char, [String; 2]>,
    /// Native directional walk collision on visibly distinct slatted bridges.
    #[serde(default)]
    pub winds: BTreeMap<char, String>,
    /// Paint native directional chevrons on every walking tile in wind lanes.
    #[serde(default)]
    pub wind_arrows: bool,
    /// Full native electric-grid art used as a solid charged barrier.
    #[serde(default)]
    pub barriers: BTreeSet<char>,
    /// A contained indoor pool may have native walk-to-water access like Cerulean.
    #[serde(default)]
    pub enclosed_pool: bool,
    pub trainers: Vec<Trainer>,
    pub stations: Vec<Station>,
    #[serde(default)]
    pub routers: Vec<Router>,
    #[serde(default)]
    pub switch_puzzle: Option<SwitchPuzzle>,
    #[serde(default)]
    pub pads: Vec<FloorPad>,
    #[serde(default)]
    pub ledges: Vec<Ledge>,
    #[serde(default)]
    pub lifts: Vec<Lift>,
    #[serde(default)]
    pub cut_gates: Vec<CutGate>,
    pub gates: Vec<Gate>,
    pub guide: Vec<String>,
    pub walkthrough: Vec<String>,
    pub dimensions_metatiles: [u16; 2],
    pub arrival_core: [u16; 2],
    pub exit_tiles_core: [[u16; 2]; 2],
    pub guide_at: [u16; 2],
    pub leader_court_locked: bool,
    /// Battle qualifications for an open court with no moving terrain.
    #[serde(default)]
    pub court_requires: Vec<String>,
    #[serde(default)]
    pub court_denied_lines: Vec<String>,
    #[serde(default)]
    pub before_lines: Vec<String>,
    #[serde(default)]
    pub victory_lines: Vec<String>,
    #[serde(default)]
    pub after_lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Member {
    pub species: String,
    pub level: u8,
    #[serde(default)]
    pub moves: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Trainer {
    pub id: String,
    pub name: String,
    pub trainer_class: String,
    pub sprite: String,
    pub at: [u16; 2],
    pub party: Vec<Member>,
    pub optional: bool,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub before_lines: Vec<String>,
    #[serde(default)]
    pub after_lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Station {
    pub id: String,
    pub at: [u16; 2],
    pub prompt: Vec<String>,
    pub options: Vec<String>,
    pub answer: u8,
    pub requires: Vec<String>,
    pub event: String,
    #[serde(default)]
    pub success_lines: Vec<String>,
    #[serde(default)]
    pub decline_lines: Vec<String>,
    #[serde(default)]
    pub already_lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Router {
    pub id: String,
    /// A defeated practice trainer can operate this control on repeat interaction.
    #[serde(default)]
    pub operator: Option<String>,
    /// Pressing a two-position physical switch immediately changes its state.
    #[serde(default)]
    pub toggle: bool,
    pub at: [u16; 2],
    pub prompt: Vec<String>,
    pub modes: Vec<RoutingMode>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoutingMode {
    pub id: String,
    pub label: String,
    pub requires: Vec<String>,
    pub event: String,
    pub lines: Vec<String>,
    pub denied_lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchPuzzle {
    pub controls: Vec<SwitchControl>,
    pub sequence: [String; 2],
    pub events: [String; 2],
    pub requires: Vec<String>,
    /// An optional first circuit which grants access to the next trainer.
    #[serde(default)]
    pub first_requires: Vec<String>,
    #[serde(default)]
    pub idle_lines: Vec<String>,
    pub denied_lines: Vec<String>,
    pub first_lines: Vec<String>,
    pub second_lines: Vec<String>,
    pub reset_lines: Vec<String>,
    pub solved_lines: Vec<String>,
}

impl SwitchPuzzle {
    pub fn step_requires(&self, index: usize) -> &[String] {
        if index == 0 && !self.first_requires.is_empty() {
            &self.first_requires
        } else {
            &self.requires
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchControl {
    pub id: String,
    pub at: [u16; 2],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Gate {
    pub at: [u16; 2],
    pub requires: Vec<String>,
    pub open_symbol: char,
    pub event: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FloorPad {
    pub id: String,
    pub tiles_core: Vec<[u16; 2]>,
    pub requires: Vec<String>,
    pub event: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Ledge {
    pub id: String,
    pub direction: String,
    pub hop_from: [u16; 2],
    pub landing: [u16; 2],
    pub return_passage_core: [u16; 4],
    pub minimum_return_steps: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Lift {
    pub id: String,
    pub ends_core: [[u16; 2]; 2],
}

/// A native tree replacement which opens a named portion of the course.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CutGate {
    pub id: String,
    /// Walking through the cut opening records the garden's permanent pruning.
    pub event: String,
    pub tree_core: [u16; 2],
    pub approach_core: [u16; 2],
    pub replacement_symbol: char,
    pub targets_core: Vec<[u16; 2]>,
    pub gated_trainers: Vec<String>,
}

impl Gym {
    /// Open-court gardens may add return doors that unlock only after victory.
    pub fn only_victory_gates(&self) -> bool {
        self.gates
            .iter()
            .all(|gate| gate.requires == [self.defeated()] && gate.event != self.court_event())
    }

    pub fn defeated(&self) -> String {
        format!("EVENT_MN_GYM_{}_DEFEATED", self.slot)
    }

    pub fn reward_event(&self) -> String {
        format!("EVENT_MN_GYM_{}_TM_RECEIVED", self.slot)
    }

    pub fn court_event(&self) -> String {
        format!("EVENT_MN_GYM_{}_COURT_OPEN", self.slot)
    }

    pub fn battle_id(&self) -> String {
        format!("MN_GYM_{}", self.slot)
    }

    pub fn tileset(&self) -> String {
        format!("mn_gym_{}", self.slot)
    }

    pub fn routing_mode(&self, id: &str) -> Option<(&Router, &RoutingMode)> {
        self.routers.iter().find_map(|router| {
            router
                .modes
                .iter()
                .find(|mode| mode.id == id)
                .map(|mode| (router, mode))
        })
    }

    pub fn switch_step(&self, id: &str) -> Option<(&SwitchPuzzle, &SwitchControl, usize)> {
        let puzzle = self.switch_puzzle.as_ref()?;
        let index = puzzle.sequence.iter().position(|step| step == id)?;
        let control = puzzle.controls.iter().find(|control| control.id == id)?;
        Some((puzzle, control, index))
    }
}

impl GymDesign {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == 1 && !self.status.is_empty(),
            "gym schema/status"
        );
        ensure!(self.gyms.len() == 8, "all eight gym rooms are required");
        let mut identities = std::collections::BTreeSet::new();
        for (index, gym) in self.gyms.iter().enumerate() {
            let [width, height] = gym.dimensions_metatiles;
            ensure!(
                (5..=16).contains(&width) && (5..=16).contains(&height),
                "{}: unsupported room extent",
                gym.id
            );
            ensure!(
                usize::from(gym.slot) == index + 1,
                "gym order must match badge slots"
            );
            ensure!(
                identities.insert(&gym.id) && !gym.city.is_empty(),
                "unique named gym cities"
            );
            ensure!(
                gym.rows.len() == usize::from(height)
                    && gym
                        .rows
                        .iter()
                        .all(|r| r.chars().count() == usize::from(width)),
                "{}: authored room extent",
                gym.id
            );
            ensure!(
                gym.art.contains_key(&'#'),
                "{}: missing boundary art",
                gym.id
            );
            ensure!(
                gym.barriers.iter().all(|symbol| {
                    gym.art.contains_key(symbol)
                        && gym.rows.iter().any(|row| row.contains(*symbol))
                        && !gym.winds.contains_key(symbol)
                }),
                "{}: solid barriers require visible, unambiguous art",
                gym.id
            );
            ensure!(
                gym.rows
                    .iter()
                    .flat_map(|r| r.chars())
                    .all(|s| gym.art.contains_key(&s)),
                "{}: missing native art recipe",
                gym.id
            );
            let court_gate = gym.gates.iter().any(|g| g.event == gym.court_event());
            ensure!(
                court_gate == gym.court_requires.is_empty(),
                "{}: use either a court gate or battle qualifications",
                gym.id
            );
            if !gym.court_requires.is_empty() {
                ensure!(
                    (!gym.leader_court_locked || !gym.cut_gates.is_empty())
                        && gym.only_victory_gates()
                        && gym.court_requires.iter().all(|token| gym
                            .trainers
                            .iter()
                            .any(|trainer| !trainer.optional
                                && token == &format!("EVENT_{}_DEFEATED", trainer.id))),
                    "{}: battle qualifications require mandatory trainer victories",
                    gym.id
                );
            }
            for gate in &gym.gates {
                ensure!(
                    gate.at[0] < width
                        && gate.at[1] < height
                        && gym.art.contains_key(&gate.open_symbol)
                        && !gate.requires.is_empty(),
                    "{}: invalid gate",
                    gym.id
                );
            }
            ensure!(
                !gym.wind_arrows || !gym.winds.is_empty(),
                "marked wind needs declared lanes"
            );
            crate::gym_cut::validate_design(gym)?;
            let mut actors = std::collections::BTreeSet::from([gym.guide_at, gym.leader_at]);
            for at in gym
                .trainers
                .iter()
                .map(|t| t.at)
                .chain(gym.stations.iter().map(|s| s.at))
                .chain(
                    gym.routers
                        .iter()
                        .filter(|r| r.operator.is_none())
                        .map(|r| r.at),
                )
            {
                ensure!(
                    at[0] < width * 2 && at[1] < height * 2 && actors.insert(at),
                    "{}: overlapping/outside actor {at:?}",
                    gym.id
                );
            }
            let mut lift_ids = std::collections::BTreeSet::new();
            let mut lift_tiles = std::collections::BTreeSet::new();
            for lift in &gym.lifts {
                ensure!(
                    !lift.id.is_empty() && lift_ids.insert(&lift.id),
                    "{}: unnamed or duplicate lift",
                    gym.id
                );
                for at in lift.ends_core {
                    ensure!(
                        at[0] < width * 2
                            && at[1] < height * 2
                            && lift_tiles.insert(at)
                            && !actors.contains(&at)
                            && at != gym.arrival_core
                            && !gym.exit_tiles_core.contains(&at),
                        "{}: overlapping or invalid lift pad {at:?}",
                        gym.id
                    );
                }
            }
            for station in &gym.stations {
                ensure!(
                    (2..=3).contains(&station.options.len())
                        && (1..=station.options.len()).contains(&usize::from(station.answer)),
                    "{}: invalid station choice",
                    gym.id
                );
            }
            let mut routing_events = gym
                .gates
                .iter()
                .map(|g| g.event.clone())
                .chain(gym.stations.iter().map(|s| s.event.clone()))
                .chain(gym.pads.iter().map(|p| p.event.clone()))
                .chain(
                    gym.trainers
                        .iter()
                        .map(|t| format!("EVENT_{}_DEFEATED", t.id)),
                )
                .chain([gym.defeated(), gym.reward_event()])
                .collect::<std::collections::BTreeSet<_>>();
            let mut router_ids = std::collections::BTreeSet::new();
            let mut operators = std::collections::BTreeSet::new();
            for router in &gym.routers {
                if let Some(operator) = &router.operator {
                    ensure!(
                        operators.insert(operator)
                            && gym
                                .trainers
                                .iter()
                                .any(|t| &t.id == operator && t.at == router.at),
                        "{}: control operator must be its existing trainer",
                        gym.id
                    );
                }
                ensure!(
                    !router.id.is_empty()
                        && router_ids.insert(&router.id)
                        && !router.prompt.is_empty()
                        && (2..=3).contains(&router.modes.len()),
                    "{}: incomplete routing desk",
                    gym.id
                );
                ensure!(
                    !router.toggle || router.modes.len() == 2,
                    "{}: a toggle switch requires two positions",
                    gym.id
                );
                for mode in &router.modes {
                    ensure!(
                        !mode.id.is_empty()
                            && !mode.label.is_empty()
                            && mode.label.len() <= 17
                            && !mode.label.contains(['\"', '@', '\n'])
                            && !mode.requires.is_empty()
                            && !mode.lines.is_empty()
                            && !mode.denied_lines.is_empty()
                            && mode
                                .event
                                .starts_with(&format!("EVENT_MN_GYM_{}_", gym.slot))
                            && routing_events.insert(mode.event.clone()),
                        "{}: invalid or colliding routing mode {}",
                        gym.id,
                        mode.id
                    );
                }
            }
            let mut pad_tiles = std::collections::BTreeSet::new();
            crate::gym_switches::validate_design(gym, &mut actors, &mut routing_events)?;
            let mut ledge_ids = std::collections::BTreeSet::new();
            for ledge in &gym.ledges {
                ensure!(
                    !ledge.id.is_empty()
                        && ledge_ids.insert(&ledge.id)
                        && [ledge.hop_from, ledge.landing]
                            .iter()
                            .all(|at| { at[0] < width * 2 && at[1] < height * 2 }),
                    "{}: unnamed, duplicate or out-of-bounds ledge",
                    gym.id
                );
            }
            for pad in &gym.pads {
                ensure!(
                    !pad.tiles_core.is_empty() && !pad.requires.is_empty() && !pad.lines.is_empty(),
                    "{}: incomplete floor pad {}",
                    gym.id,
                    pad.id
                );
                for at in &pad.tiles_core {
                    ensure!(
                        at[0] < width * 2
                            && at[1] < height * 2
                            && !actors.contains(at)
                            && pad_tiles.insert(at),
                        "{}: overlapping/outside floor pad {at:?}",
                        gym.id
                    );
                }
            }
            ensure!(
                gym.trainers.iter().any(|t| !t.optional),
                "{}: required trainers",
                gym.id
            );
            let unused_toggle_positions = gym
                .routers
                .iter()
                .filter(|r| r.toggle)
                .map(|r| {
                    r.modes
                        .iter()
                        .filter(|m| !gym.walkthrough.contains(&m.id))
                        .count()
                })
                .sum::<usize>();
            ensure!(
                gym.walkthrough.len()
                    == gym.trainers.len()
                        + gym.stations.len()
                        + gym.pads.len()
                        + gym.cut_gates.len()
                        + gym.routers.iter().map(|r| r.modes.len()).sum::<usize>()
                        - unused_toggle_positions
                        + gym.switch_puzzle.as_ref().map_or(0, |_| 2)
                    && gym
                        .routers
                        .iter()
                        .filter(|r| r.toggle)
                        .all(|r| { r.modes.iter().any(|m| gym.walkthrough.contains(&m.id)) })
                    && gym
                        .walkthrough
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == gym.walkthrough.len()
                    && gym.walkthrough.iter().all(|step| gym
                        .trainers
                        .iter()
                        .any(|t| &t.id == step)
                        || gym.stations.iter().any(|s| &s.id == step)
                        || gym.pads.iter().any(|p| &p.id == step)
                        || gym.cut_gates.iter().any(|gate| &gate.id == step)
                        || gym
                            .switch_puzzle
                            .as_ref()
                            .is_some_and(|p| p.sequence.contains(step))
                        || gym.routing_mode(step).is_some()),
                "{}: incomplete walkthrough",
                gym.id
            );
        }
        Ok(())
    }
}
