//! Optional twin-falls discovery after the independent intake restoration.
use anyhow::{Context, Result, ensure};
use crystal_core::map::CoordEvent;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{self as arc, RELEASED, RESTORED, put};

const MAP: &str = "MnDevilsKettle";
const CONSTANT: &str = "MN_DEVILS_KETTLE";
const OUTSIDE: &str = "MnMaraisCedarApproach";
const ACTOR: &str = "MN_DEVILS_KETTLURK";
const CAUGHT: &str = "EVENT_MN_DEVILS_KETTLE_CAUGHT";
const MANIFEST: &str = "minnesota-devils-kettle-v1";

/// Adds an optional wooded falls branch and a voluntary native level-60 battle.
/// Only capture permanently removes Kettlurk. Refusal retains the encounter;
/// RUN and knockout withdraw it until physical reentry. Continue preserves that
/// visit's hidden state. Neither entry nor capture advances the main story.
/// The existing Grand Marais nurse also selects a registered passive clinic
/// recovery point, preserving her original care routine and all prior spawns.
///
/// # Errors
/// Rejects occupied identities, missing reviewed art, altered entrance geometry,
/// unsafe authored anchors, malformed source or invalid compiled output.
pub fn build_devils_kettle(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let site: Value = serde_json::from_str(source)?;
    ensure!(
        site["map"] == MAP && site["tileset"] == "mn_north_shore",
        "canonical falls site required"
    );
    ensure!(
        !base.data.maps.contains_key(MAP) && !base.report.manifests.iter().any(|v| v == MANIFEST),
        "falls already installed"
    );
    ensure!(
        base.data.pokemon.contains_key("KETTLURK")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/kettlurk/overworld.2bpp"),
        "reviewed Kettlurk required"
    );
    for event in [RELEASED, RESTORED] {
        ensure!(
            base.data
                .story_event_script_constants
                .global
                .contains_key(event),
            "missing restoration identity {event}"
        );
    }
    let mut pack = base.clone();
    a::flag(&mut pack, CAUGHT)?;
    let mut outside = arc::map(&pack, OUTSIDE)?;
    ensure!(
        outside.attributes.width == 20
            && outside.attributes.height == 12
            && outside.blocks[4 * 20 + 6] == 1,
        "cedar entrance terrain changed"
    );
    for at in [[12, 8], [13, 8], [14, 9]] {
        ensure!(
            !outside.objects.iter().any(|v| [v.x, v.y] == at)
                && !outside.events.warps.iter().any(|v| [v.x, v.y] == at),
            "occupied trail entrance"
        );
    }
    ensure!(
        crate::authored_town_geometry::kind(&pack, &outside, [14, 9])? == "FLOOR",
        "passive cedar return must be dry floor"
    );
    let outside_constant = outside
        .attributes
        .map_constant
        .clone()
        .context("cedar constant")?;
    let return_id = i16::try_from(outside.events.warps.len() + 3)?;
    outside.blocks[4 * 20 + 6] = 0x82;
    authoring::warp(&mut outside, (12, 8), CONSTANT, 3);
    authoring::warp(&mut outside, (13, 8), CONSTANT, 3);
    authoring::warp(&mut outside, (14, 9), &outside_constant, return_id);
    for (x, y) in [(12, 8), (13, 8)] {
        outside.events.coord_events.push(CoordEvent {
            x,
            y,
            scene_id: "-1".into(),
            script_name: "MnKettleTrailEntry".into(),
        });
    }
    put(
        &mut outside,
        "MnKettleTrailEntry",
        vec![c("warp", &[CONSTANT, "30", "32"]), c("end", &[])],
    );

    let mut room = authoring::blank(&pack, OUTSIDE, MAP, CONSTANT)?;
    let rows = site["rows"].as_array().context("falls rows")?;
    let mut blocks = Vec::new();
    ensure!(rows.len() == 18, "falls height");
    for row in rows {
        let tokens = row
            .as_str()
            .context("terrain row")?
            .split_whitespace()
            .collect::<Vec<_>>();
        ensure!(tokens.len() == 20, "falls width");
        for token in tokens {
            blocks.push(
                site["legend"][token]
                    .as_str()
                    .context("native block token")?,
            );
        }
    }
    authoring::blocks(&mut room, 20, 18, &blocks.join(" "))?;
    room.attributes.border_block = 5;
    for at in [[30, 32], [28, 32], [20, 12], [21, 12], [6, 8]] {
        ensure!(
            crate::authored_town_geometry::kind(&pack, &room, at)? == "FLOOR",
            "unsafe falls anchor {at:?}"
        );
    }
    for y in 0..36 {
        for x in 0..40 {
            if x == 0 || x == 39 || y == 0 || y == 35 {
                ensure!(
                    crate::authored_town_geometry::kind(&pack, &room, [x, y])? == "WALL",
                    "open falls perimeter"
                );
            }
        }
    }
    authoring::warp(&mut room, (28, 32), &outside_constant, return_id);
    authoring::warp(&mut room, (28, 33), &outside_constant, return_id);
    authoring::warp(&mut room, (30, 32), CONSTANT, 3);
    for (x, y) in [(28, 32), (28, 33)] {
        room.events.coord_events.push(CoordEvent {
            x,
            y,
            scene_id: "-1".into(),
            script_name: "MnKettleTrailExit".into(),
        });
    }
    put(
        &mut room,
        "MnKettleTrailExit",
        vec![c("warp", &[&outside_constant, "14", "9"]), c("end", &[])],
    );
    a::actor(&pack, &mut room, ACTOR, "SPRITE_KETTLURK", [20, 12])?;
    // The default visible object is gated without resurrecting a saved retreat.
    for (label, appear) in [("MnKettleObjects", false), ("MnKettleNewVisit", true)] {
        let mut commands = vec![
            c("checkevent", &[CAUGHT]),
            c("iftrue", &["MnKettleHide"]),
            c("checkevent", &[RELEASED]),
            c("iffalse", &["MnKettleHide"]),
            c("checkevent", &[RESTORED]),
            c("iffalse", &["MnKettleHide"]),
        ];
        if appear {
            commands.push(c("appear", &[ACTOR]));
        }
        commands.push(c("endcallback", &[]));
        put(&mut room, label, commands);
    }
    put(
        &mut room,
        "MnKettleHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    a::callback(&mut room, "MAPCALLBACK_OBJECTS", "MnKettleObjects");
    a::callback(&mut room, "MAPCALLBACK_NEWMAP", "MnKettleNewVisit");
    put(
        &mut room,
        ACTOR,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[RELEASED]),
            c("iffalse", &["MnKettleAbsent"]),
            c("checkevent", &[RESTORED]),
            c("iffalse", &["MnKettleAbsent"]),
            c("checkevent", &[CAUGHT]),
            c("iftrue", &["MnKettleAbsent"]),
            c("opentext", &[]),
            c("writetext", &["MnKettleAskText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnKettleDeclined"]),
            c("closetext", &[]),
            c("cry", &["KETTLURK"]),
            c(
                "loadvar",
                &[
                    "VAR_BATTLETYPE",
                    crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE,
                ],
            ),
            c("loadwildmon", &["KETTLURK", "60"]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("special", &["CheckCaughtCelebi"]),
            c("iftrue", &["MnKettleCaught"]),
            c("disappear", &[ACTOR]),
            c("jumptext", &["MnKettleRetryText"]),
        ],
    );
    put(
        &mut room,
        "MnKettleCaught",
        vec![
            c("setevent", &[CAUGHT]),
            c("disappear", &[ACTOR]),
            c("jumptext", &["MnKettleCaughtText"]),
        ],
    );
    put(
        &mut room,
        "MnKettleDeclined",
        vec![c("closetext", &[]), c("end", &[])],
    );
    put(
        &mut room,
        "MnKettleAbsent",
        vec![c("jumptext", &["MnKettleAbsentText"])],
    );
    for (label, key) in [
        ("MnKettleAskText", "encounter_text"),
        ("MnKettleRetryText", "retry_text"),
        ("MnKettleCaughtText", "caught_text"),
        ("MnKettleAbsentText", "before_restoration_text"),
    ] {
        authoring::text(
            &mut room,
            label,
            &[site[key].as_str().context("falls dialogue")?],
        );
    }
    authoring::speech(
        &pack,
        &mut room,
        "Ranger",
        "SPRITE_FISHER",
        (30, 25),
        &[
            "The falls trail forks beside the pool. The short grass crossing saves a climb.",
            "A hollow current returned after the intake was sealed. Watch the rock between the falls.",
        ],
    )?;
    authoring::text(
        &mut room,
        "MnKettleRangerBeforeText",
        &[
            "The falls trail forks beside the pool. The short grass crossing saves a climb.",
            "The hollow current has been quiet. Something is missing from the falls.",
        ],
    );
    put(
        &mut room,
        "MnDevilsKettleRanger",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[RESTORED]),
            c("iffalse", &["MnKettleRangerBefore"]),
            c("jumptext", &["MnDevilsKettleRangerText"]),
        ],
    );
    put(
        &mut room,
        "MnKettleRangerBefore",
        vec![c("jumptext", &["MnKettleRangerBeforeText"])],
    );
    room.scripted_wild_battles = crate::parse_scripted_wild_battles(MAP, &room.scripts)?;
    a::publish(&mut pack, room, OUTSIDE, [30, 32])?;
    arc::publish(&mut pack, outside)?;
    let mut habitat = base
        .data
        .wild_encounters
        .get(OUTSIDE)
        .context("native cedar habitat")?
        .clone();
    habitat.map_name = MAP.into();
    habitat.zones.clear();
    habitat.swarm_overrides.clear();
    pack.data.wild_encounters.insert(MAP.into(), habitat);
    crate::devils_kettle_clinic::wire(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
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
