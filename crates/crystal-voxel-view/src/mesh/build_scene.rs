/// Join only source-authored grass at the same rendered elevation.
fn connected_grass_neighbors(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    column: usize,
    row: usize,
) -> [bool; 4] {
    let index = row * geometry.width + column;
    [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)].map(|(dx, dz)| {
        let x = column as isize + dx;
        let z = row as isize + dz;
        if x < 0 || z < 0 || x >= geometry.width as isize || z >= geometry.height as isize {
            return false;
        }
        let neighbor = z as usize * geometry.width + x as usize;
        crate::grass::grass_shape(&cells[neighbor].source).is_some()
            && matches!(shapes[neighbor], CellShape::Cutout { solid: SolidKind::Grass, .. })
            && shapes[neighbor].surface_height(geometry.tile_height)
                == shapes[index].surface_height(geometry.tile_height)
    })
}

fn append_cave_foundation(out: &mut SurfaceMeshData, g: &GridGeometry, dark: bool, holes: &[bool], origin: [i32; 2]) {
    let w = g.origin_x;
    let e = w + g.width as f32 * g.tile_width;
    let n = g.origin_z;
    let s = n + g.height as f32 * g.tile_height;
    let color = if dark {
        [0.295, 0.355, 0.38, 1.]
    } else {
        [0.43, 0.405, 0.32, 1.]
    };
    // Match the visible floor's mineral domain even where a cutaway reveals
    // this backing surface. Geometry stays below the authored walking datum.
    let paint = |out: &mut SurfaceMeshData, positions: [[f32; 3]; 4]| {
        let uvs = positions.map(|p| cave_cliff::floor_uv(Vec3::from_array(p), g, origin, false));
        let colors = positions.map(|p| cave_cliff::pigment(color, Vec3::from_array(p), g, origin, false));
        append_quad_colors(out, positions, [0., 1., 0.], uvs, colors);
    };
    if holes.iter().any(|&hole| hole) {
        for row in 0..g.height {
            let mut column = 0;
            while column < g.width {
                if holes[row*g.width+column] { column += 1; continue; }
                let start = column;
                while column < g.width && !holes[row*g.width+column] { column += 1; }
                let (w,_,n,s) = g.bounds(start,row);
                let (_,e,_,_) = g.bounds(column-1,row);
                paint(out, [[w,-0.02,n],[w,-0.02,s],[e,-0.02,s],[e,-0.02,n]]);
            }
        }
        return;
    }
    paint(out, [[w, -0.02, n], [w, -0.02, s], [e, -0.02, s], [e, -0.02, n]]);
}

#[cfg(test)]
mod cave_foundation_tests {
    use super::*;
    #[test]
    fn foundation_aperture_removes_only_the_complete_hatch_footprint() {
        let g = GridGeometry { width: 4, height: 4, tile_width: 8.,tile_height:8.,origin_x:0.,origin_z:0. };
        let mut holes = vec![false;16];
        for i in [5,6,9,10] { holes[i] = true; }
        let mut mesh = SurfaceMeshData::default();
        append_cave_foundation(&mut mesh,&g,false,&holes, [0, 0]);
        for (i,&hole) in holes.iter().enumerate() {
            let x = (i%4) as f32*8.+4.;
            let z = (i/4) as f32*8.+4.;
            let coverage = mesh.positions.chunks_exact(4).filter(|q|
                x>q[0][0] && x<q[2][0] && z>q[0][2] && z<q[2][2]).count();
            assert_eq!(coverage,usize::from(!hole),"foundation cell {i}");
        }
        assert!(mesh.positions.iter().all(|p|p[1]==-0.02));
        assert!(mesh.normals.iter().all(|n|*n==[0.,1.,0.]));
    }
    #[test]
    fn foundation_closes_exact_grid_without_copying_face_art_or_raising_support() {
        let g = GridGeometry {
            width: 8,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -32.,
            origin_z: -24.,
        };
        for dark in [false, true] {
            let mut out = SurfaceMeshData::default();
            append_cave_foundation(&mut out, &g, dark, &[], [-12, 7]);
            assert_eq!(
                out.positions,
                vec![
                    [-32., -0.02, -24.],
                    [-32., -0.02, 24.],
                    [32., -0.02, 24.],
                    [32., -0.02, -24.]
                ]
            );
            for (p, uv) in out.positions.iter().zip(&out.uvs) {
                assert_eq!(*uv, cave_cliff::floor_uv(Vec3::from_array(*p), &g, [-12, 7], false));
                assert!((400000. ..500000.).contains(&uv[1]));
            }
            // Sliding the renderer grid keeps a shared source point painted
            // identically; the backing floor must not swim with the camera.
            let shifted = GridGeometry { origin_x: g.origin_x + 8., ..g };
            let point = Vec3::new(0., -0.02, 0.);
            assert_eq!(cave_cliff::floor_uv(point, &g, [-12, 7], false),
                cave_cliff::floor_uv(point, &shifted, [-11, 7], false));
            let base = if dark { [0.295, 0.355, 0.38, 1.] } else { [0.43, 0.405, 0.32, 1.] };
            assert_eq!(cave_cliff::pigment(base, point, &g, [-12, 7], false),
                cave_cliff::pigment(base, point, &shifted, [-11, 7], false));
            assert!(out.normals.iter().all(|n| *n == [0., 1., 0.]));
            assert_eq!(out.indices.len(), 6);
            assert!(out.colors.iter().all(|c| c[3] == 1.));
        }
    }
}

