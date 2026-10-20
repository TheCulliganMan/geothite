#[test]
fn production_water_borders_share_their_dry_bank_datum() {
    use crystal_render_api::{VisualWorldClock, VisualWorldFrame};
    use crystal_voxel_view::{shape_for_source,CellShape};
    for (map,x,y,probe) in [
        ("CherrygroveCity",20,9,IVec2::new(35,36)),
        ("OlivineCity",20,18,IVec2::new(80,32)),
        ("IlexForest",15,27,IVec2::new(18,17)),
    ] {
        let root=AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
        let runtime=workspace_desktop_runtime(&root);
        let spawn_identifier=runtime.title_new_game_spawn_identifier().unwrap();
        let mut shell=initialize_bevy_runtime_shell(root,runtime,
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier,map_name:map.into(),tile_x:x,tile_y:y,
            },BevyShellConfig {
                render_test_hour:Some(12),smoke_player_name:Some("TEST".into()),..Default::default()
            }).unwrap();
        complete_visible_smoke_player_name_if_needed(&mut shell,Some("TEST")).unwrap();
        settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
        let mut app=integrated_shell_test_app(shell);
        app.init_resource::<VisualWorldFrame>().init_resource::<VisualWorldClock>()
            .add_systems(Update,publish_visual_world_frame.after(render_playfield));
        app.update();app.update();
        assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error,None);
        let frame=app.world().resource::<VisualWorldFrame>();
        let datum=frame.terrace_datum.as_ref().expect("complete shore datum");
        let corners=|p:IVec2| {
            let local=p-datum.origin;
            (local.x>=0 && local.y>=0 && local.x<datum.size.x as i32 && local.y<datum.size.y as i32)
                .then(||datum.levels[local.y as usize*datum.size.x as usize+local.x as usize])
        };
        assert!(corners(probe).is_some(),"{map}: rendered border water needs height ownership at {probe:?}");
        let width=frame.grid_size.x as usize;
        let mut checked=0;
        for tile in frame.tiles.iter().filter(|t|shape_for_source(&t.source)==CellShape::Water) {
            let p=frame.grid_origin+IVec2::new(tile.column as i32,tile.row as i32);
            let Some(water)=corners(p) else {continue;};
            let level=datum.open_water_at(p).unwrap_or(water[0]);
            for (d,edge) in [(IVec2::X,[0,1]),(-IVec2::X,[2,3]),(IVec2::Y,[0,3]),(-IVec2::Y,[1,2])] {
                let q=p+d;let local=q-frame.grid_origin;
                if local.x<0 || local.y<0 || local.x>=frame.grid_size.x as i32 || local.y>=frame.grid_size.y as i32 {continue;}
                let bank=&frame.tiles[local.y as usize*width+local.x as usize];
                if !matches!(shape_for_source(&bank.source),CellShape::Flat|CellShape::ShoreBand) {continue;}
                let Some(bank)=corners(q) else {continue;};
                for c in edge {assert!((bank[c]-level).abs()<0.0002,
                    "{map}: bank {q:?} corner {c} must meet flat water {p:?}: {} vs {level}",bank[c]);}
                checked+=1;
            }
        }
        assert!(checked>20,"{map}: audit actual native and repeated-border shore contacts");
    }
}

#[test]
fn production_connected_kanto_bank_stays_flat_across_camera_positions() {
    use crystal_render_api::{VisualWorldClock, VisualWorldFrame};
    let mut previous = None;
    for y in [8, 0] {
        let root = AssetRoot::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
        );
        let runtime = workspace_desktop_runtime(&root);
        let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
        let mut shell = initialize_bevy_runtime_shell(
            root,
            runtime,
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier,
                map_name: "Route24".into(),
                tile_x: 9,
                tile_y: y,
            },
            BevyShellConfig {
                render_test_hour: Some(12),
                smoke_player_name: Some("TEST".into()),
                ..Default::default()
            },
        )
        .unwrap();
        complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
        settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
        let mut app = integrated_shell_test_app(shell);
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldClock>()
            .add_systems(Update, publish_visual_world_frame.after(render_playfield));
        app.update();
        app.update();
        assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
        let frame = app.world().resource::<VisualWorldFrame>();
        assert!(frame.active);
        let datum = frame.terrace_datum.as_ref().expect("production native datum");
        for x in 22..28 {
            let p = IVec2::new(x, 16) - datum.origin;
            let corners = datum.levels[p.y as usize * datum.size.x as usize + p.x as usize];
            assert_eq!(corners, [0.; 4], "false raised bank at ({x},16), camera y={y}");
        }
        assert_eq!(datum.open_water_at(IVec2::new(30, 16)), Some(0.));
        assert!(datum.valid_open_water() && datum.valid_water_barriers());
        if let Some(previous) = &previous {
            assert_eq!(datum, previous, "walking north must not rebuild a different height field");
        }
        previous = Some(datum.clone());
    }
}
