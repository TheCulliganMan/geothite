//! Keep Route 3's local directions consistent with its installed bluff stairs.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

const LABELS: [&str; 2] = ["MN_R03_TERRACE_SIGNText", "MN_R03_T04Won"];

/// Updates two existing flavor texts without changing their command boundaries.
///
/// # Errors
/// Rejects stale source, missing stairs, changed text structure, invalid native
/// lines, or inconsistent global and map-local script mirrors.
pub fn revise_r03_stair_guidance(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: BTreeMap<String, Value> = serde_json::from_str(before_json)?;
    let after: BTreeMap<String, Value> = serde_json::from_str(after_json)?;
    ensure!(
        before.len() == LABELS.len()
            && after.len() == LABELS.len()
            && LABELS
                .iter()
                .all(|k| before.contains_key(*k) && after.contains_key(*k)),
        "exactly two existing Route 3 texts required"
    );
    let route = base.data.maps.get("MnR03C00R00").context("Route 3")?;
    ensure!(
        route.attributes.width == 24
            && route.attributes.height == 24
            && route.attributes.tileset_name == "mn_outdoor_headwaters_stairs"
            && route.blocks[20 * 24 + 20] == 0x86,
        "installed ordinary bluff stair required"
    );
    for x in 36..40 {
        ensure!(
            crate::authored_town_geometry::kind(base, route, [x, 41])? == "WALL",
            "former high-cliff hop must remain closed"
        );
    }
    for label in LABELS {
        ensure!(
            base.data.map_scripts.get(label) == before.get(label),
            "global text preimage drift: {label}"
        );
        let old = before[label].as_array().context("old text program")?;
        let new = after[label].as_array().context("new text program")?;
        ensure!(old.len() == new.len(), "text command count drift: {label}");
        for (old, new) in old.iter().zip(new) {
            ensure!(old["command"] == new["command"], "text page boundary drift");
            let mut expected = old.clone();
            expected["args"] = new["args"].clone();
            ensure!(expected == *new, "only text arguments may change");
            match new["command"].as_str() {
                Some("text" | "line" | "para" | "cont") => {
                    let args = new["args"].as_array().context("native text args")?;
                    ensure!(args.len() == 1, "one native text literal required");
                    let line: String =
                        serde_json::from_str(args[0].as_str().context("quoted native text")?)?;
                    ensure!(
                        !line.is_empty()
                            && line.len() <= 18
                            && line.bytes().all(|b| (b' '..=b'~').contains(&b)),
                        "native line must fit 18 characters"
                    );
                }
                Some("done" | "prompt") => ensure!(old == new, "text terminator drift"),
                _ => anyhow::bail!("only existing text programs may change"),
            }
        }
    }
    let mut pack = base.clone();
    let mut replaced = 0;
    for name in ["MnR03C00R00"] {
        let mut map = base
            .data
            .maps
            .get(name)
            .context("Route 3 text map")?
            .clone();
        for label in LABELS {
            if let Some(installed) = map.scripts.get_mut(label) {
                ensure!(
                    *installed == before[label],
                    "map text preimage drift: {label}"
                );
                *installed = after[label].clone();
                pack.data
                    .map_scripts
                    .insert(label.into(), after[label].clone());
                replaced += 1;
            }
        }
        map.script_text_commands = crate::parse_script_text_commands(name, &map.scripts)?;
        map.script_text_bodies = crate::parse_script_text_bodies(name, &map.scripts)?;
        pack.data.maps.insert(name.into(), map);
    }
    ensure!(replaced == LABELS.len(), "each local text must occur once");
    const MANIFEST: &str = "minnesota-r03-stair-guidance-v1";
    ensure!(
        !base.report.manifests.iter().any(|v| v == MANIFEST),
        "already revised"
    );
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