fn build_terrain_mesh_internal(
    frame: &VisualWorldFrame,
    images: Option<&TerrainImageSamples>,
    instance_hulls: bool,
    profiles: Option<&crate::live_profiles::Document>,
    mut grove_sink: Option<&mut dyn FnMut(SurfaceMeshData)>,
    deferred_groves: Option<&mut DeferredGroves>,
) -> Result<TerrainMeshData, TerrainMeshError> {
    let _trace = crate::loading_trace::span("region.build_source", || serde_json::json!({
        "map": frame.map_id.as_ref(), "origin": frame.grid_origin.to_array(), "grid": frame.grid_size.to_array()}));
    let mut trace_phase = None;
    crate::loading_trace::phase(&mut trace_phase, "region.classify_and_reserve");
    // The runtime's bundled/live profile document enables the full authored
    // kit. Low-level samples-only calls retain their legacy mesher contract
    // for diagnostic comparisons and regression fixtures.
    let authored_enabled = profiles.is_some();
    let jump_ledges: &[crystal_render_api::VisualJumpLedge] =
        if jump_changes_floor_level(&frame.map_id) {&frame.jump_ledges} else {&[]};
    let mut detached_groves = Vec::new();
    let scenery_metrics = (new_bark::supports_map(frame.map_id.as_ref())
        && std::env::var_os("CRYSTAL_SCENERY_METRICS").is_some())
    .then(std::time::Instant::now);
    frame
        .validate()
        .map_err(TerrainMeshError::InvalidVisualFrame)?;
    if !frame.active {
        return Err(TerrainMeshError::InactiveFrame);
    }

    let width = usize::try_from(frame.grid_size.x).map_err(|_| TerrainMeshError::GridTooLarge)?;
    let height = usize::try_from(frame.grid_size.y).map_err(|_| TerrainMeshError::GridTooLarge)?;
    let cell_count = width
        .checked_mul(height)
        .ok_or(TerrainMeshError::GridTooLarge)?;
    let mut cells = vec![None; cell_count];
    for tile in &frame.tiles {
        let column = usize::try_from(tile.column).map_err(|_| TerrainMeshError::GridTooLarge)?;
        let row = usize::try_from(tile.row).map_err(|_| TerrainMeshError::GridTooLarge)?;
        let index = row
            .checked_mul(width)
            .and_then(|base| base.checked_add(column))
            .ok_or(TerrainMeshError::GridTooLarge)?;
        if cells[index].replace(tile).is_some() {
            return Err(TerrainMeshError::DuplicateTile {
                column: tile.column,
                row: tile.row,
            });
        }
    }
    if let Some(index) = cells.iter().position(Option::is_none) {
        return Err(TerrainMeshError::MissingTile {
            column: (index % width) as u32,
            row: (index / width) as u32,
        });
    }

    let cells: Vec<&VisualTile> = cells
        .into_iter()
        .map(|tile| tile.expect("complete tile grid was checked before meshing"))
        .collect();
    let grid_width = frame.tile_size.x * frame.grid_size.x as f32;
    let grid_height = frame.tile_size.y * frame.grid_size.y as f32;
    let geometry = GridGeometry {
        width,
        height,
        tile_width: frame.tile_size.x,
        tile_height: frame.tile_size.y,
        origin_x: -grid_width * 0.5,
        origin_z: -grid_height * 0.5,
    };
    // All model matching observes the immutable, currently published source.
    // Reserve only complete drawings with their required ground sample, then
    // mask those identities from every old matcher to prevent double geometry.
    let original_cells = cells.clone();
    let mut source_shapes: Vec<_> = original_cells
        .iter()
        .map(|tile| shape_for_source_on_map(frame.map_id.as_ref(), &tile.source))
        .collect();
    cave_crowns::resolve(frame, &original_cells, &mut source_shapes, &geometry);
    resolve_rock_platform_tiers(&original_cells, &mut source_shapes, width);
    resolve_authored_mountain_tiers(&mut source_shapes, width, height);
    if jump_ledges.is_empty() {
        taper_johto_ledge_ends(&original_cells, &mut source_shapes, width);
    }
    let mut interior_placements = if images.is_some() && authored_enabled {
        modeled_interiors::resolve(&frame.map_id, &original_cells, &geometry, profiles)
    } else {
        Vec::new()
    };
    let mut authored_reserved = vec![false; cell_count];
    let olivine_stage_placement = if images.is_some() && authored_enabled {
        olivine_stage::resolve(&frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved)
    } else { None };
    if let Some(stage) = &olivine_stage_placement {
        for i in stage.indices(width) { authored_reserved[i] = true; }
    }
    // Complete institutional apparatus supersedes only the exact old partial
    // signature models; live custom profiles retain first refusal.
    let facility_radio_placements = if images.is_some() && authored_enabled {
        facility_radio::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &facility_radio_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let facility_table_placements = if images.is_some() && authored_enabled {
        facility_tables::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &facility_table_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    // Complete joined store courses supersede older individual window models.
    // Their resolver still gives all customized live objects first refusal.
    let department_store_placements = if images.is_some() && authored_enabled {
        department_store::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &department_store_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    interior_placements.retain(|placement| {
        if placement
            .indices(width)
            .any(|index| authored_reserved[index])
        {
            return false;
        }
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
        true
    });
    let gate_placements = if images.is_some() && authored_enabled {
        gate_counters::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &gate_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let cable_club_placements = if images.is_some() && authored_enabled {
        cable_club::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &cable_club_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let train_placements = if images.is_some() && authored_enabled {
        train_station_scenery::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &train_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let outdoor_sign_placements = if images.is_some() && authored_enabled {
        outdoor_signs::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &outdoor_sign_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let gym_placements = if images.is_some() && authored_enabled {
        gym_scenery::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &gym_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let violet_pits = if images.is_some() && authored_enabled {
        violet_gym_pits::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            profiles,
            &authored_reserved,
        )
    } else {
        None
    };
    if let Some(p) = &violet_pits {
        for index in p.indices() {
            authored_reserved[index] = true;
        }
    }
    let traditional_room_placements = if images.is_some() && authored_enabled {
        traditional_room::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &traditional_room_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let ship_front_placements = if images.is_some() && authored_enabled {
        ship_fronts::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &ship_front_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let ship_room_placements = if images.is_some() && authored_enabled {
        ship_rooms::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &ship_room_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let park_placements = if images.is_some() && authored_enabled {
        park_scenery::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &park_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let park_side_placements = if images.is_some() && authored_enabled {
        park_side_fence::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for p in &park_side_placements {
        for i in p.indices(width) {
            authored_reserved[i] = true;
        }
    }
    let park_flowerbed_placements = if images.is_some() && authored_enabled {
        park_flowerbed::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for p in &park_flowerbed_placements {
        for i in p.indices(width) {
            authored_reserved[i] = true;
        }
    }
    let rocket_plant_placements = if images.is_some() && authored_enabled {
        rocket_plants::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &rocket_plant_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let mut dungeon_placements = if images.is_some() && authored_enabled {
        modeled_dungeons::resolve(&frame.map_id, &original_cells, &geometry, profiles)
    } else {
        Vec::new()
    };
    dungeon_placements.retain(|placement| {
        if placement
            .indices(width)
            .any(|index| authored_reserved[index])
        {
            return false;
        }
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
        true
    });
    // Three native maps lack the catalog lawn sample. Reserve only their
    // source-complete objects with proven ground inside this map's bounds.
    let native_ground_placements = if images.is_some() && authored_enabled {
        native_ground_bindings::resolve(
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
            &authored_reserved,
        )
    } else {
        Vec::new()
    };
    for placement in &native_ground_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    crate::loading_trace::phase(&mut trace_phase, "region.resolve_profiles");
    let mut live_placements = live::resolve(
        &cells,
        width,
        height,
        frame.map_id.as_ref(),
        profiles.filter(|_| images.is_some()),
    );
    if authored_enabled {
        live_placements.retain(|placement| !modeled_exteriors::native_stone_profile(placement.object));
    }
    live_placements.retain(|placement| {
        !placement
            .indices(width)
            .any(|index| authored_reserved[index])
    });
    if images.is_some() && authored_enabled {
        let reserved =
            modeled_exteriors::preferred_cells(&frame.map_id, &original_cells, &geometry);
        live_placements.retain(|placement| !placement.indices(width).all(|index| reserved[index]));
    }
    if images.is_some() && new_bark::supports_grove(frame.map_id.as_ref()) {
        let reserved = new_bark::preferred_scenery_cells(frame.map_id.as_ref(), &cells, &geometry);
        // Complete authored trees/signs supersede their old source-art cards.
        // Mixed props and incomplete groups retain the existing live-profile path.
        live_placements.retain(|placement| !placement.indices(width).all(|index| reserved[index]));
    }
    // Mask complete overridden drawings from all compiled object matchers.
    // Texture slots stay fixed so the live mesher still uses the original art.
    let mut live_cells: Vec<VisualTile> =
        if live_placements.is_empty() && !authored_reserved.iter().any(|&v| v) {
            Vec::new()
        } else {
            cells.iter().map(|tile| (*tile).clone()).collect()
        };
    let mut live_claimed = vec![false; cell_count];
    for placement in &live_placements {
        for index in placement.indices(width) {
            live_claimed[index] = true;
            // Keep genuine ground texture identities available to other props.
            // Ownership suppresses drawing, not sampling from the immutable atlas.
            let sampleable = matches!(
                shape_for_source_on_map(frame.map_id.as_ref(), &cells[index].source),
                CellShape::Flat | CellShape::Water
            );
            live_cells[index].source.metatile_id = u16::MAX;
            if !sampleable {
                live_cells[index].source.tile_index = u16::MAX;
            }
        }
    }
    for (index, &reserved) in authored_reserved.iter().enumerate() {
        if reserved {
            live_cells[index].source.metatile_id = u16::MAX;
            if !matches!(source_shapes[index], CellShape::Flat | CellShape::Water) {
                live_cells[index].source.tile_index = u16::MAX;
            }
        }
    }
    let cells = if live_cells.is_empty() {
        cells
    } else {
        live_cells.iter().collect()
    };
    let mut shapes: Vec<_> = cells
        .iter()
        .map(|tile| shape_for_source_on_map(frame.map_id.as_ref(), &tile.source))
        .collect();
    for (i, shape) in shapes.iter_mut().enumerate() {
        if cells[i].source == original_cells[i].source
            && (original_cells[i].source.tile_index == 0x16
                || (cave_crowns::enabled(frame.map_id.as_ref()) && source_shapes[i]!=shape_for_source_on_map(frame.map_id.as_ref(), &original_cells[i].source)))
            && matches!(original_cells[i].source.tileset_id.as_ref(), "cave" | "dark_cave") {
            *shape = source_shapes[i];
        }
    }
    resolve_rock_platform_tiers(&cells, &mut shapes, width);
    // The production canyon owns complete Johto mounds as well as their
    // partial/connected forms. Only the legacy samples-only diagnostic uses
    // the old standalone object and needs its source cells flattened first.
    if !authored_enabled {
        for (column, row) in johto_closed_mound_origins(&cells, width, height) {
            for local_row in 0..8 {
                for local_column in 0..12 {
                    shapes[(row + local_row) * width + column + local_column] = CellShape::Flat;
                }
            }
        }
    }
    // Upright profiles require an exact live background cell for both the
    // vacated floor and pixel mask. A clipped viewport may not carry that
    // evidence; resolve only that shape back to the documented flat baseline
    // instead of failing the complete renderer or guessing from collision.
    let available_flat_tiles: std::collections::HashSet<_> = cells
        .iter()
        .zip(&shapes)
        .filter_map(|(tile, shape)| {
            matches!(shape, CellShape::Flat).then_some(tile.source.tile_index)
        })
        .collect();
    // A connected viewport can contain the requested number only in another
    // atlas. That is not evidence for this cutout's native background.
    let native_grounds: std::collections::HashSet<_> = cells.iter().zip(&shapes)
        .filter_map(|(tile, shape)| matches!(shape,
            CellShape::Flat | CellShape::PlaneAt { height: 0. }
            | CellShape::RaisedTop { solid: SolidKind::Bank, .. })
            .then_some((tile.source.tileset_id.as_ref(), tile.source.tile_index)))
        .collect();
    let available_relief_base_tiles: std::collections::HashSet<_> = cells
        .iter()
        .zip(&shapes)
        .filter_map(|(tile, shape)| {
            matches!(
                shape,
                CellShape::Flat
                    | CellShape::Water
                    | CellShape::RaisedTop {
                        solid: SolidKind::Bank,
                        ..
                    }
            )
            .then_some(tile.source.tile_index)
        })
        .collect();
    for (tile, shape) in cells.iter().zip(&mut shapes) {
        if let CellShape::FacadeBand { ground_tile_index, .. }
            | CellShape::Cutout { ground_tile_index, .. } = *shape
            && source_bound_outdoor_ground(&tile.source)
            && !native_grounds.contains(&(tile.source.tileset_id.as_ref(), ground_tile_index))
        {
            *shape = CellShape::Flat;
        }
        if let CellShape::FacadeBand {
            ground_tile_index, ..
        } = *shape
            && !available_flat_tiles.contains(&ground_tile_index)
        {
            *shape = CellShape::Flat;
        }
        if let CellShape::Cutout {
            ground_tile_index, ..
        } = *shape
            && !available_flat_tiles.contains(&ground_tile_index)
        {
            *shape = CellShape::Flat;
        }
        if let CellShape::Relief {
            ground_tile_index, ..
        }
        | CellShape::FloatingRelief {
            ground_tile_index, ..
        } = *shape
            && !available_relief_base_tiles.contains(&ground_tile_index)
        {
            *shape = CellShape::Flat;
        }
    }

    resolve_authored_mountain_tiers(&mut shapes, width, height);
    if jump_ledges.is_empty() {
        taper_johto_ledge_ends(&cells, &mut shapes, width);
    }

    crate::loading_trace::phase(&mut trace_phase, "region.allocate_base");
    let mut mesh = TerrainMeshData {
        tree_cache: instance_hulls.then(HashMap::new),
        authored_cells: vec![None; cell_count],
        ..Default::default()
    };
    if let Some(tileset) = cells.first().map(|tile| tile.source.tileset_id.as_ref()) {
        if !crate::interior::has_back_wall(tileset)
            && !crate::interior::has_room_perimeter(&frame.map_id,tileset)
            && !crate::interior_cutaway::is_cave(frame) {
            background::append_repeating_background_apron(
                &mut mesh, &geometry, &original_cells, &source_shapes);
        }
    }
    mesh.cave_enclosure = cave_enclosure::build(frame, &geometry);
    // Close the foundation beneath folded cave volumes. Repeating the full
    // source drawing here paints wall/ladder art onto the floor when a tall
    // foreground wall is cut away. The authenticated cave profile supplies
    // stone beneath those volumes; live authored surfaces remain above it.
    if cells
        .iter()
        .all(|tile| matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave"))
    {
        let mut foundation_holes = vec![false; cell_count];
        for hatch in dungeon_placements.iter().filter(|p| p.cuts_foundation()) {
            for index in hatch.indices(width) { foundation_holes[index] = true; }
        }
        append_cave_foundation(
            &mut mesh.solid,
            &geometry,
            cells[0].source.tileset_id.as_ref() == "dark_cave",
            &foundation_holes,
            frame.grid_origin.to_array(),
        );
    }
    mesh.footing_heights = original_cells
        .iter()
        .zip(&source_shapes)
        .map(|(tile, shape)| match shape {
            _ if crate::dance_theater::shape(frame.map_id.as_ref(), &tile.source).is_some() => {
                shape.surface_height(frame.tile_size.y)
            }
            CellShape::RaisedTop {
                solid: SolidKind::Bank,
                ..
            }
            | CellShape::LedgeBand { .. } => shape.surface_height(frame.tile_size.y),
            CellShape::RampNorth {
                north_height,
                south_height,
            } => (north_height + south_height) * 0.5 * frame.tile_size.y / SOURCE_TILE_HEIGHT,
            CellShape::RampEast {
                west_height,
                east_height,
            } => (west_height + east_height) * 0.5 * frame.tile_size.y / SOURCE_TILE_HEIGHT,
            _ => support_height(&tile.source, frame.tile_size.y),
        })
        .collect();
    let pier_field=(!jump_ledges.is_empty()).then(||terraces::Field::with_source(
        jump_ledges,&geometry,frame.grid_origin.to_array(),&original_cells).with_datum(frame.terrace_datum.as_deref()));
    if authored_enabled && images.is_some() {
        elevated_piers::support(&mut mesh,&frame.map_id,&original_cells,&geometry,pier_field.as_ref());
    }
    crate::loading_trace::phase(&mut trace_phase, "region.append_authored_assets");
    let mut claimed_by_building = vec![false; cell_count];
    let mut claimed_by_tree = live_claimed;
    if authored_enabled && crate::cave_water::outdoor(frame.map_id.as_ref()) {
        // Exterior Johto seas share the Den's exact complete whirlpool art.
        // Native funnels retain the local water phase and authoritative tiles.
        let native_whirlpools=den_whirlpool::append(&mut mesh,&original_cells,&geometry);
        let _whirlpool_trace=crate::loading_trace::span("water.whirlpool_sources",|| {
            let starts=original_cells.iter().enumerate().filter(|(_,t)|
                t.source.tileset_id.as_ref()=="johto" && t.source.metatile_id==7
                    && t.source.subtile_column==0 && t.source.subtile_row==0);
            serde_json::json!({"native_cells":native_whirlpools.len(),"blocks":starts.take(24)
                .map(|(i,_)|serde_json::json!({"origin":[i%width,i/width],
                    "selected":native_whirlpools.contains(&i),
                    "water": ([8,9,12,13].into_iter().filter_map(|j| {
                        let index=i+(j/4)*width+j%4;
                        original_cells.get(index).map(|t|serde_json::json!({
                            "source":format!("{:?}",t.source),
                            "frames":t.animation_frames.as_ref().map_or(0,|f|f.len()),
                            "current_in_family":t.animation_frames.as_ref().is_some_and(|f|f.contains(&t.texture))}))
                    }).collect::<Vec<_>>()) })).collect::<Vec<_>>()})
        });
        for i in native_whirlpools {
            claimed_by_tree[i]=true;
            mesh.authored_cells[i]=Some("johto/whirlpool");
        }
    }
    if authored_enabled {
        for (bank,donor) in crate::border_water::replacements(frame) {
            let i=bank.y as usize*width+bank.x as usize;
            if mesh.authored_cells[i].is_some() || claimed_by_tree[i] {continue;}
            claimed_by_tree[i]=true;
            mesh.authored_cells[i]=Some("outdoor/border-water");
            // Adjacent geometry sees the same water surface rather than the
            // retired corner's dry height. Native map/collision data stays put.
            shapes[i]=CellShape::Water;
            let (w,e,n,s)=geometry.bounds(bank.x as usize,bank.y as usize);
            append_top(&mut mesh.animated_textured,[w,e,n,s],
                CellShape::Water.surface_height(geometry.tile_height),
                geometry.uv(donor.x as usize,donor.y as usize));
        }
    }
    // Authenticated blank cave blocks belong to the surrounding rock mass.
    // Suppress their old black atlas planes beneath the new mineral bases.
    for i in cave_enclosure::void_cells(frame,&geometry) {
        claimed_by_tree[i]=true;
        mesh.authored_cells[i]=Some("cave:exterior-rock");
    }
    if authored_enabled && images.is_some() {
        city_gardens::append(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            &mut claimed_by_tree,
        );
    }
    if authored_enabled && images.is_some() {
        city_railway::append(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            &mut claimed_by_tree,
        );
    }
    for placement in &facility_radio_placements {
        let appended = facility_radio::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated facility/radio reservation must append");
    }
    for placement in &facility_table_placements {
        let appended = facility_tables::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated facility table reservation must append");
    }
    for placement in &interior_placements {
        let appended = modeled_interiors::append_with_tables(
            &mut mesh,
            frame.map_id.as_ref(),
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
            frame.grid_origin.to_array(),
            &interior_placements,
        );
        debug_assert!(appended, "validated interior reservation must append");
    }
    for placement in &gate_placements {
        let appended = gate_counters::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated gate network reservation must append");
    }
    for placement in &cable_club_placements {
        let appended = cable_club::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated Cable Club reservation must append");
    }
    for placement in &train_placements {
        let appended = train_station_scenery::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated station train reservation must append");
    }
    for placement in &outdoor_sign_placements {
        let appended = outdoor_signs::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated outdoor sign reservation must append");
    }
    for placement in &gym_placements {
        let appended = gym_scenery::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated Gym reservation must append");
    }
    if let Some(p) = &violet_pits {
        let appended = violet_gym_pits::append(
            &mut mesh,
            &original_cells,
            &geometry,
            p,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated Violet recess reservation must append");
    }
    for placement in &traditional_room_placements {
        let appended = traditional_room::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(
            appended,
            "validated traditional-room reservation must append"
        );
    }
    for placement in &park_placements {
        let appended = park_scenery::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated park fixture reservation must append");
    }
    for p in &park_side_placements {
        let appended = park_side_fence::append(
            &mut mesh,
            &original_cells,
            &geometry,
            p,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated park side fence must append");
    }
    for p in &park_flowerbed_placements {
        let appended = park_flowerbed::append(
            &mut mesh,
            &original_cells,
            &geometry,
            p,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated park flowerbed must append");
    }
    if let Some(stage) = &olivine_stage_placement {
        let appended = olivine_stage::append(&mut mesh, &geometry, stage, &mut claimed_by_tree);
        debug_assert!(appended, "validated Olivine terrace must append");
    }
    for placement in &department_store_placements {
        let appended = department_store::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(
            appended,
            "validated department store reservation must append"
        );
    }
    for placement in &ship_front_placements {
        let appended = ship_fronts::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated room-front reservation must append");
    }
    for placement in &ship_room_placements {
        let appended = ship_rooms::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated ship room reservation must append");
    }
    for placement in &rocket_plant_placements {
        let appended = rocket_plants::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated Rocket plant reservation must append");
    }
    modeled_dungeons::append_all_with_samples(&mut mesh, &geometry, &dungeon_placements,
        &original_cells, frame.grid_origin.to_array(), images);
    for placement in &dungeon_placements {
        for index in placement.indices(width) {
            claimed_by_tree[index] = true;
            mesh.authored_cells[index] = Some(placement.kind_label());
        }
    }
    for placement in &native_ground_placements {
        let appended = native_ground_bindings::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated native ground binding must append");
    }
    if images.is_some() && authored_enabled {
        slowpoke_well::append(
            &mut mesh,
            &original_cells,
            &source_shapes,
            &geometry,
            &mut claimed_by_tree,
            images.expect("checked image samples"),
        );
        modeled_exteriors::append_props_with_datum(
            &mut mesh,
            &cells,
            &shapes,
            &geometry,
            &mut claimed_by_tree,
            images,
            frame.grid_origin.to_array(),
            frame.terrace_datum.as_deref(),
        );
    }
    if let Some(images) = images {
        append_johto_vertical_fences(&mut mesh, images, &cells, &geometry, &mut claimed_by_tree)?;
    }
    if let Some(images) = images {
        for placement in &live_placements {
            if !authored_enabled
                || !modeled_exteriors::append_live(&mut mesh, &original_cells, &geometry, placement, frame.grid_origin.to_array())
            {
                live::append(&mut mesh, &geometry, placement, &cells, images)?;
            }
        }
    }
    let mut claimed_by_casino_stool = vec![false; cell_count];
    let mut claimed_by_house_furniture = vec![false; cell_count];
    append_lighthouse_side_walls(&mut mesh, &cells, &geometry, &mut claimed_by_tree);
    traditional_house::append_north_wall_courses(
        &mut mesh,
        &cells,
        &geometry,
        &mut claimed_by_tree,
    )?;
    ordinary_house::append_north_wall_courses(&mut mesh, &cells, &geometry, &mut claimed_by_tree)?;
    players_house::append_north_wall_courses(&mut mesh, &cells, &geometry, &mut claimed_by_tree)?;
    append_house_stairs(
        &mut mesh,
        frame.map_id.as_ref(),
        &cells,
        &geometry,
        &mut claimed_by_tree,
    );
    append_facility_divider_network(
        &mut mesh,
        frame.map_id.as_ref(),
        &cells,
        &geometry,
        &mut claimed_by_tree,
    )?;
    append_rocket_base_wall_network(
        &mut mesh,
        frame.map_id.as_ref(),
        &cells,
        &geometry,
        &mut claimed_by_tree,
    );
    for (column, row) in ice_path_closed_rock_mass_origins(&cells, geometry.width, geometry.height)
    {
        append_ice_path_closed_rock_mass(
            &mut mesh,
            &cells,
            &geometry,
            column,
            row,
            &mut claimed_by_building,
        )?;
    }
    if let Some(images) = images {
        let placements = if authored_enabled {
            crate::loading_trace::phase(&mut trace_phase, "region.append_outdoor_objects");
            modeled_exteriors::building_placements(frame.map_id.as_ref(), &cells, &geometry)
        } else {
            new_bark::building_placements(frame.map_id.as_ref(), &cells, &geometry)
        };
        for placement in &placements {
            if new_bark::append_building_with_samples(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                frame.map_id.as_ref(),
                *placement,
                &mut claimed_by_building,
                Some(images),
            ) {
                continue;
            }
            if authored_enabled
                && modeled_exteriors::append_building_with_samples(
                    &mut mesh,
                    &cells,
                    &shapes,
                    &geometry,
                    &frame.map_id,
                    *placement,
                    &mut claimed_by_building,
                    frame.grid_origin.to_array(),
                    Some(images),
                )
            {
                continue;
            }
            append_pixel_building(
                &mut mesh,
                images,
                &cells,
                &geometry,
                frame.map_id.as_ref(),
                *placement,
                &mut claimed_by_building,
            )?;
            if is_complete_rock_formation(&cells, &geometry, *placement) {
                let height =
                    crate::cave::MOUND_FACE_HEIGHT * geometry.tile_height / SOURCE_TILE_HEIGHT;
                for row in placement.row..placement.row + placement.roof_rows {
                    for column in placement.column..placement.column + placement.width {
                        mesh.footing_heights[row * geometry.width + column] = height;
                    }
                }
            }
        }
        // A partial template at the viewport edge is not enough evidence to
        // invent half a building. Preserve the faithful flat drawing until
        // the complete authored placement is available.
        for (index, shape) in shapes.iter_mut().enumerate() {
            if shape.solid_kind() == SolidKind::Building && !claimed_by_building[index] {
                *shape = CellShape::Flat;
            }
        }

        // Missing 3D trees retain exact source tiles. Never substitute the
        // legacy voxel hull for unavailable canonical tree assets.
        for placement in new_bark::tree_placements(frame.map_id.as_ref(), &cells, &geometry) {
            let appended = if grove_sink.is_some() || deferred_groves.is_some() {
                if let Some(grove) = new_bark::claim_grove(&mut mesh, &cells, &shapes,
                    &geometry, frame.map_id.as_ref(), placement, &mut claimed_by_tree,
                    frame.grid_origin.to_array()) {
                    detached_groves.push(grove);
                    true
                } else { false }
            } else {
                new_bark::append_tree(&mut mesh, &cells, &shapes, &geometry,
                    frame.map_id.as_ref(), placement, &mut claimed_by_tree,
                    frame.grid_origin.to_array())
            };
            if appended {
                continue;
            }
            if authored_enabled
                && modeled_exteriors::append_tree_with_groves(
                    &mut mesh,
                    &cells,
                    &shapes,
                    &geometry,
                    &frame.map_id,
                    placement,
                    &mut claimed_by_tree,
                    frame.grid_origin.to_array(),
                    (grove_sink.is_some() || deferred_groves.is_some()).then_some(&mut detached_groves),
                )
            {
                continue;
            }
            let _fallback_trace = crate::loading_trace::span("tree.source_card", || {
                let i = placement.row * geometry.width + placement.column;
                let owned = (placement.row..placement.row + placement.height)
                    .flat_map(|row| (placement.column..placement.column + placement.width)
                        .map(move |column| row * geometry.width + column))
                    .filter(|&index| claimed_by_tree[index]).count();
                serde_json::json!({"plot":[placement.column,placement.row,placement.width,placement.height],
                    "owned_cells":owned,"original_source":format!("{:?}", original_cells[i].source),
                    "masked_source":format!("{:?}", cells[i].source)})
            });
            modeled_exteriors::append_missing_tree_source(
                &mut mesh, &geometry, placement, &mut claimed_by_tree);
        }
        let _tree_ownership_trace = crate::loading_trace::span("tree.source_ownership", || {
            let unclaimed = original_cells.iter().enumerate().filter(|(i, tile)| {
                !claimed_by_tree[*i] && (source_shapes[*i].solid_kind() == SolidKind::Tree
                    || (matches!(tile.source.tileset_id.as_ref(), "johto" | "johto_modern")
                        && matches!(tile.source.tile_index, 0x1e | 0x1f | 0x2e | 0x2f | 0x3e | 0x3f)))
            }).collect::<Vec<_>>();
            let mut rows = std::collections::BTreeMap::<usize,usize>::new();
            for (i,_) in &unclaimed { *rows.entry(i/width).or_default() += 1; }
            let mut ground_samples = std::collections::BTreeMap::<usize,usize>::new();
            for plot in new_bark::tree_placements(frame.map_id.as_ref(),&cells,&geometry) {
                if let Some(i)=new_bark::modeled_tree_ground(frame.map_id.as_ref(),&cells,&shapes,&geometry,plot) {
                    *ground_samples.entry(i).or_default() += 1;
                }
            }
            serde_json::json!({"map":frame.map_id.as_ref(),"origin":frame.grid_origin.to_array(),
                "ground_samples":ground_samples.iter().take(16).map(|(i,count)|serde_json::json!({
                    "sample_cell":[i%width,i/width],"plots":count,
                    "height":shapes[*i].surface_height(geometry.tile_height),
                    "source":format!("{:?}",cells[*i].source)})).collect::<Vec<_>>(),
                "unclaimed_count":unclaimed.len(),"rows":rows,
                "interior_cells":unclaimed.iter().filter(|(i,_)|
                    i%width > 0 && i%width+1 < width && i/width > 0 && i/width+1 < height)
                    .take(24).map(|(i,tile)|serde_json::json!({"cell":[i%width,i/width],
                        "source":format!("{:?}",tile.source)})).collect::<Vec<_>>(),
                "first_cells":unclaimed.iter().take(24)
                    .map(|(i,tile)| serde_json::json!({"cell":[i%width,i/width],
                        "source":format!("{:?}",tile.source),"live_source":format!("{:?}",cells[*i].source)}))
                    .collect::<Vec<_>>()})
        });
        for placement in park_bench_placements(&cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in wise_trios_divider_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in kanto_round_barrier_placements(&cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in kanto_shoreline_round_barrier_placements(&cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                // A complete shoreline rock may enter the source halo before
                // a separate animated-water sample does. Leave just that
                // drawing on the faithful map plane; an optional prop must
                // never make the whole 2.5D frame disappear while scrolling.
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        if frame.map_id.as_ref() == "CeladonGym" {
            for placement in celadon_hedge_placements(&cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        if frame.map_id.as_ref() == "VermilionGym" {
            for placement in vermilion_statue_placements(frame.map_id.as_ref(), &cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        if frame.map_id.as_ref() == "ViridianGym" {
            for placement in viridian_statue_placements(frame.map_id.as_ref(), &cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        for placement in pokecom_workstation_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in pokecom_plant_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in pokecom_chair_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in ship_stool_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in ship_rack_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in ship_barrel_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in train_station_seat_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in train_station_planter_placements(frame.map_id.as_ref(), &cells, &geometry)
        {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in train_station_gate_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in warehouse_crate_placements(frame.map_id.as_ref(), &cells, &geometry) {
            append_warehouse_crate(
                &mut mesh,
                &cells,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in house_plant_placements(&cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in house_upright_fixture_placements(&cells, &geometry) {
            if (placement.row..placement.row + placement.height).any(|row| {
                (placement.column..placement.column + placement.width)
                    .any(|column| claimed_by_tree[row * geometry.width + column])
            }) {
                continue;
            }
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in house_bookcase_placements(&cells, &geometry) {
            append_house_bookcase(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in
            traditional_gift_shop_shelf_placements(frame.map_id.as_ref(), &cells, &geometry)
        {
            append_traditional_gift_shop_shelf(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in players_house_bookcase_placements(&cells, &geometry) {
            append_house_bookcase_with_ground(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
                crate::players_house::FLOOR_TILE,
            )?;
        }
        for placement in players_house_upright_fixture_placements(&cells, &geometry) {
            if (placement.row..placement.row + placement.height).any(|row| {
                (placement.column..placement.column + placement.width)
                    .any(|column| claimed_by_tree[row * geometry.width + column])
            }) {
                continue;
            }
            if cells[placement.row * geometry.width + placement.column]
                .source
                .metatile_id
                == 0x07
            {
                append_kitchen_fixture(&mut mesh, &geometry, placement, &mut claimed_by_tree);
                continue;
            }
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in players_house_tv_placements(&cells, &geometry) {
            if (placement.row..placement.row + placement.height).any(|row| {
                (placement.column..placement.column + placement.width)
                    .any(|column| claimed_by_tree[row * geometry.width + column])
            }) {
                continue;
            }
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in players_house_console_placements(&cells, &geometry) {
            if let Err(error) = append_shallow_top_group(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
                crate::players_house::FLOOR_TILE,
                3.0,
            ) && !matches!(error, TerrainMeshError::MissingGroundSample { .. })
            {
                return Err(error);
            }
        }
        for placement in players_house_bed_placements(&cells, &geometry) {
            append_player_bed_card(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in traditional_house_radio_placements(&cells, &geometry) {
            if (placement.row..placement.row + placement.height).any(|row| {
                (placement.column..placement.column + placement.width)
                    .any(|column| claimed_by_tree[row * geometry.width + column])
            }) {
                continue;
            }
            let mut placement = placement;
            placement.base_height = 4.0 * geometry.tile_height / SOURCE_TILE_HEIGHT;
            let mut overlay_claimed = vec![false; cells.len()];
            append_grouped_tree_overlay(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut overlay_claimed,
            )?;
        }
        for placement in traditional_house_cushion_placements(&frame.map_id, &cells, &geometry) {
            if let Err(error) = append_shallow_top_group(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
                crate::house::TRADITIONAL_HOUSE_FLOOR_TILE,
                2.0,
            ) && !matches!(error, TerrainMeshError::MissingGroundSample { .. })
            {
                return Err(error);
            }
        }
        if frame.map_id.as_ref() == "SoulHouse" {
            for placement in soul_house_bench_placements(&cells, &geometry) {
                if let Err(error) = append_shallow_top_group(
                    &mut mesh,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                    crate::house::HOUSE_FLOOR_TILE,
                    4.0,
                ) && !matches!(error, TerrainMeshError::MissingGroundSample { .. })
                {
                    return Err(error);
                }
            }
        }
        for placement in house_furniture_placements(&cells, &geometry) {
            append_house_furniture(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_house_furniture,
            )?;
        }
        for placement in house_table_placements(&cells, &geometry) {
            append_house_table(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_house_furniture,
            )?;
        }
        for mut placement in house_open_book_placements(&cells, &geometry) {
            placement.base_height = crate::house::FurnitureKind::Table.height()
                * geometry.tile_height
                / SOURCE_TILE_HEIGHT;
            let mut overlay_claimed = vec![false; cells.len()];
            append_grouped_tree_overlay(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut overlay_claimed,
            )?;
        }
        if crate::azalea_gym::supports_display_map(frame.map_id.as_ref()) {
            for placement in
                elite_four_gym_card_placements(frame.map_id.as_ref(), &cells, &geometry)
            {
                if let Err(error) = append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                ) {
                    if matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                        for row in placement.row..placement.row + placement.height {
                            for column in placement.column..placement.column + placement.width {
                                claimed_by_tree[row * geometry.width + column] = false;
                            }
                        }
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        if frame.map_id.as_ref() == "HallOfFame" {
            for placement in hall_of_fame_console_placements(&cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        if matches!(
            frame.map_id.as_ref(),
            "VioletGym" | "MahoganyGym" | "BlackthornGym1F"
        ) {
            for placement in violet_gym_card_placements(frame.map_id.as_ref(), &cells, &geometry) {
                if let Err(error) = append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                ) {
                    if matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                        for row in placement.row..placement.row + placement.height {
                            for column in placement.column..placement.column + placement.width {
                                claimed_by_tree[row * geometry.width + column] = false;
                            }
                        }
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        if frame.map_id.as_ref() == "CeruleanGym" {
            for placement in cerulean_statue_placements(&cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        if frame.map_id.as_ref() == "FuchsiaGym" {
            for placement in fuchsia_statue_placements(frame.map_id.as_ref(), &cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        if frame.map_id.as_ref() == "CeladonGym" {
            for placement in celadon_statue_placements(frame.map_id.as_ref(), &cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        if frame.map_id.as_ref() == "OlivineGym" {
            for placement in olivine_gym_statue_placements(frame.map_id.as_ref(), &cells, &geometry)
            {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
            for placement in olivine_gym_boulder_placements(&cells, &geometry) {
                if let Err(error) = append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                ) {
                    if matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                        for row in placement.row..placement.row + placement.height {
                            for column in placement.column..placement.column + placement.width {
                                claimed_by_tree[row * geometry.width + column] = false;
                            }
                        }
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        if frame.map_id.as_ref() == "PewterGym" {
            for placement in tower_boulder_placements(&cells, &geometry) {
                if let Err(error) = append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                ) {
                    if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                        return Err(error);
                    }
                }
            }
        }
        if frame.map_id.as_ref() == "SaffronGym" {
            for placement in saffron_gym_planter_placements(&cells, &geometry) {
                if let Err(error) = append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                ) {
                    if matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                        for row in placement.row..placement.row + placement.height {
                            for column in placement.column..placement.column + placement.width {
                                claimed_by_tree[row * geometry.width + column] = false;
                            }
                        }
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        if crate::elite_four_room::supports_boulder_map(frame.map_id.as_ref()) {
            for placement in elite_four_room_boulder_placements(&cells, &geometry) {
                if let Err(error) = append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                ) {
                    if matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                        for row in placement.row..placement.row + placement.height {
                            for column in placement.column..placement.column + placement.width {
                                claimed_by_tree[row * geometry.width + column] = false;
                            }
                        }
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        for placement in ice_path_boulder_placements(&cells, &geometry) {
            // The drawing combines eight cap rows with eight front rows.
            // Its physical rise is the front course, not both bands stacked.
            append_grouped_tree_scaled(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
                0.5,
            )?;
        }
        for placement in ice_path_edge_rock_placements(&cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        for placement in ruins_statue_placements(&cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        for placement in power_plant_plant_placements(frame.map_id.as_ref(), &cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        for placement in rocket_base_plant_placements(frame.map_id.as_ref(), &cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        if crate::casino::is_game_corner_map(frame.map_id.as_ref())
            || crate::cafe::is_cafe_map(frame.map_id.as_ref())
        {
            for placement in casino_stool_placements(&cells, &geometry) {
                append_casino_stool(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_casino_stool,
                )?;
            }
        }
        if crate::casino::is_game_corner_map(frame.map_id.as_ref()) {
            for placement in casino_slot_machine_placements(&cells, &geometry) {
                append_grouped_tree_scaled(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                    1.6,
                )?;
            }
            for placement in casino_terminal_placements(&cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
            for placement in casino_plant_placements(&cells, &geometry) {
                append_grouped_tree(
                    &mut mesh,
                    images,
                    &cells,
                    &shapes,
                    &geometry,
                    placement,
                    &mut claimed_by_tree,
                )?;
            }
        }
        for placement in player_room_fixture_placements(&cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in player_room_pc_monitor_placements(&cells, &geometry) {
            append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in player_room_pc_keyboard_placements(&cells, &geometry) {
            append_shallow_top_group(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
                0x02,
                3.0,
            )?;
        }
        for placement in player_bed_placements(&cells, &geometry) {
            append_player_bed_card(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            )?;
        }
        for placement in
            mr_pokemon_work_counter_placements(frame.map_id.as_ref(), &cells, &geometry)
        {
            if let Err(error) = append_shallow_top_group(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
                0x26,
                4.0,
            ) && !matches!(error, TerrainMeshError::MissingGroundSample { .. })
            {
                return Err(error);
            }
        }
        for placement in house_display_table_placements(&cells, &geometry) {
            if let Err(error) = append_shallow_top_group(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
                crate::house::HOUSE_FLOOR_TILE,
                5.0,
            ) && !matches!(error, TerrainMeshError::MissingGroundSample { .. })
            {
                return Err(error);
            }
        }
        if cells
            .first()
            .is_some_and(|tile| tile.source.tileset_id.as_ref() == "players_room")
        {
            append_player_room_wall(&mut mesh, &cells, &shapes, &geometry, &mut claimed_by_tree)?;
        }
        for placement in pokecenter_pc_placements(frame.map_id.as_ref(), &cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        for placement in
            pokecenter_healing_console_placements(frame.map_id.as_ref(), &cells, &geometry)
        {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        for placement in
            pokecenter_link_floor_seat_placements(frame.map_id.as_ref(), &cells, &geometry)
        {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
        for placement in mart_display_rack_placements(frame.map_id.as_ref(), &cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                // The expanded source halo can include a complete rack while
                // excluding the separate authored floor texel. Keep only
                // that rack as faithful flat art; an optional object must not
                // retire the entire 2.5D frame and produce movement flicker.
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
    }
    for placement in diagonal_cave_corner_placements(&cells, &geometry) {
        if let Err(error) = append_diagonal_cave_corner(
            &mut mesh,
            &cells,
            &shapes,
            &geometry,
            placement,
            &mut claimed_by_tree,
        ) {
            if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                return Err(error);
            }
        }
    }
    if let Some(images) = images {
        for placement in cave_small_rock_placements(&cells, &geometry) {
            if let Err(error) = append_grouped_tree(
                &mut mesh,
                images,
                &cells,
                &shapes,
                &geometry,
                placement,
                &mut claimed_by_tree,
            ) {
                if !matches!(error, TerrainMeshError::MissingGroundSample { .. }) {
                    return Err(error);
                }
            }
        }
    }
    if images.is_some() {
        new_bark::append_signs_with_samples(
            &mut mesh,
            frame.map_id.as_ref(),
            &cells,
            &shapes,
            &geometry,
            &mut claimed_by_tree,
            images,
        );
    }
    if images.is_some() && authored_enabled {
        let mut occupied: Vec<_> = (0..cell_count)
            .map(|index| {
                claimed_by_building[index]
                    || claimed_by_tree[index]
                    || claimed_by_casino_stool[index]
                    || claimed_by_house_furniture[index]
            })
            .collect();
        let joined_shapes = modeled_dungeons::joined_surface_shapes(
            &dungeon_placements, &original_cells, &source_shapes, &geometry);
        mountain_portals::append(&mut mesh, &original_cells, &joined_shapes, &geometry, &mut occupied, frame.grid_origin.to_array());
        modeled_dungeons::append_terrain(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &joined_shapes,
            &geometry,
            &mut occupied,
            frame.grid_origin.to_array(),
        );
        for (claimed, occupied) in claimed_by_tree.iter_mut().zip(occupied) {
            *claimed |= occupied;
        }
    }
    if authored_enabled && images.is_some() {
        elevated_piers::append(&mut mesh,&frame.map_id,&original_cells,&geometry,&mut claimed_by_tree,images,pier_field.as_ref(),frame.grid_origin.to_array());
    }
    // Resolve scenery first: sky may retire only unowned border/blank cells,
    // never a roof, prop, custom floor, or an authored part spanning a blank.
    if authored_enabled {
        for i in rooftop_sky::void_cells(frame,&geometry,&original_cells,images) {
            if !claimed_by_building[i] && !claimed_by_tree[i]
                && !claimed_by_casino_stool[i] && !claimed_by_house_furniture[i]
                && mesh.authored_cells[i].is_none() {
                claimed_by_tree[i]=true;
                mesh.authored_cells[i]=Some("outdoor/rooftop-sky");
            }
        }
    }
    crate::loading_trace::phase(&mut trace_phase, "region.build_tiles");
    let bank_runs = bank_column_runs(&shapes, &geometry);
    if frame.map_id.as_ref()=="UndergroundPath" && std::env::var_os("CRYSTAL_INTERIOR_FLOOR_TRACE").is_some() {
        let floors=original_cells.iter().enumerate().filter(|(_,t)|t.source.tile_index==16).collect::<Vec<_>>();
        eprintln!("interior-floor production tile={:?} live={} floor-claimed={} first={:?}",frame.tile_size,
            live_placements.len(),floors.iter().filter(|(i,_)|claimed_by_tree[*i]||claimed_by_building[*i]).count(),
            floors.iter().take(4).map(|(i,t)|(*i,t.column,t.row,shapes[*i],mesh.authored_cells[*i])).collect::<Vec<_>>());
    }

    let native_falls=waterfall_placements(&cells,width,height);
    let native_upper_pool=crate::waterfall::tohjo_upstream_pool(frame.map_id.as_ref(),&cells,width,height,&native_falls);
    let water_scale=geometry.tile_height/SOURCE_TILE_HEIGHT;
    for (i,upper) in native_upper_pool.iter().enumerate() {
        if *upper {mesh.footing_heights[i]=(crate::cave::CAVE_SHELF_HEIGHT+crate::profile::WATER_HEIGHT)*water_scale;}
    }
    for fall in &native_falls {
        if fall.row==0 || !(fall.column..fall.column+fall.width)
            .all(|x|native_upper_pool[(fall.row-1)*width+x]) {continue;}
        for row in fall.row..fall.row+fall.height {
            let along=(row-fall.row) as f32+0.5;
            let y=(crate::profile::WATER_HEIGHT+crate::cave::CAVE_SHELF_HEIGHT
                *(1.-along/fall.height as f32))*water_scale;
            for column in fall.column..fall.column+fall.width {mesh.footing_heights[row*width+column]=y;}
        }
    }

    for row in 0..height {
        for column in 0..width {
            let index = row * width + column;
            if claimed_by_building[index]
                || claimed_by_tree[index]
                || claimed_by_casino_stool[index]
                || claimed_by_house_furniture[index]
            {
                continue;
            }
            if images.is_some()
                && let CellShape::Cutout {
                    ground_tile_index,
                    solid: SolidKind::Flower,
                } = shapes[index]
                && let Some(ground) = authored_ground_for_source(&cells, &shapes, ground_tile_index,&cells[index].source)
                && (new_bark::append_flower(
                    &mut mesh.animated_solid,
                    frame.map_id.as_ref(),
                    cells[index],
                    &geometry,
                    shapes[ground].surface_height(geometry.tile_height),
                ) || modeled_exteriors::append_flower(
                        &mut mesh.animated_solid,
                        frame.map_id.as_ref(),
                        cells[index],
                        &geometry,
                        shapes[ground].surface_height(geometry.tile_height),
                        frame.grid_origin.to_array(),
                        images,
                    ))
            {
                let (west, east, north, south) = geometry.bounds(column, row);
                append_top(
                    &mut mesh.textured,
                    [west, east, north, south],
                    shapes[ground].surface_height(geometry.tile_height),
                    geometry.uv(ground % width, ground / width),
                );
                mesh.authored_cells[index] = Some(if matches!(frame.map_id.as_ref(),
                    "NationalPark" | "NationalParkBugContest")
                    && cells[index].source.tileset_id.as_ref() == "park"
                { "park/flowers" } else { "johto/flowers" });
                continue;
            }
            let grass_start = mesh.solid.positions.len();
            if images.is_some()
                && let CellShape::Cutout {
                    ground_tile_index,
                    solid: SolidKind::Grass,
                } = shapes[index]
                && let Some(ground) = authored_ground_for_source(&cells, &shapes, ground_tile_index,&cells[index].source)
                && (new_bark::append_connected_grass(
                    &mut mesh.solid,
                    frame.map_id.as_ref(),
                    cells[index],
                    &geometry,
                    shapes[ground].surface_height(geometry.tile_height),
                    frame.grid_origin.to_array(),
                    connected_grass_neighbors(&cells, &shapes, &geometry, column, row),
                ) || (authored_enabled
                    && modeled_exteriors::append_connected_grass(
                        &mut mesh.solid,
                        frame.map_id.as_ref(),
                        cells[index],
                        &geometry,
                        shapes[ground].surface_height(geometry.tile_height),
                        frame.grid_origin.to_array(),
                        connected_grass_neighbors(&cells, &shapes, &geometry, column, row),
                    )))
            {
                let (west, east, north, south) = geometry.bounds(column, row);
                append_top(
                    &mut mesh.textured,
                    [west, east, north, south],
                    shapes[ground].surface_height(geometry.tile_height),
                    geometry.uv(ground % width, ground / width),
                );
                new_bark::ledge_finish::clear_grass_at_ledges(&mut mesh.solid, grass_start,
                    shapes[ground].surface_height(geometry.tile_height), jump_ledges,
                    &geometry, frame.grid_origin.to_array());
                outdoor_signs::clear_grass(&mut mesh,grass_start,
                    shapes[ground].surface_height(geometry.tile_height),
                    [west,east,north,south],&geometry,&outdoor_sign_placements);
                mesh.authored_cells[index] = Some("johto/tall_grass");
                continue;
            }
            if native_upper_pool[index] {
                append_top(&mut mesh.textured,geometry.bounds(column,row).into(),
                    (crate::cave::CAVE_SHELF_HEIGHT+crate::profile::WATER_HEIGHT)
                        *geometry.tile_height/SOURCE_TILE_HEIGHT,
                    cave_pool_uv(cells[index],geometry.uv(column,row)));
            } else {
                append_textured_cell(
                    &mut mesh, &geometry, &cells, &shapes, &bank_runs, column, row, index, images,
                )?;
            }
        }
    }

    append_cafe_table_pedestals(&mut mesh.solid, &cells, &geometry, frame.map_id.as_ref());

    for placement in native_falls {
        let levels=(placement.row>0 && (placement.column..placement.column+placement.width)
            .all(|x|native_upper_pool[(placement.row-1)*width+x]))
            .then_some([crate::cave::CAVE_SHELF_HEIGHT+crate::profile::WATER_HEIGHT,crate::profile::WATER_HEIGHT]);
        append_waterfall(&mut mesh.textured, &geometry, &cells, placement, frame.grid_origin.to_array(),levels);
    }

    // Production uses the source-complete 3f+33 park kit. Preserve the old
    // samples-only path for low-level diagnostic comparisons, never as an
    // authored fallback on clipped, customized or incomplete beta drawings.
    if !authored_enabled {
        for placement in fountain_placements(&cells, width, height) {
            append_park_fountain(&mut mesh, &geometry, placement);
        }
    }

    for row in 0..height {
        for column in 0..width {
            if claimed_by_building[row * width + column]
                || claimed_by_tree[row * width + column]
                || claimed_by_casino_stool[row * width + column]
                || claimed_by_house_furniture[row * width + column]
            {
                continue;
            }
            append_exposed_sides(
                &mut mesh, &geometry, &cells, &shapes, &bank_runs, column, row,
            );
        }
    }

    if authored_enabled {
        new_bark::append_grove_water_contacts(&mut mesh, &original_cells, &geometry,
            frame.grid_origin.to_array());
    }

    if authored_enabled && let Some(images) = images {
        buoy_course::append(&mut mesh,&original_cells,&geometry,images,&claimed_by_tree);
    }

    if authored_enabled && let Some(images) = images {
        kanto_paving::finish_at_origin(&mut mesh, &original_cells, &source_shapes, &geometry,
            images,frame.grid_origin.to_array());
        kanto_shore::finish(&mut mesh, &original_cells, &source_shapes, &geometry,
            frame.grid_origin.to_array(), images);
        johto_shore::finish(&mut mesh, &frame.map_id, &original_cells, &source_shapes,
            &geometry, frame.grid_origin.to_array(), images);
    }

    if dragons_den::finish(&mut mesh, &frame.map_id, &original_cells, &geometry,
        frame.grid_origin.to_array()) {
        // Johto's source art belongs to this cavern, not the outdoor meadow.
    } else if authored_enabled {
        new_bark::polish_world_surfaces_with_shapes(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            &source_shapes,
        );
    } else {
        new_bark::polish_surfaces(
            &mut mesh,
            frame.map_id.as_ref(),
            &cells,
            &geometry,
            frame.grid_origin.to_array(),
        );
    }
    if authored_enabled {
        if let Some(images) = images {
            source_forest_floor::finish(&mut mesh, &original_cells, images);
        }
        modeled_interiors::finish_surfaces_with_profiles(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
        );
        modeled_interiors::append_room_perimeter(
            &mut mesh, frame, &original_cells, &geometry, &interior_placements, &dungeon_placements, images, profiles);
    }
    if authored_enabled && let Some(images) = images {
        seafoam_palette::finish(&mut mesh,&frame.map_id,&original_cells,images);
        modeled_interiors::trace_arcade_palette(&frame.map_id,&original_cells,&geometry,&interior_placements,images);
    }
    crate::loading_trace::phase(&mut trace_phase, "region.finish_terrain");
    let grove_finish_claims = (!detached_groves.is_empty() && deferred_groves.is_none())
        .then(|| mesh.authored_cells.clone());
    if jump_ledges.is_empty() {
        let _trace = crate::loading_trace::span("region.finish_bank_approaches", || serde_json::json!({}));
        ledge_approach::shape_approaches(&mut mesh, &original_cells, &geometry, &source_shapes, frame.grid_origin.to_array());
    }
    // Finish normals once, after every outdoor rock deformation. Base contact
    // rounding must not erase the mineral cap fractures.
    {
        let _trace = crate::loading_trace::span("region.finish_rock_geometry", || serde_json::json!({}));
        outdoor_stone::finish_rock_geometry(&mut mesh, &frame.map_id, &original_cells,
            &source_shapes, &geometry, frame.grid_origin.to_array());
    }
    if cave_crowns::enabled(frame.map_id.as_ref()) {
        let _trace = crate::loading_trace::span("region.finish_cave_crowns", || serde_json::json!({}));
        cave_crowns::finish_stairs(&mut mesh,&original_cells,&source_shapes,&geometry);
        cave_crowns::finish_sides(&mut mesh,&source_shapes,&geometry,frame.grid_origin.to_array());
    }
    if authored_enabled && let Some(images)=images {
        let _trace = crate::loading_trace::span("region.finish_cave_palette", || serde_json::json!({}));
        kanto_cave_palette::finish(&mut mesh,&frame.map_id,&original_cells,images,&geometry);
        kanto_cave_palette::finish_union(&mut mesh,&frame.map_id,&original_cells,images,&geometry);
    }
    {
        let _trace = crate::loading_trace::span("region.finish_shore_ecology", || serde_json::json!({}));
        shore_ecology::append(&mut mesh, &frame.map_id, &original_cells,
            &shapes, &geometry, frame.grid_origin.to_array());
    }
    if !jump_ledges.is_empty() || frame.terrace_datum.as_ref().is_some_and(|d|d.levels.iter().flatten().any(|&h|h>0.)) {
        let _trace = crate::loading_trace::span("region.finish_terraces", || serde_json::json!({}));
        let edges=jump_ledges.iter().filter(|edge|!traditional_room_placements.iter()
            .any(|p|p.owns_jump_edge(edge,frame.grid_origin.to_array())))
            .cloned().collect::<Vec<_>>();
        terraces::finish_with_datum(&mut mesh, &frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), &edges,frame.terrace_datum.as_deref());
    } else if authored_enabled {
        terraces::close_flat_rock_contacts(&mut mesh,&frame.map_id,&original_cells,&geometry,
            frame.grid_origin.to_array(),frame.terrace_datum.as_deref());
    }
    if authored_enabled && let Some(images)=images {
        outdoor_stairs::finish(&mut mesh,&original_cells,&geometry,images,frame.grid_origin.to_array());
        outdoor_entry_mat::finish(&mut mesh,&original_cells,&geometry,images);
    }
    if authored_enabled {
        let _trace = crate::loading_trace::span("region.finish_authored_landmarks", || serde_json::json!({}));
        league_terraces::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
        league_liquids::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
        league_banks::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
        lance_terraces::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
        facility_radio::finish_power_plant_equipment(&mut mesh,frame,&original_cells,&geometry,profiles,images);
        hall_machine::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
        pokecom_display::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
        pokecom_console::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
        modeled_interiors::finish_hall_room(&mut mesh,frame,&original_cells,&geometry);
        soul_offerings::finish(&mut mesh,frame,&original_cells,&geometry,profiles);
    }
    if authored_enabled {
        // Only hidden mineral backing is appended. Final source-ground support
        // must be available before a Kanto canyon's cutaway floor is projected.
        outdoor_mountains::append_kanto_reveal_floor(&mut mesh,&frame.map_id,
            &original_cells,&source_shapes,&geometry,frame.grid_origin.to_array());
        enclosed_ponds::finish_with_datum(&mut mesh,&original_cells,&geometry,&frame.jump_ledges,frame.grid_origin.to_array(),frame.terrace_datum.as_deref());
        new_bark::ledge_finish::continue_shore_cap_ground(&mut mesh,&original_cells,&geometry,frame.grid_origin.to_array());
    }
    if let Some(deferred) = deferred_groves {
        deferred.terrace_datum=frame.terrace_datum.clone();
        deferred.recipes = detached_groves.into();
        deferred.geometry = Some(geometry);
        deferred.origin = frame.grid_origin.to_array();
        deferred.jump_ledges = if jump_changes_floor_level(&frame.map_id) {frame.jump_ledges.clone()} else {Default::default()};
        if jump_ledges.is_empty() {
            deferred.cells=original_cells.iter().map(|t|(*t).clone()).collect();
            deferred.shapes=source_shapes.clone();
            deferred.claims=DeferredClaims::Static(mesh.authored_cells.clone());
        }
    } else if let Some(sink) = grove_sink.as_mut() {
        for grove in detached_groves {
            // Grove leaves can cross a bank edge. Apply the same final bank
            // shape and lip normal field using the completed source claims.
            let mut object = TerrainMeshData {
                authored_cells: grove_finish_claims.as_ref().unwrap().clone(),
                ..Default::default()
            };
            if jump_ledges.is_empty() && !frame.terrace_datum.as_ref()
                .is_some_and(|d|d.levels.iter().flatten().any(|&h|h!=0.)) {
                grove.append(&mut object);
                ledge_approach::finish_detached_grove(&mut object, &original_cells,
                    &geometry, &source_shapes, frame.grid_origin.to_array());
            } else {
                grove.on_terrace(jump_ledges, &geometry, frame.grid_origin.to_array(),frame.terrace_datum.as_deref()).append(&mut object);
            }
            sink(object.solid);
        }
    }
    if let Some(started) = scenery_metrics {
        let surfaces = [
            &mesh.textured,
            &mesh.solid,
            &mesh.animated_textured,
            &mesh.animated_solid,
        ];
        let vertices: usize = surfaces.iter().map(|m| m.positions.len()).sum();
        let triangles: usize = surfaces.iter().map(|m| m.indices.len() / 3).sum();
        let repeated: usize = mesh
            .tree_instances
            .iter()
            .map(|group| group.mesh.positions.len() * group.origins.len())
            .sum();
        eprintln!(
            "scenery_metrics map={} origin={:?} cells={}x{} vertices={} triangles={} repeated_vertices={} bytes={} detail={} build_ms={:.2}",
            frame.map_id,
            frame.grid_origin,
            width,
            height,
            vertices,
            triangles,
            repeated,
            vertices * 48 + triangles * 12,
            if crate::new_bark_models::scenery_full_detail() {
                "full"
            } else {
                "adaptive"
            },
            started.elapsed().as_secs_f64() * 1000.0
        );
    }

    #[cfg(not(target_arch="wasm32"))]
    if std::env::var_os("CRYSTAL_CAVE_SURFACE_TRACE").is_some()
        && crate::interior_cutaway::is_cave(frame) {
        static REPORTED:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
        if !REPORTED.swap(true,std::sync::atomic::Ordering::Relaxed) {
            eprintln!("cave_surface map={} grid={:?} textured_vertices={} solid_vertices={}",
                frame.map_id,frame.grid_origin,mesh.textured.positions.len(),mesh.solid.positions.len());
            if let Some(player) = frame.actors.iter().find(|a| a.id == crystal_render_api::VisualActorId::Player) {
                let foot = crate::footing::actor_foot(player);
                if let Some(cell) = crate::footing::tile_at_visual_point(frame, foot + bevy::math::Vec2::Y * 0.01) {
                    let i = cell.y as usize*width+cell.x as usize;
                    eprintln!("cave_surface player_cell={:?} source={:?} shape={:?} support={}",
                        frame.grid_origin+cell.as_ivec2(),original_cells[i].source,source_shapes[i],mesh.footing_heights[i]);
                }
            }
            let mut tilesets=std::collections::BTreeMap::new();
            for tile in &cells { *tilesets.entry(tile.source.tileset_id.as_ref()).or_insert(0usize)+=1; }
            let lower_base=mesh.solid.positions.iter().filter(|p|p[1]<0. && p[1]>-0.021).count();
            let outer_base=mesh.cave_enclosure.positions.iter().zip(&mesh.cave_enclosure.uvs)
                .filter(|(p,uv)|p[1]==0. && matches!((uv[1]/100000.).round(),6. | 7.)).count();
            eprintln!("cave_surface source_size={:?} tilesets={tilesets:?} lower_base_vertices={lower_base} outer_base_vertices={outer_base}",
                frame.source_map_size_core_tiles);
            let mut plain=std::collections::BTreeSet::new();
            for tri in mesh.solid.indices.chunks_exact(3) {
                if !tri.iter().all(|&i|mesh.solid.uvs[i as usize][0]<90000.) {continue;}
                let normal=Vec3::from_array(mesh.solid.normals[tri[0] as usize]);
                // Rail and rung geometry also has untagged UVs. Inspect
                // horizontal caps first so props cannot exhaust the bound.
                if normal.y<0.999 {continue;}
                let p:[Vec3;3]=std::array::from_fn(|i|Vec3::from_array(mesh.solid.positions[tri[i] as usize]));
                let lo=p.iter().map(|p|p.y).fold(f32::INFINITY,f32::min);
                let hi=p.iter().map(|p|p.y).fold(f32::NEG_INFINITY,f32::max);
                if hi<=0. {continue;}
                let center=(p[0]+p[1]+p[2])/3.-normal*geometry.tile_width*0.001;
                let x=((center.x-geometry.origin_x)/geometry.tile_width).floor() as i32;
                let z=((center.z-geometry.origin_z)/geometry.tile_height).floor() as i32;
                if x<0 || z<0 || x>=width as i32 || z>=height as i32 {continue;}
                let tile=original_cells[z as usize*width+x as usize];
                if !plain.insert((x,z,lo.to_bits(),hi.to_bits(),normal.to_array().map(f32::to_bits))) {continue;}
                if plain.len()>32 {break;}
                eprintln!("cave_surface plain cell={},{} block={:02x} glyph={:02x} y={}..{} uv={:?} normal={:?} vertices={:?}",
                    x+frame.grid_origin.x,z+frame.grid_origin.y,tile.source.metatile_id,tile.source.tile_index,
                    lo,hi,mesh.solid.uvs[tri[0] as usize],normal,p);
            }
            let mut seen=std::collections::BTreeSet::new();
            for q in (0..mesh.textured.positions.len()).step_by(4) {
                if q+4>mesh.textured.positions.len() {break;}
                if mesh.textured.normals[q][1]<0.999 {continue;}
                let uv=&mesh.textured.uvs[q..q+4];
                let u=uv.iter().map(|p|p[0]).sum::<f32>()/4.;
                let v=uv.iter().map(|p|p[1]).sum::<f32>()/4.;
                if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {continue;}
                let index=(v*height as f32)as usize*width+(u*width as f32)as usize;
                let tile=cells[index];
                if !seen.insert((tile.source.metatile_id,tile.source.tile_index)) {continue;}
                if seen.len()>32 {break;}
                let point=mesh.textured.positions[q];
                eprintln!("cave_surface atlas block={:02x} glyph={:02x} subtile={},{} y={} vertex={:?}",
                    tile.source.metatile_id,tile.source.tile_index,tile.source.subtile_column,
                    tile.source.subtile_row,point[1],point);
            }
        }
    }

    trace_phase.take();
    mesh.tree_neighborhood = None;
    Ok(mesh)
}
