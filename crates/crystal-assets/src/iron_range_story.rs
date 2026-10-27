//! Q06's actual mine confrontation and separate, persisted instrument recovery.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerPartyPokemon;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MINE: &str = "MnR09Cave1F";
const FOREMAN: &str = "MN_R09_DEPOT_FOREMAN";
const TRAINER: &str = "MN_LODE_FOREMAN";
const CACHE: &str = "MN_R09_SURVEY_CACHE";
const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";
const DEFEATED: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const RECOVERED: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
const PARTY: [(&str, u8); 3] = [("GRAVELER", 35), ("MACHOKE", 35), ("MAGNETON", 36)];

/// Adds Q06 to the installed, authored Iron Range mine without replacing terrain.
///
/// The foreman requires the seventh native badge and completed headwaters report.
/// An ordinary scripted trainer win records only his defeat. A separate cache
/// interaction offers the meter; declining is retryable, and acknowledging the
/// recovery text records the quest payload as a saved event. It does not invent
/// a bag item or complete the later Grand Rapids handover.
///
/// Geometry, warps, existing actors/scripts, other maps, original catalogs, audio,
/// HOME and recovery destinations are retained. Inputs remain unchanged on error.
/// Reapplying this overlay is rejected rather than duplicating actors or rewards.
///
/// # Errors
/// Returns an error for an absent or changed native mine, conflicting identities,
/// blocked source anchors, incompatible story prerequisites/party, malformed
/// dialogue, missing original assets, or native parser/pack validation failures.
pub fn build_iron_range_story_modpack(
    base: &CompiledGamePack,
    dossier_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let dossier: Value = serde_json::from_str(dossier_json).context("Q06 route source")?;
    let story: Value = serde_json::from_str(story_json).context("Q06 story source")?;
    ensure!(
        dossier["id"] == "MN_R09",
        "Q06 requires the Iron Range dossier"
    );
    let rooms = dossier["interiors"]
        .as_array()
        .context("Q06 source rooms")?;
    let rooms = rooms.iter().filter(|r| r["id"] == MINE).collect::<Vec<_>>();
    ensure!(rooms.len() == 1, "Q06 needs one authored survey mine");
    let source = rooms[0];
    let mut room = base
        .data
        .maps
        .get(MINE)
        .context("installed Q06 mine")?
        .clone();
    let (foreman, cache, texts) = validate_source(base, &room, source, &story)?;
    let mut pack = base.clone();
    allocate_events(&mut pack)?;
    add_trainer(&mut pack)?;
    for (id, tile, sprite) in [
        (FOREMAN, foreman, "SPRITE_ROCKET"),
        (CACHE, cache, "SPRITE_POKE_BALL"),
    ] {
        authoring::npc(
            &pack,
            &mut room,
            id,
            sprite,
            i16::try_from(tile[0])?,
            i16::try_from(tile[1])?,
        )?;
        let object = room.objects.last_mut().context("Q06 inserted object")?;
        object.object_identifier = Some(id.into());
        object.object_type = "OBJECTTYPE_SCRIPT".into();
        object.radius = 0;
        object.sprite_has_facings = sprite != "SPRITE_POKE_BALL";
    }
    let scripts = scripts();
    for label in scripts.keys().chain(texts.keys()) {
        ensure!(
            !room.scripts.contains_key(label),
            "Q06 script collision {label}"
        );
    }
    room.scripts.extend(scripts);
    for (label, lines) in texts {
        authoring::text(
            &mut room,
            &label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    authoring::finish(&pack, &mut room)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(MINE, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(MINE, &room.scripts)?;
    room.script_variable_commands = crate::parse_script_variable_commands(MINE, &room.scripts)?;
    room.trainer_scripts = crate::parse_trainer_scripts(MINE, &room.scripts)?;
    room.scripted_trainer_battles = crate::parse_scripted_trainer_battles(MINE, &room.scripts)?;
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect();
    room.script_map_commands = crate::parse_script_map_commands(MINE, &room.scripts, &names)?;
    pack.data.maps.insert(MINE.into(), room);
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

fn tile(value: &Value) -> Result<[u16; 2]> {
    serde_json::from_value(value.clone()).context("Q06 source walking tile")
}

fn milestone<'a>(story: &'a Value, id: &str) -> Result<&'a Value> {
    let found = story["milestones"]
        .as_array()
        .context("Q06 milestones")?
        .iter()
        .filter(|m| m["id"] == id)
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "Q06 needs unique milestone {id}");
    Ok(found[0])
}

fn strings(value: &Value) -> Result<BTreeSet<String>> {
    let values: Vec<String> = serde_json::from_value(value.clone()).context("Q06 conditions")?;
    let unique = values.iter().cloned().collect::<BTreeSet<_>>();
    ensure!(unique.len() == values.len(), "Q06 duplicate condition");
    Ok(unique)
}

pub(crate) type Dialogue = BTreeMap<String, Vec<String>>;

pub(crate) fn validate_source(
    pack: &CompiledGamePack,
    room: &MapModule,
    source: &Value,
    story: &Value,
) -> Result<([u16; 2], [u16; 2], Dialogue)> {
    ensure!(
        room.attributes.tileset_name == "cave",
        "Q06 needs native cave art"
    );
    let dims: [u16; 2] = serde_json::from_value(source["dimensions_metatiles"].clone())?;
    ensure!(
        dims == [room.attributes.width, room.attributes.height],
        "Q06 mine extent changed"
    );
    let rows: Vec<String> = serde_json::from_value(source["rows"].clone())?;
    ensure!(
        rows.len() == usize::from(dims[1])
            && rows
                .iter()
                .all(|r| r.split_whitespace().count() == usize::from(dims[0])),
        "Q06 malformed mine rows"
    );
    let blocks = rows
        .iter()
        .flat_map(|r| r.split_whitespace())
        .map(|v| u16::from_str_radix(v, 16))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        blocks == room.blocks,
        "Q06 source/native mine terrain differs"
    );
    let trainers = source["story_trainers"]
        .as_array()
        .context("Q06 foreman reservation")?;
    let objects = source["story_objects"]
        .as_array()
        .context("Q06 cache reservation")?;
    ensure!(
        trainers.len() == 1 && objects.len() == 1,
        "Q06 needs one foreman and cache"
    );
    let foreman = &trainers[0];
    let cache = &objects[0];
    ensure!(
        foreman["id"] == FOREMAN
            && foreman["battle"] == TRAINER
            && foreman["sprite"] == "SPRITE_ROCKET"
            && foreman["trainer_class"] == "GRUNTM"
            && foreman["facing"] == "south"
            && foreman["interaction"] == "talk"
            && foreman["sight_range"] == 0
            && foreman["defeated_flag"] == DEFEATED,
        "Q06 foreman identity differs from native contract"
    );
    ensure!(
        cache["id"] == CACHE
            && cache["sprite"] == "SPRITE_POKE_BALL"
            && cache["facing"] == "south"
            && cache["recovered_flag"] == RECOVERED
            && cache["payload_storage"] == "saved_event"
            && strings(&cache["requires"])? == BTreeSet::from([DEFEATED.into()]),
        "Q06 cache must use the reserved saved quest event"
    );
    let party = json!(PARTY.map(|(species, level)| json!({"species": species, "level": level})));
    let bosses = story["bosses"].as_array().context("Q06 story bosses")?;
    let bosses = bosses
        .iter()
        .filter(|b| b["id"] == TRAINER)
        .collect::<Vec<_>>();
    ensure!(
        bosses.len() == 1
            && bosses[0]["party"] == party
            && bosses[0]["win_flag"] == DEFEATED
            && foreman["party"] == party,
        "Q06 source party differs from the reserved story boss"
    );
    let win = milestone(story, "MS08")?;
    let recovery = milestone(story, "MS08_CACHE")?;
    let handover = milestone(story, "MS09")?;
    ensure!(
        win["location"] == MINE
            && strings(&win["requires"])? == BTreeSet::from(["BADGE_7".into(), REPORT.into()])
            && strings(&win["sets"])? == BTreeSet::from([DEFEATED.into()])
            && recovery["location"] == MINE
            && strings(&recovery["requires"])? == BTreeSet::from([DEFEATED.into()])
            && strings(&recovery["sets"])? == BTreeSet::from([RECOVERED.into()])
            && strings(&handover["requires"])?
                == BTreeSet::from([DEFEATED.into(), RECOVERED.into()])
            && strings(&handover["sets"])? == BTreeSet::from(["EVENT_MN_DEPOT_CLEARED".into()]),
        "Q06 milestones must separate victory, recovery and depot handover"
    );
    ensure!(
        [REPORT, DEFEATED, RECOVERED]
            .iter()
            .all(|name| story["flags"]
                .as_array()
                .is_some_and(|flags| flags.iter().any(|flag| flag == name))),
        "Q06 event is absent from the story catalog"
    );
    let mut texts = Dialogue::new();
    for m in [win, recovery] {
        let dialogue: Dialogue =
            serde_json::from_value(m["native_texts"].clone()).context("Q06 native dialogue")?;
        for (label, lines) in dialogue {
            ensure!(
                !lines.is_empty()
                    && lines
                        .iter()
                        .all(|line| !line.is_empty() && line.chars().count() <= 18),
                "Q06 text must fit18 columns: {label}"
            );
            ensure!(
                texts.insert(label.clone(), lines).is_none(),
                "Q06 duplicate text {label}"
            );
        }
    }
    let expected = [
        "ForemanSeen",
        "ForemanWon",
        "ForemanLost",
        "ForemanAfter",
        "ForemanBadgeDenied",
        "ForemanReportDenied",
        "CacheLocked",
        "CacheReady",
        "CacheDeclined",
        "CacheRecovered",
        "CacheAlready",
    ]
    .map(|suffix| format!("MnIronRange{suffix}Text"))
    .into_iter()
    .collect::<BTreeSet<_>>();
    ensure!(
        texts.keys().cloned().collect::<BTreeSet<_>>() == expected,
        "Q06 native text labels differ from supported scenes"
    );
    for (authored, label) in [
        (&foreman["before_lines"], "MnIronRangeForemanSeenText"),
        (&foreman["after_lines"], "MnIronRangeForemanAfterText"),
        (&foreman["loss_lines"], "MnIronRangeForemanLostText"),
        (&cache["denied_lines"], "MnIronRangeCacheLockedText"),
        (&cache["success_lines"], "MnIronRangeCacheRecoveredText"),
    ] {
        ensure!(
            *authored == json!(texts[label]),
            "Q06 route/story text differs: {label}"
        );
    }
    let foreman_tile = tile(&foreman["tile"])?;
    let cache_tile = tile(&cache["tile"])?;
    let occupied = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        occupied.len() == room.objects.len()
            && foreman_tile != cache_tile
            && !occupied.contains(&foreman_tile)
            && !occupied.contains(&cache_tile),
        "Q06 story actors overlap native bodies"
    );
    ensure!(
        !room
            .objects
            .iter()
            .any(|o| [FOREMAN, CACHE].contains(&o.script.as_str())
                || o.object_identifier
                    .as_deref()
                    .is_some_and(|id| [FOREMAN, CACHE].contains(&id))),
        "Q06 story is already installed"
    );
    let kind = |p: [u16; 2]| {
        crate::authored_route_modpack::collision(
            pack,
            &room.blocks,
            room.attributes.width,
            p,
            "cave",
        )
    };
    for p in [foreman_tile, cache_tile] {
        ensure!(
            p[0] < dims[0] * 2 && p[1] < dims[1] * 2 && kind(p)? == "FLOOR",
            "Q06 story actor must stand on native Floor"
        );
        ensure!(
            !room.events.warps.iter().any(|w| [w.x, w.y] == p),
            "Q06 actor overlaps a warp"
        );
    }
    let mut blocked = occupied;
    blocked.extend([foreman_tile, cache_tile]);
    let arrival = tile(&source["clear_arrival"])?;
    let floor = (0..dims[1] * 2)
        .flat_map(|y| (0..dims[0] * 2).map(move |x| [x, y]))
        .filter(|p| kind(*p).is_ok_and(|k| matches!(k, "FLOOR" | "LADDER")) && !blocked.contains(p))
        .collect::<BTreeSet<_>>();
    ensure!(
        floor.contains(&arrival)
            && kind(arrival)? == "FLOOR"
            && !room.events.warps.iter().any(|w| [w.x, w.y] == arrival),
        "Q06 arrival is blocked or active"
    );
    let mut reached = BTreeSet::from([arrival]);
    let mut queue = VecDeque::from([arrival]);
    while let Some([x, y]) = queue.pop_front() {
        for p in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if floor.contains(&p) && reached.insert(p) {
                queue.push_back(p);
            }
        }
    }
    for (actor, p) in [(foreman, foreman_tile), (cache, cache_tile)] {
        let approach = tile(&actor["approach"])?;
        ensure!(
            reached.contains(&approach)
                && p[0].abs_diff(approach[0]) + p[1].abs_diff(approach[1]) == 1,
            "Q06 interaction approach is not connected adjacent Floor"
        );
    }
    for actor in &room.objects {
        let [x, y] = [actor.x, actor.y];
        ensure!(
            [
                x.checked_sub(1).map(|x| [x, y]),
                x.checked_add(1).map(|x| [x, y]),
                y.checked_sub(1).map(|y| [x, y]),
                y.checked_add(1).map(|y| [x, y])
            ]
            .into_iter()
            .flatten()
            .any(|p| reached.contains(&p)),
            "Q06 actors block an existing mine resident"
        );
    }
    ensure!(
        room.events
            .warps
            .iter()
            .all(|w| reached.contains(&[w.x, w.y])),
        "Q06 actors block a native exit"
    );
    Ok((foreman_tile, cache_tile, texts))
}

