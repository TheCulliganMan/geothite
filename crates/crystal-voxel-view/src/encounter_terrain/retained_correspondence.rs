//! Prove walking evidence against the actual retained grid after camera scroll.
//! This only identifies shared source cells; it never rebuilds or moves terrain.
//! `terrain_tracking::refresh_animation` stamps the current live viewport's
//! revision onto the retained frame without replacing its grid. An earlier
//! witness can therefore have a different revision while describing identical
//! source cells. Exact overlapping source/priority and proven art correspondence,
//! not revision equality, are required. Different authored phases are equivalent
//! only with an exact shared family and verified geometry-invariant built profiles.
use bevy::prelude::{Handle, IVec2, Image, UVec2, Vec2};
use crystal_render_api::{VisualBattleTerrainEvidence, VisualTile, VisualWorldFrame};
use std::{collections::HashSet, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Correspondence {
    /// Built-grid center in the witnessed frame's visual coordinates (+Y north).
    pub(super) center: Vec2,
    /// Shared original-map source 8px cells, northwest inclusive/southeast exclusive.
    pub(super) min: IVec2,
    pub(super) max: IVec2,
}

#[cfg(test)]
fn resolve(
    evidence: &VisualBattleTerrainEvidence,
    built: &VisualWorldFrame,
) -> Result<Correspondence, &'static str> {
    resolve_with_profiles(evidence, built, None)
}

