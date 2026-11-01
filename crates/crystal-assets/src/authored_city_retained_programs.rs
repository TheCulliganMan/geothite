//! Certify fixed Minneapolis gates before a same-size street recomposition.
use anyhow::{Context, Result, ensure};

use crate::authored_city_expansion_design::{AnchorKind, Expansion};
use crate::{CompiledGamePack, MapModule};

/// Keeps existing coordinate programs spatially exact while ordinary doors move.
///
/// This supports only the installed40×32 Minneapolis gate/cycling epoch, without
/// installing or reparsing programs. Both sources retain native art and extent.
/// Every trigger and targeted officer stays fixed with its surrounding terrain;
/// the three river block writes and all18 reload dispatches remain unchanged.
/// Water, bridges, buoys, ledges and the perimeter cannot be edited here.
///
/// # Errors
/// Rejects another epoch, movement/warp programs with encoded coordinates,
/// altered gate or cycling anchors, changed protected terrain or program drift.
pub(super) fn validate(
    pack: &CompiledGamePack,
    original: &MapModule,
    revised: &MapModule,
    plan: &Expansion,
) -> Result<()> {
    ensure!(
        original.id == "MnMinneapolis"
            && plan.city == original.id
            && plan.old_dimensions_metatiles == [40, 32]
            && plan.dimensions_metatiles == [40, 32]
            && plan.translation_core == [0, 0]
            && original.blocks.len() == 40 * 32
            && revised.blocks.len() == 40 * 32,
        "fixed-program streets require same-size40x32 Minneapolis"
    );
    let mut normalized = revised.clone();
    normalized.blocks.clone_from(&original.blocks);
    ensure!(
        normalized == *original,
        "fixed street programs or native attributes changed"
    );
    ensure!(
        original.scenes.scenes.is_empty()
            && original
                .script_object_commands
                .iter()
                .all(|c| c.x.is_none() && c.y.is_none()),
        "unsupported coordinate-bearing city program"
    );
    ensure!(
        original.script_map_commands.len() == 18
            && original
                .script_map_commands
                .iter()
                .enumerate()
                .all(|(i, c)| {
                    c.command == "reloadmapafterbattle"
                        && c.x.is_none()
                        && c.y.is_none()
                        && c.target_map.is_none()
                        && c.facing.is_none()
                        && c.map_setup.is_none()
                        && c.source_script == format!("MN_EXTRA_MPLS_CYCLE_{:02}", i + 1)
                        && c.command_index == 11
                }),
        "unsupported native Minneapolis map dispatch epoch"
    );
    ensure!(
        original.script_block_changes.len() == 3,
        "fixed north-pool block writes required"
    );
    for (i, change) in original.script_block_changes.iter().enumerate() {
        ensure!(
            change.source_script == "MnMPUpperPoolBuoyOpen"
                && change.command_index == i
                && change.x == 32 + u16::try_from(i)? * 2
                && change.y == 2
                && change.block_id == [88, 53, 89][i],
            "unsupported north-pool block write"
        );
        fixed_footprint(original, revised, change.x, change.y)?;
    }
    let collision = &pack
        .data
        .tilesets
        .get("mn_minneapolis_city")
        .context("native Minneapolis art")?
        .collision;
    for (i, (&before, &after)) in original.blocks.iter().zip(&revised.blocks).enumerate() {
        let x = i % 40;
        let y = i / 40;
        let protected = |block: u16| -> Result<bool> {
            let tokens = collision
                .get(&format!("{block:02x}"))
                .context("native street block")?;
            Ok(tokens.iter().any(|k| {
                k.starts_with("WATER")
                    || k.starts_with("BRIDGE_")
                    || k.starts_with("HOP_")
                    || k == "BUOY"
            }))
        };
        if x == 0 || x == 39 || y == 0 || y == 31 || protected(before)? || protected(after)? {
            ensure!(
                before == after,
                "fixed street revision changes retained water/edge at {x},{y}"
            );
        }
    }
    for event in &original.events.coord_events {
        fixed_footprint(original, revised, event.x, event.y)?;
    }
    for anchor in &plan.anchor_overrides {
        match anchor.kind {
            AnchorKind::Coordinate => ensure!(
                anchor.before == anchor.after,
                "fixed city coordinate events cannot move"
            ),
            AnchorKind::Object => {
                let object = original
                    .objects
                    .get(anchor.index)
                    .context("fixed city actor index")?;
                let identifier = object.object_identifier.as_deref().unwrap_or_default();
                let targeted = original.script_object_commands.iter().any(|c| {
                    c.object_id.as_deref() == Some(identifier)
                        || c.target_object_id.as_deref() == Some(identifier)
                });
                if identifier.starts_with("MN_EXTRA_MPLS_CYCLE_") || targeted {
                    ensure!(
                        anchor.before == anchor.after,
                        "targeted gate/cycling start cannot move"
                    );
                }
            }
            AnchorKind::Background | AnchorKind::Warp => {}
        }
    }
    Ok(())
}

