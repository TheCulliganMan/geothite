//! Integrate the reviewed cave layouts with the published Pokédex release.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, LoadedCompiledGamePack};

const POLICY: &str = "data/northstar-save-compatibility.json";
const MANIFEST: &str = "northstar-cave-audit-v5";
const LIVE_ARCHIVES: [&str; 2] = [
    "9415b9ae8def05303e8cd07ade5525c9c6f92c8a775498cb69e67975a19e7f13",
    "3f1df5134910d4c8b32e31428a668736ca8fbbc8b05e7c240865abcf6c7276d8",
];
const REVIEW_ARCHIVE: &str = "63769b0e94af05fd653699486ffe26f7c8815083af0587e1162160ed0644e56e";
const ARRIVALS: [(&str, [u16; 2], &str); 19] = [
    ("MnR01Cave1F", [5, 20], "up"),
    ("MnDriftlessLampGallery", [16, 25], "up"),
    ("MnR03Cave1F", [9, 26], "up"),
    ("MnR05RockCut", [5, 26], "up"),
    ("MnR13Cave1F", [7, 20], "up"),
    ("MnR17Cave1F", [7, 24], "up"),
    ("MnR20Cave1F", [3, 16], "right"),
    ("MnR12Cave1F", [7, 4], "down"),
    ("MnR10Cave1F", [5, 24], "up"),
    ("MnInnerBasin", [4, 27], "up"),
    ("MnR09Cave1F", [5, 16], "up"),
    ("MnIronDrillApproach", [6, 29], "up"),
    ("MnIronHaulage", [12, 15], "up"),
    ("MnIronSump", [8, 23], "up"),
    ("MnIronVentilation", [10, 19], "left"),
    ("MnIronPumpGallery", [8, 5], "down"),
    ("MnIronDrillCore", [12, 25], "up"),
    ("MnVoyageursIce1F", [5, 5], "right"),
    ("MnVoyageursIceB1F", [5, 4], "down"),
];

/// Returns the raw and hosted editions, with the reviewed caves and a release
/// policy for existing saves. The hosted edition uses the server's usual
/// composition order and is checked against the exact public hosted parent.
///
/// # Errors
///
/// Rejects stale inputs, invalid arrivals, and unrelated hosted changes.
pub fn build_northstar_cave_release(
    raw_parent: &LoadedCompiledGamePack,
    hosted_parent: &LoadedCompiledGamePack,
    reviewed: &LoadedCompiledGamePack,
) -> Result<[CompiledGamePack; 2]> {
    let sources = [raw_parent, hosted_parent]
        .into_iter()
        .zip(LIVE_ARCHIVES)
        .map(|(pack, expected)| {
            let identity = pack.save_modpack_identity()?;
            ensure!(
                identity.hash() == expected,
                "unexpected live Pokédex archive"
            );
            Ok(json!({
                "modpack_id": identity.id(),
                "modpack_hash": identity.hash(),
                "content_hash": pack.pack_content_hash()?,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        reviewed.save_modpack_identity()?.hash() == REVIEW_ARCHIVE,
        "expected the visually reviewed cave archive"
    );
    let inherited = raw_parent
        .pack()
        .runtime_files
        .get(POLICY)
        .context("missing live save policy")?;
    ensure!(
        hosted_parent.pack().runtime_files.get(POLICY) == Some(inherited),
        "live editions disagree on save policy"
    );
    let inherited: Value = serde_json::from_slice(inherited)?;
    ensure!(
        inherited["schema_version"] == 11,
        "expected the Pokédex save policy"
    );
    let maps = ARRIVALS.map(|(name, [x, y], facing)| {
        json!({"map_id": name, "arrival": {"x": x, "y": y}, "facing": facing})
    });
    let policy = serde_json::to_vec(&json!({
        "schema_version": 12,
        "sources": sources,
        "inherited": inherited,
        "reentry": {"policy": "minnesota-cave-audit-v5", "maps": maps},
    }))?;

    let mut raw = raw_parent.pack().clone();
    install_review(&mut raw, reviewed.pack(), &policy)?;
    raw.report.manifests.push(MANIFEST.into());
    refresh_identity(&mut raw)?;
    let encounters = crate::build_all_pokemon_modpack(&raw)?;
    let battles = crate::build_default_battle_modpack(&encounters)?;
    let clock = crate::build_realtime_clock_modpack(&battles)?;
    let hosted = crate::build_player_customization_modpack(&clock)?;

    let mut expected = hosted_parent.pack().clone();
    install_review(&mut expected, reviewed.pack(), &policy)?;
    let old_manifests = &expected.report.manifests;
    let (_, tail) = old_manifests
        .split_at_checked(raw_parent.pack().report.manifests.len())
        .context("hosted manifest history is shorter than raw")?;
    ensure!(
        old_manifests.starts_with(&raw_parent.pack().report.manifests)
            && hosted
                .report
                .manifests
                .strip_prefix(raw.report.manifests.as_slice())
                == Some(tail),
        "unexpected hosted overlay order"
    );
    expected
        .report
        .manifests
        .clone_from(&hosted.report.manifests);
    refresh_identity(&mut expected)?;
    ensure!(
        hosted == expected,
        "hosted cave release changed unrelated data"
    );
    Ok([raw, hosted])
}

fn install_review(
    target: &mut CompiledGamePack,
    source: &CompiledGamePack,
    policy: &[u8],
) -> Result<()> {
    let mut tilesets = BTreeSet::new();
    for (name, arrival, _) in ARRIVALS {
        let map = source
            .data
            .maps
            .get(name)
            .context("missing reviewed cave")?;
        ensure!(
            crate::authored_town_geometry::kind(source, map, arrival)? == "FLOOR"
                && map
                    .objects
                    .iter()
                    .all(|actor| [actor.x, actor.y] != arrival),
            "invalid release arrival for {name}"
        );
        tilesets.insert(map.attributes.tileset_name.as_str());
        crate::authored_town_publication::replace(target, map.clone())?;
    }
    for name in tilesets {
        let bank = source
            .data
            .tilesets
            .get(name)
            .context("missing reviewed tileset")?;
        target.data.tilesets.insert(name.into(), bank.clone());
        for asset in [
            format!("data/tilesets/{name}.json"),
            format!("data/tilesets/{name}_metatiles.bin"),
            format!("data/tilesets/{name}_palette_map.json"),
            format!("gfx/tilesets/{name}.2bpp"),
            format!("gfx/tilesets/{name}.png"),
        ] {
            if let Some(bytes) = source.runtime_files.get(&asset) {
                target.runtime_files.insert(asset, bytes.clone());
            }
        }
    }
    target.runtime_files.insert(POLICY.into(), policy.to_vec());
    Ok(())
}

fn refresh_identity(pack: &mut CompiledGamePack) -> Result<()> {
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(pack)
}
