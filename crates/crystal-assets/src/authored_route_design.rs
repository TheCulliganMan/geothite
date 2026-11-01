//! Typed physical anchors from the route dossier; prose remains editor metadata.
use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use crystal_core::world::encounters::{WildEncounter, WildEncounterTable};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct RouteDesign {
    pub id: String,
    pub map_prefix: String,
    pub endpoints: Vec<Endpoint>,
    pub dimensions_metatiles: [u16; 2],
    pub chunk_metatiles: [u16; 2],
    pub chunk_columns: u16,
    pub chunk_rows: u16,
    pub required_flags: Vec<String>,
    pub trainers: Vec<RouteTrainer>,
    pub residents: Vec<Resident>,
    pub items: Vec<Pickup>,
    pub signs: Vec<Sign>,
    pub doors: Vec<Door>,
    pub interiors: Vec<Interior>,
    pub encounters: Encounters,
    pub shortcut: Option<Shortcut>,
    #[serde(default)]
    pub surf_only_actor_ids: Vec<String>,
    pub viewpoints: Vec<Anchor>,
    #[serde(default)]
    pub required_grass_crossings: Vec<GrassCrossing>,
    #[serde(default)]
    pub optional_grass_rewards: Vec<GrassReward>,
    #[serde(default)]
    pub one_way_ledges: Vec<OneWayLedge>,
    /// Former hop faces explicitly replaced by tall retaining walls.
    #[serde(default)]
    pub closed_high_cliffs: Vec<OneWayLedge>,
    #[serde(default)]
    pub battle_or_grass_choices: Vec<BattleGrassChoice>,
    #[serde(default)]
    pub cut_gates: Vec<CutGate>,
}