/// The caller restricts this relaxation to walking and separately checks the
/// original atlas provenance. Profiles must come from the actual completed
/// build; absent provenance retains strict texture equality.
pub(super) fn resolve_with_profiles(
    evidence: &VisualBattleTerrainEvidence,
    built: &VisualWorldFrame,
    profiles: Option<&crate::live_profiles::Document>,
) -> Result<Correspondence, &'static str> {
    if !built.active {
        return Err("retained walking terrain is inactive");
    }
    if evidence.map_id.is_empty() || evidence.map_id != built.map_id {
        return Err("retained walking terrain map identity does not match");
    }
    if evidence.source_map_size_core_tiles != built.source_map_size_core_tiles {
        return Err("retained walking terrain source extent does not match");
    }
    let map = evidence
        .source_map_size_core_tiles
        .ok_or("retained walking terrain source extent is unavailable")?;
    if map.min_element() == 0 || map.max_element() > i32::MAX as u32 / 2 {
        return Err("retained walking terrain source extent is invalid");
    }
    if evidence.map_texture == Handle::<Image>::default()
        || built.map_texture == Handle::<Image>::default()
    {
        return Err("retained walking terrain atlas handle is unavailable");
    }
    if !evidence.center.is_finite() || !built.center.is_finite() {
        return Err("retained walking terrain center is nonfinite");
    }
    if !evidence.tile_size.is_finite()
        || !built.tile_size.is_finite()
        || !evidence.viewport_size.is_finite()
        || !built.viewport_size.is_finite()
    {
        return Err("retained walking terrain dimensions are nonfinite");
    }
    if evidence.tile_size.min_element() <= 0.0
        || built.tile_size.min_element() <= 0.0
        || evidence.viewport_size.min_element() <= 0.0
        || built.viewport_size.min_element() <= 0.0
    {
        return Err("retained walking terrain dimensions are nonpositive");
    }
    if evidence.grid_size != built.grid_size {
        return Err("retained walking terrain grid size does not match");
    }
    if evidence.tile_size != built.tile_size {
        return Err("retained walking terrain cell size does not match");
    }
    if evidence.viewport_size != built.viewport_size {
        return Err("retained walking terrain viewport size does not match");
    }
    let invalid_grid = "retained walking terrain grid size is invalid";
    let grid = IVec2::new(
        i32::try_from(evidence.grid_size.x).map_err(|_| invalid_grid)?,
        i32::try_from(evidence.grid_size.y).map_err(|_| invalid_grid)?,
    );
    let terrain_size = evidence.grid_size.as_vec2() * evidence.tile_size;
    if grid.min_element() <= 0 {
        return Err(invalid_grid);
    }
    if !terrain_size.is_finite() {
        return Err("retained walking terrain grid dimensions are nonfinite");
    }
    if evidence.viewport_size.cmpgt(terrain_size).any() {
        return Err("retained walking terrain viewport exceeds its grid");
    }
    // Exactly terrain_tracking::can_reuse's half-halo allowance. Checked
    // arithmetic also rejects malformed origins before any cell indexing.
    let invalid_origin = "retained walking terrain grid coordinates overflow";
    let delta = IVec2::new(
        evidence
            .grid_origin
            .x
            .checked_sub(built.grid_origin.x)
            .ok_or(invalid_origin)?,
        evidence
            .grid_origin
            .y
            .checked_sub(built.grid_origin.y)
            .ok_or(invalid_origin)?,
    );
    let visible = (evidence.viewport_size / evidence.tile_size)
        .ceil()
        .as_ivec2();
    let allowance = ((grid - visible) / 4).max(IVec2::ZERO);
    if delta.cmplt(-allowance).any() || delta.cmpgt(allowance).any() {
        return Err("walking evidence exceeds retained terrain scroll allowance");
    }
    let end = |origin: IVec2| -> Result<IVec2, &'static str> {
        Ok(IVec2::new(
            origin.x.checked_add(grid.x).ok_or(invalid_origin)?,
            origin.y.checked_add(grid.y).ok_or(invalid_origin)?,
        ))
    };
    let overlap_min = built.grid_origin.max(evidence.grid_origin);
    let overlap_max = end(built.grid_origin)?.min(end(evidence.grid_origin)?);
    let min = overlap_min.max(IVec2::ZERO);
    let max = overlap_max.min(map.as_ivec2() * 2);
    if max.cmple(min).any() {
        return Err("retained walking terrain has no shared original-map cells");
    }
    let old = indexed_tiles(&built.tiles, built.grid_size)?;
    let live = indexed_tiles(&evidence.tiles, evidence.grid_size)?;
    let width = grid.x as usize;
    let index = |point: IVec2, origin: IVec2| {
        let local = point - origin;
        local.y as usize * width + local.x as usize
    };
    let mut checked_families = HashSet::new();
    // Check the whole overlap, including its halo, rather than blessing
    // mismatched retained geometry just because its cell is outside acreage.
    for y in overlap_min.y..overlap_max.y {
        for x in overlap_min.x..overlap_max.x {
            let point = IVec2::new(x, y);
            let a = old[index(point, built.grid_origin)];
            let b = live[index(point, evidence.grid_origin)];
            if a.source != b.source
                || a.priority != b.priority
                || !phase_compatible_textures(&built.map_id, a, b, profiles, &mut checked_families)
            {
                #[cfg(not(target_arch = "wasm32"))]
                trace_first_mismatch(evidence, built, &old, &live, overlap_min, overlap_max);
                return Err("retained terrain cells do not match walking evidence");
            }
        }
    }
    let offset = -delta.as_vec2() * evidence.tile_size;
    let center = evidence.center + Vec2::new(offset.x, -offset.y);
    if !center.is_finite() {
        return Err("retained walking terrain aligned center is nonfinite");
    }
    Ok(Correspondence { center, min, max })
}

