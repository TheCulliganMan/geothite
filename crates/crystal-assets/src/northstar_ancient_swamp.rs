//! Optional Vergloom track survey and a native, retryable legendary encounter.
use anyhow::{Context, Result, ensure};
use crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE;
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;

const MANIFEST: &str = "northstar-ancient-swamp-v1";
const ACTOR: &str = "MN_ANCIENT_MIRE_VERGLOOM";
const READY: &str = "EVENT_MN_ANCIENT_MIRE_TRACKS_READ";
const CAUGHT: &str = "EVENT_MN_ANCIENT_MIRE_VERGLOOM_CAUGHT";
const CLUES: [&str; 3] = [
    "EVENT_MN_ANCIENT_MIRE_TIMBER",
    "EVENT_MN_ANCIENT_MIRE_FOOTPRINT",
    "EVENT_MN_ANCIENT_MIRE_MOSS",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    id: String,
    map: String,
    creature: [u16; 2],
    clues: [[u16; 2]; 3],
    level: u8,
}

/// Adds three persistent track inspections and an optional Vergloom encounter.
///
/// Consumes the compact canonical geography with the ancient-mire terrain and
/// native Vergloom art installed. Three acknowledged observations reveal the
/// creature. Refusal, fleeing and knockout do not consume it permanently;
/// actual capture does. The route, other legendary encounters and main-story
/// flags are preserved. Apply the regional cast binder before final publication.
///
/// # Errors
/// Rejects incompatible geometry, retired/missing Vergloom, missing native art,
/// changed anchors, repeated installation or failed runtime verification.
pub fn build_northstar_ancient_swamp(
    base: &CompiledGamePack,
    source: &str,
    roster: &str,
    curation: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let source: Source = serde_json::from_str(source)?;
    ensure!(
        source.id == MANIFEST && source.map == "MnR08C00R00" && source.level == 40,
        "reviewed optional ancient-mire encounter required"
    );
    ensure!(
        source.creature == [59, 23] && source.clues == [[47, 33], [55, 29], [60, 25]],
        "reviewed ancient-mire anchors differ"
    );
    let roster: Value = serde_json::from_str(roster)?;
    let curation: Value = serde_json::from_str(curation)?;
    let family = roster["families"]
        .as_array()
        .context("Minnesota families")?
        .iter()
        .find(|f| f["id"] == "MN_F048")
        .context("Vergloom family")?;
    ensure!(
        family["rarity"] == "legendary"
            && family["members"]
                .as_array()
                .context("members")?
                .iter()
                .any(|m| m["id"] == "MN_099" && m["name"] == "Vergloom"),
        "native Minnesota Vergloom identity required"
    );
    ensure!(
        !curation["retired_families"]
            .as_array()
            .context("curation")?
            .iter()
            .any(|f| f["id"] == "MN_F048"),
        "Vergloom is retired from the active roster"
    );
    ensure!(
        base.data.pokemon.contains_key("VERGLOOM")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/vergloom/overworld.2bpp"),
        "install the active Vergloom species and reviewed native overworld art first"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "ancient-mire encounter already installed"
    );
    let mut pack = base.clone();
    for flag in CLUES.into_iter().chain([READY, CAUGHT]) {
        a::flag(&mut pack, flag)?;
    }
    let mut room = arc::map(&pack, &source.map)?;
    ensure!(
        (room.attributes.width, room.attributes.height) == (32, 24),
        "reviewed R08 chunk required"
    );
    let tileset = &pack.data.tilesets[&room.attributes.tileset_name];
    for [x, y] in [[59, 23], [59, 24], [48, 33], [56, 29], [59, 25]] {
        let block = room.blocks
            [usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2)];
        let collision = &tileset.collision[&format!("{block:02x}")]
            [usize::from(y % 2) * 2 + usize::from(x % 2)];
        ensure!(
            matches!(collision.as_str(), "FLOOR" | "TALL_GRASS"),
            "ancient-mire dry anchor changed at {x},{y}"
        );
        ensure!(
            !room.objects.iter().any(|o| o.x == x && o.y == y),
            "occupied ancient-mire approach at {x},{y}"
        );
    }
    for [x, y] in source.clues {
        let block = room.blocks
            [usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2)];
        ensure!(
            tileset.collision[&format!("{block:02x}")][usize::from(y % 2) * 2 + usize::from(x % 2)]
                == "WALL",
            "visible clue marker must not invoke a field move at {x},{y}"
        );
    }
    a::actor(&pack, &mut room, ACTOR, "SPRITE_VERGLOOM", source.creature)?;
    // Native appear/disappear also mutate an object's bound event flag. Keep
    // this actor temporary; only the actual-capture branch may set CAUGHT.
    let texts = [
        (
            "MnAncientTimberText",
            vec![
                "Petrified timber lies beside the old cedar roots. Fresh moss is caught beneath it.",
                "The moss was pulled toward the peat pools. Something broad passed between the roots.",
            ],
        ),
        (
            "MnAncientFootprintText",
            vec![
                "One deep footprint crosses the peat ribs without breaking their edges.",
                "The pools beside it are still clear. Something heavy knew where to step.",
            ],
        ),
        (
            "MnAncientMossText",
            vec![
                "Long moss strands cling to the stone. The scrape is higher than your head.",
                "The three trails meet beyond this pool.",
            ],
        ),
        (
            "MnAncientRevealedText",
            vec![
                "The branches across the pool shift against the wind.",
                "A broad figure rises beneath its hanging moss. VERGLOOM watches the raised trail.",
            ],
        ),
        (
            "MnAncientOfferText",
            vec![
                "VERGLOOM waits without crossing the peat pools.",
                "Approach the forest creature?",
            ],
        ),
        (
            "MnAncientDeclinedText",
            vec!["You step back onto the raised trail. VERGLOOM stays by the pool."],
        ),
        (
            "MnAncientRetreatText",
            vec![
                "The moss settles into the reeds. Fresh tracks remain beside the pool.",
                "You can follow them again after leaving the mire.",
            ],
        ),
        (
            "MnAncientCaughtText",
            vec![
                "VERGLOOM's trail ends here. The old peat ribs and their pools remain undisturbed.",
            ],
        ),
    ];
    for (label, lines) in texts {
        authoring::text(&mut room, label, &lines);
    }
    for (i, (flag, at)) in CLUES.iter().zip(source.clues).enumerate() {
        let label = format!("MnAncientClue{i}");
        let text = [
            "MnAncientTimberText",
            "MnAncientFootprintText",
            "MnAncientMossText",
        ][i];
        ensure!(
            !room.events.bg_events.iter().any(|e| [e.x, e.y] == at),
            "occupied clue station"
        );
        let mut rows = a::say(text);
        rows.extend([
            c("setevent", &[flag]),
            c("sjump", &["MnAncientCheckTracks"]),
        ]);
        arc::put(&mut room, &label, rows);
        a::read(&mut room, at, &label);
    }
    let mut check = vec![c("checkevent", &[READY]), c("iftrue", &["MnAncientEnd"])];
    for flag in CLUES {
        check.extend([c("checkevent", &[flag]), c("iffalse", &["MnAncientEnd"])]);
    }
    check.extend([c("cry", &["VERGLOOM"]), c("appear", &[ACTOR])]);
    check.extend(a::say("MnAncientRevealedText"));
    check.extend([c("setevent", &[READY]), c("end", &[])]);
    arc::put(&mut room, "MnAncientCheckTracks", check);
    arc::put(&mut room, "MnAncientEnd", vec![c("end", &[])]);
    arc::put(
        &mut room,
        ACTOR,
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MnAncientOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnAncientDecline"]),
            c("closetext", &[]),
            c("cry", &["VERGLOOM"]),
            // Normal mode has no caught bit; Celebi mode prevents escape.
            // This opt-in mode keeps RUN and records actual capture storage.
            c(
                "loadvar",
                &["VAR_BATTLETYPE", OPTIONAL_LEGENDARY_BATTLE_TYPE],
            ),
            c("loadwildmon", &["VERGLOOM", &source.level.to_string()]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("special", &["CheckCaughtCelebi"]),
            c("iftrue", &["MnAncientCaptured"]),
            c("disappear", &[ACTOR]),
            c("jumptext", &["MnAncientRetreatText"]),
        ],
    );
    arc::put(
        &mut room,
        "MnAncientCaptured",
        vec![
            c("setevent", &[CAUGHT]),
            c("disappear", &[ACTOR]),
            c("jumptext", &["MnAncientCaughtText"]),
        ],
    );
    arc::put(
        &mut room,
        "MnAncientDecline",
        vec![
            c("writetext", &["MnAncientDeclinedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    arc::put(
        &mut room,
        "MnAncientRestore",
        vec![
            c("checkevent", &[CAUGHT]),
            c("iftrue", &["MnAncientHide"]),
            c("checkevent", &[READY]),
            c("iffalse", &["MnAncientHide"]),
            c("appear", &[ACTOR]),
            c("endcallback", &[]),
        ],
    );
    arc::put(
        &mut room,
        "MnAncientHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    a::callback(&mut room, "MAPCALLBACK_OBJECTS", "MnAncientRestore");
    room.scripted_wild_battles = crate::parse_scripted_wild_battles(&room.id, &room.scripts)?;
    arc::publish(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
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

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires the completed native ancient-mire art pack"]
    fn ancient_mire_events_preserve_terrain_audio_and_other_encounters() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = crate::read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-ancient-mire-art-v02.crystalpack"),
        )
        .expect("native terrain/art parent");
        let read = |name: &str| std::fs::read_to_string(root.join(name)).expect("authored source");
        let source = read("planning/minnesota/revisions/ancient-swamp-v01/encounter.json");
        let roster = read("planning/minnesota/pokemon/roster.json");
        let curation = read("planning/minnesota/pokemon/roster-curation.json");
        let result = super::build_northstar_ancient_swamp(&base, &source, &roster, &curation)
            .expect("native event overlay");
        assert!(base.compiled_audio() == result.compiled_audio());
        assert!(base.audio_manifest().expect("audio") == result.audio_manifest().expect("audio"));
        assert_eq!(base.runtime_files(), result.runtime_files());
        assert_eq!(base.data.pokemon, result.data.pokemon);
        assert_eq!(base.data.trainers, result.data.trainers);
        assert_eq!(base.data.wild_encounters, result.data.wild_encounters);
        assert_eq!(base.data.field_encounters, result.data.field_encounters);
        assert_eq!(
            base.data.runtime_spawn_points,
            result.data.runtime_spawn_points
        );
        for (name, map) in &base.data.maps {
            let after = &result.data.maps[name];
            if name == "MnR08C00R00" {
                assert_eq!(map.blocks, after.blocks);
                assert_eq!(map.attributes, after.attributes);
                assert_eq!(map.events.warps, after.events.warps);
                assert_eq!(map.events.coord_events, after.events.coord_events);
                for object in &map.objects {
                    assert!(after.objects.contains(object));
                }
                assert_eq!(
                    after.objects.last().expect("Vergloom").event_flag,
                    "-1",
                    "hiding an unencountered creature must not set its capture flag"
                );
                let battle = after
                    .scripted_wild_battles
                    .iter()
                    .find(|b| b.source_script == super::ACTOR)
                    .expect("executable static-wild boundary");
                assert_eq!(battle.request.species, "VERGLOOM");
                assert_eq!(battle.request.level, 40);
                assert_eq!(
                    battle.request.battle_type,
                    super::OPTIONAL_LEGENDARY_BATTLE_TYPE
                );
            } else {
                assert_eq!(map, after, "unrelated map {name}");
            }
        }
        assert!(
            super::build_northstar_ancient_swamp(&result, &source, &roster, &curation).is_err()
        );
        let mut retired: serde_json::Value = serde_json::from_str(&curation).expect("curation");
        retired["retired_families"]
            .as_array_mut()
            .expect("families")
            .push(serde_json::json!({"id":"MN_F048"}));
        assert!(
            super::build_northstar_ancient_swamp(&base, &source, &roster, &retired.to_string())
                .is_err()
        );
    }
}
