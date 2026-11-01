//! Join the installed six-room harbor to Superior west and Gunflint north.
use std::collections::BTreeSet;

use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnGrandMarais";
const TILESET: &str = "mn_marais_shore";
const EVENTS: &str = "MnGrandMarais_MapEvents";
const WRAPPER: &str = "MnMaraisCoastWaterIn";
const SIGN: &str = "MnGrandMaraisSign1Text";
const CONTRACT_SHA: &str = "c47d93919126bc32f3aa8d6b6f9034aff5427ceacea866ba93044a9576852278";

/// Builds the reviewed 28-cell west-coast/north-inland Grand Marais revision.
///
/// Reuses the ten established old-Johto reef recipes, preserves six installed
/// rooms, and rotates the former east checkpoint north. The existing west gate
/// also gates Surf; its denial and inward steps stay on Water. No badge, signal,
/// local lamp, nurse, shop or other progression predicate is granted or changed.
/// Exact owned preimages permit composition over unrelated upstream additions.
/// The caller must supply an explicit input-file SHA at publication, write to a
/// fresh ignored path and verify strict written readback. Geography must then
/// install the matching neighbor ports: this intermediate stage alone does not
/// certify the connected game, production input, native visuals or progression.
///
/// # Errors
/// Rejects a changed contract, owned preimage, shared payload, inconsistent raw
/// mirror, unsupported inherited cache difference, unsafe water geometry or any
/// residual outside the exact town/art footprint. Borrowed inputs stay intact.
pub fn build_grand_marais_superior_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    let plan = parse_contract(base, contract_json)?;
    let art = install_art(base, &plan)?;
    let mut output = apply_town(&art, &plan)?;
    output.identity = identity(&output)?;
    preserve(base, &output, &plan)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn value_hash(value: &impl Serialize) -> Result<String> {
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

fn array(value: &Value) -> Result<&Vec<Value>> {
    value.as_array().context("reviewed array")
}

fn text(value: &Value) -> Result<&str> {
    value.as_str().context("reviewed string")
}

fn number(value: &Value) -> Result<usize> {
    usize::try_from(value.as_u64().context("reviewed unsigned coordinate")?)
        .context("coordinate fits usize")
}

fn block(value: &Value) -> Result<u16> {
    Ok(u16::from_str_radix(text(value)?, 16)?)
}

fn parse_contract(base: &CompiledGamePack, source: &str) -> Result<Value> {
    ensure!(
        hash(source.as_bytes()) == CONTRACT_SHA,
        "frozen west/north source required"
    );
    let p: Value = serde_json::from_str(source)?;
    ensure!(
        p["schema_version"] == 2 && p["map"] == MAP && p["native_alias"] == TILESET,
        "bounded native harbor profile"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(base.identity == identity(base)?, "parent identity differs");
    let old = base.data.maps.get(MAP).context("installed harbor")?;
    ensure!(
        value_hash(old)? == text(&p["base_module_sha256"])?,
        "whole harbor preimage differs"
    );
    ensure!(
        [old.attributes.width, old.attributes.height] == [20, 19]
            && old.blocks.len() == 380
            && old.attributes.tileset_name == "johto",
        "native harbor dimensions/art differ"
    );
    let guards = &p["source_guards"];
    ensure!(
        value_hash(base.data.tilesets.get("johto").context("Johto")?)?
            == text(&guards["johto_definition_sha256"])?,
        "Johto definition differs"
    );
    for (path, expected) in guards["johto_runtime_sha256"]
        .as_object()
        .context("art guards")?
    {
        ensure!(
            hash(
                base.runtime_files
                    .get(path)
                    .context("native Johto resource")?
            ) == text(expected)?,
            "Johto resource differs: {path}"
        );
    }
    for (label, expected) in guards["raw_labels_sha256"]
        .as_object()
        .context("raw guards")?
    {
        ensure!(
            value_hash(
                base.data
                    .map_scripts
                    .get(label)
                    .context("native raw label")?
            )? == text(expected)?,
            "raw harbor program differs: {label}"
        );
    }
    ensure!(
        value_hash(
            base.data
                .map_attributes
                .get(MAP)
                .context("raw harbor attributes")?
        )? == text(&guards["native_attributes_sha256"])?,
        "raw attributes differ"
    );
    ensure!(
        value_hash(base.data.npcs.get(MAP).context("raw harbor objects")?)?
            == text(&guards["native_npcs_sha256"])?,
        "raw officers differ"
    );
    for (name, expected) in p["preserved_rooms_sha256"]
        .as_object()
        .context("six rooms")?
    {
        ensure!(
            value_hash(base.data.maps.get(name).context("installed room")?)? == text(expected)?,
            "installed harbor room differs: {name}"
        );
    }
    let label = old
        .attributes
        .blocks_label
        .as_deref()
        .context("harbor payload label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(n, m)| n == MAP || m.attributes.blocks_label.as_deref() != Some(label)),
        "shared harbor payload"
    );
    ensure!(
        crate::decode_base64_bytes(base.data.map_blocks.get(label).context("harbor bytes")?)?
            .into_iter()
            .map(u16::from)
            .eq(old.blocks.iter().copied()),
        "cached/raw block mismatch"
    );
    ensure!(
        !base.data.map_scripts.contains_key(WRAPPER),
        "water wrapper already exists"
    );
    ensure!(
        array(&p["cells"])?.len() == 28 && array(&p["native_art_recipes"])?.len() == 10,
        "exact reviewed cell/recipe extent"
    );
    Ok(p)
}

fn install_art(base: &CompiledGamePack, plan: &Value) -> Result<CompiledGamePack> {
    // Reuse the existing shore installer, including Johto field-action behavior.
    let art = crate::marais_shore_art::install(
        base,
        &json!({
            "tileset": TILESET, "reef_recipes": plan["native_art_recipes"]
        }),
    )?;
    let bytes = art
        .runtime_files
        .get(&format!("data/tilesets/{TILESET}_metatiles.bin"))
        .context("installed reef metatiles")?;
    ensure!(bytes.len() == 138 * 16, "exact reef extent");
    for recipe in array(&plan["native_art_recipes"])? {
        let id = usize::from(block(&recipe["id"])?);
        let glyphs: Vec<u8> = serde_json::from_value(recipe["glyphs"].clone())?;
        let kinds: [String; 4] = serde_json::from_value(recipe["collision"].clone())?;
        ensure!(
            bytes[id * 16..(id + 1) * 16] == glyphs
                && art.data.tilesets[TILESET].collision[text(&recipe["id"])?] == kinds,
            "native reef recipe differs"
        );
    }
    Ok(art)
}

fn rotated_args(value: &mut Value, plan: &Value) -> Result<()> {
    for edit in array(&plan["north_rotation"]["argument_edits"])? {
        let label = text(&edit["label"])?;
        let command = number(&edit["command_index"])?;
        let arg = number(&edit["arg_index"])?;
        let slot = value
            .get_mut(label)
            .context("rotated script")?
            .get_mut(command)
            .context("rotated command")?
            .get_mut("args")
            .context("command args")?
            .get_mut(arg)
            .context("rotated argument")?;
        ensure!(*slot == edit["before"], "rotated argument preimage differs");
        *slot = edit["after"].clone();
    }
    Ok(())
}

fn patch_events(raw: &Value, plan: &Value) -> Result<Value> {
    let mut commands = array(raw)?.clone();
    for command in &mut commands {
        match command["command"].as_str() {
            Some("coord_event") => {
                let script = command["args"][3].as_str().context("coordinate script")?;
                if script == "MnCohesionMaraisEast" || script == "MnCohesionMaraisEastInward" {
                    let old_y: u16 = text(&command["args"][1])?.parse()?;
                    ensure!([12, 13].contains(&old_y), "old north gate row");
                    let inward = script.ends_with("Inward");
                    command["args"][0] = json!((18 + old_y - 12).to_string());
                    command["args"][1] = json!(if inward { "1" } else { "3" });
                }
            }
            Some("object_event") if command["args"][11] == "MnCohesionMaraisEast" => {
                let old_y: u16 = text(&command["args"][1])?.parse()?;
                ensure!([12, 13].contains(&old_y), "old north guard row");
                command["args"][0] = json!((18 + old_y - 12).to_string());
                command["args"][1] = json!("2");
                command["args"][3] = json!("SPRITEMOVEDATA_STANDING_DOWN");
            }
            _ => {}
        }
    }
    let index = commands
        .iter()
        .position(|v| v["command"] == "def_bg_events")
        .context("coordinate section end")?;
    let mut water = Vec::new();
    for (key, script) in [
        ("outbound_core", "MnCohesionMaraisWest"),
        ("inward_core", WRAPPER),
    ] {
        for at in array(&plan["water_gate"][key])? {
            water.push(json!({"command":"coord_event", "args":[
                number(&at[0])?.to_string(), number(&at[1])?.to_string(), "-1", script]}));
        }
    }
    ensure!(water.len() == 42, "complete two-column water checkpoint");
    commands.splice(index..index, water);
    Ok(Value::Array(commands))
}

fn uncached(pack: &CompiledGamePack) -> Result<MapModule> {
    let mut data = pack.data.clone();
    data.maps.remove(MAP).context("cached harbor")?;
    data.assemble_map_module_from_compiled_payloads(MAP)
}

// Apply only actual wire changes to the inherited cache. An unchanged wire
// subtree retains its old cached representation, including inherited parser
// differences unrelated to this revision. Changed entries need exact preimages.
fn merge_wire_delta(cached: &mut Value, before: &Value, after: &Value) -> Result<()> {
    if before == after {
        return Ok(());
    }
    if cached == before {
        *cached = after.clone();
        return Ok(());
    }
    match (cached, before, after) {
        (Value::Object(cache), Value::Object(old), Value::Object(new)) => {
            for key in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
                if old.get(key) == new.get(key) {
                    continue;
                }
                // The installed cache omits this transport-only script label;
                // its parsed events are merged through the typed event fields.
                // Keep that exact inherited omission while the raw header is
                // updated and checked separately by the full residual audit.
                if key == EVENTS && !cache.contains_key(key) {
                    continue;
                }
                match (old.get(key), new.get(key)) {
                    (Some(a), Some(b)) => merge_wire_delta(
                        cache
                            .get_mut(key)
                            .with_context(|| format!("inherited cache key {key}"))?,
                        a,
                        b,
                    )?,
                    (None, Some(b)) => {
                        ensure!(
                            cache.insert(key.clone(), b.clone()).is_none(),
                            "new cached key already occupied: {key}"
                        );
                    }
                    (Some(a), None) => {
                        ensure!(
                            cache.remove(key).as_ref() == Some(a),
                            "removed cached key differs: {key}"
                        );
                    }
                    (None, None) => unreachable!("union contains a key"),
                }
            }
        }
        (Value::Array(cache), Value::Array(old), Value::Array(new))
            if old.len() == new.len() && cache.len() == old.len() =>
        {
            for ((c, a), b) in cache.iter_mut().zip(old).zip(new) {
                merge_wire_delta(c, a, b)?;
            }
        }
        (Value::Array(cache), Value::Array(old), Value::Array(new)) => {
            // Typed variable/control indexes grow by the single new wrapper.
            let key = |v: &Value| -> Result<(String, usize)> {
                Ok((
                    text(&v["source_script"])?.to_string(),
                    number(&v["command_index"])?,
                ))
            };
            for a in old {
                let k = key(a)?;
                let b = new.iter().find(|b| key(b).ok().as_ref() == Some(&k));
                if b == Some(a) {
                    continue;
                }
                let i = cache
                    .iter()
                    .position(|c| key(c).ok().as_ref() == Some(&k))
                    .context("changed typed cache entry")?;
                match b {
                    Some(b) => merge_wire_delta(&mut cache[i], a, b)?,
                    None => {
                        ensure!(cache[i] == *a, "deleted typed cache differs");
                        cache.remove(i);
                    }
                }
            }
            for b in new {
                let k = key(b)?;
                if !old.iter().any(|a| key(a).ok().as_ref() == Some(&k)) {
                    ensure!(
                        !cache.iter().any(|c| key(c).ok().as_ref() == Some(&k)),
                        "new typed key occupied"
                    );
                    cache.push(b.clone());
                }
            }
        }
        _ => bail!("owned wire change touches an unsupported inherited cache difference"),
    }
    Ok(())
}

fn apply_town(art: &CompiledGamePack, plan: &Value) -> Result<CompiledGamePack> {
    let old = art.data.maps.get(MAP).context("native town")?;
    let before = uncached(art)?;
    let mut output = art.clone();
    let mut blocks = old.blocks.clone();
    let mut touched = BTreeSet::new();
    for cell in array(&plan["cells"])? {
        let x = number(&cell["metatile"][0])?;
        let y = number(&cell["metatile"][1])?;
        ensure!(
            x < 20 && y < 19 && touched.insert((x, y)),
            "distinct bounded town cell"
        );
        ensure!(
            blocks[y * 20 + x] == block(&cell["before"])?,
            "town cell preimage differs"
        );
        blocks[y * 20 + x] = block(&cell["after"])?;
    }
    let label = old
        .attributes
        .blocks_label
        .as_deref()
        .context("town block label")?;
    let bytes = blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    output
        .data
        .map_blocks
        .insert(label.into(), crate::encode_base64_bytes(&bytes));
    output.data.map_attributes.insert(
        MAP.into(),
        serde_json::from_value(plan["attributes_after"].clone())?,
    );
    let mut raw = serde_json::to_value(&output.data.map_scripts)?;
    rotated_args(&mut raw, plan)?;
    raw[EVENTS] = patch_events(&raw[EVENTS], plan)?;
    raw[SIGN] = json!([
        {"command":"text", "args":["\"SUPERIOR HARBOR\""]},
        {"command":"line", "args":["\"NORTH SHORE: WEST\""]},
        {"command":"para", "args":["\"GUNFLINT: NORTH\""]},
        {"command":"line", "args":["\"ELY: INLAND SNOW\""]},
        {"command":"done", "args":[]}
    ]);
    raw[WRAPPER] = plan["water_gate"]["inward_wrapper"]["native_commands"].clone();
    output.data.map_scripts = serde_json::from_value(raw)?;
    let objects = output.data.npcs.get_mut(MAP).context("native NPC mirror")?;
    for object in objects.as_array_mut().context("NPC array")? {
        if object["script"] == "MnCohesionMaraisEast" {
            let old_y = number(&object["y"])?;
            ensure!([12, 13].contains(&old_y), "exact old officer row");
            object["x"] = json!(18 + old_y - 12);
            object["y"] = json!(2);
            object["spritemovedata"] = json!("SPRITEMOVEDATA_STANDING_DOWN");
        }
    }
    let after = uncached(&output)?;
    let mut cached = serde_json::to_value(old)?;
    merge_wire_delta(
        &mut cached,
        &serde_json::to_value(&before)?,
        &serde_json::to_value(&after)?,
    )?;
    let replacement: MapModule = serde_json::from_value(cached)?;
    ensure!(replacement.blocks == blocks, "cached terrain disagrees");
    ensure!(
        replacement.events.warps == old.events.warps
            && replacement.events.bg_events == old.events.bg_events,
        "original six doors/sign positions changed"
    );
    ensure!(
        replacement.events.coord_events.len() == 50,
        "all land/water event indexes required"
    );
    output.data.maps.insert(MAP.into(), replacement);
    validate_geometry(&output)?;
    Ok(output)
}

fn kind(pack: &CompiledGamePack, x: usize, y: usize) -> &str {
    let m = &pack.data.maps[MAP];
    let b = m.blocks[y / 2 * 20 + x / 2];
    &pack.data.tilesets[TILESET].collision[&format!("{b:02x}")][y % 2 * 2 + x % 2]
}

fn validate_geometry(pack: &CompiledGamePack) -> Result<()> {
    for y in 0..38 {
        ensure!(
            (kind(pack, 0, y) == "WATER") == (16..36).contains(&y),
            "west Superior aperture differs"
        );
        ensure!(
            !kind(pack, 39, y).starts_with("WATER"),
            "unmatched east water"
        );
    }
    for x in 0..40 {
        ensure!(
            !kind(pack, x, 37).starts_with("WATER"),
            "unmatched south water"
        );
        ensure!(
            (kind(pack, x, 0) == "FLOOR") == [18, 19].contains(&x),
            "north inland lane differs"
        );
    }
    for y in 16..37 {
        for x in 1..4 {
            ensure!(kind(pack, x, y) == "WATER", "Surf step would land ashore");
        }
    }
    for y in [8, 9] {
        for x in 0..5 {
            ensure!(kind(pack, x, y) == "FLOOR", "west Foot entry");
        }
    }
    ensure!(
        pack.data.maps[MAP]
            .blocks
            .iter()
            .all(
                |b| pack.data.tilesets[TILESET].collision[&format!("{b:02x}")]
                    .iter()
                    .all(|q| q != "CUT_TREE")
            ),
        "incidental town Cut gate"
    );
    Ok(())
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack, plan: &Value) -> Result<()> {
    let art = install_art(base, plan)?;
    let expected = apply_town(&art, plan)?;
    ensure!(
        output.data.maps.get(MAP) == expected.data.maps.get(MAP),
        "cached harbor outside reviewed delta"
    );
    let mut residual = output.clone();
    residual
        .data
        .maps
        .insert(MAP.into(), art.data.maps[MAP].clone());
    ensure!(
        output.data.map_attributes.get(MAP) == expected.data.map_attributes.get(MAP),
        "harbor attrs differ"
    );
    residual
        .data
        .map_attributes
        .insert(MAP.into(), art.data.map_attributes[MAP].clone());
    ensure!(
        output.data.npcs.get(MAP) == expected.data.npcs.get(MAP),
        "harbor objects differ"
    );
    residual
        .data
        .npcs
        .insert(MAP.into(), art.data.npcs[MAP].clone());
    let label = art.data.maps[MAP]
        .attributes
        .blocks_label
        .as_deref()
        .context("town label")?;
    ensure!(
        output.data.map_blocks.get(label) == expected.data.map_blocks.get(label),
        "encoded town cells differ"
    );
    residual
        .data
        .map_blocks
        .insert(label.into(), art.data.map_blocks[label].clone());
    let mut labels = plan["source_guards"]["raw_labels_sha256"]
        .as_object()
        .context("raw owned labels")?
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    labels.push(WRAPPER.into());
    for name in labels {
        ensure!(
            output.data.map_scripts.get(&name) == expected.data.map_scripts.get(&name),
            "raw program outside reviewed delta: {name}"
        );
        match art.data.map_scripts.get(&name) {
            Some(original) => {
                residual.data.map_scripts.insert(name, original.clone());
            }
            None => {
                residual.data.map_scripts.remove(&name);
            }
        }
    }
    residual.identity = art.identity.clone();
    ensure!(
        residual == art,
        "full pack residual differs beyond bounded town/art edit"
    );
    // Rebuilding each side without its cache preserves inherited parser drift.
    ensure!(
        uncached(output)? == uncached(&expected)?,
        "uncached native delta differs"
    );
    ensure!(
        output.identity == identity(output)?,
        "derived harbor identity differs"
    );
    Ok(())
}

#[cfg(test)]
#[path = "grand_marais_superior_tests.rs"]
mod tests;
