//! Hand-authored west-entry Superior coast and the inland Gunflint connector.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::grand_marais_modpack::authoring;
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, RuntimeMapMetadata, RuntimeSpawnPoint,
};

const CONTRACT_SHA: &str = "ee195bcfc318fa55e5c31d6f37f75ebbf1bb033760c98267303bc46a6beecc5c";
const CEDAR: &str = "MnMaraisCedarApproach";
const NECK: &str = "MnMaraisInlandNeck";
const CONSTANT: &str = "MN_MARAIS_INLAND_NECK";
const FLAG: &str = "EVENT_MN_GUNFLINT_CAMP_SUPPLY";
const SIGN: &str = "MN_COHESION_MARAIS_SIGNText";
const MANIFEST: &str = "minnesota-superior-inland-geography-v1";
const OWNED: [&str; 4] = [CEDAR, "MnR11C00R00", "MnR11C01R00", "MnR12C01R01"];

#[path = "superior_inland_geography_validation.rs"]
mod validation;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u16,
    id: String,
    base_map_count: usize,
    preimages: BTreeMap<String, String>,
    inland: Value,
    headland: Value,
}

/// Installs the reviewed warm coastal bend and snowy inland walking connection.
///
/// Requires the already-published Grand Marais west/north ports, additive Ely,
/// and native cold-pine art. The frozen source baseline guards four unchanged
/// owned modules while allowing a completed town parent to compose. Adds one
/// 65×19 map in existing group40, its copied Cedar encounter table, one native
/// three-Berry pickup and fresh event2387. No recovery, party, art or audio is
/// added. Existing Cedar trainers and programs remain unchanged except the two
/// explicitly corrected sign directions. The caller must bind its actual file
/// SHA, use a fresh output and validate written readback. This is construction,
/// not a controller, campaign, production-camera or visual acceptance claim.
///
/// # Errors
/// Rejects unreviewed source, stale identity or preimages, missing town ports,
/// occupied allocations, unsupported terrain, inconsistent native mirrors,
/// unsafe graph geometry or mutations outside the exact declared scope. The
/// borrowed parent remains unchanged on success and failure.
pub fn build_superior_inland_geography_modpack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    let plan = parse(base, contract_json)?;
    let revised = revisions(base, &plan)?;
    let mut output = base.clone();
    for (name, module) in &revised {
        let old = &base.data.maps[name];
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("owned block label")?;
        ensure!(
            base.data.maps.iter().all(
                |(other, m)| other == name || m.attributes.blocks_label.as_ref() != Some(label)
            ),
            "shared geography block payload"
        );
        ensure!(
            crate::decode_base64_bytes(base.data.map_blocks.get(label).context("old blocks")?)?
                .into_iter()
                .map(u16::from)
                .eq(old.blocks.iter().copied()),
            "owned cached/encoded block preimage differs"
        );
        if module.blocks != old.blocks {
            output
                .data
                .map_blocks
                .insert(label.clone(), encode(module)?);
        }
        output
            .data
            .map_attributes
            .insert(name.clone(), module.attributes.clone());
        output.data.maps.insert(name.clone(), module.clone());
    }
    correct_directions(
        output
            .data
            .map_scripts
            .get_mut(SIGN)
            .context("global Cedar sign")?,
    )?;
    ensure!(
        output
            .data
            .story_event_script_constants
            .global
            .insert(FLAG.into(), 2387)
            .is_none(),
        "camp flag occupied"
    );
    let module = new_map(&output, &plan)?;
    publish_neck(&mut output, module)?;
    output.report.maps = output.data.maps.len();
    output.identity = identity(&output)?;
    validation::validate(base, &output, &plan)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn parse(base: &CompiledGamePack, text: &str) -> Result<Contract> {
    ensure!(
        hash(text.as_bytes()) == CONTRACT_SHA,
        "exact west-entry source contract required"
    );
    let plan: Contract = serde_json::from_str(text).context("west-entry source")?;
    ensure!(
        plan.schema_version == 1
            && plan.id == "MN_SUPERIOR_INLAND_GEOGRAPHY_V1"
            && plan.base_map_count == 693
            && base.data.maps.len() == 693
            && plan.preimages.len() == OWNED.len(),
        "693-map geography profile required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity == identity(base)?,
        "geography parent identity is stale"
    );
    for name in OWNED {
        let module = base.data.maps.get(name).context("owned native module")?;
        ensure!(
            plan.preimages.get(name) == Some(&value_hash(module)?),
            "geography native module preimage differs: {name}"
        );
        ensure!(
            base.data.map_attributes.get(name) == Some(&module.attributes),
            "geography attribute mirror differs: {name}"
        );
    }
    ensure!(
        !base.data.maps.contains_key(NECK)
            && !base.data.runtime_map_metadata.contains_key(CONSTANT)
            && !base
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == 40 && m.map_id == 8)
            && !base
                .data
                .story_event_script_constants
                .global
                .contains_key(FLAG)
            && base
                .data
                .story_event_script_constants
                .global
                .iter()
                .filter(|(k, _)| k.starts_with("EVENT_"))
                .map(|(_, v)| *v)
                .max()
                == Some(2386)
            && base
                .data
                .runtime_spawn_points
                .values()
                .map(|s| s.identifier)
                .max()
                == Some(279)
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "fresh connector allocation required"
    );
    ensure!(
        base.data.maps.contains_key("MnEly")
            && base
                .data
                .tilesets
                .get("mn_winter_north_shore")
                .and_then(|t| t.collision.get("8b"))
                .is_some_and(|c| c.iter().all(|p| p == "WALL")),
        "installed Ely and native cold pines required"
    );
    let town = base
        .data
        .maps
        .get("MnGrandMarais")
        .context("published coastal town")?;
    ensure!(
        town.attributes.connections.len() == 2
            && has_connection(town, "west", "MnR11C01R00", 0)
            && has_connection(town, "north", CEDAR, -19),
        "published town west/north ports required"
    );
    for (tile, permission) in [
        ([0, 8], "FLOOR"),
        ([0, 9], "FLOOR"),
        ([18, 0], "FLOOR"),
        ([19, 0], "FLOOR"),
    ] {
        ensure!(
            crate::authored_town_geometry::kind(base, town, tile)? == permission,
            "published town Foot port differs"
        );
    }
    for y in 16..36 {
        ensure!(
            crate::authored_town_geometry::kind(base, town, [0, y])? == "WATER",
            "published town Superior port differs"
        );
    }
    ensure!(
        value_hash(
            base.data
                .wild_encounters
                .get(CEDAR)
                .context("Cedar encounters")?
        )? == plan.inland["neck"]["wild_encounters"]["source_sha256"]
            .as_str()
            .context("encounter source")?,
        "copied encounter preimage differs"
    );
    Ok(plan)
}

