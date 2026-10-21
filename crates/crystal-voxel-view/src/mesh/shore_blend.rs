//! Source-scoped damp soil and wet sand on the dry side of Johto water edges.
use super::*;
use bevy::prelude::Vec2;

#[derive(Clone, Default)]
pub(super) struct ShoreBlend {
    edges: Vec<([f32; 2], [f32; 2], bool, bool)>,
    candidates: Vec<Vec<usize>>,
    land: Vec<bool>,
}
impl ShoreBlend {
    pub(super) fn new(map: &str, cells: &[&VisualTile], g: &GridGeometry) -> Self {
        if !ordinary_lawn_map(map) {
            return Self::default();
        }
        let land: Vec<_> = cells
            .iter()
            .map(|t| {
                matches!(
                    t.source.tileset_id.as_ref(),
                    "johto" | "johto_modern" | "kanto"
                ) && (matches!(
                    ground_material(&t.source),
                    Some(GroundMaterial::Lawn | GroundMaterial::Path)
                ) || super::super::kanto_shore::source_bank(&t.source)
                    || shape_for_source(&t.source) == CellShape::ShoreBand)
            })
            .collect();
        let mut result = Self {
            edges: Vec::new(),
            candidates: vec![Vec::new(); cells.len()],
            land,
        };
        for row in 0..g.height {
            for col in 0..g.width {
                let i = row * g.width + col;
                if !result.land[i] {
                    continue;
                }
                let (w, e, n, s) = g.bounds(col, row);
                for direction in Direction::ALL {
                    let (dx, dz) = direction.offset();
                    let x = col as isize + dx;
                    let z = row as isize + dz;
                    if x < 0 || z < 0 || x >= g.width as isize || z >= g.height as isize {
                        continue;
                    }
                    let other = &cells[z as usize * g.width + x as usize].source;
                    if other.tileset_id != cells[i].source.tileset_id
                        || !matches!(shape_for_source(other), CellShape::Water)
                    {
                        continue;
                    }
                    let edge = match direction {
                        Direction::North => ([w, n], [e, n]),
                        Direction::South => ([w, s], [e, s]),
                        Direction::West => ([w, n], [w, s]),
                        Direction::East => ([e, n], [e, s]),
                    };
                    let index = result.edges.len();
                    result.edges.push((edge.0, edge.1,
                        ground_material(&cells[i].source) == Some(GroundMaterial::Path)
                            || super::super::kanto_shore::source_bank(&cells[i].source),
                        cells[i].source.tileset_id.as_ref()=="kanto"));
                    for y in row.saturating_sub(1)..=(row + 1).min(g.height - 1) {
                        for x in col.saturating_sub(1)..=(col + 1).min(g.width - 1) {
                            result.candidates[y * g.width + x].push(index);
                        }
                    }
                }
            }
        }
        result
    }
    pub(super) fn near(&self, p: [f32; 3], g: &GridGeometry) -> bool {
        cell_at(g, p[0], p[2]).is_some_and(|i| {
            self.land.get(i).copied().unwrap_or(false)
                && self.candidates.get(i).is_some_and(|v| !v.is_empty())
        })
    }
    pub(super) fn color(
        &self,
        p: [f32; 3],
        g: &GridGeometry,
        origin: [i32; 2],
        base: [f32; 4],
    ) -> [f32; 4] {
        if p[1].abs() > g.tile_height * 0.0001 {
            return base;
        }
        let Some(i) = cell_at(g, p[0], p[2]) else {
            return base;
        };
        let Some(candidates) = self.candidates.get(i) else {
            // Excluded maps intentionally have no shoreline index. Their
            // ground finish must remain a no-op instead of indexing it.
            return base;
        };
        let owned = self.land.get(i).copied().unwrap_or(false);
        let x = (p[0] - g.origin_x) / g.tile_width + origin[0] as f32;
        let z = (p[2] - g.origin_z) / g.tile_height + origin[1] as f32;
        let roughness = meadow_noise(x * 2.1 + 37., z * 2.1 - 17.);
        let mut amount = 0.0_f32;
        let mut sandy = false;
        let mut source_tinted = false;
        for &index in candidates {
            let (a, b, sand, source) = self.edges[index];
            let width = g.tile_width.min(g.tile_height)
                * (if sand { 1.15 } else { 0.55 } + roughness * 0.22);
            let a = Vec2::from_array(a);
            let b = Vec2::from_array(b);
            let q = Vec2::new(p[0], p[2]);
            let d = b - a;
            let t = ((q - a).dot(d) / d.length_squared()).clamp(0., 1.);
            let distance = (q - a - d * t).length();
            // A shared dry vertex can lie exactly on the water cell's
            // half-open grid boundary. Retain its bank color on that line,
            // without painting any point inside the water or unknown cells.
            if !owned && distance > g.tile_width.min(g.tile_height) * 0.0001 {
                continue;
            }
            let weight = (1. - distance / width).clamp(0., 1.);
            let strength = weight * weight * (3. - 2. * weight) * 0.62;
            if strength > amount {
                amount = strength;
                sandy = sand;
                source_tinted = source;
            }
        }
        // Damp Kanto sand retains its received pigment and time-of-day color.
        // Mixing a fixed daylight brown into native night paint lights the rim.
        let soil = if source_tinted { [base[0]*0.50,base[1]*0.52,base[2]*0.55] }
            else if sandy { [0.28, 0.205, 0.12] }
            else { ledge_finish::soil_color(x, z, 0.45) };
        [
            base[0] + (soil[0] - base[0]) * amount,
            base[1] + (soil[1] - base[1]) * amount,
            base[2] + (soil[2] - base[2]) * amount,
            base[3],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{frame, source_with_tile};
    use super::*;
    #[test]
    fn shared_johto_bank_caps_receive_damp_soil_only_at_the_actual_water_edge() {
        for family in ["johto", "johto_modern"] {
            let tiles: Vec<_> = [0x14, 0x3b, 0x06]
                .into_iter()
                .enumerate()
                .map(|(i, tile)| VisualTile {
                    column: i as u32,
                    row: 0,
                    source: VisualTileSource {
                        tileset_id: family.into(),
                        metatile_id: if i < 2 { 0x59 } else { 0x01 },
                        subtile_column: if i == 1 { 3 } else { 0 },
                        subtile_row: 0,
                        tile_index: tile,
                    },
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                })
                .collect();
            let g = GridGeometry {
                width: 3,
                height: 1,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let cells: Vec<_> = tiles.iter().collect();
            let blend = ShoreBlend::new("Route34", &cells, &g);
            let base = [0.5, 0.4, 0.25, 1.];
            let wet = [8.1, 0., 4.];
            assert!(
                blend.near(wet, &g),
                "{family} bank caps must participate after material finishing"
            );
            let color = blend.color(wet, &g, [0, 0], base);
            assert!(color[0] < base[0] && color[1] < base[1] && color[2] < base[2]);
            assert_eq!(
                blend.color([15.9, 0., 4.], &g, [0, 0], base),
                base,
                "damp soil cannot cross the whole walking cap"
            );
            assert_eq!(
                blend.color([7.9, 0., 4.], &g, [0, 0], base),
                base,
                "dry pigment cannot spill into the water"
            );
            assert_eq!(
                blend.color([8.1, 1., 4.], &g, [0, 0], base),
                base,
                "the blend belongs to the existing zero-height shore cap"
            );
        }
    }

    fn fixture() -> VisualWorldFrame {
        frame(
            3,
            2,
            vec![
                source_with_tile(0x01, 0, 0, 0x05),
                source_with_tile(0x35, 0, 0, 0x14),
                source_with_tile(0x35, 0, 0, 0x14),
                source_with_tile(0x01, 0, 1, 0x05),
                source_with_tile(0x35, 0, 1, 0x14),
                source_with_tile(0x35, 0, 1, 0x14),
            ],
        )
    }
    fn grid() -> GridGeometry {
        GridGeometry {
            width: 3,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    #[test]
    fn native_kanto_bank_caps_carry_the_damp_transition_to_the_real_waterline() {
        let sources=(0..9).map(|i| {
            let (x,z)=((i%3)as u8,(i/3)as u8);
            let mut s=source_with_tile(if z==0 {1}else{0x1f},x,z.saturating_sub(1),
                if z==0 {0x23}else if z==1 {0x33}else{0x14});
            s.tileset_id="kanto".into();s
        }).collect();
        let f=frame(3,3,sources);
        let g=GridGeometry {width:3,height:3,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let make=|| {
            let mut mesh=TerrainMeshData {authored_cells:vec![None;9],footing_heights:vec![0.;9],..Default::default()};
            super::super::ledge_finish::append_natural_bank(&mut mesh.solid,
                [[8.,0.,8.],[8.,0.,16.],[16.,0.,16.],[16.,0.,8.]],
                [0.,1.,0.],&g,[0,0],[[0.6,0.5,0.3],[0.3,0.2,0.1]]);
            mesh
        };
        let mut dry_frame=f.clone();
        for tile in &mut dry_frame.tiles {if tile.row==2 {tile.source.tile_index=0x99;}}
        let mut wet=make();let mut dry=make();
        let before=wet.clone();
        super::super::polish_world_surfaces(&mut wet,"Route20",&f.tiles.iter().collect::<Vec<_>>(),&g,[0,0]);
        super::super::polish_world_surfaces(&mut dry,"Route20",&dry_frame.tiles.iter().collect::<Vec<_>>(),&g,[0,0]);
        assert_eq!(wet.solid.positions,before.solid.positions);
        assert_eq!(wet.solid.indices,before.solid.indices);
        assert_eq!(wet.footing_heights,before.footing_heights);
        for i in 0..wet.solid.positions.len() {
            if wet.solid.positions[i][2]==16. {
                assert!((0..3).all(|c|wet.solid.colors[i][c]<dry.solid.colors[i][c]*0.85),
                    "the source bank cap must reach the damp edge instead of inheriting dry sand");
            }
        }
    }

    #[test]
    fn kanto_native_sand_gets_a_damp_edge_with_its_received_day_or_night_pigment() {
        let mut f=fixture();
        for tile in &mut f.tiles {
            let s=&mut tile.source;s.tileset_id="kanto".into();
            if tile.column==0 {
                s.metatile_id=0x7b;s.subtile_column=0;s.subtile_row=tile.row as u8;
                s.tile_index=if tile.row%2==0 {0x30}else{0x39};
            } else {s.metatile_id=0x1f;s.subtile_row=1;}
        }
        let g=grid();
        let cells=f.tiles.iter().collect::<Vec<_>>();
        let blend=ShoreBlend::new("Route20",&cells,&g);
        assert_eq!(blend.edges.len(),2);
        let day=[0.68,0.52,0.31,1.];let night=[0.08,0.12,0.19,1.];
        for base in [day,night] {
            let wet=blend.color([7.,0.,4.],&g,[0,0],base);
            assert!((0..3).all(|c|wet[c]<base[c] && wet[c]>base[c]*0.65));
            assert_eq!(wet[3],base[3]);
            assert_eq!(blend.color([8.1,0.,4.],&g,[0,0],base),base);
            assert_eq!(blend.color([7.,1.,4.],&g,[0,0],base),base);
        }
        let day_wet=blend.color([7.,0.,4.],&g,[0,0],day);
        let night_wet=blend.color([7.,0.,4.],&g,[0,0],night);
        for c in 0..3 {assert!((day_wet[c]/day[c]-night_wet[c]/night[c]).abs()<0.00001);}
        for tile in &mut f.tiles {if tile.column==0 {tile.source.tile_index^=1;}}
        let changed=f.tiles.iter().collect::<Vec<_>>();
        assert!(ShoreBlend::new("Route20",&changed,&g).edges.is_empty(),
            "changed sand art cannot acquire the shoreline finish");
    }

    #[test]
    fn sand_has_a_wider_damp_transition_without_painting_the_water() {
        let mut f = fixture();
        for tile in &mut f.tiles {
            if tile.column == 0 { tile.source.tile_index = 0x06; }
        }
        let g = grid();
        let cells: Vec<_> = f.tiles.iter().collect();
        let sand = ShoreBlend::new("CherrygroveCity", &cells, &g);
        assert_eq!(sand.edges.len(), 2);
        let base = [0.68, 0.52, 0.31, 1.];
        let wet = sand.color([7., 0., 4.], &g, [0, 0], base);
        let inland = sand.color([0., 0., 4.], &g, [0, 0], base);
        assert!(wet[0] < inland[0] && wet[1] < inland[1]);
        assert_ne!(sand.color([3., 0., 4.], &g, [0, 0], base), base);
        assert_eq!(sand.color([8.1, 0., 4.], &g, [0, 0], base), base);
        assert_eq!(sand.color([7., 2., 4.], &g, [0, 0], base), base);
        let moved = GridGeometry { origin_x: -80., origin_z: 48., ..g };
        let shifted = ShoreBlend::new("CherrygroveCity", &cells, &moved);
        assert_eq!(wet, shifted.color([-73., 0., 52.], &moved, [0, 0], base));
    }

    #[test]
    fn excluded_maps_leave_ground_colors_unchanged_at_in_bounds_cells() {
        let f = fixture();
        let g = grid();
        let cells: Vec<_> = f.tiles.iter().collect();
        let base = [0.31, 0.42, 0.12, 1.];
        for map in ["IlexForest", "UnionCave1F", "Route43Gate"] {
            let blend = ShoreBlend::new(map, &cells, &g);
            for row in 0..g.height {
                for column in 0..g.width {
                    let p = [(column as f32 + 0.5) * 8., 0., (row as f32 + 0.5) * 8.];
                    assert!(!blend.near(p, &g));
                    assert_eq!(blend.color(p, &g, [0, 0], base), base, "{map}");
                }
            }
        }
    }
    #[test]
    fn wet_edge_is_bounded_continuous_and_map_anchored() {
        let f = fixture();
        let g = grid();
        let cells: Vec<_> = f.tiles.iter().collect();
        let blend = ShoreBlend::new("LakeOfRage", &cells, &g);
        let base = [0.31, 0.42, 0.12, 1.];
        assert_eq!(blend.edges.len(), 2);
        assert_eq!(blend.color([0., 0., 4.], &g, [0, 0], base), base);
        assert_eq!(blend.color([8.5, 0., 4.], &g, [0, 0], base), base);
        assert_eq!(blend.color([7., 2., 4.], &g, [0, 0], base), base);
        let edge = blend.color([8., 0., 4.], &g, [0, 0], base);
        assert_ne!(edge, base);
        let nearby = blend.color([7.999, 0., 4.], &g, [0, 0], base);
        assert!((0..4).all(|i| (edge[i] - nearby[i]).abs() < 0.001));
        let a = blend.color([7., 0., 7.999], &g, [0, 0], base);
        let b = blend.color([7., 0., 8.001], &g, [0, 0], base);
        assert!((0..4).all(|i| (a[i] - b[i]).abs() < 0.001));
        let moved = GridGeometry {
            origin_x: -80.,
            origin_z: 48.,
            ..g
        };
        let shifted = ShoreBlend::new("LakeOfRage", &cells, &moved);
        assert_eq!(
            blend.color([7., 0., 4.], &g, [13, 17], base),
            shifted.color([-73., 0., 52.], &moved, [13, 17], base)
        );
        let padded = frame(
            4,
            2,
            vec![
                source_with_tile(0x01, 0, 0, 0x05),
                f.tiles[0].source.clone(),
                f.tiles[1].source.clone(),
                f.tiles[2].source.clone(),
                source_with_tile(0x01, 0, 1, 0x05),
                f.tiles[3].source.clone(),
                f.tiles[4].source.clone(),
                f.tiles[5].source.clone(),
            ],
        );
        let padded_grid = GridGeometry {
            width: 4,
            origin_x: -8.,
            ..g
        };
        let padded_cells: Vec<_> = padded.tiles.iter().collect();
        let shifted = ShoreBlend::new("LakeOfRage", &padded_cells, &padded_grid);
        assert_eq!(
            blend.color([7., 0., 4.], &g, [13, 17], base),
            shifted.color([7., 0., 4.], &padded_grid, [12, 17], base)
        );
    }
    #[test]
    fn finished_damp_bank_retains_native_floor_corners_heights_and_claims() {
        let mut f = fixture();
        f.map_id = std::sync::Arc::from("LakeOfRage");
        let g = GridGeometry {
            origin_x: -12.,
            origin_z: -8.,
            ..grid()
        };
        let cells: Vec<_> = f.tiles.iter().collect();
        let mut mesh =
            build_terrain_mesh_with_samples(&f, &TerrainImageSamples::default()).unwrap();
        let corners: Vec<_> = mesh
            .textured
            .positions
            .iter()
            .zip(&mesh.textured.normals)
            .filter(|(p, n)| p[1] == 0. && n[1] > 0.999)
            .map(|(p, _)| *p)
            .collect();
        assert!(!corners.is_empty());
        let heights = mesh.footing_heights.clone();
        let claims = mesh.authored_cells.clone();
        polish_world_surfaces(&mut mesh, "LakeOfRage", &cells, &g, [0, 0]);
        assert_eq!(mesh.footing_heights, heights);
        assert_eq!(mesh.authored_cells, claims);
        for corner in corners {
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .zip(&mesh.solid.normals)
                    .chain(mesh.textured.positions.iter().zip(&mesh.textured.normals))
                    .any(|(p, n)| *p == corner && n[1] > 0.999)
            );
        }
    }
    #[test]
    fn missing_water_unknown_atlas_and_indoor_views_keep_original_ground() {
        let f = fixture();
        let g = grid();
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(ShoreBlend::new("Route43Gate", &cells, &g).edges.is_empty());
        let mut f = f;
        for t in &mut f.tiles {
            t.source = source_with_tile(0x01, 0, 0, 0x05);
        }
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(ShoreBlend::new("LakeOfRage", &cells, &g).edges.is_empty());
        let mut f = fixture();
        for t in &mut f.tiles {
            t.source.tileset_id = std::sync::Arc::from("unknown");
        }
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(ShoreBlend::new("LakeOfRage", &cells, &g).edges.is_empty());
    }
}
