//! Recompose installed city terrain while preserving every room and activity.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::authored_city_expansion_anchors as anchors;
use crate::authored_city_expansion_design::{AnchorKind, Expansion};
use crate::authored_city_recomposition_design::Recomposition;
use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Refines or tightens an installed major city's exterior and indexed positions.
///
/// The old module hash, all anchor indices and outdoor spawn records must match
/// the supplied contract. Original programs, event ordering, rooms, activities,
/// identities and unrelated content are retained. Only contracted terrain,
/// dimensions and positions, reciprocal seam offsets and outdoor recovery
/// positions change. An optional bounded St Anthony Falls recipe appends exact
/// native waterfall art in unused glyph storage; exact neighbor block revisions
/// and reciprocal Water lanes permit the river to continue across city edges.
/// A north river connection past these falls requires the fixed native buoy
/// checkpoint. Its appended scripts check the same two badges as the road and
/// restore the opened shoreline through a silent map tile callback.
/// This returns an in-memory pack and does not migrate saves,
/// write a pack, establish earned access or certify visual quality.
/// A same-size40×32 Minneapolis street revision may retain the existing north
/// pool gate and18 cyclist return dispatches only after their exact programs,
/// fixed anchors, recoil surroundings, water and boundary terrain are certified.
///
/// # Errors
/// Returns an error for an unsupported/stale city or extent, incomplete anchor
/// inventory, unsupported coordinate-bearing scripts, changed trigger collision,
/// unsafe geometry, invalid reciprocal connections or failed native verification.
/// Borrowed inputs are unchanged on failure.
pub fn recompose_authored_city(
    base: &CompiledGamePack,
    layout_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    let mut proposal: Recomposition =
        serde_json::from_str(contract_json).context("installed-city recomposition contract")?;
    ensure!(
        proposal.schema_version == 1
            && matches!(
                proposal.city.as_str(),
                "MnMinneapolis" | "MnStPaul" | "MnDuluth"
            ),
        "unsupported installed city recomposition"
    );
    let [width, height] = proposal.dimensions_metatiles;
    let same_extent = proposal.dimensions_metatiles == proposal.old_dimensions_metatiles;
    ensure!(
        (proposal.old_dimensions_metatiles == [64, 48] || same_extent)
            && (32..=64).contains(&width)
            && (24..=48).contains(&height)
            && (same_extent || proposal.dimensions_metatiles != [64, 48]),
        "city recomposition must refine its extent or tighten an installed64x48 exterior"
    );
    let original = base
        .data
        .maps
        .get(&proposal.city)
        .context("installed expanded city")?;
    ensure!(
        [original.attributes.width, original.attributes.height]
            == proposal.old_dimensions_metatiles,
        "installed city recomposition dimensions differ"
    );
    let hash = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
    );
    ensure!(
        hash == proposal.native_module_sha256,
        "installed city recomposition preimage differs: {hash}"
    );
    let value: serde_json::Value = serde_json::from_str(layout_json)?;
    ensure!(
        value["source_pixels_modified"] == false,
        "recomposition reuses installed city pixels"
    );
    let layout: Layout = serde_json::from_value(value).context("explicit compact city layout")?;
    ensure!(
        layout.schema_version == 2
            && layout.id == proposal.city
            && layout.dimensions_metatiles == proposal.dimensions_metatiles
            && layout.compiled_tileset == original.attributes.tileset_name,
        "recomposition layout identity/extent/art differs"
    );
    let waterfall = proposal.waterfall_block.take();
    let water_checkpoint = proposal.north_water_checkpoint;
    let seams = std::mem::take(&mut proposal.water_seams);
    let neighbors = std::mem::take(&mut proposal.neighbor_terrain);
    ensure!(
        waterfall.is_none() || proposal.city == "MnMinneapolis",
        "St Anthony Falls belongs to Minneapolis"
    );
    ensure!(
        waterfall.is_none()
            || !seams.iter().any(|seam| seam.direction == "north")
            || water_checkpoint,
        "connected St Anthony Falls needs the north river badge checkpoint"
    );
    let mut plan = proposal.into_relocation();
    validate_inventory(base, original, &plan)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut city = original.clone();
    city.attributes.width = width;
    city.attributes.height = height;
    city.blocks = layout.compile()?;
    if original.id == "MnMinneapolis"
        && same_extent
        && (!original.script_map_commands.is_empty() || !original.script_block_changes.is_empty())
    {
        anchors::relocate_with_fixed_programs(base, original, &mut city, &plan)?;
    } else {
        anchors::relocate(&mut city, &plan)?;
    }
    let mut pack = if let Some(block) = waterfall {
        crate::authored_city_recomposition_art::waterfall(base, block)?
    } else {
        base.clone()
    };
    crate::authored_city_recomposition_water::revise_neighbors(
        &mut pack, &plan, &seams, &neighbors,
    )?;
    let retained_water = seams
        .iter()
        .filter(|seam| {
            !plan
                .connections
                .iter()
                .any(|link| link.direction == seam.direction && link.target == seam.target)
        })
        .map(|seam| (seam.direction.as_str(), seam.target.as_str()))
        .collect();
    anchors::connections_with_retained_water(&mut pack, &mut city, &plan, &retained_water)?;
    anchors::spawns(&mut pack, &plan)?;
    crate::authored_city_expansion_geometry::validate_relocations(base, original, &pack, &city)?;
    for (before, after) in original.events.bg_events.iter().zip(&city.events.bg_events) {
        ensure!(
            crate::authored_town_geometry::kind(base, original, [before.x, before.y])?
                == crate::authored_town_geometry::kind(&pack, &city, [after.x, after.y])?,
            "recomposed city background {} changed its native standing collision",
            before.script
        );
    }
    ensure!(
        city.objects
            .iter()
            .map(|object| [object.x, object.y])
            .collect::<BTreeSet<_>>()
            .len()
            == city.objects.len(),
        "recomposed city objects overlap"
    );
    if water_checkpoint {
        crate::authored_city_recomposition_gate::install(&pack, &mut city)?;
    }
    authoring::refresh_event_section(&mut city);
    crate::authored_town_publication::replace(&mut pack, city)?;
    anchors::validate_connections(&pack, &plan)?;
    let water_edges =
        crate::authored_city_recomposition_water::validate_seams(&pack, &plan, &seams)?;
    crate::authored_city_expansion_geometry::validate_with_water_edges(&pack, &plan, &water_edges)?;
    for spawn in &plan.spawn_overrides {
        plan.arrival_core = spawn.after;
        crate::authored_city_expansion_geometry::validate_with_water_edges(
            &pack,
            &plan,
            &water_edges,
        )
        .context("recomposed outdoor recovery must reach the city")?;
    }
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