fn allocate_events(pack: &mut CompiledGamePack) -> Result<()> {
    let globals = &mut pack.data.story_event_script_constants.global;
    ensure!(
        globals.contains_key(BADGE_ENGINE_FLAGS[6]),
        "Q06 seventh badge is absent"
    );
    for name in [REPORT, DEFEATED, RECOVERED] {
        if let Some(value) = globals.get(name) {
            ensure!(
                !globals
                    .iter()
                    .any(|(other, id)| other.starts_with("EVENT_") && other != name && id == value),
                "Q06 existing event numeric collision {name}"
            );
        } else {
            let next = globals
                .iter()
                .filter(|(key, _)| key.starts_with("EVENT_"))
                .map(|(_, id)| *id)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .context("Q06 event namespace overflow")?;
            globals.insert(name.into(), next);
        }
    }
    Ok(())
}

pub(crate) fn add_trainer(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        !pack.data.trainers.trainers.contains_key(TRAINER)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(TRAINER),
        "Q06 trainer identity already exists"
    );
    let mut trainer = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == "GRUNTM")
        .context("original male grunt trainer class")?
        .clone();
    let dvs = trainer.party.first().context("original grunt DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == "GRUNTM")
        .map(|t| {
            pack.data
                .story_event_script_constants
                .global
                .get(&t.trainer_id)
                .copied()
                .context("source grunt numeric identity")
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("Q06 trainer namespace overflow")?;
    trainer.trainer_id = TRAINER.into();
    trainer.name = "LODE FOREMAN".into();
    trainer.party = PARTY
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
        .insert(TRAINER.into(), next);
    Ok(())
}

pub(crate) fn scripts() -> BTreeMap<String, Value> {
    let mut scripts = BTreeMap::from([
        (
            FOREMAN.into(),
            json!([
                authoring::command("faceplayer", &[]),
                authoring::command("checkevent", &[DEFEATED]),
                authoring::command("iftrue", &["MnIronRangeForemanAfter"]),
                authoring::command("checkflag", &[BADGE_ENGINE_FLAGS[6]]),
                authoring::command("iffalse", &["MnIronRangeForemanBadgeDenied"]),
                authoring::command("checkevent", &[REPORT]),
                authoring::command("iffalse", &["MnIronRangeForemanReportDenied"]),
                authoring::command("opentext", &[]),
                authoring::command("writetext", &["MnIronRangeForemanSeenText"]),
                authoring::command("waitbutton", &[]),
                authoring::command("closetext", &[]),
                authoring::command(
                    "winlosstext",
                    &["MnIronRangeForemanWonText", "MnIronRangeForemanLostText"]
                ),
                authoring::command("loadtrainer", &["GRUNTM", TRAINER]),
                authoring::command("startbattle", &[]),
                authoring::command("reloadmapafterbattle", &[]),
                authoring::command("setevent", &[DEFEATED]),
                authoring::command("sjump", &["MnIronRangeForemanAfter"])
            ]),
        ),
        (
            CACHE.into(),
            json!([
                authoring::command("faceplayer", &[]),
                authoring::command("checkevent", &[RECOVERED]),
                authoring::command("iftrue", &["MnIronRangeCacheAlready"]),
                authoring::command("checkevent", &[DEFEATED]),
                authoring::command("iffalse", &["MnIronRangeCacheLocked"]),
                authoring::command("opentext", &[]),
                authoring::command("writetext", &["MnIronRangeCacheReadyText"]),
                authoring::command("yesorno", &[]),
                authoring::command("iffalse", &["MnIronRangeCacheDeclined"]),
                authoring::command("writetext", &["MnIronRangeCacheRecoveredText"]),
                authoring::command("waitbutton", &[]),
                authoring::command("setevent", &[RECOVERED]),
                authoring::command("closetext", &[]),
                authoring::command("end", &[])
            ]),
        ),
    ]);
    for suffix in [
        "ForemanAfter",
        "ForemanBadgeDenied",
        "ForemanReportDenied",
        "CacheAlready",
        "CacheLocked",
    ] {
        scripts.insert(
            format!("MnIronRange{suffix}"),
            json!([authoring::command(
                "jumptextfaceplayer",
                &[&format!("MnIronRange{suffix}Text")]
            )]),
        );
    }
    scripts.insert(
        "MnIronRangeCacheDeclined".into(),
        json!([
            authoring::command("writetext", &["MnIronRangeCacheDeclinedText"]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[]),
        ]),
    );
    scripts
}