fn revisions(base: &CompiledGamePack, plan: &Contract) -> Result<BTreeMap<String, MapModule>> {
    let mut maps = BTreeMap::new();
    for name in OWNED {
        maps.insert(name.to_string(), revise(&base.data.maps[name], plan)?);
    }
    Ok(maps)
}

fn revise(old: &MapModule, plan: &Contract) -> Result<MapModule> {
    let mut module = old.clone();
    match old.id.as_str() {
        CEDAR => {
            cells(&mut module, &plan.inland["cedar"]["cells"])?;
            module.attributes = serde_json::from_value(plan.inland["cedar"]["attributes"].clone())?;
            correct_directions(module.scripts.get_mut(SIGN).context("Cedar raw sign")?)?;
            let body = module
                .script_text_bodies
                .get_mut(SIGN)
                .context("Cedar typed sign")?;
            for (index, before, after) in direction_changes() {
                let command = body.commands.get_mut(index).context("typed sign line")?;
                ensure!(
                    command.args.first().map(String::as_str) == Some(before),
                    "typed sign preimage"
                );
                command.args[0] = after.into();
            }
        }
        "MnR11C00R00" | "MnR11C01R00" => {
            let change = plan.headland["changes"]
                .as_array()
                .context("two headland revisions")?
                .iter()
                .find(|v| v["map"] == old.id)
                .context("headland map")?;
            cells(&mut module, &change["cells"])?;
            if old.id == "MnR11C00R00" {
                set_connection(&mut module, "north", None)?;
            } else {
                set_connection(&mut module, "east", Some(("MnGrandMarais", 0)))?;
            }
        }
        "MnR12C01R01" => set_connection(&mut module, "east", Some((NECK, 0)))?,
        _ => anyhow::bail!("unowned geography revision"),
    }
    Ok(module)
}

