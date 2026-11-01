//! Fit the exterior's upper eaves to the live playable roof, rather than
//! attaching a room-sized roof to the exterior sprite's much smaller plot.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TowerAttachment {
    pub anchor: Vec2,
    pub span: Vec2,
    pub eave: f32,
}

fn bounds(points: impl Iterator<Item = Vec3>) -> (Vec3, Vec3) {
    points.fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(low, high), p| (low.min(p), high.max(p)),
    )
}

/// Only the panorama copy changes. The ordinary exterior, source maps and
/// foreground roof (including every walking/support coordinate) stay exact.
pub(crate) fn prepare(
    source: &VisualWorldFrame,
    terrain: &mut TerrainMeshData,
    tower_core: IVec2,
) -> Option<TowerAttachment> {
    if source.map_id.as_ref() != "EcruteakCity" {
        return None;
    }
    let half = source.grid_size.as_vec2() * source.tile_size * 0.5;
    let door = (tower_core.as_vec2() + Vec2::splat(0.5)) * source.tile_size * 2. - half;
    let building = terrain.rigid_buildings.iter().min_by(|a, b| {
        a.doorway
            .distance_squared(door)
            .total_cmp(&b.doorway.distance_squared(door))
    })?;
    if building.doorway.distance(door) > source.tile_size.max_element() * 1.5 {
        return None;
    }
    let range = building.vertices.clone();
    fit_building(source, terrain, range)
}

fn fit_building(
    source: &VisualWorldFrame,
    terrain: &mut TerrainMeshData,
    range: std::ops::Range<usize>,
) -> Option<TowerAttachment> {
    let half = source.grid_size.as_vec2() * source.tile_size * 0.5;
    let model = crate::exterior_models::model(crate::exterior_models::Kind::TinTower);
    if range.len() != model.surface.positions.len() {
        return None;
    }

    // Each closed hip backing has a flat underside. Its highest underside
    // identifies the uppermost roof without hard-coding a model height.
    let base = model
        .surface
        .positions
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            model.vertex_material(*i) == "violet_dark" && model.surface.normals[*i][1] < -0.99
        })
        .map(|(_, p)| p[1])
        .max_by(f32::total_cmp)?;
    let top = model
        .surface
        .positions
        .iter()
        .enumerate()
        .filter(|(i, p)| model.vertex_material(*i) == "violet_dark" && p[1] >= base - 0.001)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let (low, high) = bounds(
        terrain
            .solid
            .positions
            .get(range.clone())?
            .iter()
            .map(|p| Vec3::from_array(*p)),
    );
    // A tall sprite's vertical drawing is not the building's plan depth.
    // Restore the authored pagoda's plan aspect before fitting its summit.
    // Keep its south doorway seated on the original town approach.
    let depth_scale = (high.x - low.x) / (high.z - low.z) * (model.max[2] - model.min[2])
        / (model.max[0] - model.min[0]);
    for i in range.clone() {
        terrain.solid.positions[i][2] =
            high.z + (terrain.solid.positions[i][2] - high.z) * depth_scale;
        let mut normal = Vec3::from_array(terrain.solid.normals[i]);
        normal.z /= depth_scale;
        terrain.solid.normals[i] = normal.normalize().to_array();
    }
    let (roof_low, roof_high) = bounds(
        top.iter()
            .map(|&i| Vec3::from_array(terrain.solid.positions[range.start + i])),
    );
    let eave = top
        .iter()
        .filter(|&&i| {
            let z = terrain.solid.positions[range.start + i][2];
            (z - roof_low.z).abs() < 0.001 || (z - roof_high.z).abs() < 0.001
        })
        .map(|&i| terrain.solid.positions[range.start + i][1])
        .max_by(f32::total_cmp)?;

    // The playable roof replaces the exterior cap and finial. Keeping both
    // roofs would drive the enlarged exterior ridge through the walking deck.
    terrain.solid.indices = terrain
        .solid
        .indices
        .chunks_exact(3)
        .filter(|tri| {
            !tri.iter().all(|&i| {
                let i = i as usize;
                if !range.contains(&i) {
                    return false;
                }
                let local = i - range.start;
                let material = model.vertex_material(local);
                (material.starts_with("violet") || material == "yellow")
                    && model.surface.positions[local][1] >= base - 0.001
            })
        })
        .flatten()
        .copied()
        .collect();
    Some(TowerAttachment {
        anchor: Vec2::new(
            (roof_low.x + roof_high.x) * 0.5,
            (roof_low.z + roof_high.z) * 0.5,
        ) + half,
        span: Vec2::new(roof_high.x - roof_low.x, roof_high.z - roof_low.z),
        eave,
    })
}

