//! Private composition of the five independently reviewed Minnesota field revisions.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, LoadedCompiledGamePack};

const PARENT_SHA256: &str = "e56ded3d55b8364e8756a4a02eb530fb74f52c03951c1c01d962441f7e2193d9";
const MANIFEST: &str = "northstar-field-review-v2";
const POLICY: &str = "data/northstar-save-compatibility.json";

/// Composes the five field revisions on the verified raw Lakeside pack.
///
/// Gives the candidate a separate save namespace and removes the inherited
/// public-save migration policy. Does not select, deploy, or read player saves.
///
/// # Errors
/// Rejects a different parent, any leaf preimage failure, unrelated changes, or
/// invalid compiled content. The borrowed parent remains unchanged on failure.
pub fn build_northstar_field_review(base: &LoadedCompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.save_modpack_identity()?.hash() == PARENT_SHA256,
        "field review requires the verified raw Lakeside parent"
    );
    let original = base.pack();
    let mut pack = original.clone();
    crate::northstar_river_terrace::install(&mut pack).context("river terrace")?;
    crate::northstar_mine_return::install(&mut pack).context("mine return drift")?;
    crate::minnesota_hidden_route_clues::install_minnesota_hidden_route_clues(&mut pack)
        .context("route discoveries")?;
    crate::northstar_route_challenges::install_northstar_route_challenges(&mut pack)
        .context("route trainer challenges")?;
    crate::northstar_town_responses::install(&mut pack).context("town return replies")?;
    ensure!(
        pack.report == original.report,
        "leaf changed compile report"
    );
    pack.report.manifests.push(MANIFEST.into());
    ensure!(
        pack.runtime_files.remove(POLICY).is_some(),
        "Lakeside save policy missing"
    );
    ensure!(
        pack.runtime_modpack_id()? == format!("{}+{MANIFEST}", original.runtime_modpack_id()?),
        "private review save namespace must extend Lakeside exactly once"
    );
    field_review_manifest(original, &pack)?;
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

/// Audits composition boundaries and returns the actual changed-map manifest.
///
/// # Errors
/// Rejects changes outside the five authored lanes, including species, audio,
/// encounters, map topology, unrelated trainers, or non-clue runtime art.
pub fn field_review_manifest(before: &CompiledGamePack, after: &CompiledGamePack) -> Result<Value> {
    let clues: Vec<Value> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/discoveries/route-07-11-clues-v01/caches.json"
    ))?;
    let challenges: Vec<Value> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/route-challenges-v01/trainers.json"
    ))?;
    let mut expected = BTreeSet::from(["MnR03C00R00", "MnR09Cave1F", "MnStCloud", "MnDuluth"]);
    for entry in clues.iter().chain(&challenges) {
        expected.insert(entry["map"].as_str().context("owned map")?);
    }
    ensure!(
        before.data.maps.keys().eq(after.data.maps.keys()),
        "map set changed"
    );
    let mut changed_maps = Vec::new();
    for (name, old) in &before.data.maps {
        let new = &after.data.maps[name];
        if old == new {
            continue;
        }
        ensure!(
            expected.contains(name.as_str()),
            "unowned map changed: {name}"
        );
        ensure!(
            old.attributes == new.attributes && old.events == new.events,
            "map dimensions, entrances or events changed: {name}"
        );
        let a = serde_json::to_value(old)?;
        let b = serde_json::to_value(new)?;
        let fields: Vec<_> = a
            .as_object()
            .context("map object")?
            .keys()
            .filter(|key| a[*key] != b[*key])
            .cloned()
            .collect();
        let terrain = old
            .blocks
            .iter()
            .zip(&new.blocks)
            .filter(|(a, b)| a != b)
            .count();
        changed_maps.push(json!({"map":name,"changed_fields":fields,"changed_metatiles":terrain}));
    }
    let actual: BTreeSet<_> = changed_maps
        .iter()
        .filter_map(|m| m["map"].as_str())
        .collect();
    ensure!(actual == expected, "missing authored map changes");
    let a = serde_json::to_value(&before.data)?;
    let b = serde_json::to_value(&after.data)?;
    let allowed = [
        "maps",
        "map_scripts",
        "map_blocks",
        "npcs",
        "tilesets",
        "trainers",
    ];
    let mut changed_data_fields = Vec::new();
    for (key, value) in a.as_object().context("game data object")? {
        if value != &b[key] {
            ensure!(
                allowed.contains(&key.as_str()),
                "unowned game data changed: {key}"
            );
            changed_data_fields.push(key.clone());
        }
    }
    let trainers: BTreeSet<_> = challenges.iter().filter_map(|t| t["id"].as_str()).collect();
    ensure!(
        before
            .data
            .trainers
            .trainers
            .keys()
            .eq(after.data.trainers.trainers.keys()),
        "trainer set changed"
    );
    for (id, old) in &before.data.trainers.trainers {
        ensure!(
            trainers.contains(id.as_str()) || after.data.trainers.trainers.get(id) == Some(old),
            "unrelated trainer changed: {id}"
        );
    }
    let banks: BTreeSet<_> = clues
        .iter()
        .map(|c| {
            before.data.maps[c["map"].as_str().expect("validated map")]
                .attributes
                .tileset_name
                .clone()
        })
        .collect();
    let mut allowed_files = BTreeSet::from([POLICY.to_owned()]);
    for bank in &banks {
        for path in [
            format!("data/tilesets/{bank}.json"),
            format!("data/tilesets/{bank}_metatiles.bin"),
            format!("data/tilesets/{bank}_palette_map.json"),
            format!("gfx/tilesets/{bank}.2bpp"),
            format!("gfx/tilesets/{bank}.png"),
        ] {
            allowed_files.insert(path);
        }
    }
    let files: BTreeSet<_> = before
        .runtime_files
        .keys()
        .chain(after.runtime_files.keys())
        .collect();
    let mut changed_runtime_files = Vec::new();
    for path in files {
        if before.runtime_files.get(path) != after.runtime_files.get(path) {
            ensure!(
                allowed_files.contains(path),
                "unrelated runtime file changed: {path}"
            );
            changed_runtime_files.push(path);
        }
    }
    for (bank, atlas) in &before.data.tilesets {
        ensure!(
            banks.contains(bank) || after.data.tilesets.get(bank) == Some(atlas),
            "unrelated tile bank changed: {bank}"
        );
    }
    ensure!(
        before.audio_manifest == after.audio_manifest
            && before.compiled_audio == after.compiled_audio
            && before.audio_compression == after.audio_compression,
        "audio changed"
    );
    ensure!(
        !after.runtime_files.contains_key(POLICY),
        "private review must not inherit public save migration"
    );
    Ok(
        json!({"schema_version":1,"review_manifest":MANIFEST,"parent_sha256":PARENT_SHA256,
        "parent_content_identity":before.identity()?.content_hash,
        "save_id":after.runtime_modpack_id()?, "map_count":after.data.maps.len(),
        "changed_maps":changed_maps,"changed_data_fields":changed_data_fields,
        "changed_runtime_files":changed_runtime_files,
        "future_saved_terrain_review":["MnR03C00R00","MnR09Cave1F"],
        "selected":false,"deployed":false}),
    )
}