fn anchor_key(kind: AnchorKind, index: usize) -> (u8, usize) {
    let category = match kind {
        AnchorKind::Object => 0,
        AnchorKind::Background => 1,
        AnchorKind::Coordinate => 2,
        AnchorKind::Warp => 3,
    };
    (category, index)
}

fn validate_inventory(base: &CompiledGamePack, city: &MapModule, plan: &Expansion) -> Result<()> {
    let mut expected = BTreeSet::new();
    for (kind, count) in [
        (AnchorKind::Object, city.objects.len()),
        (AnchorKind::Background, city.events.bg_events.len()),
        (AnchorKind::Coordinate, city.events.coord_events.len()),
        (AnchorKind::Warp, city.events.warps.len()),
    ] {
        expected.extend((0..count).map(|index| anchor_key(kind, index)));
    }
    let actual = plan
        .anchor_overrides
        .iter()
        .map(|anchor| anchor_key(anchor.kind, anchor.index))
        .collect::<BTreeSet<_>>();
    ensure!(
        actual == expected && actual.len() == plan.anchor_overrides.len(),
        "recomposition requires every original indexed anchor exactly once"
    );
    let expected = base
        .data
        .runtime_spawn_points
        .values()
        .filter(|spawn| spawn.map_name == city.id)
        .map(|spawn| spawn.identifier)
        .collect::<BTreeSet<_>>();
    let actual = plan
        .spawn_overrides
        .iter()
        .map(|spawn| spawn.identifier)
        .collect::<BTreeSet<_>>();
    ensure!(
        actual == expected && actual.len() == plan.spawn_overrides.len(),
        "recomposition requires every outdoor city spawn exactly once"
    );
    Ok(())
}
