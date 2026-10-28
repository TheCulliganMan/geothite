//! Publish the reviewed Minneapolis river bend without rebuilding old scripts.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const CONTRACT_SHA: &str = "32757c9db294ce6ae58bacfda7107d807b4ecc77a2123c07e5d09efae854bb89";
const PARENT_SHA: &str = "92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33";
const PARENT_IDENTITY: &str = "043b57b8814714bcf7b14f1571931eadbd139c12ad3e9afba596c193034c358f";
const ROAD: &str = "MnR03C00R00";
const CONFLUENCE: &str = "MnMississippiConfluenceC00R00";
const SIGN: &str = "MN_R03_NORTH_SIGN";
const EVENTS: &str = "MnR03C00R00_MapEvents";

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    parent_pack_sha256: String,
    parent_identity: String,
    changes: Vec<Change>,
}

#[derive(Deserialize)]
struct Change {
    map: String,
    module_sha256: String,
    dimensions_metatiles: [u16; 2],
    tileset: String,
    cells: Vec<Cell>,
}

#[derive(Deserialize)]
struct Cell {
    metatile: [u16; 2],
    before: String,
    after: String,
}

/// Applies the frozen 51-cell river bend and relocates its existing north sign.
///
/// Requires the exact reviewed winter parent and source contract. Native art,
/// connections, scripts, text, objects, rooms, gates, audio, report and unrelated
/// maps remain exact. The only declaration edit is the north sign's y argument
/// from4 to8; its raw and typed mirrors are updated together. The caller owns
/// fresh publication and strict written readback. This does not certify native
/// input, visual acceptance or earned campaign access.
///
/// # Errors
/// Rejects stale source/module preimages, duplicate or unsupported terrain,
/// shared payloads, inconsistent native mirrors or changes outside the contract.
/// The borrowed parent is unchanged on success and failure.
pub fn build_river_continuity_bend_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA,
        "reviewed bend contract required"
    );
    let contract: Contract = serde_json::from_str(contract_json).context("river bend contract")?;
    ensure!(
        contract.schema_version == 1
            && contract.parent_pack_sha256 == PARENT_SHA
            && contract.parent_identity == PARENT_IDENTITY
            && base.identity.content_hash == PARENT_IDENTITY
            && base.data.maps.len() == 683
            && contract
                .changes
                .iter()
                .map(|c| c.map.as_str())
                .eq([ROAD, CONFLUENCE])
            && contract.changes[0].cells.len() == 41
            && contract.changes[1].cells.len() == 10,
        "exact 683-map bend parent/profile required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut output = base.clone();
    for change in &contract.changes {
        let original = base
            .data
            .maps
            .get(&change.map)
            .context("river bend module")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == change.module_sha256
                && [original.attributes.width, original.attributes.height]
                    == change.dimensions_metatiles
                && original.attributes.tileset_name == change.tileset
                && base.data.map_attributes.get(&change.map) == Some(&original.attributes),
            "river bend module/attribute preimage differs: {}",
            change.map
        );
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("bend block label")?;
        ensure!(
            base.data.maps.iter().all(|(name, m)| name == &change.map
                || m.attributes.blocks_label.as_ref() != Some(label)),
            "shared river block payload"
        );
        ensure!(
            crate::decode_base64_bytes(
                base.data
                    .map_blocks
                    .get(label)
                    .context("bend block payload")?
            )?
            .into_iter()
            .map(u16::from)
            .eq(original.blocks.iter().copied()),
            "river block mirror differs"
        );
        let mut module = original.clone();
        let mut visited = BTreeSet::new();
        for cell in &change.cells {
            let [x, y] = cell.metatile;
            ensure!(
                x < original.attributes.width
                    && y < original.attributes.height
                    && visited.insert([x, y]),
                "invalid or duplicate river bend cell"
            );
            let before = u16::from_str_radix(&cell.before, 16)?;
            let after = u16::from_str_radix(&cell.after, 16)?;
            let index = usize::from(y) * usize::from(original.attributes.width) + usize::from(x);
            ensure!(
                module.blocks[index] == before && before != after,
                "stale/no-op river cell"
            );
            ensure!(
                base.data.tilesets[&change.tileset]
                    .collision
                    .contains_key(&format!("{after:02x}")),
                "unsupported bend metatile"
            );
            module.blocks[index] = after;
        }
        if change.map == ROAD {
            ensure!(
                original.attributes.map_events_label.as_deref() == Some(EVENTS),
                "river sign event label changed"
            );
            ensure!(
                base.data.maps.iter().all(|(name, m)| name == ROAD
                    || m.attributes.map_events_label.as_deref() != Some(EVENTS)),
                "shared sign declaration payload"
            );
            move_module_sign(&mut module)?;
        }
        let bytes = module
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        output
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
        output.data.maps.insert(change.map.clone(), module);
    }
    move_raw_sign(
        output
            .data
            .map_scripts
            .get_mut(EVENTS)
            .context("native sign declaration")?,
    )?;
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    preserve(base, &output, &contract)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn move_raw_sign(payload: &mut Value) -> Result<()> {
    let entries = payload
        .as_array_mut()
        .context("native event command array")?;
    let mut matched = 0;
    for entry in entries {
        if entry.get("command").and_then(Value::as_str) == Some("bg_event")
            && entry
                .get("args")
                .and_then(|a| a.get(3))
                .and_then(Value::as_str)
                == Some(SIGN)
        {
            ensure!(
                entry["args"] == serde_json::json!(["22", "4", "BGEVENT_READ", SIGN]),
                "native sign command preimage differs"
            );
            entry["args"][1] = Value::String("8".into());
            matched += 1;
        }
    }
    ensure!(
        matched == 1,
        "exactly one native north sign declaration required"
    );
    Ok(())
}