fn roof_layout(frame: &VisualWorldFrame) -> Option<(Vec2, Vec2, f32)> {
    if !enabled(frame) {
        return None;
    }
    let mut low = Vec2::splat(f32::INFINITY);
    let mut high = Vec2::splat(f32::NEG_INFINITY);
    let mut wings = [[f32::INFINITY, f32::NEG_INFINITY]; 2];
    for tile in &frame.tiles {
        let s = &tile.source;
        let side = match (s.tileset_id.as_ref(), s.metatile_id) {
            ("tower", 0x1c) => 0,
            ("tower", 0x23) => 1,
            _ => continue,
        };
        let art = if side == 0 {
            [0x16, 0x24, 0x16, 6]
        } else {
            [0x25, 0x35, 0x34, 0x35]
        };
        if s.subtile_column >= 4
            || s.subtile_row >= 4
            || s.tile_index != art[s.subtile_column as usize]
        {
            continue;
        }
        let cell = Vec2::new(tile.column as f32, tile.row as f32);
        low = low.min(cell);
        high = high.max(cell + Vec2::ONE);
        wings[side][0] = wings[side][0].min(cell.x);
        wings[side][1] = wings[side][1].max(cell.x + 1.);
    }
    if !low.is_finite() || !wings.iter().all(|w| w[0].is_finite() && w[1] > w[0]) {
        return None;
    }
    let center = ((low + high) * 0.5 - frame.grid_size.as_vec2() * 0.5) * frame.tile_size
        + Vec2::new(frame.center.x, -frame.center.y);
    let span = (high - low) * frame.tile_size;
    let eave = -wings.iter().map(|w| w[1] - w[0]).fold(0., f32::max) * frame.tile_size.x * 0.25;
    Some((center, span, eave))
}