fn cells(module: &mut MapModule, cells: &Value) -> Result<()> {
    let mut visited = BTreeSet::new();
    for cell in cells.as_array().context("explicit terrain cells")? {
        let point: [u16; 2] = serde_json::from_value(cell["metatile"].clone())?;
        let [x, y] = point;
        ensure!(
            x < module.attributes.width && y < module.attributes.height && visited.insert(point),
            "invalid/repeated geography cell"
        );
        let before = u16::from_str_radix(cell["before"].as_str().context("before block")?, 16)?;
        let after = u16::from_str_radix(cell["after"].as_str().context("after block")?, 16)?;
        let index = usize::from(y) * usize::from(module.attributes.width) + usize::from(x);
        ensure!(
            module.blocks[index] == before && before != after && u8::try_from(after).is_ok(),
            "stale/no-op geography cell"
        );
        module.blocks[index] = after;
    }
    Ok(())
}

fn direction_changes() -> [(usize, &'static str, &'static str); 2] {
    [
        (1, "\"GRAND MARAIS west.\"", "\"GRAND MARAIS south\""),
        (2, "\"NORTH SHORE south.\"", "\"GUNFLINT west.\""),
    ]
}

fn correct_directions(script: &mut Value) -> Result<()> {
    for (index, before, after) in direction_changes() {
        let arg = script
            .as_array_mut()
            .context("sign command array")?
            .get_mut(index)
            .context("sign command")?
            .get_mut("args")
            .context("sign arguments")?
            .as_array_mut()
            .context("sign argument array")?
            .first_mut()
            .context("sign argument")?;
        ensure!(arg.as_str() == Some(before), "raw sign preimage differs");
        *arg = Value::String(after.into());
    }
    Ok(())
}