#[derive(Deserialize)]
pub(crate) struct CutGate {
    pub id: String,
    pub tile: [u16; 2],
    pub approach: [u16; 2],
    pub replacement_block: String,
    pub targets: Vec<[u16; 2]>,
    pub gated_actor_ids: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct BattleGrassChoice {
    pub id: String,
    pub trainer_id: String,
    pub bounds_core: [u16; 4],
    pub minimum_grass_steps: u16,
}

#[derive(Deserialize)]
pub(crate) struct OneWayLedge {
    pub id: String,
    pub bounds_core: [u16; 4],
    pub direction: String,
    pub upper_approach: [u16; 2],
    pub hop_from: [u16; 2],
    pub landing: [u16; 2],
    pub reverse_blocked: [u16; 2],
    pub return_passage_core: [u16; 4],
    /// Optional local walking loop; global cheap ascents are still rejected.
    #[serde(default)]
    pub return_region_core: Option<[u16; 4]>,
    pub minimum_return_steps: u16,
    #[serde(default)]
    pub side_barriers: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
pub(crate) struct GrassCrossing {
    pub id: String,
    pub bounds_core: [u16; 4],
    pub minimum_grass_steps: u16,
}

#[derive(Deserialize)]
pub(crate) struct GrassReward {
    pub actor_id: String,
    pub bounds_core: [u16; 4],
    /// Additional parts of the same encounter threshold, including solid edges.
    #[serde(default)]
    pub additional_bounds_core: Vec<[u16; 4]>,
    pub minimum_grass_steps: u16,
}

#[derive(Deserialize)]
pub(crate) struct RouteTrainer {
    pub id: String,
    pub display_name: String,
    pub trainer_class: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub facing: Facing,
    pub sight_range: u8,
    pub defeated_flag: String,
    pub party: Vec<PartyMember>,
    pub before_lines: Vec<String>,
    pub after_lines: Vec<String>,
    pub loss_lines: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct PartyMember {
    pub species: String,
    pub level: u8,
}

#[derive(Deserialize)]
pub(crate) struct Resident {
    pub id: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub facing: Facing,
    pub lines: Vec<String>,
    #[serde(default)]
    pub service: Option<ResidentService>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ResidentService {
    HealParty,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Facing {
    North,
    South,
    East,
    West,
}

impl Facing {
    pub fn movement(self) -> &'static str {
        match self {
            Self::North => "SPRITEMOVEDATA_STANDING_UP",
            Self::South => "SPRITEMOVEDATA_STANDING_DOWN",
            Self::East => "SPRITEMOVEDATA_STANDING_RIGHT",
            Self::West => "SPRITEMOVEDATA_STANDING_LEFT",
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct Pickup {
    pub id: String,
    pub item: String,
    pub quantity: u16,
    pub flag: String,
    pub tile: [u16; 2],
}

#[derive(Deserialize)]
pub(crate) struct Sign {
    pub id: String,
    pub tile: [u16; 2],
    pub lines: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct Door {
    pub warp_index: u16,
    #[serde(default)]
    pub return_warp_index: Option<u16>,
    #[serde(default)]
    pub return_target_warp: Option<i16>,
    pub tile: [u16; 2],
    pub target: String,
    pub target_warp: i16,
    pub return_tile: [u16; 2],
    pub arrival_tile: [u16; 2],
    #[serde(default = "door_collision")]
    pub collision: String,
}

#[derive(Deserialize)]
pub(crate) struct Interior {
    pub id: String,
    pub template: String,
    pub dimensions_metatiles: [u16; 2],
    pub rows: Vec<String>,
    pub exits: Vec<[u16; 2]>,
    /// Optional exterior door for each interior exit, in native warp order.
    #[serde(default)]
    pub exit_doors: Vec<[u16; 2]>,
    /// Optional nontriggering destination following the active exit records.
    #[serde(default)]
    pub passive_arrival: Option<PassiveArrival>,
    #[serde(default)]
    pub keeper_tile: Option<[u16; 2]>,
    #[serde(default)]
    pub keeper_lines: Vec<String>,
    #[serde(default)]
    pub residents: Vec<Resident>,
    #[serde(default)]
    pub items: Vec<Pickup>,
    #[serde(default)]
    pub signs: Vec<Sign>,
    #[serde(default)]
    pub clear_arrival: Option<[u16; 2]>,
    #[serde(default = "carpet_collision")]
    pub exit_collision: String,
    #[serde(default)]
    pub border_block: Option<u8>,
}

#[derive(Deserialize)]
pub(crate) struct PassiveArrival {
    pub index: u16,
    pub tile: [u16; 2],
}

#[derive(Deserialize)]
pub(crate) struct Encounters {
    pub grass_rates: BTreeMap<String, u8>,
    pub grass: WildEncounterTable,
    pub water_rate: u8,
    pub water: Vec<WildEncounter>,
}

impl RouteDesign {
    pub fn parse(text: &str) -> Result<Self> {
        let design: Self = serde_json::from_str(text)?;
        let closed_r10_carrier = design.id == "MN_R10"
            && design.one_way_ledges.is_empty()
            && design.closed_high_cliffs.len() == 1
            && design.closed_high_cliffs[0].id == "CARRIER_BANK_DROP"
            && design.closed_high_cliffs[0].direction == "south"
            && design.closed_high_cliffs[0].bounds_core == [24, 20, 28, 22]
            && design.closed_high_cliffs[0].upper_approach == [25, 19]
            && design.closed_high_cliffs[0].hop_from == [25, 20]
            && design.closed_high_cliffs[0].landing == [25, 22]
            && design.closed_high_cliffs[0].reverse_blocked == [25, 21]
            && design.closed_high_cliffs[0].return_passage_core == [10, 20, 12, 22]
            && design.closed_high_cliffs[0].minimum_return_steps >= 25;
        ensure!(
            design.closed_high_cliffs.is_empty() || closed_r10_carrier,
            "closed high cliffs must retain the explicit R10 carrier wall and stair return"
        );
        let (dimensions, columns, rows, trainers, residents, rooms, prefix) =
            match design.id.as_str() {
                "MN_R01" => ([32, 20], 1, 1, 3, 2, 2, "MnR01"),
                "MN_R02" => ([30, 22], 1, 1, 3, 2, 2, "MnR02"),
                "MN_R03" => ([24, 32], 1, 2, 4, 2, 1, "MnR03"),
                "MN_R04" => ([28, 18], 1, 1, 2, 2, 0, "MnR04"),
                "MN_R05" => ([32, 56], 1, 3, 5, 3, 1, "MnR05"),
                "MN_R06" => ([32, 44], 1, 2, 5, 3, 1, "MnR06"),
                "MN_R07" => ([32, 48], 1, 2, 8, 3, 0, "MnR07"),
                "MN_R08" => ([40, 48], 2, 2, 8, 3, 0, "MnR08"),
                "MN_R09" => ([64, 36], 2, 2, 8, 3, 1, "MnR09"),
                "MN_R10" => ([48, 20], 2, 1, 6, 3, 1, "MnR10"),
                "MN_R16" => ([48, 24], 2, 1, 10, 3, 0, "MnR16"),
                "MN_R11" if design.dimensions_metatiles == [56, 96] => {
                    let rooms = if design
                        .interiors
                        .iter()
                        .any(|room| room.id == "MnRyansHouse")
                    {
                        2
                    } else {
                        1
                    };
                    ([56, 96], 2, 4, 10, 5, rooms, "MnR11")
                }
                "MN_R11" => ([32, 72], 1, 3, 10, 5, 1, "MnR11"),
                "MN_R12" if design.dimensions_metatiles == [32, 32] => {
                    ([32, 32], 1, 2, 10, 6, 2, "MnR12")
                }
                "MN_R12" => ([64, 56], 2, 3, 10, 6, 2, "MnR12"),
                "MN_R13" => ([32, 64], 1, 3, 7, 6, 1, "MnR13"),
                "MN_R14" => ([48, 32], 2, 2, 6, 6, 0, "MnR14"),
                "MN_R18" => ([40, 24], 2, 1, 6, 4, 0, "MnR18"),
                "MN_R19" => ([32, 30], 1, 2, 6, 2, 0, "MnR19"),
                "MN_R20" => ([64, 32], 2, 2, 10, 3, 1, "MnR20"),
                "MN_R17" => ([32, 56], 1, 3, 6, 3, 2, "MnR17"),
                "MN_R15" => ([32, 18], 1, 1, 2, 2, 1, "MnR15"),
                _ => anyhow::bail!("unsupported authored route {}", design.id),
            };
        ensure!(
            design.dimensions_metatiles == dimensions
                && design.chunk_metatiles == [32, 24]
                && design.chunk_columns == columns
                && design.chunk_rows == rows
                && design.map_prefix == prefix
                && design.endpoints.len() == 2,
            "retain the complete authored route extent"
        );
        ensure!(
            design.trainers.len() == trainers
                && design.residents.len() == residents
                && design.items.len()
                    == match design.id.as_str() {
                        "MN_R11" if design.dimensions_metatiles == [56, 96] => 6,
                        "MN_R12" if design.dimensions_metatiles == [32, 32] => 5,
                        "MN_R01" | "MN_R02" | "MN_R15" => 2,
                        "MN_R03" | "MN_R04" | "MN_R05" | "MN_R06" | "MN_R10" | "MN_R17"
                        | "MN_R20" => 3,
                        _ => 4,
                    }
                && design.signs.len()
                    == match design.id.as_str() {
                        "MN_R11" if design.dimensions_metatiles == [56, 96] => 8 + rooms,
                        "MN_R12" if design.dimensions_metatiles == [32, 32] => 7,
                        "MN_R01" | "MN_R02" | "MN_R03" | "MN_R04" => 5,
                        "MN_R15" => 3,
                        "MN_R07" => 7,
                        "MN_R09" => 8,
                        "MN_R16" | "MN_R20" => 5,
                        "MN_R17" => 9,
                        _ => 6,
                    }
                && design.doors.len()
                    == if matches!(design.id.as_str(), "MN_R05" | "MN_R20") {
                        2
                    } else {
                        rooms
                    }
                && design.interiors.len() == rooms,
            "missing authored route events"
        );
        if design.id == "MN_R19" {
            ensure!(
                design.required_flags == ["BADGE_6"]
                    && design.required_grass_crossings.len() == 1
                    && design.battle_or_grass_choices.len() == 1
                    && (design.one_way_ledges.is_empty()
                        || (design.one_way_ledges.len() == 1
                            && design.one_way_ledges[0].id == "MN_R19_OAK_EDGE_DESCENT"))
                    && design.cut_gates.is_empty()
                    && design.shortcut.is_none()
                    && design.surf_only_actor_ids.is_empty(),
                "retain the prairie/oak route, observation loop and grass choice"
            );
            return Ok(design);
        }
        if design.id == "MN_R18" {
            ensure!(
                design.required_flags == ["BADGE_6"]
                    && design.required_grass_crossings.len() == 1
                    && design.one_way_ledges.is_empty()
                    && design.cut_gates.is_empty()
                    && design.shortcut.is_none()
                    && design.surf_only_actor_ids == ["MN_R18_ITEM_04"],
                "retain the sixth-badge shore route and optional Surf-islet pickup"
            );
            return Ok(design);
        }
        ensure!(
            design.surf_only_actor_ids.is_empty(),
            "unexpected Surf-only actor contract"
        );
        if design.id == "MN_R20" {
            let expected = (1..=8)
                .map(|n| format!("BADGE_{n}"))
                .chain(["EVENT_MN_LIGHTHOUSE_SIGNAL".into()])
                .collect::<Vec<_>>();
            ensure!(
                design.required_flags == expected
                    && design.required_grass_crossings.len() == 1
                    && design.one_way_ledges.len() == 1
                    && design.cut_gates.is_empty()
                    && design.shortcut.is_none(),
                "retain the free border route gates, habitat and directed bank"
            );
            return Ok(design);
        }
        if design.id == "MN_R01" {
            ensure!(
                design.required_flags == ["EVENT_MN_STARTER_RECEIVED"]
                    && design.required_grass_crossings.len() == 1
                    && design.one_way_ledges.len() == 1,
                "retain Route 1's starter gate, grass crossing and one-way return"
            );
            return Ok(design);
        }
        if design.id == "MN_R02" {
            ensure!(
                design.required_flags == ["BADGE_1"]
                    && design.required_grass_crossings.len() == 1
                    && design.one_way_ledges.len() == 1,
                "retain the first-badge gate, pasture crossing and nursery drop"
            );
            return Ok(design);
        }
        if design.id == "MN_R03" {
            ensure!(
                design.required_flags == ["BADGE_2"]
                    && design.required_grass_crossings.len() == 1
                    && design
                        .one_way_ledges
                        .iter()
                        .any(|ledge| ledge.id == "TERRACE_DROP"),
                "retain the second-badge gate, habitat shelf and optional terrace"
            );
            return Ok(design);
        }
        if design.id == "MN_R04" {
            ensure!(
                design.required_flags == ["BADGE_3"]
                    && design.battle_or_grass_choices.len() == 1
                    && design.required_grass_crossings.is_empty()
                    && design.one_way_ledges.is_empty(),
                "retain the third-badge gate and two-way civic park choice"
            );
            return Ok(design);
        }
        if design.id == "MN_R05" {
            ensure!(
                design.required_flags == ["BADGE_3", "BADGE_4"]
                    && design.required_grass_crossings.len() == 1
                    && design.one_way_ledges.len() == 2,
                "retain both Twin Cities badges, shared habitat and two return drops"
            );
            return Ok(design);
        }
        if matches!(design.id.as_str(), "MN_R06" | "MN_R07" | "MN_R08") {
            let badge = if design.id == "MN_R06" {
                "BADGE_5"
            } else {
                "BADGE_6"
            };
            ensure!(
                design.required_flags == [badge]
                    && design.required_grass_crossings.len() == 1
                    && (design.one_way_ledges.len() == 1
                        || (design.id == "MN_R06"
                            && design
                                .one_way_ledges
                                .iter()
                                .map(|ledge| ledge.id.as_str())
                                .eq(["PINE_LOOKOUT_RETURN", "BIRD_GLADE_RETURN"]))
                        || (design.id == "MN_R08"
                            && design.one_way_ledges.len() == 2
                            && design.one_way_ledges[0].id == "BIRCH_RETURN_DROP"
                            && design.one_way_ledges[1].id == "CEDAR_PIER_BANK_DROP"))
                    && design.cut_gates.len() == 1,
                "retain the authored regional badge, habitat, ledge and real Cut gate"
            );
            return Ok(design);
        }
        if matches!(design.id.as_str(), "MN_R09" | "MN_R10" | "MN_R16") {
            let expected_flags: &[&str] = match design.id.as_str() {
                "MN_R09" => &["BADGE_7", "EVENT_MN_HEADWATERS_REPORT"],
                "MN_R10" => &["BADGE_7", "EVENT_MN_DEPOT_CLEARED"],
                _ => &["BADGE_6"],
            };
            let retained_or_mine_apron_ledges = closed_r10_carrier
                || design.one_way_ledges.len() == 1
                || (design.id == "MN_R09"
                    && design.one_way_ledges.len() == 2
                    && design.one_way_ledges[0].id == "MN_R09_SPOIL_RETURN"
                    && design.one_way_ledges[1].id == "MN_R09_MINE_APRON_RETURN"
                    && design.one_way_ledges[1].bounds_core == [38, 34, 40, 36]
                    && design.one_way_ledges[1].upper_approach == [39, 33]
                    && design.one_way_ledges[1].landing == [39, 36]
                    && design.one_way_ledges[1].return_passage_core == [36, 34, 37, 36]);
            ensure!(
                design.required_flags == expected_flags
                    && design.required_grass_crossings.len() == 1
                    && retained_or_mine_apron_ledges
                    && design.cut_gates.len() == usize::from(design.id == "MN_R09")
                    && design.battle_or_grass_choices.len() == usize::from(design.id != "MN_R10"),
                "retain the authored regional gates and distinct terrain choices"
            );
            return Ok(design);
        }
        if design.id == "MN_R17" {
            ensure!(
                design.required_flags == ["BADGE_7"]
                    && design.required_grass_crossings.len() == 1
                    && design.one_way_ledges.is_empty()
                    && design.cut_gates.is_empty()
                    && design.shortcut.is_some(),
                "retain Border Pines badge, marsh habitat and useful Cut return"
            );
            return Ok(design);
        }
        if matches!(design.id.as_str(), "MN_R11" | "MN_R12") {
            ensure!(
                if design.id == "MN_R11" && design.dimensions_metatiles == [56, 96] {
                    design.required_grass_crossings.len() == 2 && design.one_way_ledges.len() == 3
                } else {
                    design.required_grass_crossings.len() == 1 && design.one_way_ledges.is_empty()
                } && design.shortcut.is_none()
                    && design.cut_gates.len() == usize::from(design.id == "MN_R12"),
                "retain the authored coast or forest habitat and native Cut contract"
            );
        }
        if matches!(design.id.as_str(), "MN_R14" | "MN_R15") {
            if design.id == "MN_R14" {
                ensure!(
                    design.required_grass_crossings.len() == 1
                        && design.one_way_ledges.len() == 1
                        && design.cut_gates.len() == 1
                        && design.shortcut.is_some(),
                    "retain Headwaters habitat, bank and useful Cut circuit"
                );
            }
            if design.id == "MN_R15" {
                ensure!(
                    design.required_grass_crossings.len() == 1 && design.one_way_ledges.len() == 1,
                    "Central Lakes requires its grass crossing and one-way return"
                );
            }
            ensure!(
                design.required_flags == ["BADGE_6"],
                "retain the six-badge side-route gate"
            );
            return Ok(design);
        }
        let story_gate = if design.id == "MN_R13" {
            "EVENT_MN_SURVEY_RESTORED"
        } else {
            "EVENT_MN_LIGHTHOUSE_SIGNAL"
        };
        for flag in (1..=8)
            .map(|n| format!("BADGE_{n}"))
            .chain([story_gate.into()])
        {
            ensure!(
                design.required_flags.contains(&flag),
                "regional approach lost {flag}"
            );
        }
        Ok(design)
    }
}

#[derive(Deserialize)]
pub(crate) struct Endpoint {
    pub gate_metatile: [u16; 2],
}

#[derive(Deserialize)]
pub(crate) struct Shortcut {
    pub tile: [u16; 2],
    pub approach: [u16; 2],
    #[serde(default)]
    pub compare_from: Option<[u16; 2]>,
    #[serde(default)]
    pub compare_to: Option<[u16; 2]>,
    #[serde(default)]
    pub minimum_saved_steps: Option<u16>,
    #[serde(default)]
    pub useful_target_actor_ids: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct Anchor {
    pub tile: [u16; 2],
}

fn door_collision() -> String {
    "DOOR".into()
}
fn carpet_collision() -> String {
    "WARP_CARPET_DOWN".into()
}