fn move_module_sign(module: &mut MapModule) -> Result<()> {
    let mut matched = 0;
    for event in &mut module.events.bg_events {
        if event.script == SIGN {
            ensure!(
                [event.x, event.y] == [22, 4] && event.event_type == "BGEVENT_READ",
                "typed river sign preimage differs"
            );
            event.y = 8;
            matched += 1;
        }
    }
    ensure!(matched == 1, "exactly one typed north sign required");
    let mut commands = 0;
    for command in &mut module.map_event_section_commands {
        if command.command == "bg_event" && command.args.get(3).is_some_and(|a| a == SIGN) {
            ensure!(
                command
                    .args
                    .iter()
                    .map(String::as_str)
                    .eq(["22", "4", "BGEVENT_READ", SIGN]),
                "typed native sign declaration differs"
            );
            command.args[1] = "8".into();
            commands += 1;
        }
    }
    ensure!(
        commands == 1,
        "exactly one typed sign section command required"
    );
    if let Some(payload) = module.scripts.get_mut(EVENTS) {
        move_raw_sign(payload)?;
    }
    Ok(())
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack, contract: &Contract) -> Result<()> {
    let mut normalized = output.clone();
    for change in &contract.changes {
        let old = base.data.maps.get(&change.map).context("old bend module")?;
        let mut expected = old.clone();
        for cell in &change.cells {
            let [x, y] = cell.metatile;
            let index = usize::from(y) * usize::from(old.attributes.width) + usize::from(x);
            expected.blocks[index] = u16::from_str_radix(&cell.after, 16)?;
        }
        if change.map == ROAD {
            move_module_sign(&mut expected)?;
        }
        ensure!(
            output.data.maps.get(&change.map) == Some(&expected),
            "bend module changed outside terrain/sign declaration"
        );
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("old block label")?;
        let bytes = expected
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            output.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&bytes)),
            "written bend block mirror differs"
        );
        normalized.data.maps.insert(change.map.clone(), old.clone());
        normalized.data.map_blocks.insert(
            label.clone(),
            base.data
                .map_blocks
                .get(label)
                .context("original block payload")?
                .clone(),
        );
    }
    let mut expected_raw = base
        .data
        .map_scripts
        .get(EVENTS)
        .context("old event declaration")?
        .clone();
    move_raw_sign(&mut expected_raw)?;
    ensure!(
        output.data.map_scripts.get(EVENTS) == Some(&expected_raw),
        "raw sign declaration differs outside one argument"
    );
    normalized.data.map_scripts.insert(
        EVENTS.into(),
        base.data
            .map_scripts
            .get(EVENTS)
            .context("old declaration")?
            .clone(),
    );
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "river bend changed unrelated maps/programs/art/audio/report"
    );
    let mut before = base.data.clone();
    let mut after = output.data.clone();
    for change in &contract.changes {
        before
            .maps
            .remove(&change.map)
            .context("remove old cached bend")?;
        after
            .maps
            .remove(&change.map)
            .context("remove new cached bend")?;
    }
    for change in &contract.changes {
        let mut expected = before.assemble_map_module_from_compiled_payloads(&change.map)?;
        expected
            .blocks
            .clone_from(&output.data.maps[&change.map].blocks);
        if change.map == ROAD {
            move_module_sign(&mut expected)?;
        }
        ensure!(
            after.assemble_map_module_from_compiled_payloads(&change.map)? == expected,
            "uncached bend lost native program/event behavior"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "river_continuity_bend_tests.rs"]
mod tests;