/// Only the current authored program families (at most 32 frames) can prove a
/// phase change. Equal textures keep the old strict path without needing a family.
fn phase_compatible_textures(
    map: &str,
    a: &VisualTile,
    b: &VisualTile,
    profiles: Option<&crate::live_profiles::Document>,
    checked: &mut HashSet<(usize, usize)>,
) -> bool {
    if a.texture == b.texture {
        return true;
    }
    let Some(profiles) = profiles else {
        return false;
    };
    if !matches!(
        crate::profile::shape_for_source_on_map(map, &a.source),
        crate::profile::CellShape::Flat
            | crate::profile::CellShape::Water
            | crate::profile::CellShape::Waterfall
    ) {
        return false;
    }
    // A live drawing can override otherwise flat cells or derive its mask
    // from a ground sample. Conservatively deny any matching profile reference,
    // even when the complete drawing may not be placed in this particular grid.
    if profiles.objects.iter().any(|object| {
        object.tileset == a.source.tileset_id.as_ref()
            && object.map.as_deref().is_none_or(|id| id == map)
            && object
                .maps
                .as_ref()
                .is_none_or(|ids| ids.iter().any(|id| id == map))
            && (object.ground == a.source.tile_index
                || object
                    .tiles
                    .iter()
                    .flatten()
                    .any(|&id| id == a.source.tile_index))
    }) {
        return false;
    }
    let (Some(first), Some(second)) = (&a.animation_frames, &b.animation_frames) else {
        return false;
    };
    if first.is_empty() || first.len() > 32 || second.is_empty() || second.len() > 32 {
        return false;
    }
    // The slices are held by the two frozen frame inputs for this entire call.
    // Validate each shared pair once; never allocate/copy frame handles per cell.
    let key = (
        Arc::as_ptr(first) as *const () as usize,
        Arc::as_ptr(second) as *const () as usize,
    );
    if !checked.contains(&key) {
        if !(Arc::ptr_eq(first, second) || first.as_ref() == second.as_ref())
            || first
                .iter()
                .any(|handle| *handle == Handle::<Image>::default())
        {
            return false;
        }
        checked.insert(key);
    }
    first.contains(&a.texture) && first.contains(&b.texture)
}

/// One opt-in report per native process, even if a rejected encounter retries
/// every render tick. Raw difference classification is diagnostic only; failed
/// proofs still reject, and neither source evidence nor retained cells change.
#[cfg(not(target_arch = "wasm32"))]
fn trace_first_mismatch(
    evidence: &VisualBattleTerrainEvidence,
    built: &VisualWorldFrame,
    old: &[&VisualTile],
    live: &[&VisualTile],
    min: IVec2,
    max: IVec2,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_none()
        || REPORTED.swap(true, Ordering::Relaxed)
    {
        return;
    }
    let width = built.grid_size.x as usize;
    let index = |point: IVec2, origin: IVec2| {
        let local = point - origin;
        local.y as usize * width + local.x as usize
    };
    let mut total = 0;
    let mut source = 0;
    let mut texture = 0;
    let mut priority = 0;
    let mut texture_only = 0;
    let mut samples = Vec::with_capacity(4);
    for y in min.y..max.y {
        for x in min.x..max.x {
            let point = IVec2::new(x, y);
            let a = old[index(point, built.grid_origin)];
            let b = live[index(point, evidence.grid_origin)];
            let changed_source = a.source != b.source;
            let changed_texture = a.texture != b.texture;
            let changed_priority = a.priority != b.priority;
            if changed_source || changed_texture || changed_priority {
                total += 1;
                source += usize::from(changed_source);
                texture += usize::from(changed_texture);
                priority += usize::from(changed_priority);
                texture_only +=
                    usize::from(changed_texture && !changed_source && !changed_priority);
                if samples.len() < 4 {
                    samples.push((point, a, b));
                }
            }
        }
    }
    eprintln!(
        "retained correspondence mismatch (one-shot): map={} map_extent={:?} grid={:?} overlap={min:?}..{max:?} built_origin={:?} witness_origin={:?} built_center={:?} witness_center={:?} built_revision={} witness_revision={} total={total} source={source} texture={texture} priority={priority} texture_only={texture_only}",
        evidence.map_id,
        evidence.source_map_size_core_tiles,
        built.grid_size,
        built.grid_origin,
        evidence.grid_origin,
        built.center,
        evidence.center,
        built.terrain_revision,
        evidence.terrain_revision,
    );
    for (point, a, b) in samples {
        let in_map = evidence.source_map_size_core_tiles.is_some_and(|size| {
            point.cmpge(IVec2::ZERO).all() && point.cmplt(size.as_ivec2() * 2).all()
        });
        eprintln!(
            "retained correspondence mismatch cell={point:?} original_map={in_map} built_source={:?} witness_source={:?} built_texture={:?} witness_texture={:?} built_priority={} witness_priority={}",
            a.source,
            b.source,
            a.texture.id(),
            b.texture.id(),
            a.priority,
            b.priority,
        );
    }
}

