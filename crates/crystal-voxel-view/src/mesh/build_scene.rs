fn build_terrain_mesh_internal(
    frame: &VisualWorldFrame,
    images: Option<&TerrainImageSamples>,
    instance_hulls: bool,
    profiles: Option<&crate::live_profiles::Document>,
) -> Result<TerrainMeshData, TerrainMeshError> {
    // The runtime's bundled/live profile document enables the full authored
    // kit. Low-level samples-only calls retain their legacy mesher contract
    // for diagnostic comparisons and regression fixtures.
    let authored_enabled = profiles.is_some();
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
    resolve_rock_platform_tiers(&original_cells, &mut source_shapes, width);
    resolve_authored_mountain_tiers(&mut source_shapes, width, height);
    taper_johto_ledge_ends(&original_cells, &mut source_shapes, width);
    let mut interior_placements = if images.is_some() && authored_enabled {
        modeled_interiors::resolve(&frame.map_id, &original_cells, &geometry, profiles)
    } else {
        Vec::new()
    };
    let mut authored_reserved = vec![false; cell_count];
    // Complete institutional apparatus supersedes only the exact old partial
    // signature models; live custom profiles retain first refusal.
    let facility_radio_placements = if images.is_some() && authored_enabled {
        facility_radio::resolve(
            &frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved,
        )
    } else { Vec::new() };
    for placement in &facility_radio_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    // Complete joined store courses supersede older individual window models.
    // Their resolver still gives all customized live objects first refusal.
    let department_store_placements = if images.is_some() && authored_enabled {
        department_store::resolve(
            &frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved,
        )
    } else { Vec::new() };
    for placement in &department_store_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    interior_placements.retain(|placement| {
        if placement.indices(width).any(|index| authored_reserved[index]) { return false; }
        for index in placement.indices(width) { authored_reserved[index] = true; }
        true
    });
    let gate_placements = if images.is_some() && authored_enabled {
        gate_counters::resolve(&frame.map_id, &original_cells, &geometry, frame.grid_origin.to_array(), profiles, &authored_reserved)
    } else {
        Vec::new()
    };
    for placement in &gate_placements {
        for index in placement.indices(width) {
            authored_reserved[index] = true;
        }
    }
    let cable_club_placements = if images.is_some() && authored_enabled {
        cable_club::resolve(&frame.map_id, &original_cells, &geometry, frame.grid_origin.to_array(), profiles, &authored_reserved)
    } else { Vec::new() };
    for placement in &cable_club_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    let train_placements = if images.is_some() && authored_enabled {
        train_station_scenery::resolve(&frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved)
    } else { Vec::new() };
    for placement in &train_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    let outdoor_sign_placements = if images.is_some() && authored_enabled {
        outdoor_signs::resolve(&frame.map_id, &original_cells, &geometry, profiles, &authored_reserved)
    } else { Vec::new() };
    for placement in &outdoor_sign_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    let gym_placements = if images.is_some() && authored_enabled {
        gym_scenery::resolve(&frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved)
    } else { Vec::new() };
    for placement in &gym_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    let traditional_room_placements = if images.is_some() && authored_enabled {
        traditional_room::resolve(&frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved)
    } else { Vec::new() };
    for placement in &traditional_room_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    let ship_room_placements = if images.is_some() && authored_enabled {
        ship_rooms::resolve(&frame.map_id, &original_cells, &geometry, profiles, &authored_reserved)
    } else { Vec::new() };
    for placement in &ship_room_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    let park_placements = if images.is_some() && authored_enabled {
        park_scenery::resolve(&frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved)
    } else { Vec::new() };
    for placement in &park_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
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
            &frame.map_id, &original_cells, &geometry,
            frame.grid_origin.to_array(), profiles, &authored_reserved,
        )
    } else { Vec::new() };
    for placement in &native_ground_placements {
        for index in placement.indices(width) { authored_reserved[index] = true; }
    }
    let mut live_placements = live::resolve(
        &cells,
        width,
        height,
        frame.map_id.as_ref(),
        profiles.filter(|_| images.is_some()),
    );
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
    if images.is_some() && new_bark::supports_map(frame.map_id.as_ref()) {
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
    resolve_rock_platform_tiers(&cells, &mut shapes, width);
    // Blackthorn's closed `[6a 70 6b; 6c 72 6d]` drawing is a free-standing
    // mound, not part of the surrounding mountain datum graph. Claim the
    // complete drawing only; the same individual metatiles remain valid
    // cliff pieces everywhere else.
    for (column, row) in johto_closed_mound_origins(&cells, width, height) {
        for local_row in 0..8 {
            for local_column in 0..12 {
                shapes[(row + local_row) * width + column + local_column] = CellShape::Flat;
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
    for shape in &mut shapes {
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
    taper_johto_ledge_ends(&cells, &mut shapes, width);

    let mut mesh = TerrainMeshData {
        tree_cache: instance_hulls.then(HashMap::new),
        authored_cells: vec![None; cell_count],
        ..Default::default()
    };
    if let Some(tileset) = cells.first().map(|tile| tile.source.tileset_id.as_ref()) {
        if !crate::interior::has_back_wall(tileset) {
            background::append_repeating_background_apron(&mut mesh, &geometry, &cells, &shapes);
        }
    }
    // Cave faces are folds cut from the same continuous map drawing. Keep a
    // faithful copy of that drawing just below the modeled shelves so a fold,
    // diagonal corner, or clipped neighbor can never punch the clear color
    // through the cave floor. This is the connected-volume rule used by the
    // rock/cliff mesher: authored face art changes elevation, not topology.
    if cells
        .iter()
        .all(|tile| matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave"))
    {
        append_top(
            &mut mesh.textured,
            [
                geometry.origin_x,
                geometry.origin_x + grid_width,
                geometry.origin_z,
                geometry.origin_z + grid_height,
            ],
            -0.02,
            (0.0, 1.0, 0.0, 1.0),
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
    let mut claimed_by_building = vec![false; cell_count];
    let mut claimed_by_tree = live_claimed;
    for placement in &facility_radio_placements {
        let appended = facility_radio::append(
            &mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated facility/radio reservation must append");
    }
    for placement in &interior_placements {
        let appended = modeled_interiors::append(
            &mut mesh,
            &original_cells,
            &geometry,
            placement,
            &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated interior reservation must append");
    }
    for placement in &gate_placements {
        let appended = gate_counters::append(
            &mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated gate network reservation must append");
    }
    for placement in &cable_club_placements {
        let appended = cable_club::append(&mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree);
        debug_assert!(appended, "validated Cable Club reservation must append");
    }
    for placement in &train_placements {
        let appended = train_station_scenery::append(
            &mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated station train reservation must append");
    }
    for placement in &outdoor_sign_placements {
        let appended = outdoor_signs::append(&mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree);
        debug_assert!(appended, "validated outdoor sign reservation must append");
    }
    for placement in &gym_placements {
        let appended = gym_scenery::append(&mut mesh, &original_cells, &geometry,
            placement, &mut claimed_by_tree);
        debug_assert!(appended, "validated Gym reservation must append");
    }
    for placement in &traditional_room_placements {
        let appended = traditional_room::append(&mut mesh, &original_cells, &geometry,
            placement, &mut claimed_by_tree);
        debug_assert!(appended, "validated traditional-room reservation must append");
    }
    for placement in &park_placements {
        let appended = park_scenery::append(&mut mesh, &original_cells, &geometry,
            placement, &mut claimed_by_tree);
        debug_assert!(appended, "validated park fixture reservation must append");
    }
    for placement in &department_store_placements {
        let appended = department_store::append(
            &mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated department store reservation must append");
    }
    for placement in &ship_room_placements {
        let appended = ship_rooms::append(
            &mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated ship room reservation must append");
    }
    for placement in &dungeon_placements {
        modeled_dungeons::append(&mut mesh, &geometry, placement, &original_cells);
        for index in placement.indices(width) {
            claimed_by_tree[index] = true;
            mesh.authored_cells[index] = Some(placement.kind_label());
        }
    }
    for placement in &native_ground_placements {
        let appended = native_ground_bindings::append(
            &mut mesh, &original_cells, &geometry, placement, &mut claimed_by_tree,
        );
        debug_assert!(appended, "validated native ground binding must append");
    }
    if images.is_some() && authored_enabled {
        modeled_exteriors::append_props(
            &mut mesh,
            &cells,
            &shapes,
            &geometry,
            &mut claimed_by_tree,
        );
    }
    if let Some(images) = images {
        append_johto_vertical_fences(&mut mesh, images, &cells, &geometry, &mut claimed_by_tree)?;
    }
    if let Some(images) = images {
        for placement in &live_placements {
            if !authored_enabled
                || !modeled_exteriors::append_live(&mut mesh, &original_cells, &geometry, placement)
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
            modeled_exteriors::building_placements(frame.map_id.as_ref(), &cells, &geometry)
        } else {
            new_bark::building_placements(frame.map_id.as_ref(), &cells, &geometry)
        };
        for placement in &placements {
            if new_bark::append_building(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                frame.map_id.as_ref(),
                *placement,
                &mut claimed_by_building,
            ) {
                continue;
            }
            if authored_enabled
                && modeled_exteriors::append_building(
                    &mut mesh,
                    &cells,
                    &shapes,
                    &geometry,
                    &frame.map_id,
                    *placement,
                    &mut claimed_by_building,
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

        // Verified originals take priority. A complete legacy tree still owns
        // its closed source-derived hull if no authored model is available;
        // discarding its back and depth here made fallback trees disappear
        // from an orbiting camera and broke the instancing geometry contract.
        for placement in new_bark::tree_placements(frame.map_id.as_ref(), &cells, &geometry) {
            if new_bark::append_tree(
                &mut mesh,
                &cells,
                &shapes,
                &geometry,
                frame.map_id.as_ref(),
                placement,
                &mut claimed_by_tree,
                frame.grid_origin.to_array(),
            ) {
                continue;
            }
            if authored_enabled
                && modeled_exteriors::append_tree(
                    &mut mesh,
                    &cells,
                    &shapes,
                    &geometry,
                    &frame.map_id,
                    placement,
                    &mut claimed_by_tree,
                    frame.grid_origin.to_array(),
                )
            {
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
        for placement in kanto_round_barrier_placements(&cells, &geometry)
            .into_iter()
            .chain(kanto_round_path_barrier_placements(&cells, &geometry))
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
        for placement in tower_statue_placements(&cells, &geometry) {
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
        new_bark::append_signs(
            &mut mesh,
            frame.map_id.as_ref(),
            &cells,
            &shapes,
            &geometry,
            &mut claimed_by_tree,
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
        modeled_dungeons::append_terrain(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &source_shapes,
            &geometry,
            &mut occupied,
        );
        for (claimed, occupied) in claimed_by_tree.iter_mut().zip(occupied) {
            *claimed |= occupied;
        }
    }
    let bank_runs = bank_column_runs(&shapes, &geometry);

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
                && let Some(ground) = authored_ground_cell(&cells, &shapes, ground_tile_index)
                && (new_bark::append_flower(
                    &mut mesh.animated_solid,
                    frame.map_id.as_ref(),
                    cells[index],
                    &geometry,
                    shapes[ground].surface_height(geometry.tile_height),
                ) || (authored_enabled
                    && modeled_exteriors::append_flower(
                        &mut mesh.animated_solid,
                        frame.map_id.as_ref(),
                        cells[index],
                        &geometry,
                        shapes[ground].surface_height(geometry.tile_height),
                    )))
            {
                let (west, east, north, south) = geometry.bounds(column, row);
                append_top(
                    &mut mesh.textured,
                    [west, east, north, south],
                    shapes[ground].surface_height(geometry.tile_height),
                    geometry.uv(ground % width, ground / width),
                );
                mesh.authored_cells[index] = Some("johto/flowers");
                continue;
            }
            if images.is_some()
                && let CellShape::Cutout {
                    ground_tile_index,
                    solid: SolidKind::Grass,
                } = shapes[index]
                && let Some(ground) = authored_ground_cell(&cells, &shapes, ground_tile_index)
                && (new_bark::append_grass(
                    &mut mesh.solid,
                    frame.map_id.as_ref(),
                    cells[index],
                    &geometry,
                    shapes[ground].surface_height(geometry.tile_height),
                    frame.grid_origin.to_array(),
                ) || (authored_enabled
                    && modeled_exteriors::append_grass(
                        &mut mesh.solid,
                        frame.map_id.as_ref(),
                        cells[index],
                        &geometry,
                        shapes[ground].surface_height(geometry.tile_height),
                        frame.grid_origin.to_array(),
                    )))
            {
                let (west, east, north, south) = geometry.bounds(column, row);
                append_top(
                    &mut mesh.textured,
                    [west, east, north, south],
                    shapes[ground].surface_height(geometry.tile_height),
                    geometry.uv(ground % width, ground / width),
                );
                mesh.authored_cells[index] = Some("johto/tall_grass");
                continue;
            }
            append_textured_cell(
                &mut mesh, &geometry, &cells, &shapes, &bank_runs, column, row, index, images,
            )?;
        }
    }

    append_cafe_table_pedestals(&mut mesh.solid, &cells, &geometry, frame.map_id.as_ref());

    for placement in waterfall_placements(&cells, width, height) {
        append_waterfall(&mut mesh.textured, &geometry, &cells, placement);
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
        new_bark::polish_world_surfaces(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
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
        modeled_interiors::finish_surfaces_with_profiles(
            &mut mesh,
            &frame.map_id,
            &original_cells,
            &geometry,
            frame.grid_origin.to_array(),
            profiles,
        );
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

    Ok(mesh)
}
