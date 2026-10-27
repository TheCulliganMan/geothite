//! Q07 chief and receiver bindings for the caller's actual free survey shelter.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerPartyPokemon;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Mandatory free story shelter, separate from the optional paid preserve.
pub const BOUNDARY_SURVEY_SHELTER: &str = "MnBoundaryWatersSurveyShelter";
/// Native constant supplied by the real shelter publisher.
pub const BOUNDARY_SURVEY_SHELTER_CONSTANT: &str = "MN_BOUNDARY_WATERS_SURVEY_SHELTER";
/// Existing story reservation for the chief's actual opposing party.
pub const BOUNDARY_SURVEY_CHIEF_TRAINER: &str = "MN_LODE_CHIEF";
/// Existing logical chief object/script identity in the authored shelter.
pub const BOUNDARY_SURVEY_CHIEF: &str = "MN_SURVEY_CHIEF";
/// Separate receiver background interaction in the authored shelter.
pub const BOUNDARY_SURVEY_RECEIVER: &str = "MN_BOUNDARY_SURVEY_RECEIVER";
/// Upstream Q07 prerequisite; this binder allocates metadata but never sets it.
pub const BOUNDARY_LIGHTHOUSE_SIGNAL_EVENT: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
/// Set only on the chief's native post-victory continuation.
pub const BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT: &str = "EVENT_MN_SURVEY_CHIEF_DEFEATED";
/// Set only by the separate post-victory receiver interaction.
pub const BOUNDARY_SURVEY_RESTORED_EVENT: &str = "EVENT_MN_SURVEY_RESTORED";

const BADGES: [&str; 8] = [
    "ENGINE_ZEPHYRBADGE",
    "ENGINE_HIVEBADGE",
    "ENGINE_PLAINBADGE",
    "ENGINE_FOGBADGE",
    "ENGINE_STORMBADGE",
    "ENGINE_MINERALBADGE",
    "ENGINE_GLACIERBADGE",
    "ENGINE_RISINGBADGE",
];
const AFTER: &str = "MnBoundarySurveyChiefAfter";
const BADGE_DENIED: &str = "MnBoundarySurveyChiefBadgeDenied";
const SIGNAL_DENIED: &str = "MnBoundarySurveyChiefSignalDenied";
const RECEIVER_LOCKED: &str = "MnBoundarySurveyReceiverLocked";
const RECEIVER_ALREADY: &str = "MnBoundarySurveyReceiverAlready";
const RECEIVER_DECLINED: &str = "MnBoundarySurveyReceiverDeclined";

/// Frozen room anchors supplied by the terrain author, in16px walking tiles.
///
/// The binder compares these with the actual caller-supplied native object,
/// background event and collision. It never invents a substitute room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryBasecampStoryAnchors {
    pub chief_tile: [u16; 2],
    pub chief_approaches: [[u16; 2]; 3],
    pub receiver_tile: [u16; 2],
    pub receiver_approach: [u16; 2],
    pub clear_arrival: [u16; 2],
}

/// Prepared story/catalog additions and the caller's actual shelter module.
///
/// The caller must atomically publish the returned module with its basecamp,
/// real warps and metadata. No map, spawn, recovery, hub gate or Safari session
/// is registered by this preparation step.
pub struct PreparedBoundaryBasecampStory {
    pub pack: CompiledGamePack,
    pub shelter: MapModule,
}

/// Binds MS11/MS12 to the real, freely accessible Boundary Waters shelter.
///
/// MS11 checks all eight Johto badges and the existing lighthouse signal. The
/// native scripted trainer battle uses the reserved Magneton44/Weezing44/
/// Crobat46 party. Its post-battle reload follows the supplied game's boss
/// pattern: loss whiteouts do not continue to the victory event. MS12 is a
/// separate receiver interaction requiring that victory, with decline/repeat
/// behavior and no paid admission or capture conditions.
///
/// Missing story-event identifiers are allocated additively; existing unique
/// identifiers are preserved. No saved flag is granted during preparation.
/// Existing source maps, files, audio, catalogs and shelter geometry/warps/
/// unrelated actors remain unchanged. Both inputs remain unchanged on error.
///
/// # Errors
/// Returns an error for changed room identities/anchors, blocked approaches,
/// absent or ambiguous native actors, wrong console art/collision, conflicting
/// script/trainer/event identities, unavailable source content or native parser
/// failures. The returned pack is prepared catalog data, not a published town.
pub fn prepare_boundary_basecamp_story(
    base: &CompiledGamePack,
    shelter: &MapModule,
    anchors: BoundaryBasecampStoryAnchors,
) -> Result<PreparedBoundaryBasecampStory> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    validate_anchors(base, shelter, anchors)?;
    let mut pack = base.clone();
    let mut authored = shelter.clone();
    allocate_events(&mut pack)?;
    add_chief_party(&mut pack)?;
    let scripts = story_scripts();
    let texts = story_texts();
    for label in scripts.keys().chain(texts.keys()) {
        if label == BOUNDARY_SURVEY_CHIEF || label == BOUNDARY_SURVEY_RECEIVER {
            continue;
        }
        ensure!(
            !authored.scripts.contains_key(label),
            "Q07 script namespace collision {label}"
        );
    }
    for (label, commands) in scripts {
        authored.scripts.insert(label, commands);
    }
    for (label, lines) in texts {
        ensure!(
            lines
                .iter()
                .all(|line| !line.is_empty() && line.chars().count() <= 18),
            "Q07 dialogue exceeds18columns"
        );
        authoring::text(&mut authored, &label, &lines);
    }
    authoring::finish(&pack, &mut authored)?;
    authored.script_runtime_commands =
        crate::parse_script_runtime_commands(&authored.id, &authored.scripts)?;
    authored.script_object_commands =
        crate::parse_script_object_commands(&authored.id, &authored.scripts)?;
    authored.script_variable_commands =
        crate::parse_script_variable_commands(&authored.id, &authored.scripts)?;
    authored.trainer_scripts = crate::parse_trainer_scripts(&authored.id, &authored.scripts)?;
    authored.scripted_trainer_battles =
        crate::parse_scripted_trainer_battles(&authored.id, &authored.scripts)?;
    let map_names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect();
    authored.script_map_commands =
        crate::parse_script_map_commands(&authored.id, &authored.scripts, &map_names)?;
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(PreparedBoundaryBasecampStory {
        pack,
        shelter: authored,
    })
}

