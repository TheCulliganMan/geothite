//! Snowy Ely, its short wooded coastal pass and the public Split Rock clinic.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_town_publication::{self, CityIdentity};
use crate::ely_league_rooms;
use crate::grand_marais_modpack::authoring;

const ELY: &str = "MnEly";
const SHORE: &str = "MnR11C00R02";
const PASS: &str = "MnElyShorePass";
const CLINIC: &str = "MnSplitRockPokecenter1F";
const CLINIC_CONSTANT: &str = "MN_SPLIT_ROCK_POKECENTER_1F";
const SNOW: &str = "mn_ely_snow";
const MANIFEST: &str = "minnesota-ely-shore-pass-v1";

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    id: String,
    base_content_hash: String,
    module_preimages: BTreeMap<String, String>,
    patches: BTreeMap<String, Vec<Patch>>,
    pass: Pass,
    connections: Vec<Connection>,
    clinic: Clinic,
}
#[derive(Deserialize)]
struct Patch {
    position: [usize; 2],
    before: u16,
    after: u16,
}
#[derive(Deserialize)]
struct Pass {
    id: String,
    constant: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
}
#[derive(Deserialize)]
struct Clinic {
    entrance: [u16; 2],
    #[serde(rename = "return")]
    return_tile: [u16; 2],
}
#[derive(Deserialize)]
struct Connection {
    map: String,
    direction: String,
    target: String,
    offset: i32,
}

