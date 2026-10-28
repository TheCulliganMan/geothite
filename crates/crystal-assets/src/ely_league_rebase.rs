//! Exact Ely inputs and additive preservation for the connected-state lineage.
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::ely_league_rooms::MAPS;
use crate::ely_league_scripts::{ACTIVE, CHAMPION, Member};
use crate::{CompiledGamePack, MapModule};

pub(super) const CONNECTED_PARENT: &str =
    "11a5f3b39ffba6252fb2246af17433ed70b6b4d722d0b3a67394a1526845b03a";
const TERRAIN_PARENT: &str = "53ae7d31d3f73eab9341b2be2b0bdc6ba782a13ea56584162b8ca32d66c050f6";
const WATER_REPAIRED_PARENT: &str =
    "174f1942c42bd590e292e5a95340ffb9d93758bf0ab1ec31d83dadfd9fc9c722";
// The completed winter/interior publication retains all eleven module and eight
// tileset preimages below. Its added residents and furniture use fresh events;
// certify its actual allocation boundary rather than reusing the old event IDs.
const INTERIOR_PARENT: &str = "846584fc49e42f436f3aef4e77eea913b12a61c44e1582528858086b9cc1460c";
// v5 moves two read-only Duluth details from floor cells onto their bookcases.
// The Ely module/tileset preimages and allocation boundary are unchanged.
const INTERIOR_FURNITURE_PARENT: &str =
    "4722d2d85aa6387e56429b608c5f75f3558925dbf90f8275551004acbb33da8f";
// Canonical story text is refreshed before the v4 local stories/v6 room pass.
// All eleven module/eight tileset preimages and the event boundary stay exact.
const INTERIOR_STORY_PARENT: &str =
    "42e325856c412b4713898db5ec89822cd7fb50eb079a369ce04ddda0bc3ee0c0";
// This published coast parent retains all template/art preimages. Its existing
// event catalog ends at 2384; allocate after that actual boundary, not after an
// older interior lineage's 2391 boundary.
const WARM_COAST_PARENT: &str = "5009f2898563ecf699d7036b19d988ef5f4d43a8b39cbd48f8616cc9c45eca32";
const ROAD: &str = "MnR13C00R02";
const GROUP: &str = "MAPGROUP_MINNESOTA_ELY";
const LANDMARK: &str = "LANDMARK_MN_LEAGUE_APPROACH";
const TRAINERS: [&str; 5] = [
    "MN_LEAGUE_1",
    "MN_LEAGUE_2",
    "MN_LEAGUE_3",
    "MN_LEAGUE_4",
    "MN_CHAMPION",
];
const TEMPLATES: [(&str, &str); 11] = [
    (
        ROAD,
        "db1df63b6e16737f9538657427e7a88300962a85b9ed604df1f3a669128ac8dc",
    ),
    (
        "Route23",
        "8555ad874d23e1ff6d372924c1e79a4a24ebdf605955c2344656fc1ea0be8623",
    ),
    (
        "GoldenrodPokecenter1F",
        "f303eee737460eacdc1ee76f7054ffc37fb28555751dd32bd9ee1bfa2d54a232",
    ),
    (
        "CherrygroveMart",
        "85ec2ff192fe83a4ccdd67935b80a58dbbb28042d2958935ecf0545758d6c8cc",
    ),
    (
        "IlexForestAzaleaGate",
        "7a4a75354f412bcf7d36417b1660112d9f499b87ac8b7f031764fd34957e8feb",
    ),
    (
        "KogasRoom",
        "077b2ea45fe652bda0d85d98f29d9cfd4ccef67c12cfb56103ce703ac090df7a",
    ),
    (
        "BrunosRoom",
        "4cc2b169c233d2e29aba341ca0d4c38a1d77205687526f9b361f0af2e0f1cc26",
    ),
    (
        "WillsRoom",
        "391fd57cffe55f86f24034fe9d5822a9b9a790891a54a948a750512a3bdc382f",
    ),
    (
        "KarensRoom",
        "92971a6f60fe91432412603ecc6d599acf7061a92e24cfaaf289348f6541c9a5",
    ),
    (
        "LancesRoom",
        "3f8ed98449361bb83f46fe8fe6acfdb09ce88c75524be8dbfb6c5f6f845d2d14",
    ),
    (
        "HallOfFame",
        "10aa8287b621c9500e80f2d228c21327d6fcf816a15ba4f6fce91d18fb48afdf",
    ),
];
const TILESETS: [(&str, &str); 8] = [
    (
        "champions_room",
        "f0827aa9c9c347751ccb900010feeec1ef5b90fc6f004b496f4c3040757f1972",
    ),
    (
        "elite_four_room",
        "9f5bbf4dc8b8be5b8c43d43bf49461f3c480009a62fb2a5e1e4a92604bb4301a",
    ),
    (
        "gate",
        "45c956470106320be6b39c1975785c95d7a4d9900dab614351ccf3bbb62b24ec",
    ),
    (
        "ice_path",
        "3ff648ff4a4d65f7a00c322b76da7cd5ac19cda4636e01acb338cb03fc99a10c",
    ),
    (
        "johto",
        "d49be2437d396e1a5f69905a3297254b30d6b73d04bbba49ec4e39383dcc1c40",
    ),
    (
        "kanto",
        "985a118f297b3b88484a23b779053bad567e4ad604c87b278bdf9c295ceafaa0",
    ),
    (
        "mart",
        "aca2d04b7a96918b668cebd94e107636fdd39de034181e8fee7a047bc53f5d49",
    ),
    (
        "pokecenter",
        "3107b35ede1c58f0caa537542538c9f5f14a48ab6989a3b965d99fa59f033362",
    ),
];