/// Refreshes the installed Q07 finale's dialogue to the current Northstar story.
///
/// Requires the existing supported chief/receiver script logic. Only the scene's
/// text and derived text tables change; maps, geometry, events, trainer parties,
/// progression commands, audio and recovery destinations remain intact. No
/// saved game state is created or granted. Repeating the refresh is idempotent.
///
/// # Errors
/// Returns an error for an absent finale, changed script logic, missing text
/// labels, or native parser/pack validation failures. The input is unchanged.
pub fn refresh_boundary_basecamp_story_dialogue(
    base: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut room = base
        .data
        .maps
        .get(BOUNDARY_SURVEY_SHELTER)
        .context("installed native Q07 finale")?
        .clone();
    for (label, commands) in story_scripts() {
        ensure!(
            room.scripts.get(&label) == Some(&commands),
            "Q07 dialogue refresh requires unchanged native scene logic: {label}"
        );
    }
    for (label, lines) in story_texts() {
        ensure!(
            room.scripts.contains_key(&label) && room.script_text_bodies.contains_key(&label),
            "Q07 dialogue refresh requires existing text {label}"
        );
        authoring::text(&mut room, &label, &lines);
    }
    // Text replacement cannot alter any event or control command. Rebuild only
    // the two text tables so original map-section formatting is retained too.
    room.script_text_commands = crate::parse_script_text_commands(&room.id, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(&room.id, &room.scripts)?;
    let mut pack = base.clone();
    pack.data.maps.insert(room.id.clone(), room);
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn validate_anchors(
    base: &CompiledGamePack,
    room: &MapModule,
    a: BoundaryBasecampStoryAnchors,
) -> Result<()> {
    ensure!(
        room.id == BOUNDARY_SURVEY_SHELTER
            && room.attributes.map_constant.as_deref() == Some(BOUNDARY_SURVEY_SHELTER_CONSTANT)
            && room.attributes.tileset_name == "lab"
            && [room.attributes.width, room.attributes.height] == [8, 6]
            && room.blocks.len() == 48,
        "Q07 requires the actual8×6 lab survey shelter"
    );
    let chiefs: Vec<_> = room
        .objects
        .iter()
        .filter(|o| {
            o.object_identifier.as_deref() == Some(BOUNDARY_SURVEY_CHIEF)
                || o.script == BOUNDARY_SURVEY_CHIEF
        })
        .collect();
    ensure!(chiefs.len() == 1, "Q07 needs one actual survey chief");
    let chief = chiefs[0];
    ensure!(
        chief.object_identifier.as_deref() == Some(BOUNDARY_SURVEY_CHIEF)
            && chief.script == BOUNDARY_SURVEY_CHIEF
            && chief.sprite == "SPRITE_ROCKET"
            && chief.object_type == "OBJECTTYPE_SCRIPT"
            && chief.event_flag == "-1"
            && chief.spritemovedata == "SPRITEMOVEDATA_STANDING_UP"
            && [chief.x, chief.y] == a.chief_tile,
        "Q07 chief object differs from authored contract"
    );
    ensure!(
        room.scripts.get(BOUNDARY_SURVEY_CHIEF)
            == Some(&json!([authoring::command(
                "jumptextfaceplayer",
                &["MN_SURVEY_CHIEFText"]
            )]))
            && room.scripts.get(BOUNDARY_SURVEY_RECEIVER)
                == Some(&json!([authoring::command(
                    "jumptext",
                    &["MN_BOUNDARY_SURVEY_RECEIVERText"]
                )])),
        "Q07 may replace only ordinary authored chief/receiver placeholders"
    );
    let receivers: Vec<_> = room
        .events
        .bg_events
        .iter()
        .filter(|e| e.script == BOUNDARY_SURVEY_RECEIVER)
        .collect();
    ensure!(
        receivers.len() == 1
            && [receivers[0].x, receivers[0].y] == a.receiver_tile
            && receivers[0].event_type == "BGEVENT_READ"
            && room
                .events
                .bg_events
                .iter()
                .filter(|event| [event.x, event.y] == a.receiver_tile)
                .count()
                == 1,
        "Q07 receiver needs one actual native read event"
    );
    let kind = |tile| {
        crate::authored_route_modpack::collision(
            base,
            &room.blocks,
            room.attributes.width,
            tile,
            "lab",
        )
    };
    ensure!(
        kind(a.chief_tile)? == "FLOOR"
            && kind(a.receiver_tile)? == "WALL"
            && room.blocks
                [usize::from(a.receiver_tile[1] / 2) * 8 + usize::from(a.receiver_tile[0] / 2)]
                == 0x08,
        "Q07 chief must stand on Floor beside an authentic native labPC08 receiver"
    );
    let occupied: BTreeSet<_> = room.objects.iter().map(|o| [o.x, o.y]).collect();
    ensure!(
        occupied.len() == room.objects.len(),
        "Q07 room actor bodies overlap"
    );
    ensure!(
        a.chief_approaches
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len()
            == 3,
        "Q07 needs three distinct authored chief approaches"
    );
    let walkable: BTreeSet<_> = (0..12)
        .flat_map(|y| (0..16).map(move |x| [x, y]))
        .filter(|tile| kind(*tile).is_ok_and(|k| k == "FLOOR") && !occupied.contains(tile))
        .collect();
    ensure!(
        walkable.contains(&a.clear_arrival),
        "Q07 shelter needs its actual unoccupied Floor arrival"
    );
    let mut reached = BTreeSet::from([a.clear_arrival]);
    let mut todo = VecDeque::from([a.clear_arrival]);
    while let Some([x, y]) = todo.pop_front() {
        for next in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if walkable.contains(&next) && reached.insert(next) {
                todo.push_back(next);
            }
        }
    }
    for approach in a.chief_approaches {
        ensure!(
            reached.contains(&approach)
                && approach[0].abs_diff(a.chief_tile[0]) + approach[1].abs_diff(a.chief_tile[1])
                    == 1,
            "Q07 chief approach must be real connected adjacent Floor"
        );
    }
    ensure!(
        reached.contains(&a.receiver_approach)
            && a.receiver_approach[0].abs_diff(a.receiver_tile[0])
                + a.receiver_approach[1].abs_diff(a.receiver_tile[1])
                == 1,
        "Q07 receiver approach must be real connected adjacent Floor"
    );
    Ok(())
}

fn allocate_events(pack: &mut CompiledGamePack) -> Result<()> {
    let globals = &mut pack.data.story_event_script_constants.global;
    for badge in BADGES {
        ensure!(
            globals.contains_key(badge),
            "missing native Johto badge {badge}"
        );
    }
    for name in [
        BOUNDARY_LIGHTHOUSE_SIGNAL_EVENT,
        BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT,
        BOUNDARY_SURVEY_RESTORED_EVENT,
    ] {
        if let Some(value) = globals.get(name) {
            ensure!(
                !globals
                    .iter()
                    .any(|(other, id)| other.starts_with("EVENT_") && other != name && id == value),
                "Q07 existing event numeric collision {name}"
            );
        } else {
            let next = globals
                .iter()
                .filter(|(k, _)| k.starts_with("EVENT_"))
                .map(|(_, v)| *v)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .context("Q07 event identifier overflow")?;
            globals.insert(name.into(), next);
        }
    }
    Ok(())
}

fn add_chief_party(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        !pack
            .data
            .trainers
            .trainers
            .contains_key(BOUNDARY_SURVEY_CHIEF_TRAINER)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(BOUNDARY_SURVEY_CHIEF_TRAINER),
        "Q07 chief trainer identity already exists"
    );
    let mut trainer = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == "EXECUTIVEM")
        .context("source EXECUTIVEM class")?
        .clone();
    let dvs = trainer.party.first().context("source chief-class DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == "EXECUTIVEM")
        .map(|t| {
            pack.data
                .story_event_script_constants
                .global
                .get(&t.trainer_id)
                .copied()
                .context("source executive numeric identity")
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("Q07 chief identifier overflow")?;
    trainer.trainer_id = BOUNDARY_SURVEY_CHIEF_TRAINER.into();
    trainer.name = "LODE CHIEF".into();
    trainer.party = [("MAGNETON", 44), ("WEEZING", 44), ("CROBAT", 46)]
        .into_iter()
        .map(|(species, level)| {
            let pokemon = pack.data.create_pokemon(species, level, dvs)?;
            Ok(TrainerPartyPokemon {
                species: species.into(),
                level,
                item: None,
                moves: pokemon.moves,
                dvs,
            })
        })
        .collect::<Result<_>>()?;
    trainer.win_quote.clear();
    trainer.lose_quote.clear();
    pack.data.trainers.insert(trainer)?;
    pack.data
        .story_event_script_constants
        .global
        .insert(BOUNDARY_SURVEY_CHIEF_TRAINER.into(), next);
    Ok(())
}

fn story_scripts() -> std::collections::BTreeMap<String, Value> {
    let mut chief = vec![
        authoring::command("faceplayer", &[]),
        authoring::command("checkevent", &[BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT]),
        authoring::command("iftrue", &[AFTER]),
    ];
    for badge in BADGES {
        chief.extend([
            authoring::command("checkflag", &[badge]),
            authoring::command("iffalse", &[BADGE_DENIED]),
        ]);
    }
    chief.extend([
        authoring::command("checkevent", &[BOUNDARY_LIGHTHOUSE_SIGNAL_EVENT]),
        authoring::command("iffalse", &[SIGNAL_DENIED]),
        authoring::command("opentext", &[]),
        authoring::command("writetext", &["MnBoundarySurveyChiefSeenText"]),
        authoring::command("waitbutton", &[]),
        authoring::command("closetext", &[]),
        authoring::command(
            "winlosstext",
            &[
                "MnBoundarySurveyChiefWonText",
                "MnBoundarySurveyChiefLostText",
            ],
        ),
        authoring::command(
            "loadtrainer",
            &["EXECUTIVEM", BOUNDARY_SURVEY_CHIEF_TRAINER],
        ),
        authoring::command("startbattle", &[]),
        authoring::command("reloadmapafterbattle", &[]),
        authoring::command("setevent", &[BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT]),
        authoring::command("sjump", &[AFTER]),
    ]);
    let mut scripts =
        std::collections::BTreeMap::from([(BOUNDARY_SURVEY_CHIEF.into(), Value::Array(chief))]);
    for (script, text) in [
        (AFTER, "MnBoundarySurveyChiefAfterText"),
        (BADGE_DENIED, "MnBoundarySurveyChiefBadgeDeniedText"),
        (SIGNAL_DENIED, "MnBoundarySurveyChiefSignalDeniedText"),
        (RECEIVER_LOCKED, "MnBoundarySurveyReceiverLockedText"),
        (RECEIVER_ALREADY, "MnBoundarySurveyReceiverAlreadyText"),
        (RECEIVER_DECLINED, "MnBoundarySurveyReceiverDeclinedText"),
    ] {
        scripts.insert(
            script.into(),
            json!([authoring::command(
                if script.starts_with("MnBoundarySurveyChief") {
                    "jumptextfaceplayer"
                } else {
                    "jumptext"
                },
                &[text]
            )]),
        );
    }
    scripts.insert(
        BOUNDARY_SURVEY_RECEIVER.into(),
        json!([
            authoring::command("checkevent", &[BOUNDARY_SURVEY_RESTORED_EVENT]),
            authoring::command("iftrue", &[RECEIVER_ALREADY]),
            authoring::command("checkevent", &[BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT]),
            authoring::command("iffalse", &[RECEIVER_LOCKED]),
            authoring::command("opentext", &[]),
            authoring::command("writetext", &["MnBoundarySurveyReceiverReadyText"]),
            authoring::command("yesorno", &[]),
            authoring::command("iffalse", &[RECEIVER_DECLINED]),
            authoring::command("writetext", &["MnBoundarySurveyReceiverRestoreText"]),
            authoring::command("waitbutton", &[]),
            authoring::command("setevent", &[BOUNDARY_SURVEY_RESTORED_EVENT]),
            authoring::command("writetext", &["MnBoundarySurveyReceiverDoneText"]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[]),
        ]),
    );
    scripts
}

fn story_texts() -> std::collections::BTreeMap<String, Vec<&'static str>> {
    [
        (
            "MnBoundarySurveyChiefSeenText",
            vec![
                "You kept the first",
                "survey readings.",
                "Ore under lakes:",
                "Copper and nickel.",
                "I read the risks.",
                "Drain them anyway.",
                "ICE takes anyone",
                "they don't trust.",
                "KETTLURK will",
                "drive the pumps.",
            ],
        ),
        (
            "MnBoundarySurveyChiefWonText",
            vec!["You have cost me", "enough already."],
        ),
        ("MnBoundarySurveyChiefLostText", vec!["The work goes on."]),
        (
            "MnBoundarySurveyChiefAfterText",
            vec![
                "I lost the relay.",
                "I have not changed",
                "my mind.",
                "The receiver is",
                "at the east desk.",
            ],
        ),
        (
            "MnBoundarySurveyChiefBadgeDeniedText",
            vec!["Eight BADGES first", "Then face my team."],
        ),
        (
            "MnBoundarySurveyChiefSignalDeniedText",
            vec![
                "The harbor signal",
                "has not reached us",
                "Check the workshop",
            ],
        ),
        (
            "MnBoundarySurveyReceiverLockedText",
            vec!["The survey chief", "holds this line.", "Win the desk first"],
        ),
        (
            "MnBoundarySurveyReceiverAlreadyText",
            vec![
                "SURVEY LINK ACTIVE",
                "Real readings sent",
                "Drainage stopped.",
            ],
        ),
        (
            "MnBoundarySurveyReceiverDeclinedText",
            vec!["Leave the receiver", "for another time."],
        ),
        (
            "MnBoundarySurveyReceiverReadyText",
            vec!["The desk is clear.", "Cut the pump link?"],
        ),
        (
            "MnBoundarySurveyReceiverRestoreText",
            vec!["You align the dial", "with the harbor."],
        ),
        (
            "MnBoundarySurveyReceiverDoneText",
            vec![
                "SURVEY LINK ACTIVE",
                "Real readings sent",
                "Crews stop the",
                "drainage pumps.",
                "KETTLURK escapes",
                "the waiting trap.",
                "The ELY trail is",
                "ready to survey.",
            ],
        ),
    ]
    .into_iter()
    .map(|(name, lines)| (name.into(), lines))
    .collect()
}

#[cfg(test)]
mod boundary_basecamp_story_component_tests {
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use crystal_core::battle::turn::BattleAction;
    use crystal_core::models::{Dv, LearnedMove};
    use crystal_core::random::ReplayDivider;
    use crystal_core::state::{GameState, OverworldMemory};
    use crystal_core::systems::script_control::ScriptControlAction;
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::movement::MovementMode;

    use super::*;
    use crate::{
        GameDataSet, RuntimeMutationCommand, RuntimePendingYesNoResolutionCommand, ScriptMapAction,
    };

    fn inputs() -> &'static (CompiledGamePack, MapModule, BoundaryBasecampStoryAnchors) {
        static INPUTS: OnceLock<(CompiledGamePack, MapModule, BoundaryBasecampStoryAnchors)> =
            OnceLock::new();
        INPUTS.get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("repository root");
            let source_pack = std::env::var_os("CRYSTAL_BOUNDARY_BASECAMP_BASE_PACK")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("content-packs/minnesota-region-review.crystalpack"))
                .canonicalize()
                .expect("completed regional source pack");
            let base = crate::read_verified_compiled_game_pack(source_pack)
                .expect("supplied original content pack");
            let source: Value = serde_json::from_str(
                &std::fs::read_to_string(
                    root.join("planning/minnesota/towns/MnBoundaryWaters.json"),
                )
                .expect("actual town source"),
            )
            .expect("town dossier");
            let room: crate::authored_route_design::Interior =
                serde_json::from_value(source["interiors"][0].clone())
                    .expect("Northern's actual shelter rows and anchors");
            let mut module = authoring::blank(
                &base,
                &room.template,
                &room.id,
                BOUNDARY_SURVEY_SHELTER_CONSTANT,
            )
            .expect("actual lab template");
            authoring::blocks(&mut module, 8, 6, &room.rows.join(" "))
                .expect("exact8×6 source rows");
            module.attributes.border_block = room.border_block.expect("actual source border");
            crate::authored_route_events::room_events(&base, &mut module, &room)
                .expect("native source residents/medic/sign placeholders");
            let anchors = BoundaryBasecampStoryAnchors {
                chief_tile: [9, 5],
                chief_approaches: [[8, 5], [10, 5], [9, 6]],
                receiver_tile: [12, 3],
                receiver_approach: [12, 4],
                clear_arrival: [8, 9],
            };
            (base, module, anchors)
        })
    }

    fn prepared_data() -> &'static GameDataSet {
        static DATA: OnceLock<GameDataSet> = OnceLock::new();
        DATA.get_or_init(|| {
            let (base, room, anchors) = inputs();
            let prepared = prepare_boundary_basecamp_story(base, room, *anchors)
                .expect("native Q07 preparation");
            let mut data = prepared.pack.data;
            // Component-only installation of the actual supplied room. The town
            // publisher owns full metadata, warps and final pack verification.
            data.maps
                .insert(prepared.shelter.id.clone(), prepared.shelter);
            data
        })
    }

    fn component_state() -> GameState {
        let mut state = GameState::reset_wram_for_new_game();
        state.money = 0;
        state.overworld = OverworldMemory::Active {
            map_name: BOUNDARY_SURVEY_SHELTER.into(),
            tile: TilePosition::new(8, 5),
            facing: Direction::Right,
            mode: MovementMode::Normal,
        };
        state
    }

    fn flag(state: &GameState, name: &str) -> bool {
        state
            .flags
            .is_event_flag_set(name)
            .expect("valid story event")
    }

    fn check(data: &GameDataSet, state: &mut GameState, root: &str, index: usize) {
        data.check_script_flag(
            state,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            root,
            index,
        )
        .expect("native checkflag/checkevent");
    }

    fn branch(
        data: &GameDataSet,
        state: &mut GameState,
        root: &str,
        index: usize,
    ) -> ScriptControlAction {
        data.apply_script_control_command(
            state,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            root,
            index,
        )
        .expect("native conditional")
    }

    #[test]
    fn boundary_basecamp_q07_preserves_source_room_and_exact_reserved_party() {
        let (base, room, a) = inputs();
        let prepared = prepare_boundary_basecamp_story(base, room, *a).expect("prepare Q07");
        assert_eq!(prepared.pack.data.maps, base.data.maps);
        assert_eq!(prepared.pack.runtime_files, base.runtime_files);
        assert_eq!(prepared.pack.data.tilesets, base.data.tilesets);
        assert_eq!(prepared.pack.data.items, base.data.items);
        assert_eq!(prepared.pack.data.marts, base.data.marts);
        assert_eq!(prepared.pack.data.pokemon, base.data.pokemon);
        assert_eq!(
            prepared.pack.data.runtime_spawn_points,
            base.data.runtime_spawn_points
        );
        assert_eq!(prepared.pack.compiled_audio, base.compiled_audio);
        assert_eq!(prepared.pack.audio_manifest, base.audio_manifest);
        assert_eq!(prepared.shelter.blocks, room.blocks);
        assert_eq!(prepared.shelter.objects, room.objects);
        assert_eq!(prepared.shelter.events, room.events);
        assert_eq!(prepared.shelter.attributes, room.attributes);
        for (id, trainer) in &base.data.trainers.trainers {
            assert_eq!(prepared.pack.data.trainers.trainers.get(id), Some(trainer));
        }
        for (name, value) in &base.data.story_event_script_constants.global {
            assert_eq!(
                prepared
                    .pack
                    .data
                    .story_event_script_constants
                    .global
                    .get(name),
                Some(value)
            );
        }
        let trainer = &prepared.pack.data.trainers.trainers[BOUNDARY_SURVEY_CHIEF_TRAINER];
        assert_eq!(trainer.trainer_class, "EXECUTIVEM");
        assert_eq!(
            trainer
                .party
                .iter()
                .map(|p| (p.species.as_str(), p.level))
                .collect::<Vec<_>>(),
            vec![("MAGNETON", 44), ("WEEZING", 44), ("CROBAT", 46)]
        );
        let battle = prepared
            .shelter
            .scripted_trainer_battles
            .iter()
            .find(|b| b.source_script == BOUNDARY_SURVEY_CHIEF)
            .expect("real native scripted boss descriptor");
        assert_eq!(
            (
                battle.loadtrainer_command_index,
                battle.startbattle_command_index
            ),
            (26, 27)
        );
        assert_eq!(battle.request.trainer_id, BOUNDARY_SURVEY_CHIEF_TRAINER);
        assert_eq!(battle.request.battle_type, "BATTLETYPE_TRAINER");
        let flags = &prepared.shelter.script_flag_commands;
        assert_eq!(
            flags
                .iter()
                .filter(|c| c.command == "setevent"
                    && c.flag_id == BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT)
                .map(|c| (c.source_script.as_str(), c.command_index))
                .collect::<Vec<_>>(),
            vec![(BOUNDARY_SURVEY_CHIEF, 29)]
        );
        assert_eq!(
            flags
                .iter()
                .filter(|c| c.command == "setevent" && c.flag_id == BOUNDARY_SURVEY_RESTORED_EVENT)
                .map(|c| (c.source_script.as_str(), c.command_index))
                .collect::<Vec<_>>(),
            vec![(BOUNDARY_SURVEY_RECEIVER, 10)]
        );
        assert!(
            !flags
                .iter()
                .any(|c| c.command == "setevent" && c.flag_id == BOUNDARY_LIGHTHOUSE_SIGNAL_EVENT)
        );
        assert!(prepared.shelter.script_economy_commands.is_empty());
        assert!(prepared.shelter.script_item_grants.is_empty());
        let story: Value =
            serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
                .expect("authored Minnesota story");
        let expected_texts = story["milestones"]
            .as_array()
            .expect("story milestones")
            .iter()
            .filter(|milestone| matches!(milestone["id"].as_str(), Some("MS11" | "MS12")))
            .flat_map(|milestone| {
                milestone["native_texts"]
                    .as_object()
                    .expect("authored Q07 dialogue")
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(expected_texts.len(), story_texts().len());
        for (label, expected) in expected_texts {
            let actual = prepared.shelter.scripts[label]
                .as_array()
                .expect("native text body")
                .iter()
                .filter(|c| matches!(c["command"].as_str(), Some("text" | "line" | "para")))
                .map(|c| {
                    c["args"][0]
                        .as_str()
                        .expect("text argument")
                        .trim_matches('"')
                })
                .collect::<Vec<_>>();
            assert!(actual.iter().all(|line| line.chars().count() <= 18));
            assert_eq!(json!(actual), *expected, "native story text {label}");
        }
        let medic = "MN_BOUNDARY_SURVEY_MEDIC";
        assert_eq!(
            prepared.shelter.scripts[medic], room.scripts[medic],
            "actual free HealParty must remain untouched"
        );
        assert_eq!(
            prepared.shelter.scripts["MN_BOUNDARY_SURVEY_ASSISTANT"],
            room.scripts["MN_BOUNDARY_SURVEY_ASSISTANT"]
        );
    }

    #[test]
    fn boundary_basecamp_q07_native_guards_require_every_badge_and_signal_without_fee() {
        let data = prepared_data();
        for missing in 0..8 {
            let mut state = component_state();
            for (i, name) in BADGES.into_iter().enumerate() {
                state
                    .flags
                    .set_engine_flag(name, i != missing)
                    .expect("explicit badge component setup");
            }
            state
                .flags
                .set_event_flag(BOUNDARY_LIGHTHOUSE_SIGNAL_EVENT, true)
                .expect("explicit upstream component setup");
            let before = state.clone();
            check(data, &mut state, BOUNDARY_SURVEY_CHIEF, 3 + missing * 2);
            assert!(
                matches!(branch(data,&mut state,BOUNDARY_SURVEY_CHIEF,4+missing*2),ScriptControlAction::Jump{target_script,..} if target_script==BADGE_DENIED)
            );
            assert!(!flag(&state, BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT));
            assert!(!flag(&state, BOUNDARY_SURVEY_RESTORED_EVENT));
            assert_eq!(state.money, before.money);
            assert_eq!(state.bag, before.bag);
        }
        let mut state = component_state();
        for name in BADGES {
            state
                .flags
                .set_engine_flag(name, true)
                .expect("badge component setup");
        }
        check(data, &mut state, BOUNDARY_SURVEY_CHIEF, 19);
        assert!(
            matches!(branch(data,&mut state,BOUNDARY_SURVEY_CHIEF,20),ScriptControlAction::Jump{target_script,..} if target_script==SIGNAL_DENIED)
        );
        state
            .flags
            .set_event_flag(BOUNDARY_LIGHTHOUSE_SIGNAL_EVENT, true)
            .expect("upstream component setup");
        check(data, &mut state, BOUNDARY_SURVEY_CHIEF, 19);
        assert!(matches!(
            branch(data, &mut state, BOUNDARY_SURVEY_CHIEF, 20),
            ScriptControlAction::Continue { .. }
        ));
        assert!(!flag(&state, BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT));
        assert!(!flag(&state, BOUNDARY_SURVEY_RESTORED_EVENT));
    }

    #[test]
    fn boundary_basecamp_q07_receiver_native_refusal_decline_and_repeat_are_separate() {
        let data = prepared_data();
        let mut state = component_state();
        check(data, &mut state, BOUNDARY_SURVEY_RECEIVER, 2);
        assert!(
            matches!(branch(data,&mut state,BOUNDARY_SURVEY_RECEIVER,3),ScriptControlAction::Jump{target_script,..} if target_script==RECEIVER_LOCKED)
        );
        assert!(!flag(&state, BOUNDARY_SURVEY_RESTORED_EVENT));
        // This is isolated console precondition setup, not an earned win claim.
        state
            .flags
            .set_event_flag(BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT, true)
            .expect("console component setup");
        let mut session = data
            .overworld_session(BOUNDARY_SURVEY_SHELTER, TilePosition::new(12, 4), 0)
            .expect("actual PC approach");
        let audio = data.audio_ids().into_iter().map(str::to_owned).collect();
        data.apply_script_text_command(
            &mut state,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_RECEIVER,
            6,
        )
        .expect("native yesorno prompt");
        data.apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::ResolvePendingYesNo(RuntimePendingYesNoResolutionCommand {
                accepted: false,
            }),
            &audio,
            &audio,
            &audio,
        )
        .expect("native decline choice");
        assert!(
            matches!(branch(data,&mut state,BOUNDARY_SURVEY_RECEIVER,7),ScriptControlAction::Jump{target_script,..} if target_script==RECEIVER_DECLINED)
        );
        assert!(!flag(&state, BOUNDARY_SURVEY_RESTORED_EVENT));
        data.apply_script_text_command(
            &mut state,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_RECEIVER,
            6,
        )
        .expect("native prompt retry");
        data.apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::ResolvePendingYesNo(RuntimePendingYesNoResolutionCommand {
                accepted: true,
            }),
            &audio,
            &audio,
            &audio,
        )
        .expect("native affirmative choice");
        assert!(matches!(
            branch(data, &mut state, BOUNDARY_SURVEY_RECEIVER, 7),
            ScriptControlAction::Continue { .. }
        ));
        data.apply_script_flag_mutation(
            &mut state,
            &mut session,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_RECEIVER,
            10,
        )
        .expect("native receiver acknowledgment event");
        check(data, &mut state, BOUNDARY_SURVEY_RECEIVER, 0);
        assert!(
            matches!(branch(data,&mut state,BOUNDARY_SURVEY_RECEIVER,1),ScriptControlAction::Jump{target_script,..} if target_script==RECEIVER_ALREADY)
        );
        assert_eq!(state.money, 0);
        assert!(state.bag.items.is_empty());
    }

    #[test]
    fn boundary_basecamp_q07_real_battle_turns_precede_victory_event_and_loss_whiteouts() {
        // Rewards resolve the actual town landmark, so this battle must use the
        // complete publisher rather than the command-only room fixture above.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let published = crate::build_boundary_basecamp_modpack(
            &inputs().0,
            &std::fs::read_to_string(root.join("planning/minnesota/towns/MnBoundaryWaters.json"))
                .expect("actual town source"),
            &std::fs::read_to_string(
                root.join("planning/minnesota/towns/MnBoundaryWaters.layout.json"),
            )
            .expect("actual town layout"),
        )
        .expect("complete native town, shelter and landmark registration");
        let data = published.data();
        let mut state = component_state();
        let mut partner = data
            .create_pokemon("MEW", 100, Dv::default())
            .expect("explicit legal component partner");
        partner.moves = vec![LearnedMove {
            name: "SURF".into(),
            current_pp: data.moves["SURF"].pp,
            pp_ups: 0,
        }];
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
        data.start_scripted_trainer_battle(
            &mut state,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_CHIEF,
            27,
        )
        .expect("actual reserved native chief party");
        assert!(!flag(&state, BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT));
        let mut divider = ReplayDivider::new((0_u32..65536).map(|n| ((n * 73 + 19) % 256) as u8));
        let initial_pp = state.storage.party.pokemon[0]
            .as_ref()
            .expect("partner")
            .moves[0]
            .current_pp;
        let mut defeated = false;
        for _ in 0..32 {
            let turn = data
                .resolve_active_battle_turn_with_divider(
                    &mut state,
                    BattleAction::Move { slot: 0 },
                    BattleAction::Move { slot: 0 },
                    &mut divider,
                )
                .expect("native selected Surf turn, no HP rewriting");
            assert!(!flag(&state, BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT));
            if turn.state.enemy.hp == 0 {
                data.claim_active_trainer_battle_rewards_now(&mut state)
                    .expect("native faint rewards");
                defeated = data
                    .advance_active_trainer_battle(&mut state)
                    .expect("native next opposing party member")
                    .trainer_defeated;
                if defeated {
                    break;
                }
            }
        }
        assert!(
            defeated,
            "all three real opponents must faint through battle turns"
        );
        assert!(
            state.storage.party.pokemon[0]
                .as_ref()
                .expect("partner")
                .moves[0]
                .current_pp
                < initial_pp
        );
        data.complete_scripted_trainer_battle(
            &mut state,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_CHIEF,
            27,
            true,
            false,
            &mut divider,
        )
        .expect("native terminal completion after actual victory");
        assert!(
            !flag(&state, BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT),
            "the separate post-reload source opcode owns this event"
        );
        let mut session = data
            .overworld_session(BOUNDARY_SURVEY_SHELTER, TilePosition::new(8, 5), 0)
            .expect("native chief approach");
        data.apply_script_flag_mutation(
            &mut state,
            &mut session,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_SHELTER,
            BOUNDARY_SURVEY_CHIEF,
            29,
        )
        .expect("native post-victory event");
        assert!(flag(&state, BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT));
        assert!(
            !flag(&state, BOUNDARY_SURVEY_RESTORED_EVENT),
            "victory must not restore the receiver"
        );
        check(data, &mut state, BOUNDARY_SURVEY_CHIEF, 1);
        assert!(
            matches!(branch(data,&mut state,BOUNDARY_SURVEY_CHIEF,2),ScriptControlAction::Jump{target_script,..} if target_script==AFTER)
        );
        // Separate explicit losing-terminal component state exercises the
        // original reloadmapafterbattle whiteout branch. This is a terminal-
        // state component check, not an actual played losing battle.
        let mut lost = component_state();
        lost.battle_result = 1;
        lost.script_runtime
            .memory
            .insert("wBattleScriptFlags".into(), "129".into());
        let action = data
            .apply_script_map_command_with_divider_in_session(
                &mut lost,
                &session,
                BOUNDARY_SURVEY_SHELTER,
                BOUNDARY_SURVEY_CHIEF,
                28,
                &mut divider,
            )
            .expect("native losing boss reload");
        assert!(matches!(action, ScriptMapAction::BattleWhiteout { .. }));
        assert!(!flag(&lost, BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT));
        assert!(!flag(&lost, BOUNDARY_SURVEY_RESTORED_EVENT));
    }

    #[test]
    fn boundary_basecamp_q07_rejects_real_anchor_and_namespace_defects_atomically() {
        let (base, room, a) = inputs();
        type RoomMutation = fn(&mut MapModule);
        let changes: [(&str, RoomMutation); 6] = [
            ("chief on cabinet", |m| {
                let o = m
                    .objects
                    .iter_mut()
                    .find(|o| o.script == BOUNDARY_SURVEY_CHIEF)
                    .expect("chief");
                o.x = 12;
                o.y = 3;
            }),
            ("buried receiver", |m| m.blocks[14] = 0x10),
            ("blocked receiver approach", |m| m.blocks[22] = 0x08),
            ("missing actual receiver", |m| {
                m.events
                    .bg_events
                    .retain(|e| e.script != BOUNDARY_SURVEY_RECEIVER)
            }),
            ("duplicate chief", |m| {
                let o = m
                    .objects
                    .iter()
                    .find(|o| o.script == BOUNDARY_SURVEY_CHIEF)
                    .expect("chief")
                    .clone();
                m.objects.push(o);
            }),
            ("foreign chief root", |m| {
                m.scripts.insert(
                    BOUNDARY_SURVEY_CHIEF.into(),
                    json!([authoring::command(
                        "setevent",
                        &[BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT]
                    )]),
                );
            }),
        ];
        for (label, change) in changes {
            let mut changed = room.clone();
            change(&mut changed);
            let before = changed.clone();
            assert!(
                prepare_boundary_basecamp_story(base, &changed, *a).is_err(),
                "accepted {label}"
            );
            assert_eq!(changed, before, "mutated caller room on failure {label}");
        }
        let mut collision = base.clone();
        let id = collision
            .data
            .story_event_script_constants
            .global
            .iter()
            .find(|(k, _)| k.starts_with("EVENT_"))
            .expect("source event")
            .1;
        let id = *id;
        collision
            .data
            .story_event_script_constants
            .global
            .insert(BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT.into(), id);
        assert!(
            allocate_events(&mut collision).is_err(),
            "an existing event alias must not be reused"
        );
        assert_eq!(base.data.maps.len(), inputs().0.data.maps.len());
    }
}
