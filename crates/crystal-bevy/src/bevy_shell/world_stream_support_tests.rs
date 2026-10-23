// Compare the actual source-composed windows involved in a late worker handoff.
#[test]
fn connected_world_windows_publish_identical_walking_support() {
    let root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root"),
    );
    let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            root.repository_root
                .join("content-packs/text-tui.crystalpack")
        });
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack)
        .expect("verified external pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)
        .expect("runtime from external pack");
    let mut controller = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root,
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime
                    .title_new_game_spawn_identifier()
                    .expect("trainer spawn"),
                map_name: "Route28".into(),
                tile_x: 31,
                tile_y: 16,
            },
            BevyShellConfig {
                render_test_hour: Some(12),
                ..default()
            },
        )
        .expect("production controller"),
    };
    let snapshot = controller.snapshot().expect("source catalog");
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut windows = Vec::new();
    for (name, x, y) in [("Route28", -13, 16), ("SilverCaveOutside", 39, 34)] {
        let map = snapshot
            .maps
            .iter()
            .find(|map| map.map_name == name)
            .expect("source map");
        let frame = source_frame(
            &snapshot,
            &controller.shell,
            map,
            TilePosition::new(x, y),
            "day",
            &mut art,
            &mut images,
        )
        .expect("complete source window");
        assert_eq!(
            frame.grid_origin,
            if name == "Route28" {
                IVec2::new(-84, -32)
            } else {
                IVec2::new(20, 4)
            }
        );
        let mesh = crystal_voxel_view::audit_terrain_mesh_with_images(&frame, &images)
            .expect("production authored terrain");
        windows.push((frame, mesh));
    }
    let ((before, old_mesh), (after, new_mesh)) = (&windows[0], &windows[1]);
    let old_heights = &old_mesh.footing_heights;
    let new_heights = &new_mesh.footing_heights;
    let shift = before
        .connected_source_shift(after)
        .expect("reciprocal source placement");
    let mut aligned = Vec::with_capacity(new_heights.len());
    for row in 0..after.grid_size.y {
        for column in 0..after.grid_size.x {
            let p = after.grid_origin + IVec2::new(column as i32, row as i32)
                - shift
                - before.grid_origin;
            aligned.push(
                if p.cmpge(IVec2::ZERO).all() && p.cmplt(before.grid_size.as_ivec2()).all() {
                    old_heights[(p.y as u32 * before.grid_size.x + p.x as u32) as usize]
                } else {
                    f32::NAN
                },
            );
        }
    }
    let mut differences = Vec::new();
    for y in 66..70 {
        for x in 42..54 {
            let p = IVec2::new(x, y);
            let a = p - shift - before.grid_origin;
            let b = p - after.grid_origin;
            let old = old_heights[(a.y as u32 * before.grid_size.x + a.x as u32) as usize];
            let new = new_heights[(b.y as u32 * after.grid_size.x + b.x as u32) as usize];
            if (old - new).abs() > 0.001 {
                differences.push((p, old, new));
            }
            for fraction in [0.25, 0.5, 0.75] {
                let local = p.as_vec2() + Vec2::new(fraction, 0.5)
                    - after.grid_origin.as_vec2()
                    - after.grid_size.as_vec2() * 0.5;
                let point = after.center + local * after.tile_size * Vec2::new(1., -1.);
                let old = crystal_voxel_view::resolved_footing_height(after, point, &aligned)
                    .expect("retained walking support");
                let new = crystal_voxel_view::resolved_footing_height(after, point, new_heights)
                    .expect("replacement walking support");
                if (old - new).abs() > 0.001 {
                    differences.push((p, old, new));
                }
            }
        }
    }
    assert!(
        differences.is_empty(),
        "identical native walking cells changed height: {differences:?}"
    );
    // Check real rendered triangles as well as agreement between support grids.
    // Equal support that floats above both meshes would still be a regression.
    for (frame, mesh) in &windows {
        let source = Vec2::new(45.5, 66.5)
            - if frame.map_id.as_ref() == "Route28" {
                shift.as_vec2()
            } else {
                Vec2::ZERO
            };
        let local = source - frame.grid_origin.as_vec2() - frame.grid_size.as_vec2() * 0.5;
        let foot = frame.center + local * frame.tile_size * Vec2::new(1., -1.);
        let point = local * frame.tile_size;
        let source_cell = source.floor().as_ivec2();
        let local_cell = source_cell - frame.grid_origin;
        let mut surface = None::<f32>;
        let mut contacts = Vec::new();
        for triangle in mesh.solid.indices.chunks_exact(3) {
            let mineral = !triangle
                .iter()
                .any(|&i| mesh.solid.uvs[i as usize][0] >= -9000.);
            let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
                .map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
            let (ab, ac, ap) = (
                Vec2::new(b.x - a.x, b.z - a.z),
                Vec2::new(c.x - a.x, c.z - a.z),
                point - Vec2::new(a.x, a.z),
            );
            let area = ab.perp_dot(ac);
            if area.abs() < 0.00001 {
                continue;
            }
            let (wb, wc) = (ap.perp_dot(ac) / area, ab.perp_dot(ap) / area);
            if wb >= -0.0001 && wc >= -0.0001 && wb + wc <= 1.0001 {
                let height = a.y + (b.y - a.y) * wb + (c.y - a.y) * wc;
                contacts.push((height, mesh.solid.uvs[triangle[0] as usize]));
                if mineral {
                    surface = Some(surface.map_or(height, |old| old.max(height)));
                }
            }
        }
        let surface = surface.unwrap_or_else(|| panic!(
            "{} walking source {source:?}, local {point:?}, grid {:?}/{:?}, tile {:?}, walking={} has no mineral triangle; contacts={contacts:?}",
            frame.map_id, frame.grid_origin, frame.grid_size,
            frame.tiles[(local_cell.y as u32*frame.grid_size.x+local_cell.x as u32) as usize].source,
            frame.terrace_datum.as_ref().is_some_and(|datum| datum.walkable_mineral.contains(&source_cell))
        ));
        let support =
            crystal_voxel_view::resolved_footing_height(frame, foot, &mesh.footing_heights)
                .expect("native walking support");
        assert!(
            (surface - support).abs() < 0.01,
            "{} support {support} must meet rendered surface {surface}",
            frame.map_id
        );
    }
}