fn hash(value: &impl Serialize) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(value)?)?)
    ))
}
fn bytes_hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn names() -> impl Iterator<Item = &'static str> {
    std::iter::once("MnEly").chain(MAPS.iter().map(|(name, _, _)| *name))
}

pub(super) fn parent(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        matches!(
            base.identity.content_hash.as_str(),
            super::BASE
                | CONNECTED_PARENT
                | TERRAIN_PARENT
                | WATER_REPAIRED_PARENT
                | INTERIOR_PARENT
                | INTERIOR_FURNITURE_PARENT
                | INTERIOR_STORY_PARENT
                | WARM_COAST_PARENT
        ),
        "exact completed North Shore or certified connected-state Ely parent required"
    );
    if matches!(
        base.identity.content_hash.as_str(),
        CONNECTED_PARENT | TERRAIN_PARENT | WATER_REPAIRED_PARENT
    ) {
        ensure!(
            base.data.maps.len() == 681
                && base.data.pokemon.len() == 251
                && base.data.moves.len() == 251
                && base
                    .data
                    .runtime_spawn_points
                    .values()
                    .map(|s| s.identifier)
                    .max()
                    == Some(277)
                && next_event(base)? == 2382,
            "connected-state Ely allocation preimage changed"
        );
    }
    if matches!(
        base.identity.content_hash.as_str(),
        INTERIOR_PARENT | INTERIOR_FURNITURE_PARENT | INTERIOR_STORY_PARENT
    ) {
        ensure!(
            base.data.maps.len() == 683
                && base.data.pokemon.len() == 251
                && base.data.moves.len() == 251
                && base
                    .data
                    .runtime_spawn_points
                    .values()
                    .map(|s| s.identifier)
                    .max()
                    == Some(277)
                && next_event(base)? == 2392,
            "winter/interior Ely allocation preimage changed"
        );
    }
    if base.identity.content_hash == WARM_COAST_PARENT {
        ensure!(
            base.data.maps.len() == 683
                && base.data.pokemon.len() == 251
                && base.data.moves.len() == 251
                && base
                    .data
                    .runtime_spawn_points
                    .values()
                    .map(|s| s.identifier)
                    .max()
                    == Some(277)
                && next_event(base)? == 2385,
            "warm-coast Ely allocation preimage changed"
        );
    }
    preimages(base)
}