fn set_connection(module: &mut MapModule, side: &str, target: Option<(&str, i32)>) -> Result<()> {
    module
        .attributes
        .connections
        .retain(|c| c.direction != side);
    if let Some((name, offset)) = target {
        crate::lake_country_geometry::connect(module, side, name, offset)?;
    }
    module.attributes.connection_flags = if module.attributes.connections.is_empty() {
        None
    } else {
        Some(
            module
                .attributes
                .connections
                .iter()
                .map(|c| c.direction.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join(" | "),
        )
    };
    Ok(())
}

fn has_connection(module: &MapModule, side: &str, target: &str, offset: i32) -> bool {
    module
        .attributes
        .connections
        .iter()
        .any(|c| c.direction == side && c.target_map == target && c.offset == offset)
}

fn new_map(base: &CompiledGamePack, plan: &Contract) -> Result<MapModule> {
    let mut module = authoring::blank(base, CEDAR, NECK, CONSTANT)?;
    module.attributes = serde_json::from_value(plan.inland["neck"]["attributes"].clone())?;
    module.blocks = plan.inland["neck"]["rows"]
        .as_array()
        .context("65x19 rows")?
        .iter()
        .map(|v| v.as_str().context("terrain row"))
        .collect::<Result<Vec<_>>>()?
        .join(" ")
        .split_whitespace()
        .map(|b| u16::from_str_radix(b, 16).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        [module.attributes.width, module.attributes.height] == [65, 19]
            && module.blocks.len() == 65 * 19,
        "exact whole connector dimensions required"
    );
    module.scripts.insert(
        "MN_INLAND_CAMP_SUPPLY".into(),
        json!([authoring::command("itemball", &["BERRY", "3"])]),
    );
    authoring::npc(
        base,
        &mut module,
        "MN_INLAND_CAMP_SUPPLY",
        "SPRITE_POKE_BALL",
        63,
        27,
    )?;
    let object = module.objects.last_mut().context("camp supply object")?;
    object.sprite_has_facings = false;
    object.spritemovedata = "SPRITEMOVEDATA_STILL".into();
    object.object_type = "OBJECTTYPE_ITEMBALL".into();
    object.object_identifier = Some("MN_INLAND_CAMP_SUPPLY".into());
    object.event_flag = FLAG.into();
    module.scripts.insert(
        "MN_INLAND_CAMP_SIGN".into(),
        json!([
            authoring::command("opentext", &[]),
            authoring::command("writetext", &["MN_INLAND_CAMP_SIGNText"]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    );
    authoring::text(
        &mut module,
        "MN_INLAND_CAMP_SIGNText",
        &["PORTAGE CAMP", "Three BERRIES for", "the trail ahead."],
    );
    module.events.bg_events.push(serde_json::from_value(json!({
        "x":69,"y":25,"event_type":"BGEVENT_READ","script":"MN_INLAND_CAMP_SIGN"
    }))?);
    authoring::finish(base, &mut module)?;
    module.script_field_pickups =
        crate::parse_script_field_pickups(NECK, &module.scripts, &module.objects)?;
    Ok(module)
}

fn publish_neck(output: &mut CompiledGamePack, module: MapModule) -> Result<()> {
    let mut encounters = output
        .data
        .wild_encounters
        .get(CEDAR)
        .context("source encounters")?
        .clone();
    encounters.map_name = NECK.into();
    let location = output
        .data
        .pokegear_landmark_for_map("MnGrandMarais")?
        .constant
        .clone();
    ensure!(
        output
            .data
            .pokegear_landmarks
            .map_to_landmark
            .insert(NECK.into(), location)
            .is_none(),
        "connector landmark already occupied"
    );
    let extension = CompiledMapExtension {
        manifest_id: MANIFEST.into(),
        map_name: NECK.into(),
        map_constant: CONSTANT.into(),
        metadata: RuntimeMapMetadata {
            name: NECK.into(),
            constant: CONSTANT.into(),
            group_name: "MINNESOTA".into(),
            group_id: 40,
            map_id: 8,
            width: 65,
            height: 19,
            environment: "ROUTE".into(),
            phone_service: module.attributes.phone_service,
        },
        spawn_key: "280".into(),
        spawn: RuntimeSpawnPoint {
            identifier: 280,
            map_name: NECK.into(),
            map_constant: CONSTANT.into(),
            group_name: "MINNESOTA".into(),
            group_id: 40,
            map_id: 8,
            tile_x: 0,
            tile_y: 8,
            metatile_x: 0,
            metatile_y: 4,
            subtile_x: 0,
            subtile_y: 0,
        },
        module,
        wild_encounters: Some(encounters),
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(output, extension)?;
    ensure!(
        output.data.runtime_spawn_points.remove("280").is_some(),
        "temporary recovery missing"
    );
    Ok(())
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn value_hash(value: &impl serde::Serialize) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}

fn identity(pack: &CompiledGamePack) -> Result<crate::CompiledGamePackIdentity> {
    crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )
}

fn encode(module: &MapModule) -> Result<String> {
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(crate::encode_base64_bytes(&bytes))
}

#[cfg(test)]
#[path = "superior_inland_geography_tests.rs"]
mod tests;