fn fixed_footprint(original: &MapModule, revised: &MapModule, x: u16, y: u16) -> Result<()> {
    let bx = usize::from(x / 2);
    let by = usize::from(y / 2);
    ensure!(bx < 40 && by < 32, "fixed gate coordinate outside city");
    // Two metatiles cover the retained one/two-step PLAYER recoil movements,
    // both officers, their shoulders and the native buoy callback's landings.
    for y in by.saturating_sub(2)..=(by + 2).min(31) {
        for x in bx.saturating_sub(2)..=(bx + 2).min(39) {
            ensure!(
                original.blocks[y * 40 + x] == revised.blocks[y * 40 + x],
                "fixed gate footprint changed at {x},{y}"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::authored_city_recomposition_design::Recomposition;
    use crate::authored_route_layout::Layout;

    #[test]
    #[ignore = "requires CRYSTAL_CITY_STREETS_PARENT with the current Minneapolis gates/cyclists"]
    fn retained_minneapolis_programs_allow_streets_and_reject_gate_or_dispatch_drift() {
        let path = std::env::var_os("CRYSTAL_CITY_STREETS_PARENT")
            .expect("set CRYSTAL_CITY_STREETS_PARENT to the completed street parent");
        let pack = crate::read_verified_compiled_game_pack(Path::new(&path))
            .expect("verified installed Minneapolis parent");
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/revisions/minneapolis-mill-arts-v01");
        let layout_json = std::fs::read_to_string(source.join("after.layout.json"))
            .expect("runtime-loaded hand-authored street fixture");
        let contract_json = std::fs::read_to_string(source.join("recomposition.contract.json"))
            .expect("runtime-loaded native anchor fixture");
        let old = &pack.data.maps["MnMinneapolis"];
        let layout: Layout = serde_json::from_str(&layout_json).expect("street layout");
        let contract: Recomposition =
            serde_json::from_str(&contract_json).expect("street contract");
        let mut plan = contract.into_relocation();
        let mut new = old.clone();
        new.blocks = layout.compile().expect("complete native city terrain");
        super::validate(&pack, old, &new, &plan).expect("fixed-program street revision");
        let child = crate::recompose_authored_city(&pack, &layout_json, &contract_json)
            .expect("strict native publication of all moved facades");
        let city = &child.data.maps["MnMinneapolis"];
        assert_eq!(city.script_map_commands, old.script_map_commands);
        assert_eq!(city.script_block_changes, old.script_block_changes);
        assert_eq!(city.script_object_commands, old.script_object_commands);
        assert_eq!(city.events.coord_events, old.events.coord_events);
        assert_eq!(child.runtime_files, pack.runtime_files);
        assert_eq!(child.audio_manifest, pack.audio_manifest);
        assert_eq!(child.data.map_attributes, pack.data.map_attributes);

        for (index, expected) in [
            (40 * 17 + 16, "retained water/edge"),
            (40 + 21, "fixed gate footprint"),
        ] {
            let mut unsafe_map = new.clone();
            unsafe_map.blocks[index] = 1;
            let error = super::validate(&pack, old, &unsafe_map, &plan)
                .expect_err("changing buoy or recoil shoulders must fail");
            assert!(error.to_string().contains(expected), "{error:#}");
        }
        let cycling = plan
            .anchor_overrides
            .iter_mut()
            .find(|a| {
                a.kind == crate::authored_city_expansion_design::AnchorKind::Object && a.index == 28
            })
            .expect("first original cyclist");
        cycling.after[0] += 1;
        assert!(
            super::validate(&pack, old, &new, &plan)
                .expect_err("moving a cyclist start")
                .to_string()
                .contains("cycling start")
        );
        let cycling = plan
            .anchor_overrides
            .iter_mut()
            .find(|a| {
                a.kind == crate::authored_city_expansion_design::AnchorKind::Object && a.index == 28
            })
            .expect("first original cyclist");
        cycling.after[0] -= 1;
        let coordinate = plan
            .anchor_overrides
            .iter_mut()
            .find(|a| a.kind == crate::authored_city_expansion_design::AnchorKind::Coordinate)
            .expect("native fixed guard trigger");
        coordinate.after[1] += 1;
        assert!(
            super::validate(&pack, old, &new, &plan)
                .expect_err("moving a guard trigger")
                .to_string()
                .contains("coordinate events")
        );
        let mut drifted = old.clone();
        drifted.script_map_commands[0].command_index += 1;
        let mut drifted_candidate = drifted.clone();
        drifted_candidate.blocks.clone_from(&new.blocks);
        assert!(
            super::validate(&pack, &drifted, &drifted_candidate, &plan)
                .expect_err("battle-return dispatch drift")
                .to_string()
                .contains("dispatch epoch")
        );
    }
}