fn preimages(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        !base
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == 81)
            && !base
                .data
                .story_event_script_constants
                .global
                .contains_key(GROUP),
        "Ely group81 already occupied"
    );
    ensure!(
        !base.data.marts.0.contains_key("MART_MN_ELY"),
        "Ely mart already occupied"
    );
    for name in names() {
        ensure!(
            !base.data.maps.contains_key(name),
            "Ely map already occupied: {name}"
        );
    }
    for name in [ACTIVE, CHAMPION].into_iter().chain(TRAINERS) {
        ensure!(
            !base
                .data
                .story_event_script_constants
                .global
                .contains_key(name)
                && !base.data.trainers.trainers.contains_key(name),
            "Ely event/trainer identity already occupied: {name}"
        );
    }
    ensure!(
        base.data
            .story_event_script_constants
            .global
            .contains_key("EVENT_MN_SURVEY_RESTORED"),
        "Ely requires the existing restored-survey event"
    );
    for (name, expected) in TEMPLATES {
        ensure!(
            hash(
                base.data
                    .maps
                    .get(name)
                    .with_context(|| format!("Ely native template {name}"))?
            )? == expected,
            "Ely native module preimage differs: {name}"
        );
    }
    for (name, expected) in TILESETS {
        ensure!(
            hash(
                base.data
                    .tilesets
                    .get(name)
                    .with_context(|| format!("Ely native tileset {name}"))?
            )? == expected,
            "Ely native tileset preimage differs: {name}"
        );
    }
    Ok(())
}

pub(super) fn sources(
    base: &CompiledGamePack,
    town: &str,
    layout: &str,
    story: &Value,
) -> Result<()> {
    ensure!(
        bytes_hash(town) == "aca04118e1b4206de2a2c504e9862e8cee0a8d61f801971f6c57ee3b4165e912"
            && bytes_hash(layout)
                == "42916dee1f0daa6b899c6fe79891ab973a185437a71bf9ac4c85dd101dc1e784"
            && hash(&story["league"])?
                == "9fe88edb2abada391a04d6c612bf474a2768932f54339c0e039bd43b1ed71c1b",
        "complete reviewed Ely town/layout/five-member League source required"
    );
    for member in story["league"].as_array().context("five League members")? {
        for pokemon in member["party"].as_array().context("League party")? {
            let species = pokemon["species"].as_str().context("League species")?;
            ensure!(
                base.data.pokemon.contains_key(species),
                "Ely League species missing: {species}"
            );
        }
    }
    Ok(())
}

fn next_event(base: &CompiledGamePack) -> Result<i64> {
    base.data
        .story_event_script_constants
        .global
        .iter()
        .filter(|(k, _)| k.starts_with("EVENT_"))
        .map(|(_, v)| *v)
        .max()
        .context("native event table")?
        .checked_add(1)
        .context("Ely event overflow")
}

