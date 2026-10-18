//! Native clinic appliances in the PokéCom lobby; source-identical to Centers.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Appliance {
    Healing,
    Pc,
    Cabinet,
    Console,
}

fn spec(
    role: Appliance,
) -> (
    ModelKind,
    u16,
    usize,
    usize,
    usize,
    usize,
    &'static [u16],
    f32,
    f32,
    f32,
) {
    match role {
        Appliance::Healing => (
            ModelKind::HealingMachine,
            5,
            0,
            0,
            4,
            4,
            &[
                28, 29, 30, 31, 44, 45, 46, 47, 60, 61, 61, 63, 76, 77, 78, 79,
            ],
            12.,
            30.,
            0.,
        ),
        Appliance::Pc => (
            ModelKind::Computer,
            7,
            2,
            1,
            2,
            3,
            &[32, 33, 48, 49, 64, 65],
            12.,
            22.,
            0.,
        ),
        Appliance::Cabinet => (
            ModelKind::WallClinical,
            6,
            0,
            0,
            2,
            2,
            &[4, 5, 20, 21],
            6.,
            16.,
            13.,
        ),
        Appliance::Console => (
            ModelKind::HealingConsole,
            6,
            0,
            2,
            2,
            2,
            &[10, 11, 26, 27],
            13.,
            13.,
            0.,
        ),
    }
}
fn roles() -> [Appliance; 4] {
    [
        Appliance::Healing,
        Appliance::Pc,
        Appliance::Cabinet,
        Appliance::Console,
    ]
}
fn donor(role: Appliance, p: &Placement, g: &GridGeometry) -> Option<(usize, usize)> {
    match role {
        Appliance::Healing => Some((p.column.checked_sub(1)?, p.row)),
        Appliance::Pc => Some((p.column.checked_sub(2)?, p.row.checked_sub(1)?)),
        Appliance::Cabinet => {
            (p.column + 2 < g.width && p.row + 1 < g.height).then_some((p.column + 2, p.row + 1))
        }
        Appliance::Console => {
            (p.column + 2 < g.width).then_some((p.column + 2, p.row.checked_sub(1)?))
        }
    }
}
pub(super) fn source(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<Appliance> {
    if map != crate::pokecom::ADMIN_OFFICE
        || cells.len() != g.width * g.height
        || p.column + p.width > g.width
        || p.row + p.height > g.height
    {
        return None;
    }
    roles().into_iter().find(|&role| {
        let (kind, block, x, y, w, h, art, depth, height, base) = spec(role);
        if p.kind != kind
            || p.width != w
            || p.height != h
            || p.depth_pixels != depth
            || p.height_pixels != height
            || p.base_pixels != base
        {
            return false;
        }
        if !p.indices(g.width).enumerate().all(|(j, i)| {
            let s = &cells[i].source;
            s.tileset_id.as_ref() == "pokecom_center"
                && s.metatile_id == block
                && s.subtile_column == (x + j % w) as u8
                && s.subtile_row == (y + j / w) as u8
                && s.tile_index == art[j]
        }) {
            return false;
        }
        let Some((x, y)) = donor(role, p, g) else {
            return false;
        };
        let s = &cells[y * g.width + x].source;
        s.tileset_id.as_ref() == "pokecom_center"
            && s.tile_index == 2
            && match role {
                Appliance::Healing => {
                    s.metatile_id == 4 && s.subtile_column == 3 && s.subtile_row == 0
                }
                Appliance::Pc => s.metatile_id == 7 && s.subtile_column == 0 && s.subtile_row == 0,
                Appliance::Cabinet | Appliance::Console => {
                    s.metatile_id == 6 && s.subtile_column == 2 && s.subtile_row == 1
                }
            }
    })
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if map != crate::pokecom::ADMIN_OFFICE {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "pokecom_center", 0x11) else {
        return;
    };
    for role in roles() {
        let (kind, _, _, _, width, height, _, depth_pixels, height_pixels, base_pixels) =
            spec(role);
        if width > g.width || height > g.height {
            continue;
        }
        for row in 0..=g.height - height {
            for column in 0..=g.width - width {
                let p = Placement {
                    kind,
                    column,
                    row,
                    width,
                    height,
                    ground,
                    depth_pixels,
                    height_pixels,
                    base_pixels,
                    footing_pixels: None,
                };
                if source(map, &p, cells, g) != Some(role) || p.indices(g.width).any(|i| claimed[i])
                {
                    continue;
                }
                for i in p.indices(g.width) {
                    claimed[i] = true;
                }
                out.push(p);
            }
        }
    }
}
fn backing(out: &mut SurfaceMeshData, g: &GridGeometry, p: &Placement, role: Appliance) {
    if role == Appliance::Console {
        return;
    }
    native_pokecom_clinic_wall::append_backing(
        out,
        g,
        p,
        p.row - spec(role).3,
        donor(role, p, g).unwrap(),
    );
}
pub(super) fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    role: Appliance,
    b: [f32; 4],
) {
    backing(out, g, p, role);
    let scale = g.tile_height / 8.;
    let height = p.height_pixels * scale;
    match role {
        Appliance::Healing => native_clinic::append_healing(out, g, p, b, height),
        Appliance::Cabinet => {
            native_clinic::append_cabinet(out, g, p, [b[0], b[1], b[3] - 6. * scale, b[3]], height)
        }
        Appliance::Pc | Appliance::Console => native_clinic::append_terminal(out, g, p, b, height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_clinic_uses_complete_native_appliances_and_backing() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let map = crate::pokecom::ADMIN_OFFICE;
        let data = &pack.data().maps[map];
        let art = &pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
        let (w, h) = (
            data.attributes.width as usize * 4,
            data.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let b = data.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "pokecom_center".into(),
                        metatile_id: b,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect::<Vec<_>>();
        let refs = tiles.iter().collect::<Vec<_>>();
        assert!(
            !crate::interior::has_room_perimeter(map, "pokecom_center"),
            "separated source rooms must not receive a rectangular wrapper through the void"
        );
        let g = GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let all = resolve(map, &refs, &g, None);
        let fixtures = all
            .iter()
            .filter_map(|p| source(map, p, &refs, &g).map(|role| (p, role)))
            .collect::<Vec<_>>();
        assert_eq!(fixtures.len(), 4);
        let walls = all
            .iter()
            .filter(|p| {
                native_pokecom_clinic_wall::wall(map, p, &refs, &g)
                    || native_pokecom_clinic_wall::mounted(map, p, &refs, &g)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            walls
                .iter()
                .filter(|p| p.kind == ModelKind::WallClinical)
                .count(),
            15,
            "wall columns: {:?}",
            walls
                .iter()
                .map(|p| (p.column, p.row, p.width, p.height, p.kind))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            walls
                .iter()
                .filter(|p| p.kind == ModelKind::PictureFrame)
                .count(),
            4
        );
        assert_eq!(walls.iter().map(|p| p.width * p.height).sum::<usize>(), 32);
        let mut joined = TerrainMeshData {
            footing_heights: vec![2.75; w * h],
            authored_cells: vec![None; w * h],
            ..Default::default()
        };
        let mut joined_claims = vec![false; w * h];
        for p in walls
            .iter()
            .copied()
            .chain(fixtures.iter().map(|(p, _)| *p))
        {
            assert!(append_with_tables(
                &mut joined,
                map,
                &refs,
                &g,
                p,
                &mut joined_claims,
                [0, 0],
                &all
            ));
        }
        assert_eq!(joined_claims.iter().filter(|&&v| v).count(), 62);
        assert_eq!(joined.footing_heights, vec![2.75; w * h]);
        for x in 0..24 {
            for y in [4., 12., 20., 28.] {
                let x = (x as f32 + 0.5) * 8.;
                assert!(
                    joined.textured.indices.chunks_exact(3).any(|triangle| {
                        let v = triangle
                            .iter()
                            .map(|&i| joined.textured.positions[i as usize])
                            .collect::<Vec<_>>();
                        v.iter().all(|p| (p[2] - 112.).abs() < 0.001)
                            && v.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min) <= x
                            && v.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max) >= x
                            && v.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min) <= y
                            && v.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max) >= y
                    }),
                    "rear wall must continue across every source column and ceiling-height course"
                );
            }
        }
        for p in &walls {
            if p.kind == ModelKind::PictureFrame {
                let mut expected = SurfaceMeshData::default();
                native_pokecom_clinic_wall::append(&mut expected, &g, p, &refs);
                let mut actual = TerrainMeshData::default();
                let mut claims = vec![false; w * h];
                assert!(append_with_tables(
                    &mut actual,
                    map,
                    &refs,
                    &g,
                    p,
                    &mut claims,
                    [0, 0],
                    &all
                ));
                assert_eq!(
                    actual.textured.positions.len(),
                    expected.positions.len() + p.width * p.height * 4,
                    "native mounted art must not also emit the generic floating picture overlay"
                );
                assert!(actual.solid.positions.is_empty());
            }
            for i in p.indices(w) {
                let mut damaged = tiles.clone();
                damaged[i].source.tile_index ^= 0x80;
                let r = damaged.iter().collect::<Vec<_>>();
                assert!(!native_pokecom_clinic_wall::wall(map, p, &r, &g));
                assert!(!native_pokecom_clinic_wall::mounted(map, p, &r, &g));
            }
            assert!(!native_pokecom_clinic_wall::wall(
                "PewterPokecenter1F",
                p,
                &refs,
                &g
            ));
            assert!(!native_pokecom_clinic_wall::mounted(
                "PewterPokecenter1F",
                p,
                &refs,
                &g
            ));
        }
        for role in roles() {
            assert_eq!(fixtures.iter().filter(|(_, r)| *r == role).count(), 1);
        }
        for (p, role) in fixtures {
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            let mut claims = vec![false; w * h];
            assert!(append_with_tables(
                &mut mesh,
                map,
                &refs,
                &g,
                p,
                &mut claims,
                [0, 0],
                &all
            ));
            assert_eq!(claims.iter().filter(|&&c| c).count(), p.width * p.height);
            assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite())
            );
            assert!(
                mesh.textured
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
            );
            let mut wall = SurfaceMeshData::default();
            backing(&mut wall, &g, p, role);
            if role != Appliance::Console {
                assert!(wall.positions.iter().any(|v| v[1] == 32.));
                let source_row = p.row - spec(role).3;
                assert!(
                    wall.positions
                        .iter()
                        .all(|v| v[2] <= (source_row + 2) as f32 * 8.)
                );
            } else {
                assert!(wall.positions.is_empty());
            }
            let mut body = SurfaceMeshData::default();
            let b = [0., p.width as f32 * 8., 0., p.depth_pixels];
            match role {
                Appliance::Healing => native_clinic::append_healing(&mut body, &g, p, b, 30.),
                Appliance::Cabinet => native_clinic::append_cabinet(&mut body, &g, p, b, 16.),
                Appliance::Pc | Appliance::Console => {
                    native_clinic::append_terminal(&mut body, &g, p, b, p.height_pixels)
                }
            }
            let low = body
                .positions
                .iter()
                .map(|v| v[1])
                .fold(f32::INFINITY, f32::min);
            let high = body
                .positions
                .iter()
                .map(|v| v[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert_eq!(
                (low, high),
                (p.base_pixels, p.base_pixels + p.height_pixels)
            );
            for i in p.indices(w) {
                let mut damaged = tiles.clone();
                damaged[i].source.tile_index ^= 0x80;
                let r = damaged.iter().collect::<Vec<_>>();
                assert_eq!(source(map, p, &r, &g), None);
                let mut reserved = vec![false; w * h];
                reserved[i] = true;
                let mut out = Vec::new();
                candidates(map, &refs, &g, &mut reserved, &mut out);
                assert!(!out.iter().any(|q| q.column == p.column && q.row == p.row));
            }
            let (x, y) = donor(role, p, &g).unwrap();
            let mut damaged = tiles.clone();
            damaged[y * w + x].source.tile_index ^= 1;
            assert_eq!(
                source(map, p, &damaged.iter().collect::<Vec<_>>(), &g),
                None
            );
            assert_eq!(source("PewterPokecenter1F", p, &refs, &g), None);
        }
    }
}
