use super::*;
use bevy::prelude::Vec2;
#[test]
fn distant_map_removes_only_repeated_aprons_preserving_all_finite_source_geometry() {
    use bevy::{
        asset::RenderAssetUsages,
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    let mut source = super::tests::source_with_tile(0, 0, 0, 5);
    source.tileset_id = "johto".into();
    let mut frame = super::tests::frame(40, 40, vec![source; 1600]);
    frame.map_id = "EcruteakCity".into();
    frame.source_map_size_core_tiles = Some(bevy::math::UVec2::new(20, 20));
    let mut images = Assets::<Image>::default();
    let texture = images.add(Image::new_fill(
        Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[87, 150, 41, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD,
    ));
    for tile in &mut frame.tiles {
        tile.texture = texture.clone();
    }
    let samples = TerrainImageSamples::capture(&frame, &images);
    let profiles = crate::live_profiles::LiveProfiles::default();
    let normal = begin_runtime_terrain_mesh_with_profiles(&frame, &samples, &profiles.document)
        .unwrap()
        .into_streamed()
        .terrain;
    let distant = begin_distant_terrain_mesh_with_profiles(&frame, &samples, &profiles.document)
        .unwrap()
        .into_streamed()
        .terrain;
    let inside = |p: [f32; 3]| p[0] >= -160. && p[0] <= 160. && p[2] >= -160. && p[2] <= 160.;
    assert!(
        normal
            .solid
            .positions
            .iter()
            .chain(&normal.textured.positions)
            .any(|&p| !inside(p))
            || normal.background.is_some(),
        "ordinary world rendering must retain its normal continuation"
    );
    assert!(distant.background.is_none());
    assert!(
        distant
            .solid
            .positions
            .iter()
            .chain(&distant.textured.positions)
            .all(|&p| inside(p))
    );
    assert_eq!(normal.footing_heights, distant.footing_heights);
    assert_eq!(normal.authored_cells, distant.authored_cells);
    let triangles = |mesh: &TerrainMeshData| {
        let mut all = Vec::new();
        for surface in [&mesh.solid, &mesh.textured] {
            for tri in surface.indices.chunks_exact(3) {
                if !tri.iter().all(|&i| inside(surface.positions[i as usize])) {
                    continue;
                }
                all.push(
                    tri.iter()
                        .flat_map(|&i| {
                            let i = i as usize;
                            surface.positions[i]
                                .into_iter()
                                .chain(surface.normals[i])
                                .chain(surface.uvs[i])
                                .chain(surface.colors[i])
                                .map(f32::to_bits)
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }
        all.sort();
        all
    };
    let expected = triangles(&normal);
    assert!(!expected.is_empty());
    assert_eq!(
        triangles(&distant),
        expected,
        "positions, normals, UVs and pigments inside the actual map stay exact"
    );
}

#[test]
fn summit_attachment_uses_the_native_roof_center_and_preserves_its_terrace_height() {
    use crate::summit_vista::{SummitBackdropMap, tower_attachment};
    let source = SummitBackdropMap {
        frame: std::sync::Arc::new(VisualWorldFrame {
            map_id: "EcruteakCity".into(),
            grid_size: bevy::math::UVec2::new(80, 72),
            tile_size: Vec2::splat(8.),
            ..Default::default()
        }),
        origin_source: bevy::math::IVec2::ZERO,
    };
    // These are the measured source-model envelope and doorway from the
    // actual Ecruteak headless render, including its raised terrace.
    let mut terrain = TerrainMeshData::default();
    terrain.solid.positions = vec![[256., 30.046732, -288.], [320., 191.64674, -160.]];
    terrain.rigid_buildings.push(rigid_buildings::Building {
        vertices: 0..2,
        doorway: Vec2::new(280., -164.),
    });
    let (anchor, drop) =
        tower_attachment(&source, &terrain, bevy::math::IVec2::new(37, 7)).unwrap();
    assert_eq!(anchor, Vec2::new(608., 64.));
    assert!(
        (191.64674 - drop + 4.).abs() < 0.001,
        "the exterior roof must stay below the summit"
    );
    let mut production_source = source.clone();
    std::sync::Arc::make_mut(&mut production_source.frame).tile_size *= 4.;
    for p in &mut terrain.solid.positions {
        for value in p {
            *value *= 4.;
        }
    }
    terrain.rigid_buildings[0].doorway *= 4.;
    let production =
        tower_attachment(&production_source, &terrain, bevy::math::IVec2::new(37, 7)).unwrap();
    assert_eq!(production.0, anchor * 4.);
    assert!(
        (production.1 - drop * 4.).abs() < 0.001,
        "production tile scale must include the terrace and roof height"
    );
    terrain.rigid_buildings[0].doorway = Vec2::ZERO;
    assert!(
        tower_attachment(&source, &terrain, bevy::math::IVec2::new(37, 7)).is_none(),
        "an edited-away tower must not attach to a distant house"
    );
}