pub(super) fn validate(
    base: &CompiledGamePack,
    pack: &CompiledGamePack,
    members: &[Member],
) -> Result<()> {
    ensure!(
        pack.data.maps.len() == base.data.maps.len() + 10,
        "all ten Ely maps required"
    );
    let mut expected_road = base.data.maps[ROAD].clone();
    crate::lake_country_geometry::connect(&mut expected_road, "south", "MnEly", 5)?;
    ensure!(
        pack.data.maps[ROAD] == expected_road,
        "old R13 changed beyond its exact Ely seam"
    );
    let mut uncached = pack.data.clone();
    for name in names() {
        let map = &pack.data.maps[name];
        uncached
            .maps
            .remove(name)
            .context("remove new Ely cache for native parser")?;
        let mut rebuilt = uncached.assemble_map_module_from_compiled_payloads(name)?;
        for label in [
            &map.attributes.map_scripts_label,
            &map.attributes.map_events_label,
        ] {
            rebuilt
                .scripts
                .remove(label.as_ref().context("Ely transport label")?);
        }
        ensure!(
            &rebuilt == map,
            "Ely uncached module differs: {name}: {}",
            differing_fields(map, &rebuilt)?
        );
        uncached.maps.insert(name.into(), map.clone());
        let landmark = pack.data.pokegear_landmark_for_map(name)?;
        ensure!(
            landmark.constant == LANDMARK
                && landmark.id == 100
                && map.attributes.location.as_deref() == Some(LANDMARK),
            "Ely geographic registration differs: {name}"
        );
    }
    drop(uncached);
    let mut normalized = pack.data.clone();
    for name in names() {
        let map = normalized.maps.remove(name).context("new Ely module")?;
        normalized.map_attributes.remove(name);
        let constant = map
            .attributes
            .map_constant
            .as_ref()
            .context("Ely map constant")?;
        normalized.map_dimensions.remove(constant);
        normalized.runtime_map_metadata.remove(constant);
        normalized.map_blocks.remove(
            map.attributes
                .blocks_label
                .as_ref()
                .context("Ely block label")?,
        );
        normalized.npcs.remove(name);
        normalized.pokegear_landmarks.map_to_landmark.remove(name);
        for label in map
            .scripts
            .keys()
            .chain(map.script_text_bodies.keys())
            .chain(map.attributes.map_scripts_label.iter())
            .chain(map.attributes.map_events_label.iter())
        {
            ensure!(
                !base.data.map_scripts.contains_key(label),
                "Ely replaced an old global label: {label}"
            );
            normalized.map_scripts.remove(label);
        }
    }
    normalized
        .maps
        .insert(ROAD.into(), base.data.maps[ROAD].clone());
    normalized
        .map_attributes
        .insert(ROAD.into(), base.data.map_attributes[ROAD].clone());
    let first = next_event(base)?;
    for (name, value) in [(ACTIVE, first), (CHAMPION, first + 1)] {
        ensure!(
            pack.data.story_event_script_constants.global.get(name) == Some(&value),
            "Ely fresh event allocation differs: {name}"
        );
        normalized.story_event_script_constants.global.remove(name);
    }
    ensure!(
        normalized.story_event_script_constants.global.remove(GROUP) == Some(81),
        "Ely native group registration"
    );
    for member in members {
        normalized
            .story_event_script_constants
            .global
            .remove(&member.id);
        normalized.trainers.trainers.remove(&member.id);
    }
    normalized.marts.0.remove("MART_MN_ELY");
    let first_spawn = base
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("original recovery table")?
        .checked_add(1)
        .context("Ely recovery overflow")?
        .max(256);
    for (name, id, position) in [
        ("MnEly", first_spawn, [24, 4]),
        ("MnElyPokecenter1F", first_spawn + 1, [3, 5]),
    ] {
        let spawn = pack
            .data
            .runtime_spawn_points
            .values()
            .find(|s| s.map_name == name)
            .context("new Ely recovery")?;
        ensure!(
            spawn.identifier == id && [spawn.tile_x, spawn.tile_y] == position,
            "fresh Ely recovery differs"
        );
        normalized.runtime_spawn_points.remove(&id.to_string());
    }
    let scoped = normalized
        .story_event_script_constants
        .maps
        .remove("MnLeagueHallOfFame")
        .context("Ely post-credits declaration")?;
    ensure!(
        scoped.len() == 1
            && scoped.get("GEOTHITE_POST_CREDITS_SPAWN") == Some(&i64::from(first_spawn + 1)),
        "Ely post-credits recovery must use actual fresh clinic"
    );
    ensure!(
        serde_json::to_value(&normalized)? == serde_json::to_value(&base.data)?,
        "Ely changed unrelated original maps/programs/catalogs/recovery/constants"
    );
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_manifest == base.audio_manifest
            && pack.audio_compression == base.audio_compression,
        "Ely changed original art or audio"
    );
    let mut expected_report = base.report.clone();
    expected_report.maps += 10;
    for (index, name) in names().enumerate() {
        expected_report
            .manifests
            .push(format!("{}-{index}", super::MANIFEST));
        if !expected_report.reachable_maps.iter().any(|s| s == name) {
            expected_report.reachable_maps.push(name.into());
        }
        if !expected_report.solvable_maps.iter().any(|s| s == name) {
            expected_report.solvable_maps.push(name.into());
        }
    }
    expected_report.manifests.push(super::MANIFEST.into());
    ensure!(
        pack.report == expected_report,
        "Ely changed unrelated compile report"
    );
    Ok(())
}

