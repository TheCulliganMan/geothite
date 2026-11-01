//! Guarded structural revision of the compact Superior coast and Gunflint route.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

#[path = "compact_superior_art.rs"]
mod art;
#[path = "compact_superior_programs.rs"]
mod programs;
#[path = "compact_superior_validation.rs"]
mod validation;

const PARENT: &str = "f34280a5fc5d6ab8ccc3ec4ef4317a6ea9321070df342ea95aca5745514a0223";
const TOWN: &str = "MnGrandMarais";
const CEDAR: &str = "MnMaraisCedarApproach";
const NECK: &str = "MnMaraisInlandNeck";
const UPPER: &str = "MnR12C00R00";
const LOWER: &str = "MnR12C00R01";
const COAST: &str = "MnR11C01R00";
const LOWER_COAST: &str = "MnR11C01R01";
const BOUNDARY: &str = "MnBoundaryWaters";
const OWNED: [&str; 14] = [
    COAST,
    LOWER_COAST,
    TOWN,
    CEDAR,
    NECK,
    UPPER,
    LOWER,
    "MnR12C01R00",
    "MnR12C01R01",
    "MnR12C00R02",
    "MnR12C01R02",
    "MnR12Cave1F",
    "MnR12RangerHut",
    BOUNDARY,
];
const RETIRED: [&str; 5] = [
    NECK,
    "MnR12C01R00",
    "MnR12C01R01",
    "MnR12C00R02",
    "MnR12C01R02",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u16,
    id: String,
    module_preimages: BTreeMap<String, String>,
    source_sha256: BTreeMap<String, String>,
}