/// Grid order is not an API guarantee. Index explicit coordinates, rejecting
/// holes, duplicates and unavailable tile identities even outside the overlap.
fn indexed_tiles(tiles: &[VisualTile], size: UVec2) -> Result<Vec<&VisualTile>, &'static str> {
    let invalid = "retained walking terrain has incomplete or invalid cells";
    let width = usize::try_from(size.x).map_err(|_| invalid)?;
    let height = usize::try_from(size.y).map_err(|_| invalid)?;
    let count = width.checked_mul(height).ok_or(invalid)?;
    if count == 0 || count != tiles.len() {
        return Err(invalid);
    }
    let mut indexed = vec![None; count];
    for tile in tiles {
        if tile.column >= size.x
            || tile.row >= size.y
            || tile.texture == Handle::<Image>::default()
            || tile.source.tileset_id.is_empty()
            || tile.source.subtile_column >= 4
            || tile.source.subtile_row >= 4
        {
            return Err(invalid);
        }
        let slot = tile.row as usize * width + tile.column as usize;
        if indexed[slot].replace(tile).is_some() {
            return Err(invalid);
        }
    }
    indexed
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::VisualTileSource;
    use std::sync::Arc;

    fn frame(origin: IVec2) -> VisualWorldFrame {
        let mut frame = VisualWorldFrame {
            active: true,
            map_id: "Route29".into(),
            source_map_size_core_tiles: Some(UVec2::splat(16)),
            terrain_revision: 37,
            grid_origin: origin,
            grid_size: UVec2::splat(12),
            tile_size: Vec2::splat(32.0),
            viewport_size: Vec2::splat(128.0),
            center: Vec2::new(123.0, -87.0),
            map_texture: Handle::weak_from_u128(1),
            ..Default::default()
        };
        for row in 0..12 {
            for column in 0..12 {
                let point = origin + IVec2::new(column as i32, row as i32);
                let identity = ((point.y + 100) * 1000 + point.x + 100) as u32;
                frame.tiles.push(VisualTile {
                    animation_frames: None,
                    column,
                    row,
                    source: VisualTileSource {
                        tileset_id: "johto".into(),
                        metatile_id: (identity / 4) as u16,
                        subtile_column: point.x.rem_euclid(4) as u8,
                        subtile_row: point.y.rem_euclid(4) as u8,
                        tile_index: identity as u16,
                    },
                    texture: Handle::weak_from_u128(u128::from(identity) + 100),
                    priority: point.x.rem_euclid(3) == 0,
                });
            }
        }
        frame
    }

    fn evidence(frame: &VisualWorldFrame) -> VisualBattleTerrainEvidence {
        VisualBattleTerrainEvidence {
            map_id: frame.map_id.clone(),
            source_map_size_core_tiles: frame.source_map_size_core_tiles,
            terrain_revision: frame.terrain_revision,
            grid_origin: frame.grid_origin,
            grid_size: frame.grid_size,
            center: frame.center,
            viewport_size: frame.viewport_size,
            tile_size: frame.tile_size,
            map_texture: frame.map_texture.clone(),
            tiles: frame.tiles.clone().into(),
        }
    }

    fn phase_pair(
        tileset: &str,
        metatile: u16,
        art: u16,
    ) -> (VisualWorldFrame, VisualBattleTerrainEvidence) {
        let mut built = frame(IVec2::new(6, 5));
        let mut witness = evidence(&frame(IVec2::new(8, 5)));
        built.map_id = "UnionCave1F".into();
        witness.map_id = built.map_id.clone();
        let family: Arc<[Handle<Image>]> = (0..32)
            .map(|i| Handle::weak_from_u128(700_000 + i))
            .collect::<Vec<_>>()
            .into();
        let source = VisualTileSource {
            tileset_id: tileset.into(),
            metatile_id: metatile,
            subtile_column: 1,
            subtile_row: 1,
            tile_index: art,
        };
        // These slots represent the same absolute source cell (8,5).
        built.tiles[2].source = source.clone();
        built.tiles[2].texture = family[0].clone();
        built.tiles[2].animation_frames = Some(family.clone());
        let tile = &mut Arc::make_mut(&mut witness.tiles)[0];
        tile.source = source;
        tile.texture = family[17].clone();
        tile.animation_frames = Some(family);
        (built, witness)
    }

    #[test]
    fn retained_correspondence_accepts_only_proven_invariant_phase_families() {
        let profiles = crate::live_profiles::LiveProfiles::default();
        for (tileset, metatile, art) in [
            ("cave", 0x3e, 0x14),
            ("johto", 0x54, 0x14),
            ("cave", 0x2c, 0x40),
        ] {
            let (built, mut witness) = phase_pair(tileset, metatile, art);
            assert!(
                resolve(&witness, &built).is_err(),
                "missing actual build profiles"
            );
            let accepted =
                resolve_with_profiles(&witness, &built, Some(&profiles.document)).unwrap();
            assert_eq!(accepted.min, IVec2::new(8, 5));
            // Equivalent complete families need not share an allocation.
            let original = witness.tiles[0].animation_frames.as_ref().unwrap().clone();
            Arc::make_mut(&mut witness.tiles)[0].animation_frames = Some(original.to_vec().into());
            assert!(!Arc::ptr_eq(
                &original,
                witness.tiles[0].animation_frames.as_ref().unwrap()
            ));
            assert_eq!(
                resolve_with_profiles(&witness, &built, Some(&profiles.document)).unwrap(),
                accepted
            );
        }
        for (tileset, metatile, art) in [
            ("johto", 0, 3),
            ("forest", 0x20, 0x0c),
            ("cave", 0x18, 0x1d),
        ] {
            let (built, witness) = phase_pair(tileset, metatile, art);
            let empty_profiles = crate::live_profiles::Document::default();
            assert!(
                resolve_with_profiles(&witness, &built, Some(&empty_profiles)).is_err(),
                "geometry-derived {tileset}/{metatile}/{art}"
            );
        }
    }

    #[test]
    fn retained_correspondence_rejects_missing_malformed_and_different_phase_families() {
        let profiles = crate::live_profiles::Document::default();
        let (built, witness) = phase_pair("cave", 0x3e, 0x14);
        for case in 0..9 {
            let mut bad = witness.clone();
            let tile = &mut Arc::make_mut(&mut bad.tiles)[0];
            let mut family = tile.animation_frames.as_ref().unwrap().to_vec();
            match case {
                0 => tile.animation_frames = None,
                1 => tile.animation_frames = Some(Arc::from([])),
                2 => {
                    family[31] = Handle::default();
                    tile.animation_frames = Some(family.into());
                }
                3 => {
                    family.push(Handle::weak_from_u128(900_000));
                    tile.animation_frames = Some(family.into());
                }
                4 => {
                    tile.texture = family[1].clone();
                    family.truncate(16);
                    tile.animation_frames = Some(family.into());
                }
                5 => {
                    family.reverse();
                    tile.animation_frames = Some(family.into());
                }
                6 => tile.texture = Handle::weak_from_u128(900_001),
                7 => {
                    family[31] = Handle::weak_from_u128(900_002);
                    tile.animation_frames = Some(family.into());
                }
                _ => {
                    tile.texture = Handle::weak_from_u128(900_003);
                    tile.animation_frames =
                        Some(vec![built.tiles[2].texture.clone(), tile.texture.clone()].into());
                }
            }
            assert!(
                resolve_with_profiles(&bad, &built, Some(&profiles)).is_err(),
                "case {case}"
            );
        }
        let mut static_built = built.clone();
        static_built.tiles[2].animation_frames = None;
        assert!(resolve_with_profiles(&witness, &static_built, Some(&profiles)).is_err());
        let mut equal_texture = witness.clone();
        let tile = &mut Arc::make_mut(&mut equal_texture.tiles)[0];
        tile.texture = built.tiles[2].texture.clone();
        tile.animation_frames = None;
        assert!(
            resolve(&equal_texture, &static_built).is_ok(),
            "strict static equality remains sufficient"
        );
    }

    #[test]
    fn retained_correspondence_family_never_excuses_source_priority_or_palette_change() {
        let profiles = crate::live_profiles::Document::default();
        let (built, witness) = phase_pair("cave", 0x3e, 0x14);
        for case in 0..3 {
            let mut bad = witness.clone();
            let tile = &mut Arc::make_mut(&mut bad.tiles)[0];
            match case {
                0 => tile.source.metatile_id += 1,
                1 => tile.priority ^= true,
                _ => {
                    let other: Arc<[Handle<Image>]> = (0..32)
                        .map(|i| Handle::weak_from_u128(800_000 + i))
                        .collect::<Vec<_>>()
                        .into();
                    tile.texture = other[17].clone();
                    tile.animation_frames = Some(other);
                }
            }
            assert!(
                resolve_with_profiles(&bad, &built, Some(&profiles)).is_err(),
                "case {case}"
            );
        }
    }

    #[test]
    fn retained_correspondence_denies_actual_live_profile_drawings_and_ground_dependencies() {
        let (built, witness) = phase_pair("cave", 0x3e, 0x14);
        let mut profiles: crate::live_profiles::Document =
            serde_json::from_value(serde_json::json!({
                "objects": [{"name": "water override", "tileset": "cave", "map": "UnionCave1F",
                    "metatile": 62, "origin": [0, 0], "tiles": [[20]], "ground": 22,
                    "top_pixels": 0, "depth_pixels": 1.0}]
            }))
            .unwrap();
        assert!(resolve_with_profiles(&witness, &built, Some(&profiles)).is_err());
        profiles.objects[0].tiles = vec![vec![12]];
        profiles.objects[0].ground = 20;
        assert!(
            resolve_with_profiles(&witness, &built, Some(&profiles)).is_err(),
            "mask ground can affect geometry"
        );
        profiles.objects[0].map = Some("OtherMap".into());
        assert!(resolve_with_profiles(&witness, &built, Some(&profiles)).is_ok());
        profiles.objects[0].map = None;
        profiles.objects[0].maps = Some(vec!["UnionCave1F".into()]);
        assert!(resolve_with_profiles(&witness, &built, Some(&profiles)).is_err());
    }

    #[test]
    fn retained_correspondence_old_witness_survives_actual_shifted_phase_refresh() {
        use bevy::render::{
            render_asset::RenderAssetUsages,
            render_resource::{Extent3d, TextureDimension, TextureFormat},
        };
        let (mut built, witness) = phase_pair("cave", 0x3e, 0x14);
        let profiles = crate::live_profiles::Document::default();
        let family = built.tiles[2].animation_frames.as_ref().unwrap().clone();
        let mut live = frame(IVec2::new(7, 5));
        live.map_id = built.map_id.clone();
        live.terrain_revision += 1;
        live.tiles[1].source = built.tiles[2].source.clone();
        live.tiles[1].texture = family[31].clone();
        live.tiles[1].animation_frames = Some(family.clone());
        let image = |size: UVec2, color: [u8; 4]| {
            Image::new_fill(
                Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &color,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )
        };
        let mut images = bevy::prelude::Assets::<Image>::default();
        images.insert(
            built.map_texture.id(),
            image(built.grid_size * 8, [0, 0, 0, 255]),
        );
        images.insert(family[31].id(), image(UVec2::splat(8), [17, 31, 47, 255]));
        assert!(crate::terrain_tracking::can_reuse(&built, &live));
        assert!(
            !crate::terrain_tracking::refresh_animation(&mut built, &live, &mut images).unwrap()
        );
        assert_eq!(built.tiles[2].texture, family[31]);
        assert!(Arc::ptr_eq(
            built.tiles[2].animation_frames.as_ref().unwrap(),
            &family
        ));
        assert_ne!(built.terrain_revision, witness.terrain_revision);
        assert!(resolve_with_profiles(&witness, &built, Some(&profiles)).is_ok());
        let mut changed = witness.clone();
        Arc::make_mut(&mut changed.tiles)[0].priority ^= true;
        assert!(resolve_with_profiles(&changed, &built, Some(&profiles)).is_err());
    }

    #[test]
    fn retained_correspondence_aligns_all_scroll_directions_and_subtile_center() {
        let built = frame(IVec2::new(6, 5));
        for delta in [
            IVec2::ZERO,
            IVec2::X,
            IVec2::NEG_X,
            IVec2::Y,
            IVec2::NEG_Y,
            IVec2::ONE,
            -IVec2::ONE,
            IVec2::new(1, -1),
            IVec2::new(-1, 1),
        ] {
            let mut live = frame(built.grid_origin + delta * 2);
            live.center = Vec2::new(137.5, -42.25);
            live.map_texture = Handle::weak_from_u128(2);
            let result = resolve(&evidence(&live), &built).unwrap();
            let offset = crate::terrain_tracking::offset(&built, &live);
            assert_eq!(result.center, live.center + Vec2::new(offset.x, -offset.y));
            assert_eq!(result.min, built.grid_origin.max(live.grid_origin));
            assert_eq!(
                result.max,
                (built.grid_origin + IVec2::splat(12)).min(live.grid_origin + IVec2::splat(12))
            );
            assert!(crate::terrain_tracking::can_reuse(&built, &live));
        }
    }

    #[test]
    fn retained_correspondence_limits_support_to_both_grids_and_original_acreage() {
        let mut built = frame(IVec2::new(-2, -1));
        built.source_map_size_core_tiles = Some(UVec2::new(4, 5));
        let mut live = frame(IVec2::new(-1, -2));
        live.source_map_size_core_tiles = built.source_map_size_core_tiles;
        let result = resolve(&evidence(&live), &built).unwrap();
        assert_eq!(result.min, IVec2::ZERO);
        assert_eq!(result.max, IVec2::new(8, 10));
        let mut built = frame(IVec2::new(-11, 4));
        let live = frame(IVec2::new(-12, 4));
        assert!(
            resolve(&evidence(&live), &built).is_err(),
            "overlap is only halo"
        );
        built.grid_origin = IVec2::new(i32::MAX - 5, 4);
        let mut bad = evidence(&built);
        assert!(resolve(&bad, &built).is_err(), "grid end overflow");
        bad.grid_origin = IVec2::new(i32::MIN, 4);
        assert!(
            resolve(&bad, &built).is_err(),
            "origin subtraction overflow"
        );
    }

    #[test]
    fn retained_correspondence_requires_exact_cells_with_matching_or_different_revision() {
        let built = frame(IVec2::new(6, 5));
        let live = evidence(&frame(IVec2::new(8, 5)));
        for revision in [built.terrain_revision, built.terrain_revision + 1] {
            for case in 0..3 {
                let mut changed = live.clone();
                changed.terrain_revision = revision;
                match case {
                    0 => Arc::make_mut(&mut changed.tiles)[0].source.tile_index += 1,
                    1 => {
                        Arc::make_mut(&mut changed.tiles)[0].texture =
                            Handle::weak_from_u128(999999)
                    }
                    _ => Arc::make_mut(&mut changed.tiles)[0].priority ^= true,
                }
                assert!(
                    resolve(&changed, &built).is_err(),
                    "revision {revision}, case {case}"
                );
            }
        }
        // Newly exposed cells cannot authorize support in the old mesh.
        let mut changed = live.clone();
        Arc::make_mut(&mut changed.tiles)[11].source.tile_index += 1;
        let result = resolve(&changed, &built).unwrap();
        assert_eq!(result.max.x, 18);
    }

    #[test]
    fn retained_correspondence_accepts_previous_witness_after_real_refresh_revision_change() {
        let mut built = frame(IVec2::new(6, 5));
        let original_tiles = built.tiles.clone();
        let original_origin = built.grid_origin;
        let mut witnessed = frame(IVec2::new(7, 5));
        witnessed.terrain_revision = 38;
        let witness = evidence(&witnessed);
        let before = resolve(&witness, &built).unwrap();
        let mut live = frame(IVec2::new(8, 5));
        live.terrain_revision = 39;
        assert!(crate::terrain_tracking::can_reuse(&built, &live));
        // No image lookup is required: all overlapping textures are unchanged.
        let mut images = bevy::prelude::Assets::<Image>::default();
        assert!(
            !crate::terrain_tracking::refresh_animation(&mut built, &live, &mut images).unwrap()
        );
        assert_eq!(built.grid_origin, original_origin);
        assert_eq!(built.tiles, original_tiles);
        assert_eq!(built.terrain_revision, live.terrain_revision);
        assert_ne!(built.terrain_revision, witness.terrain_revision);
        assert_eq!(resolve(&witness, &built).unwrap(), before);
        for revision in [witness.terrain_revision, built.terrain_revision] {
            let mut changed = witness.clone();
            changed.terrain_revision = revision;
            Arc::make_mut(&mut changed.tiles)[0].source.tile_index += 1;
            assert!(resolve(&changed, &built).is_err());
        }
    }

    #[test]
    fn retained_correspondence_rejects_holes_duplicates_and_unavailable_cells() {
        let built = frame(IVec2::new(6, 5));
        let live = evidence(&frame(IVec2::new(8, 5)));
        for case in 0..7 {
            let mut changed = live.clone();
            match case {
                0 => changed.tiles = changed.tiles[..143].to_vec().into(),
                1 => Arc::make_mut(&mut changed.tiles)[1] = changed.tiles[0].clone(),
                2 => Arc::make_mut(&mut changed.tiles)[0].column = 12,
                3 => Arc::make_mut(&mut changed.tiles)[0].texture = Handle::default(),
                4 => Arc::make_mut(&mut changed.tiles)[0].source.tileset_id = "".into(),
                5 => Arc::make_mut(&mut changed.tiles)[0].source.subtile_row = 4,
                _ => changed.tiles = Arc::from([]),
            }
            assert!(resolve(&changed, &built).is_err(), "live case {case}");
            let mut bad_built = built.clone();
            bad_built.tiles = changed.tiles.to_vec();
            assert!(resolve(&live, &bad_built).is_err(), "built case {case}");
        }
        let mut reordered = live.clone();
        Arc::make_mut(&mut reordered.tiles).reverse();
        let mut reordered_built = built.clone();
        reordered_built.tiles.reverse();
        assert_eq!(
            resolve(&reordered, &reordered_built),
            resolve(&live, &built)
        );
    }

    #[test]
    fn retained_correspondence_rejects_incompatible_or_invalid_frame_geometry() {
        let built = frame(IVec2::new(6, 5));
        let live = evidence(&frame(IVec2::new(8, 5)));
        for case in 0..14 {
            let mut changed = live.clone();
            match case {
                0 => changed.map_id = "Route30".into(),
                1 => changed.source_map_size_core_tiles = None,
                2 => changed.source_map_size_core_tiles = Some(UVec2::ZERO),
                3 => changed.source_map_size_core_tiles = Some(UVec2::splat(u32::MAX)),
                4 => changed.grid_size.x += 1,
                5 => changed.tile_size.x += 1.0,
                6 => changed.viewport_size.y += 1.0,
                7 => changed.grid_origin.x += 1,
                8 => changed.center.x = f32::NAN,
                9 => changed.map_texture = Handle::default(),
                10 => changed.tile_size = Vec2::ZERO,
                11 => changed.viewport_size = Vec2::splat(f32::INFINITY),
                12 => changed.grid_size = UVec2::ZERO,
                _ => changed.viewport_size = Vec2::splat(1000.0),
            }
            assert!(resolve(&changed, &built).is_err(), "case {case}");
        }
        let mut inactive = built.clone();
        inactive.active = false;
        assert!(resolve(&live, &inactive).is_err());
    }
}