fn differing_fields(expected: &MapModule, actual: &MapModule) -> Result<String> {
    let a = serde_json::to_value(expected)?;
    let b = serde_json::to_value(actual)?;
    Ok(a.as_object()
        .context("Ely expected module")?
        .iter()
        .filter(|(key, value)| b[*key] != **value)
        .map(|(key, _)| key.as_str())
        .collect::<Vec<_>>()
        .join(", "))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use super::{ACTIVE, preimages};
    use crate::{CompiledGamePack, read_verified_compiled_game_pack};

    fn base() -> &'static CompiledGamePack {
        static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
        BASE.get_or_init(|| {
            let path = std::env::var_os("CRYSTAL_ELY_COHESION_BASE")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../content-packs/minnesota-state-cohesion-v01.crystalpack")
                });
            read_verified_compiled_game_pack(path.canonicalize().expect("external Ely parent"))
                .expect("verified external Ely parent")
        })
    }

    #[test]
    #[ignore = "requires the external connected-state parent; tests precise guards without stale-identity shortcuts"]
    fn ely_rebase_rejects_native_template_group_flag_and_mart_collisions() {
        preimages(base()).expect("exact native Ely inputs");
        for name in ["MnR13C00R02", "IlexForestAzaleaGate", "HallOfFame"] {
            let mut bad = base().clone();
            bad.data.maps.get_mut(name).expect("template").blocks[0] ^= 1;
            assert!(
                preimages(&bad)
                    .expect_err("native template drift")
                    .to_string()
                    .contains("module preimage")
            );
        }
        let mut bad = base().clone();
        bad.data
            .runtime_map_metadata
            .get_mut("MN_ST_PAUL")
            .expect("existing city")
            .group_id = 81;
        assert!(
            preimages(&bad)
                .expect_err("occupied map group")
                .to_string()
                .contains("group81")
        );
        let mut bad = base().clone();
        bad.data
            .story_event_script_constants
            .global
            .insert(ACTIVE.into(), 2310);
        assert!(
            preimages(&bad)
                .expect_err("old numeric League flag cannot overwrite a current club win")
                .to_string()
                .contains("identity already occupied")
        );
        let mut bad = base().clone();
        bad.data
            .marts
            .0
            .insert("MART_MN_ELY".into(), vec!["POTION".into()]);
        assert!(
            preimages(&bad)
                .expect_err("existing mart")
                .to_string()
                .contains("mart already occupied")
        );
        let mut bad = base().clone();
        let collision = &mut bad
            .data
            .tilesets
            .get_mut("gate")
            .expect("original art")
            .collision
            .get_mut("00")
            .expect("native block")[0];
        *collision = if collision == "FLOOR" {
            "WALL"
        } else {
            "FLOOR"
        }
        .into();
        assert!(
            preimages(&bad)
                .expect_err("native collision drift")
                .to_string()
                .contains("tileset preimage")
        );
    }
}