pub(super) fn transform(
    frame: &VisualWorldFrame,
    tower_core: IVec2,
    attachment: Option<TowerAttachment>,
) -> Transform {
    if let (Some(a), Some((center, span, eave))) = (attachment, roof_layout(frame)) {
        // One uniform scale preserves every neighboring building, tree and
        // route's proportions. A small eave overhang seats the real roof.
        let scale = (span / a.span).min_element() * 0.96;
        return Transform::from_xyz(
            center.x - a.anchor.x * scale,
            eave - a.eave * scale,
            center.y - a.anchor.y * scale,
        )
        .with_scale(Vec3::splat(scale));
    }
    let map_zero =
        -(frame.grid_origin.as_vec2() + frame.grid_size.as_vec2() * 0.5) * frame.tile_size;
    let roof_center = frame
        .source_map_size_core_tiles
        .unwrap_or(UVec2::new(20, 18))
        .as_vec2()
        * frame.tile_size;
    let anchor = (tower_core.as_vec2() + Vec2::splat(0.5)) * frame.tile_size * 2.;
    Transform::from_xyz(
        map_zero.x + roof_center.x - anchor.x + frame.center.x,
        -DROP * world_units(frame),
        map_zero.y + roof_center.y - anchor.y - frame.center.y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualTile, VisualTileSource};

    fn roof(unit: f32) -> VisualWorldFrame {
        let mut tiles = Vec::new();
        for y in 0..32 {
            for x in 0..36 {
                let side = if x < 16 {
                    Some(0)
                } else if x >= 20 {
                    Some(1)
                } else {
                    None
                };
                let art = if side == Some(0) {
                    [0x16, 0x24, 0x16, 6]
                } else {
                    [0x25, 0x35, 0x34, 0x35]
                };
                tiles.push(VisualTile {
                    column: x,
                    row: y,
                    texture: Handle::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "tower".into(),
                        metatile_id: match side {
                            Some(0) => 0x1c,
                            Some(_) => 0x23,
                            None => 2,
                        },
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: if side.is_some() {
                            art[(x % 4) as usize]
                        } else {
                            2
                        },
                    },
                });
            }
        }
        VisualWorldFrame {
            active: true,
            map_id: "TinTowerRoof".into(),
            grid_size: UVec2::new(36, 32),
            tile_size: Vec2::splat(unit),
            tiles,
            ..default()
        }
    }

    #[test]
    fn playable_roof_fits_upper_eaves_at_native_scales_without_changing_source_or_support() {
        let model = crate::exterior_models::model(crate::exterior_models::Kind::TinTower);
        for unit in [8., 32.] {
            let source = VisualWorldFrame {
                map_id: "EcruteakCity".into(),
                grid_size: UVec2::new(80, 72),
                tile_size: Vec2::splat(unit),
                ..default()
            };
            let mut terrain = TerrainMeshData::default();
            terrain.footing_heights = vec![3., 7.];
            // An unrelated surface before the building must remain byte-exact.
            terrain
                .solid
                .positions
                .extend([[1., 2., 3.], [4., 2., 3.], [1., 2., 6.]]);
            terrain.solid.normals.extend([[0., 1., 0.]; 3]);
            terrain.solid.colors.extend([[1.; 4]; 3]);
            terrain.solid.uvs.extend([[0.; 2]; 3]);
            terrain.solid.indices.extend([0, 1, 2]);
            model.append_fitted(
                &mut terrain.solid,
                [0., 8. * unit, 0., 16. * unit],
                3. * unit,
                2. * unit,
                Some(4. * unit),
            );
            let end = terrain.solid.positions.len();
            let before = terrain.clone();
            let attachment = fit_building(&source, &mut terrain, 3..end).unwrap();
            assert_eq!(terrain.footing_heights, before.footing_heights);
            assert_eq!(terrain.solid.positions[..3], before.solid.positions[..3]);
            assert_eq!(terrain.solid.normals[..3], before.solid.normals[..3]);
            assert_eq!(terrain.solid.colors, before.solid.colors);
            assert_eq!(terrain.solid.uvs, before.solid.uvs);
            assert_eq!(terrain.solid.indices[..3], [0, 1, 2]);
            let south = |mesh: &TerrainMeshData| {
                mesh.solid.positions[3..]
                    .iter()
                    .map(|p| p[2])
                    .fold(f32::NEG_INFINITY, f32::max)
            };
            assert_eq!(
                south(&terrain),
                south(&before),
                "the real town approach stays on the doorway seam"
            );
            assert!(
                terrain.solid.indices.len() < before.solid.indices.len(),
                "replace the duplicate exterior cap"
            );
            assert!(
                terrain
                    .solid
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.0001)
            );

            let mut frame = roof(unit);
            let cells = frame.tiles.clone();
            let fitted = transform(&frame, IVec2::ZERO, Some(attachment));
            let span = attachment.span * fitted.scale.x;
            // Both dimensions must actually support the roof, including the
            // side view that previously exposed a three-times wider roof.
            assert!((span / (Vec2::new(36., 32.) * unit)).min_element() > 0.85);
            assert!((span / (Vec2::new(36., 32.) * unit)).max_element() < 0.97);
            let seated = fitted.transform_point(Vec3::new(
                attachment.anchor.x,
                attachment.eave,
                attachment.anchor.y,
            ));
            assert!(seated.distance(Vec3::new(0., -4. * unit, 0.)) < 0.001);
            assert_eq!(frame.tiles, cells);
            frame.center = Vec2::new(11., -7.);
            let moved = transform(&frame, IVec2::ZERO, Some(attachment));
            assert_eq!(moved.scale, fitted.scale);
            assert!(
                (moved.translation - fitted.translation - Vec3::new(11., 0., 7.)).length() < 0.001
            );
            frame.map_id = "TinTower9F".into();
            assert_eq!(
                transform(&frame, IVec2::ZERO, Some(attachment)).scale,
                Vec3::ONE
            );
        }
    }
}