/// Publishes the reviewed public back route and clinic over the compact coast.
///
/// Keeps all existing story programs, parties, doors, recovery and audio. Ely's
/// private alias retains native buildings and imports exact snowy pines. New maps
/// receive fresh native identities; the clinic uses the production nurse/PC.
/// The caller binds the reviewed contract hash, publishes to a fresh ignored
/// path and verifies strict readback. This does not certify earned progression.
///
/// # Errors
/// Rejects a stale parent or contract, occupied identities, changed preimages,
/// invalid patches, disconnected seams or any mutation of unrelated content.
pub fn build_ely_shore_pass_pack(
    base: &CompiledGamePack,
    contract_json: &str,
    expected_contract_sha256: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == expected_contract_sha256,
        "reviewed contract hash differs"
    );
    let source: Contract = serde_json::from_str(contract_json)?;
    ensure!(
        source.schema_version == 1
            && source.id == "MN_ELY_SHORE_PASS_V1"
            && source.base_content_hash
                == "f2ac93f14af05a2935947e1696385a26a16b2d2bd509e5da3e9223af02f98d63"
            && base.identity.content_hash == source.base_content_hash,
        "compact coast parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        source.module_preimages.len() == 2 && source.patches.len() == 2,
        "two owned exteriors required"
    );
    for name in [ELY, SHORE] {
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(
                &base.data.maps[name]
            )?)?)
                == source.module_preimages[name],
            "stale exterior preimage: {name}"
        );
    }
    ensure!(
        !base.data.maps.contains_key(PASS)
            && !base.data.maps.contains_key(CLINIC)
            && !base.data.tilesets.contains_key(SNOW),
        "back route already installed"
    );
    let mut pack = crate::ely_snow_art::append(base)?;
    let mut exteriors = BTreeMap::new();
    for name in [ELY, SHORE] {
        let mut map = base.data.maps[name].clone();
        for patch in &source.patches[name] {
            let [x, y] = patch.position;
            ensure!(
                x < usize::from(map.attributes.width) && y < usize::from(map.attributes.height),
                "patch outside map"
            );
            if name == SHORE {
                let definition = &base.data.tilesets[&map.attributes.tileset_name];
                let key = format!("{:02x}", patch.before);
                ensure!(
                    definition.collision[&key]
                        .iter()
                        .all(|kind| matches!(kind.as_str(), "FLOOR" | "TALL_GRASS" | "WALL")),
                    "cannot replace native water, cliffs, ice or field gates"
                );
            }
            let cell = &mut map.blocks[y * usize::from(map.attributes.width) + x];
            ensure!(*cell == patch.before, "patch preimage differs");
            *cell = patch.after;
        }
        if name == ELY {
            map.attributes.tileset_name = SNOW.into();
            map.attributes.border_block = 0x83;
        }
        exteriors.insert(name.to_string(), map);
    }
    ensure!(
        source.pass.id == PASS
            && source.pass.constant == "MN_ELY_SHORE_PASS"
            && source.pass.tileset == "mn_winter_north_shore"
            && source.pass.dimensions_metatiles == [7, 12],
        "exact gap-fitted pass required"
    );
    let mut pass = authoring::blank(&pack, "MnR11C00R00", PASS, &source.pass.constant)?;
    pass.attributes.tileset_name = source.pass.tileset.clone();
    pass.attributes.border_block = 0x8b;
    authoring::blocks(&mut pass, 7, 12, &source.pass.rows.join(" "))?;
    authoring::speech(
        &pack,
        &mut pass,
        "TrailGuide",
        "SPRITE_FISHER",
        (8, 14),
        &[
            "ELY lies west. SPLIT ROCK and its POKEMON CENTER are east.",
            "This back trail saves the long walk south. Snow stays inland.",
        ],
    )?;
    authoring::finish(&pack, &mut pass)?;
    exteriors.insert(PASS.into(), pass);
    ensure!(
        source.connections.len() == 4,
        "two reciprocal seams required"
    );
    for (link, (from, direction, to, offset)) in source.connections.iter().zip([
        (ELY, "east", PASS, 0),
        (PASS, "west", ELY, 0),
        (PASS, "east", SHORE, -10),
        (SHORE, "west", PASS, 10),
    ]) {
        ensure!(
            link.map == from
                && link.direction == direction
                && link.target == to
                && link.offset == offset,
            "pass geometry changed"
        );
        let map = exteriors.get_mut(from).context("owned exterior")?;
        ensure!(
            !map.attributes
                .connections
                .iter()
                .any(|c| c.direction == direction),
            "occupied seam"
        );
        map.attributes.connections.push(MapConnection {
            direction: direction.into(),
            target_map: to.into(),
            offset,
        });
        map.attributes.connection_flags = Some(
            map.attributes
                .connections
                .iter()
                .map(|c| c.direction.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    let shore = exteriors.get_mut(SHORE).context("shore")?;
    let return_warp = i16::try_from(shore.events.warps.len() + 2)?;
    let [door_x, door_y] = source.clinic.entrance;
    let [return_x, return_y] = source.clinic.return_tile;
    ensure!(
        door_x == return_x && return_y == door_y + 1,
        "clinic needs a passive step outside its door"
    );
    authoring::warp(shore, (door_x, door_y), CLINIC_CONSTANT, 3);
    authoring::warp(shore, (return_x, return_y), CLINIC_CONSTANT, 3);
    let mut clinic = authoring::blank(&pack, "CherrygrovePokecenter1F", CLINIC, CLINIC_CONSTANT)?;
    // Original clinic shell, with its link-club stairs replaced by ordinary floor.
    clinic.blocks[15] = 0x04;
    for tile in [(3, 7), (4, 7), (3, 5)] {
        authoring::warp(&mut clinic, tile, "MN_R11_C00R02", return_warp);
    }
    clinic.scripts.insert(
        "MnSplitRockNurse".into(),
        json!([
            authoring::command("blackoutmod", &[CLINIC_CONSTANT]),
            authoring::command("jumpstd", &["PokecenterNurseScript"])
        ]),
    );
    authoring::npc(&pack, &mut clinic, "MnSplitRockNurse", "SPRITE_NURSE", 3, 1)?;
    clinic.scripts.insert(
        "MnSplitRockPC".into(),
        json!([
            authoring::command("opentext", &[]),
            authoring::command("special", &["PokemonCenterPC"]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    );
    clinic.events.bg_events.push(serde_json::from_value(
        json!({"x":8,"y":0,"event_type":"BGEVENT_READ","script":"MnSplitRockPC"}),
    )?);
    ely_league_rooms::finish(&pack, &mut clinic)?;
    for map in exteriors.values_mut() {
        authoring::refresh_event_section(map);
    }
    let pass = exteriors.remove(PASS).context("new pass")?;
    let group_id = (100..=255)
        .find(|id| {
            !pack
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == *id)
        })
        .context("free native map group")?;
    authored_town_publication::publish(
        &mut pack,
        vec![(pass, [0, 14]), (clinic, [3, 5])],
        CityIdentity {
            group: "MINNESOTA_ELY_SHORE_PASS",
            group_id: i16::try_from(group_id)?,
            landmark: "LANDMARK_MN_ELY_SHORE_PASS",
            fallback_landmark: Some("LANDMARK_MN_LEAGUE_APPROACH"),
            constant: "MN_ELY_SHORE_PASS",
            name: "ELY SHORE PASS",
            position: [82, 22],
            manifest: MANIFEST,
            town: CLINIC,
            clinic: CLINIC,
            gym: "",
        },
    )?;
    for map in exteriors.into_values() {
        authored_town_publication::replace(&mut pack, map)?;
    }
    if let Some(encounters) = base.data.wild_encounters.get(SHORE) {
        let mut encounters = encounters.clone();
        encounters.map_name = PASS.into();
        encounters.zones.clear();
        pack.data.wild_encounters.insert(PASS.into(), encounters);
    }
    validate_seams(&pack, source.clinic.return_tile)?;
    ensure!(
        crate::authored_town_geometry::kind(&pack, &pack.data.maps[SHORE], source.clinic.entrance)?
            == "DOOR",
        "clinic facade door collision required"
    );
    pack.report.maps = pack.data.maps.len();
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    preserve(base, &pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn validate_seams(pack: &CompiledGamePack, clinic_return: [u16; 2]) -> Result<()> {
    for (name, points) in [
        (ELY, vec![[39, 14], [39, 15]]),
        (PASS, vec![[0, 14], [0, 15], [13, 10], [13, 11]]),
        (SHORE, vec![[0, 30], [0, 31], clinic_return]),
    ] {
        for point in points {
            ensure!(
                crate::authored_town_geometry::kind(pack, &pack.data.maps[name], point)? == "FLOOR",
                "unsafe native seam {name} {point:?}"
            );
        }
    }
    let p = &pack.data.maps[PASS];
    for y in 0..24 {
        for x in 0..14 {
            if (x == 0 || x == 13 || y == 0 || y == 23)
                && !([[0, 14], [0, 15], [13, 10], [13, 11]].contains(&[x, y]))
            {
                ensure!(
                    crate::authored_town_geometry::kind(pack, p, [x, y])? == "WALL",
                    "unbound pass boundary"
                );
            }
        }
    }
    Ok(())
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack) -> Result<()> {
    ensure!(
        base.compiled_audio == output.compiled_audio
            && base.audio_manifest == output.audio_manifest,
        "audio changed"
    );
    for (path, bytes) in &base.runtime_files {
        ensure!(
            output.runtime_files.get(path) == Some(bytes),
            "original runtime art changed: {path}"
        );
    }
    ensure!(
        output
            .runtime_files
            .keys()
            .filter(|k| !base.runtime_files.contains_key(*k))
            .all(|k| k.contains(SNOW)),
        "unrelated new runtime file"
    );
    let mut before = serde_json::to_value(&base.data)?;
    let mut after = serde_json::to_value(&output.data)?;
    for table in ["maps", "map_attributes"] {
        for name in [ELY, SHORE] {
            after[table][name] = before[table][name].clone();
        }
    }
    for name in [ELY, SHORE] {
        let label = base.data.maps[name]
            .attributes
            .blocks_label
            .as_deref()
            .context("block mirror")?;
        after["map_blocks"][label] = before["map_blocks"][label].clone();
        // Existing scripts are retained exactly; the wire event section gains only clinic warps.
        let section = base.data.maps[name]
            .attributes
            .map_events_label
            .as_deref()
            .context("event mirror")?;
        after["map_scripts"][section] = before["map_scripts"][section].clone();
        after["npcs"][name] = before["npcs"][name].clone();
    }
    let recoveries = output
        .data
        .runtime_spawn_points
        .iter()
        .filter(|(key, _)| !base.data.runtime_spawn_points.contains_key(*key))
        .collect::<Vec<_>>();
    ensure!(
        recoveries.len() == 1,
        "exactly one new clinic recovery required"
    );
    for (key, point) in recoveries {
        ensure!(
            point.map_name == CLINIC
                && point.map_constant == CLINIC_CONSTANT
                && point.tile_x == 3
                && point.tile_y == 5,
            "new recovery must be the clinic's passive floor arrival"
        );
        after["runtime_spawn_points"]
            .as_object_mut()
            .context("spawn catalog")?
            .remove(key);
    }
    prune_additions(&before, &mut after, "")?;
    // The optional native landmark is appended; no previous caught area can change.
    let old = before["pokegear_landmarks"]["landmarks"]
        .as_array_mut()
        .context("landmarks")?;
    let new = after["pokegear_landmarks"]["landmarks"]
        .as_array_mut()
        .context("landmarks")?;
    ensure!(new.starts_with(old), "old landmarks changed");
    for extra in &new[old.len()..] {
        ensure!(
            extra["constant"] == "LANDMARK_MN_ELY_SHORE_PASS",
            "unexpected landmark"
        );
    }
    new.truncate(old.len());
    ensure!(
        before == after,
        "unrelated data changed in Ely pass publication: {}",
        first_difference(&before, &after, "data").unwrap_or_else(|| "unknown difference".into())
    );
    Ok(())
}

fn first_difference(before: &Value, after: &Value, path: &str) -> Option<String> {
    if before == after {
        return None;
    }
    if let (Some(a), Some(b)) = (before.as_object(), after.as_object()) {
        for (key, value) in a {
            let child = format!("{path}/{key}");
            let Some(other) = b.get(key) else {
                return Some(child);
            };
            if let Some(difference) = first_difference(value, other, &child) {
                return Some(difference);
            }
        }
        for key in b.keys() {
            if !a.contains_key(key) {
                return Some(format!("{path}/{key}"));
            }
        }
    }
    Some(path.into())
}

fn prune_additions(before: &Value, after: &mut Value, path: &str) -> Result<()> {
    if let (Some(old), Some(new)) = (before.as_object(), after.as_object_mut()) {
        let extras = new
            .keys()
            .filter(|key| !old.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        for key in extras {
            let child = format!("{path}/{key}");
            let owned = [
                PASS,
                CLINIC,
                "MN_ELY_SHORE_PASS",
                CLINIC_CONSTANT,
                SNOW,
                "MnSplitRock",
                "MAPGROUP_MINNESOTA_ELY_SHORE_PASS",
                "LANDMARK_MN_ELY_SHORE_PASS",
            ]
            .iter()
            .any(|name| key.contains(name));
            ensure!(owned, "unexpected new data: {child}");
            new.remove(&key);
        }
        for (key, value) in old {
            if let Some(child) = new.get_mut(key) {
                prune_additions(value, child, &format!("{path}/{key}"))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_ely_shore_pass_pack, hash};
    #[test]
    #[ignore = "requires CRYSTAL_ELY_PASS_PARENT_PACK and external source-v08/native.contract.json"]
    fn ely_pass_publisher_preserves_parent_and_rejects_changed_contract() {
        let path = std::env::var_os("CRYSTAL_ELY_PASS_PARENT_PACK").expect("compact parent");
        let base = crate::read_verified_compiled_game_pack(path).expect("parent pack");
        let source = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../planning/minnesota/revisions/ely-backroute-v01/source-v08/native.contract.json",
        )).expect("external reviewed Ely pass contract");
        let output = build_ely_shore_pass_pack(&base, &source, &hash(source.as_bytes()))
            .expect("bounded pass publication");
        assert_eq!(output.data.maps.len(), base.data.maps.len() + 2);
        assert_eq!(output.compiled_audio, base.compiled_audio);
        assert_eq!(
            output.data.maps["MnEly"].objects,
            base.data.maps["MnEly"].objects
        );
        let changed = format!("{source} ");
        assert!(build_ely_shore_pass_pack(&base, &changed, &hash(source.as_bytes())).is_err());
        assert!(build_ely_shore_pass_pack(&output, &source, &hash(source.as_bytes())).is_err());
    }
}
