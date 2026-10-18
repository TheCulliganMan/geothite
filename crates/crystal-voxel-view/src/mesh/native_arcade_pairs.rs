//! Turn the two original closed cabinets toward their source stool aisles.
//! The native source matcher owns admission; this adapter changes no footprint,
//! collision, support, source UV, index topology or cabinet height.
use super::*;

pub(super) fn append(
    target: &mut SurfaceMeshData,
    model: &crate::interior_models::Model,
    fitted: [f32; 4],
    owner: [f32; 4],
    height: f32,
    swatches: [[f32; 2]; 9],
) -> bool {
    if fitted.iter().chain(&owner).any(|v| !v.is_finite())
        || !height.is_finite()
        || height <= 0.
        || fitted[0] >= fitted[1]
        || fitted[2] >= fitted[3]
        || owner[0] >= owner[1]
        || owner[2] >= owner[3]
    {
        return false;
    }
    let mut part = SurfaceMeshData::default();
    if !model.append_native_arcade(&mut part, fitted, height, swatches)
        || part.indices.is_empty()
        || part.indices.len() % 3 != 0
        || part.normals.len() != part.positions.len()
        || part
            .indices
            .iter()
            .any(|&i| i as usize >= part.positions.len())
    {
        return false;
    }
    let split = (fitted[0] + fitted[1]) * 0.5;
    let epsilon = (owner[1] - owner[0]).min(owner[3] - owner[2]) * 0.00001;
    if part
        .positions
        .iter()
        .any(|p| p.iter().any(|v| !v.is_finite()) || (p[0] - split).abs() <= epsilon)
    {
        return false;
    }
    let side = |x: f32| usize::from(x > split);
    // A later connected/plinth model must not be torn apart at the midline.
    if part.indices.chunks_exact(3).any(|t| {
        let a = side(part.positions[t[0] as usize][0]);
        t[1..]
            .iter()
            .any(|&i| side(part.positions[i as usize][0]) != a)
    }) {
        return false;
    }
    let mut extents = [[
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ]; 2];
    for p in &part.positions {
        let e = &mut extents[side(p[0])];
        e[0] = e[0].min(p[0]);
        e[1] = e[1].max(p[0]);
        e[2] = e[2].min(p[2]);
        e[3] = e[3].max(p[2]);
    }
    if extents.iter().flatten().any(|v| !v.is_finite()) {
        return false;
    }
    let middle = (owner[0] + owner[1]) * 0.5;
    for i in 0..part.positions.len() {
        let p = part.positions[i];
        let unit = side(p[0]);
        let e = extents[unit];
        let x = p[0] - (e[0] + e[1]) * 0.5;
        let z = p[2] - (e[2] + e[3]) * 0.5;
        let sign = if unit == 0 { -1. } else { 1. };
        let cx = owner[0] + (owner[1] - owner[0]) * (unit as f32 + 0.5) * 0.5;
        let cz = (owner[2] + owner[3]) * 0.5;
        let turned = [cx + sign * z, p[1], cz - sign * x];
        let unit_bounds = if unit == 0 {
            [owner[0], middle]
        } else {
            [middle, owner[1]]
        };
        if turned[0] < unit_bounds[0] - epsilon
            || turned[0] > unit_bounds[1] + epsilon
            || turned[2] < owner[2] - epsilon
            || turned[2] > owner[3] + epsilon
        {
            return false;
        }
        part.positions[i] = turned;
        let n = part.normals[i];
        part.normals[i] = [sign * n[2], n[1], -sign * n[0]];
        part.colors[i] =
            crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(part.normals[i]));
    }
    let first = target.positions.len();
    target.positions.extend(part.positions);
    target.normals.extend(part.normals);
    target.uvs.extend(part.uvs);
    target.colors.extend(part.colors);
    target
        .indices
        .extend(part.indices.into_iter().map(|i| i + first as u32));
    target.cutaway_ranges.extend(
        part.cutaway_ranges
            .into_iter()
            .map(|range| first + range.start..first + range.end),
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_arcade_pair_uses_its_actual_pink_body_and_yellow_trim_pigments() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack = loaded.pack();
        let drawing = &pack.runtime_files()["data/tilesets/game_corner_metatiles.bin"];
        let image = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/game_corner.png"])
            .unwrap()
            .to_luma8();
        let palettes = &pack.data().tilesets["game_corner"].palette_map;
        let g = GridGeometry {
            width: 6,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut tiles = (0..24)
            .map(|i| VisualTile {
                column: (i % 6) as u32,
                row: (i / 6) as u32,
                texture: Default::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "game_corner".into(),
                    metatile_id: 0,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: 1,
                },
            })
            .collect::<Vec<_>>();
        for y in 0..2 {
            for x in 0..4 {
                tiles[(y + 1) * 6 + x + 1].source = VisualTileSource {
                    tileset_id: "game_corner".into(),
                    metatile_id: 7,
                    subtile_column: x as u8,
                    subtile_row: y as u8,
                    tile_index: drawing[7 * 16 + y * 4 + x] as u16,
                };
            }
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        let p = Placement {
            kind: ModelKind::ArcadePair,
            column: 1,
            row: 1,
            width: 4,
            height: 2,
            ground: 0,
            depth_pixels: 14.,
            height_pixels: 21.,
            base_pixels: 0.,
            footing_pixels: None,
        };
        for map in ["CeladonGameCorner", "GoldenrodGameCorner"] {
            let swatches = native_arcade_source(map, &p, &refs, &g).unwrap();
            let pigment = |uv: [f32; 2]| {
                let x = uv[0] * g.width as f32;
                let y = uv[1] * g.height as f32;
                let raw = refs[y.floor() as usize * g.width + x.floor() as usize]
                    .source
                    .tile_index as usize;
                let bank = palettes[raw];
                let count = (image.width() / 8) * (image.height() / 8);
                let tile = if bank & 8 != 0 {
                    (raw & 127) + count as usize / 2
                } else {
                    raw
                };
                (
                    bank & 7,
                    image.get_pixel(
                        (tile % 16 * 8) as u32 + ((x.fract() * 8.).floor() as u32),
                        (tile / 16 * 8) as u32 + ((y.fract() * 8.).floor() as u32),
                    )[0],
                )
            };
            assert_eq!(
                pigment(swatches[0]),
                (1, 170),
                "pink native panel uses the red palette midtone, not its cream highlight"
            );
            assert_eq!(
                pigment(swatches[1]),
                (4, 170),
                "yellow native trim remains distinct"
            );
        }
    }

    #[test]
    fn native_arcade_pair_faces_both_aisles_without_deforming_closed_units() {
        let model = model(ModelKind::ArcadePair);
        let swatches = std::array::from_fn(|i| [i as f32 / 10., 0.75]);
        for scale in [0.5, 1., 2.] {
            let owner = [-32. * scale, 0., 16. * scale, 32. * scale];
            let fitted = [-31.64 * scale, -0.36 * scale, 18. * scale, 32. * scale];
            let mut old = SurfaceMeshData::default();
            assert!(model.append_native_arcade(&mut old, fitted, 21. * scale, swatches));
            let mut turned = SurfaceMeshData::default();
            assert!(append(
                &mut turned,
                model,
                fitted,
                owner,
                21. * scale,
                swatches
            ));
            assert_eq!(old.indices, turned.indices);
            assert_eq!(old.uvs, turned.uvs);
            assert_eq!(old.positions.len(), turned.positions.len());
            let split = (fitted[0] + fitted[1]) * 0.5;
            let mut screens = [0; 2];
            for i in 0..old.positions.len() {
                assert_eq!(old.positions[i][1], turned.positions[i][1]);
                let side = usize::from(old.positions[i][0] > split);
                if old.uvs[i] == swatches[3] && old.normals[i][2] > 0.99 {
                    screens[side] += 1;
                    assert!(
                        if side == 0 {
                            turned.normals[i][0] < -0.99
                        } else {
                            turned.normals[i][0] > 0.99
                        },
                        "screens face their outer stool aisles"
                    );
                }
                assert!(turned.positions[i][0] >= owner[0] - 0.001);
                assert!(turned.positions[i][0] <= owner[1] + 0.001);
                assert!(turned.positions[i][2] >= owner[2] - 0.001);
                assert!(turned.positions[i][2] <= owner[3] + 0.001);
                assert!((Vec3::from_array(turned.normals[i]).length_squared() - 1.).abs() < 0.001);
            }
            assert!(screens.into_iter().all(|count| count > 0));
            for t in old.indices.chunks_exact(3) {
                for pair in [[0, 1], [1, 2], [2, 0]] {
                    let [a, b] = pair.map(|i| t[i] as usize);
                    let before = Vec3::from_array(old.positions[a])
                        .distance_squared(Vec3::from_array(old.positions[b]));
                    let after = Vec3::from_array(turned.positions[a])
                        .distance_squared(Vec3::from_array(turned.positions[b]));
                    assert!(
                        (before - after).abs() < 0.001 * scale * scale,
                        "cabinet bevels, controls and closed faces retain rigid edge lengths"
                    );
                }
            }
            let mut cached = SurfaceMeshData::default();
            assert!(model.append_native_arcade(&mut cached, fitted, 21. * scale, swatches));
            assert_eq!(old, cached, "shared cached model is immutable");
            let mut rejected = SurfaceMeshData::default();
            assert!(!append(
                &mut rejected,
                model,
                fitted,
                [0., 1., 0., 1.],
                21. * scale,
                swatches
            ));
            assert_eq!(rejected, SurfaceMeshData::default());
        }
        let mut rejected = SurfaceMeshData::default();
        assert!(
            !append(
                &mut rejected,
                crate::interior_models::model(ModelKind::ArcadeMachine),
                [0., 32., 0., 14.],
                [0., 32., 0., 16.],
                21.,
                swatches
            ),
            "a single cabinet must not split into two"
        );
        assert_eq!(rejected, SurfaceMeshData::default());
    }
}