/// Apply the exact caller-reviewed compact source to the verified 694-map parent.
///
/// Retains existing numeric identities, programs, trainer parties and flags;
/// relocates authored R12/Cedar anchors, rebases their connections, removes the
/// explicitly retired five maps and appends one private native pine recipe.
/// All other maps, programs, art, audio, catalogs and recovery data are retained.
/// No output is written. The caller must bind reviewed contract bytes explicitly,
/// write to a fresh path and verify strict readback. This is not visual,
/// controller, earned progression or old-save compatibility acceptance.
///
/// # Errors
/// Rejects stale parent/source/module preimages, changed program identities,
/// unsupported relocation, dangling retired references, invalid graph geometry,
/// unexpected wire mutations or failed runtime verification. Borrowed inputs
/// remain unchanged on success and failure.
pub fn build_compact_superior_pack(
    base: &CompiledGamePack,
    contract_json: &str,
    expected_contract_sha256: &str,
    coast_contract_json: &str,
    r12_layout_json: &str,
    r12_dossier_json: &str,
    cedar_contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == expected_contract_sha256,
        "reviewed compact contract SHA differs"
    );
    let plan: Contract = serde_json::from_str(contract_json)?;
    ensure!(
        plan.schema_version == 1 && plan.id == "MN_COMPACT_SUPERIOR_V1",
        "compact source profile differs"
    );
    ensure!(
        base.identity()?.content_hash == PARENT && base.data.maps.len() == 694,
        "exact completed Superior parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity
            == crate::derive_compiled_game_pack_identity_from_manifest(
                base.format_version,
                &base.data,
                &base.audio_manifest,
                &base.runtime_files,
                &base.report
            )?,
        "compact parent derived identity is stale"
    );
    ensure!(
        plan.module_preimages
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == OWNED.into_iter().collect(),
        "complete compact module guard set required"
    );
    for name in OWNED {
        ensure!(
            json_hash(&base.data.maps[name])? == plan.module_preimages[name],
            "compact native preimage differs: {name}"
        );
    }
    let sources = [
        ("coast", coast_contract_json),
        ("r12_layout", r12_layout_json),
        ("r12_dossier", r12_dossier_json),
        ("cedar", cedar_contract_json),
    ];
    ensure!(
        plan.source_sha256.len() == sources.len(),
        "exact compact source guard set required"
    );
    for (name, text) in sources {
        ensure!(
            plan.source_sha256.get(name) == Some(&hash(text.as_bytes())),
            "reviewed compact source differs: {name}"
        );
    }
    let coast: Value = serde_json::from_str(coast_contract_json)?;
    let cedar: Value = serde_json::from_str(cedar_contract_json)?;
    let layout = Layout::parse(r12_layout_json)?;
    ensure!(
        layout.id == "MN_R12"
            && layout.dimensions_metatiles == [32, 32]
            && layout.compiled_tileset == "mn_winter_gunflint",
        "compact R12 identity/extent/alias differs"
    );
    let mut output = base.clone();
    art::append_pine(base, &mut output)?;
    let mut revised = programs::redistribute(base, &layout.compile()?, r12_dossier_json)?;
    let mut shoreline = base.data.maps[COAST].clone();
    ensure!(
        coast["dimensions_metatiles"] == serde_json::json!([4, 24]),
        "reviewed coast dimensions differ"
    );
    shoreline.blocks = rows(&coast, 4, 24)?;
    shoreline.attributes.width = 4;
    revised.insert(COAST.into(), shoreline);
    let boundary = &coast["lower_north_boundary"];
    ensure!(
        boundary["map"].as_str() == Some(LOWER_COAST),
        "explicit lower coastal boundary required"
    );
    let mut lower_coast = base.data.maps[LOWER_COAST].clone();
    ensure!(
        boundary["old_module_sha256"].as_str() == Some(json_hash(&lower_coast)?.as_str()),
        "lower coastal preimage differs"
    );
    let cells = boundary["cells"]
        .as_array()
        .context("lower coastal boundary cells")?;
    ensure!(
        cells.len() == 20,
        "complete lower coastal overhang boundary required"
    );
    let mut seen = BTreeSet::new();
    for cell in cells {
        let x = usize::try_from(cell["metatile"][0].as_u64().context("lower coastal x")?)?;
        ensure!(
            (4..24).contains(&x) && cell["metatile"][1].as_u64() == Some(0) && seen.insert(x),
            "lower coast exact northern overhang scope differs"
        );
        ensure!(
            cell["before"].as_u64() == Some(u64::from(lower_coast.blocks[x]))
                && cell["after"].as_u64() == Some(10),
            "lower coastal cell preimage or permanent rock differs"
        );
        lower_coast.blocks[x] = 10;
    }
    revised.insert(LOWER_COAST.into(), lower_coast);

    let mut approach = base.data.maps[CEDAR].clone();
    ensure!(
        cedar["dimensions"] == serde_json::json!([20, 12]),
        "reviewed Cedar dimensions differ"
    );
    approach.blocks = rows(&cedar, 20, 12)?;
    approach.attributes.width = 20;
    approach.attributes.height = 12;
    approach.attributes.border_block = 0x8b;
    for actor in &mut approach.objects {
        let id = actor
            .object_identifier
            .as_ref()
            .context("Cedar actor identity")?;
        let xy = cedar["actor_anchors"]
            .get(id)
            .context("reviewed Cedar actor anchor")?;
        actor.x = u16::try_from(xy[0].as_u64().context("Cedar actor x")?)?;
        actor.y = u16::try_from(xy[1].as_u64().context("Cedar actor y")?)?;
    }
    ensure!(
        approach.events.bg_events.len() == 1,
        "Cedar sign preimage differs"
    );
    approach.events.bg_events[0].x =
        u16::try_from(cedar["sign_anchor"][0].as_u64().context("Cedar sign x")?)?;
    approach.events.bg_events[0].y =
        u16::try_from(cedar["sign_anchor"][1].as_u64().context("Cedar sign y")?)?;
    authoring::refresh_event_section(&mut approach);
    revised.insert(CEDAR.into(), approach);
    revised.insert(TOWN.into(), base.data.maps[TOWN].clone());
    revised.insert(BOUNDARY.into(), base.data.maps[BOUNDARY].clone());
    set_connections(
        revised.get_mut(UPPER).context("upper R12")?,
        serde_json::json!([
            {"direction":"south","target_map":LOWER,"offset":0},
            {"direction":"west","target_map":BOUNDARY,"offset":2}
        ]),
    )?;
    set_connections(
        revised.get_mut(LOWER).context("lower R12")?,
        serde_json::json!([
            {"direction":"north","target_map":UPPER,"offset":0},
            {"direction":"east","target_map":CEDAR,"offset":-4}
        ]),
    )?;
    set_connections(
        revised.get_mut(CEDAR).context("Cedar")?,
        serde_json::json!([
            {"direction":"west","target_map":LOWER,"offset":4},
            {"direction":"south","target_map":TOWN,"offset":4}
        ]),
    )?;
    for link in &mut revised
        .get_mut(TOWN)
        .context("town")?
        .attributes
        .connections
    {
        if link.direction == "north" {
            ensure!(link.target_map == CEDAR, "town north target differs");
            link.offset = -4;
        }
    }
    for link in &mut revised
        .get_mut(BOUNDARY)
        .context("Boundary Waters")?
        .attributes
        .connections
    {
        if link.direction == "east" {
            ensure!(
                link.target_map == "MnR12C00R01",
                "Boundary old east target differs"
            );
            link.target_map = UPPER.into();
            link.offset = -2;
        }
    }
    // Remove every old source program before inserting its new owner: otherwise
    // replacing a later old owner could delete a program already migrated earlier.
    for name in [UPPER, LOWER].into_iter().chain(RETIRED) {
        remove_owned_labels(&mut output, &base.data.maps[name])?;
    }
    for name in RETIRED {
        retire(&mut output, &base.data.maps[name])?;
    }
    for name in [UPPER, LOWER] {
        let cached = output
            .data
            .maps
            .get_mut(name)
            .context("retained cached chunk")?;
        cached.scripts.clear();
        cached.script_text_bodies.clear();
    }
    for (name, module) in &revised {
        if [UPPER, LOWER, CEDAR, "MnR12Cave1F", "MnR12RangerHut"].contains(&name.as_str()) {
            crate::authored_town_publication::replace(&mut output, module.clone())?;
        } else {
            geometry(&mut output, module.clone())?;
        }
    }
    output.report.maps = output.data.maps.len();
    output
        .report
        .reachable_maps
        .retain(|n| !RETIRED.contains(&n.as_str()));
    output
        .report
        .solvable_maps
        .retain(|n| !RETIRED.contains(&n.as_str()));
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    validation::validate(base, &output, &revised)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn json_hash(value: &impl serde::Serialize) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}
fn rows(value: &Value, width: u16, height: u16) -> Result<Vec<u16>> {
    let lines = value["rows"].as_array().context("explicit compact rows")?;
    ensure!(
        lines.len() == usize::from(height),
        "compact source row count differs"
    );
    let mut blocks = Vec::new();
    for line in lines {
        let row = line
            .as_str()
            .context("compact row text")?
            .split_whitespace()
            .map(|s| u16::from_str_radix(s, 16).context("native compact block"))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            row.len() == usize::from(width),
            "compact source row width differs"
        );
        blocks.extend(row);
    }
    Ok(blocks)
}
fn set_connections(map: &mut MapModule, connections: Value) -> Result<()> {
    map.attributes.connections = serde_json::from_value(connections)?;
    map.attributes.connection_flags = Some(
        map.attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    Ok(())
}
fn geometry(pack: &mut CompiledGamePack, map: MapModule) -> Result<()> {
    let label = map
        .attributes
        .blocks_label
        .as_ref()
        .context("compact blocks label")?;
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    pack.data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    pack.data
        .map_attributes
        .insert(map.id.clone(), map.attributes.clone());
    let constant = map
        .attributes
        .map_constant
        .as_ref()
        .context("compact map constant")?;
    pack.data.map_dimensions.insert(
        constant.clone(),
        serde_json::json!({"width":map.attributes.width,"height":map.attributes.height}),
    );
    let meta = pack
        .data
        .runtime_map_metadata
        .get_mut(constant)
        .context("compact metadata")?;
    meta.width = map.attributes.width;
    meta.height = map.attributes.height;
    pack.data.maps.insert(map.id.clone(), map);
    Ok(())
}
fn remove_owned_labels(pack: &mut CompiledGamePack, original: &MapModule) -> Result<()> {
    for label in original
        .scripts
        .keys()
        .chain(original.script_text_bodies.keys())
        .chain(original.attributes.map_scripts_label.iter())
        .chain(original.attributes.map_events_label.iter())
    {
        ensure!(
            pack.data
                .maps
                .values()
                .all(|m| m.id == original.id || !m.scripts.contains_key(label)),
            "owned compact label shared with another map"
        );
        pack.data.map_scripts.remove(label);
    }
    Ok(())
}
fn retire(pack: &mut CompiledGamePack, original: &MapModule) -> Result<()> {
    let name = &original.id;
    let constant = original
        .attributes
        .map_constant
        .as_ref()
        .context("retired constant")?;
    ensure!(
        pack.data
            .runtime_spawn_points
            .values()
            .all(|s| s.map_name != *name && s.map_constant != *constant),
        "retired map retains a spawn/recovery"
    );
    pack.data.maps.remove(name);
    pack.data.map_attributes.remove(name);
    pack.data.map_dimensions.remove(constant);
    pack.data.runtime_map_metadata.remove(constant);
    pack.data.map_blocks.remove(
        original
            .attributes
            .blocks_label
            .as_ref()
            .context("retired block label")?,
    );
    pack.data.npcs.remove(name);
    pack.data.wild_encounters.remove(name);
    pack.data.pokegear_landmarks.map_to_landmark.remove(name);
    Ok(())
}

#[cfg(test)]
#[path = "compact_superior_tests.rs"]
mod tests;
