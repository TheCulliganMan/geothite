//! Publish the reviewed three warm Superior headlands without replaying stale events.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const DELTA_SHA: &str = "e81a36e944296934fab730e9060becb69584936660e3705dbfce102743833bbb";

#[derive(Deserialize)]
struct Cell {
    metatile: [u16; 2],
    before: String,
    after: String,
}

/// Applies the frozen 179-cell native shoreline review to compact warm R11.
///
/// Existing actors, events, scripts, native boundary cells, connections, art,
/// audio and unrelated terrain remain exact. All old cell preimages must match;
/// disjoint changes elsewhere in an approved compact parent are retained.
/// This bounded operation does not replay stale route dossiers or claim earned
/// progression. The caller must inspect native views and exercise the controller.
///
/// # Errors
/// Rejects an unverified/noncompact parent, a changed frozen delta, duplicate or
/// out-of-bounds cells, changed native preimages, event anchors, shared block
/// labels, unsafe publication scope, or invalid derived output.
pub fn build_north_shore_coves_pack(
    base: &CompiledGamePack,
    delta_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        format!("{:x}", Sha256::digest(delta_json)) == DELTA_SHA,
        "reviewed coastal delta required"
    );
    let cells: Vec<Cell> = serde_json::from_str(delta_json)?;
    ensure!(cells.len() == 179, "reviewed coastal cell count");
    let mut output = base.clone();
    let mut positions = BTreeSet::new();
    let mut touched = BTreeSet::new();
    for cell in cells {
        let [x, y] = cell.metatile;
        ensure!(positions.insert([x, y]), "duplicate coastal cell");
        ensure!(
            (18..=37).contains(&x)
                && ((29..=43).contains(&y) || (64..=71).contains(&y) || (84..=92).contains(&y)),
            "coastal cell outside reviewed capes"
        );
        let name = format!("MnR11C{:02}R{:02}", x / 32, y / 24);
        let module = output
            .data
            .maps
            .get_mut(&name)
            .context("existing coastal chunk")?;
        ensure!(
            module.attributes.tileset_name == "mn_north_shore",
            "warm native coast required"
        );
        let [lx, ly] = [x % 32, y % 24];
        ensure!(
            (1..55).contains(&x) && (1..95).contains(&y),
            "preserve native route outer boundary cells"
        );
        ensure!(
            module
                .objects
                .iter()
                .all(|o| o.x / 2 != lx || o.y / 2 != ly)
                && module
                    .events
                    .warps
                    .iter()
                    .all(|w| w.x / 2 != lx || w.y / 2 != ly)
                && module
                    .events
                    .bg_events
                    .iter()
                    .all(|e| e.x / 2 != lx || e.y / 2 != ly),
            "coastal delta touches native event anchor at {x},{y}"
        );
        let index = usize::from(ly * module.attributes.width + lx);
        let before = u16::from_str_radix(&cell.before, 16)?;
        let after = u16::from_str_radix(&cell.after, 16)?;
        ensure!(
            module.blocks[index] == before && before != after,
            "coastal cell preimage drift at {x},{y}"
        );
        module.blocks[index] = after;
        touched.insert(name);
    }
    for name in &touched {
        let module = &output.data.maps[name];
        let label = module
            .attributes
            .blocks_label
            .as_ref()
            .context("coastal block label")?;
        ensure!(
            base.data.maps.iter().all(
                |(other, m)| other == name || m.attributes.blocks_label.as_ref() != Some(label)
            ),
            "shared coastal block label"
        );
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
    }
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    crate::validate_compact_superior_geography(&output)?;
    let mut normalized = output.clone();
    normalized.identity = base.identity.clone();
    for name in touched {
        let old = &base.data.maps[&name];
        normalized
            .data
            .maps
            .get_mut(&name)
            .context("normalized coastal chunk")?
            .blocks = old.blocks.clone();
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("old coastal block label")?;
        normalized
            .data
            .map_blocks
            .insert(label.clone(), base.data.map_blocks[label].clone());
    }
    ensure!(
        &normalized == base,
        "coastal publication changed unrelated content"
    );
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DELTA: &str =
        include_str!("../../../planning/minnesota/revisions/coast-coves-v01/delta.json");

    fn seal(pack: &mut CompiledGamePack) {
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )
        .expect("derive native fixture identity");
    }

    #[test]
    fn coast_coves_publish_and_reject_stale_terrain_actors_and_changed_perimeter() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("actual repository root");
        let base = crate::read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-r10-rock-gallery-v01.crystalpack"),
        )
        .expect("actual approved coast fixture");
        let output =
            build_north_shore_coves_pack(&base, DELTA).expect("bounded native cape publication");
        let source = crate::authored_route_layout::Layout::parse(include_str!(
            "../../../planning/minnesota/revisions/coast-coves-v01/MN_R11.candidate.layout.json"
        ))
        .expect("reviewed explicit coastal source")
        .compile()
        .expect("compile coastal source");
        for column in 0..2_u16 {
            for row in 0..4_u16 {
                let name = format!("MnR11C{column:02}R{row:02}");
                let module = &output.data.maps[&name];
                for y in 0..module.attributes.height {
                    for x in 0..module.attributes.width {
                        assert_eq!(
                            module.blocks[usize::from(y * module.attributes.width + x)],
                            source[usize::from((row * 24 + y) * 56 + column * 32 + x)],
                            "published cell must match explicit authored source"
                        );
                    }
                }
            }
        }
        assert_eq!(output.data.maps.len(), base.data.maps.len());
        assert_ne!(output.identity, base.identity);
        let cells: Vec<Cell> = serde_json::from_str(DELTA).expect("reviewed source cells");
        let [x, y] = cells[0].metatile;
        let name = format!("MnR11C{:02}R{:02}", x / 32, y / 24);
        let mut stale = base.clone();
        let module = stale.data.maps.get_mut(&name).expect("existing chunk");
        module.blocks[usize::from((y % 24) * module.attributes.width + x % 32)] = 5;
        let label = module
            .attributes
            .blocks_label
            .clone()
            .expect("native block label");
        let bytes = module
            .blocks
            .iter()
            .copied()
            .map(|v| u8::try_from(v).expect("native byte"))
            .collect::<Vec<_>>();
        stale
            .data
            .map_blocks
            .insert(label, crate::encode_base64_bytes(&bytes));
        seal(&mut stale);
        assert!(
            build_north_shore_coves_pack(&stale, DELTA)
                .expect_err("stale terrain rejected")
                .to_string()
                .contains("preimage")
        );
        let mut actor = base.clone();
        let object = actor
            .data
            .maps
            .get_mut("MnR11C00R01")
            .expect("existing cape chunk")
            .objects
            .first_mut()
            .expect("native trainer fixture");
        // This is ordinary Floor in the valid parent and belongs to the delta.
        // Relocate the existing actor within its owning script module, retaining
        // unique identity, party and flags so the anchor guard is exercised.
        object.x = 42;
        object.y = 18;
        seal(&mut actor);
        assert!(
            build_north_shore_coves_pack(&actor, DELTA)
                .expect_err("new actor anchor cannot be paved over")
                .to_string()
                .contains("event anchor"),
            "actor on an otherwise compatible new cape cell must be rejected"
        );
        let mut perimeter: serde_json::Value = serde_json::from_str(DELTA).expect("reviewed cells");
        perimeter[0]["metatile"] = serde_json::json!([55, 33]);
        assert!(
            build_north_shore_coves_pack(
                &base,
                &serde_json::to_string(&perimeter).expect("perimeter tampering fixture")
            )
            .expect_err("frozen perimeter rejected")
            .to_string()
            .contains("reviewed coastal delta")
        );
    }
}
