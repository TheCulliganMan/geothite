#[cfg(feature = "voxel-view")]
#[test]
fn retained_visual_windows_keep_whole_source_blocks_without_resizing_between_phases() {
    let base=visual_world_grid_window(true,IVec2::ZERO).size;
    for y in -19..=19 {for x in -19..=19 {
        let start=IVec2::new(x,y);let w=visual_world_grid_window(true,start);
        assert_eq!(w.size,base,"a source phase must not resize the retained terrain");
        assert_eq!(w.origin.x.rem_euclid(4),0);assert_eq!(w.origin.y.rem_euclid(4),0);
        assert_eq!(w.size.x%4,0);assert_eq!(w.size.y%4,0);
        let old=start-IVec2::splat(i32::from(VISUAL_WORLD_HALO_TILES));
        assert!(w.origin.cmple(old).all());
        assert!((w.origin+w.size.as_ivec2()).cmpge(old+IVec2::new(
            VISUAL_WORLD_TILES_X as i32,VISUAL_WORLD_TILES_Y as i32)).all());
        let classic=visual_world_grid_window(false,start);
        assert_eq!(classic.origin,start-IVec2::splat(i32::from(CLASSIC_SCROLL_HALO_TILES)));
        assert_eq!(classic.size,UVec2::new(CLASSIC_SCROLL_TILES_X as u32,CLASSIC_SCROLL_TILES_Y as u32));
    }}
}

#[cfg(feature = "voxel-view")]
#[test]
#[ignore = "requires an external production content pack"]
fn aligned_source_windows_publish_the_player_on_its_authoritative_gameplay_tile() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap());
    for (map,x,y) in [("Route20",13,9),("Route40",6,11),("Route31",14,9),("NewBarkTown",13,6)] {
        let root=AssetRoot::new(path.parent().unwrap());
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
        let spawn_identifier=runtime.title_new_game_spawn_identifier().unwrap();
        let shell=initialize_bevy_runtime_shell(root,runtime,BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,map_name:map.into(),tile_x:x,tile_y:y,
        },BevyShellConfig {smoke_player_name:Some("CHRIS".into()),..Default::default()}).unwrap();
        let before=shell.shell.snapshot().unwrap().state_checksum;
        let mut app=App::new();app.insert_resource(shell)
            .init_resource::<RenderedViewport>().init_resource::<RenderedTilesetArt>()
            .init_resource::<Assets<Image>>()
            .init_resource::<crystal_render_api::VisualWorldFrame>()
            .init_resource::<crystal_render_api::VisualTileAnimationFrame>()
            .insert_resource(crystal_voxel_view::VoxelViewSettings {enabled:true,..Default::default()})
            .add_systems(Update,(render_playfield,sync_visible_player_sprite,publish_visual_world_frame).chain());
        for _ in 0..3 {app.update();}
        let frame=app.world().resource::<crystal_render_api::VisualWorldFrame>();
        assert!(frame.active,"{map}");assert!(frame.validate().is_ok(),"{map}");
        assert_eq!(frame.grid_origin.x.rem_euclid(4),0);assert_eq!(frame.grid_origin.y.rem_euclid(4),0);
        let player=frame.actors.iter().find(|a|a.id==crystal_render_api::VisualActorId::Player).unwrap();
        let foot=crystal_voxel_view::actor_foot(player)+Vec2::Y*frame.tile_size.y;
        let cell=crystal_voxel_view::tile_at_visual_point(frame,foot+Vec2::Y*0.01).unwrap();
        let source=frame.grid_origin+cell.as_ivec2();
        assert_eq!((source.x.div_euclid(2),source.y.div_euclid(2)),(i32::from(x),i32::from(y)),"{map}");
        assert_eq!(app.world().resource::<crystal_render_api::VisualTileAnimationFrame>().grid_origin,frame.grid_origin);
        if map=="Route20" {
            let mut checked=0;
            for tile in &frame.tiles {
                let s=&tile.source;
                if s.tileset_id.as_ref()!="kanto" || s.metatile_id!=0x0f {continue;}
                let anchor=IVec2::new(tile.column as i32-s.subtile_column as i32,
                    tile.row as i32-s.subtile_row as i32);
                for z in 0..4 {for x in 0..4 {
                    let i=(anchor.y+z)as usize*frame.grid_size.x as usize+(anchor.x+x)as usize;
                    let other=&frame.tiles[i].source;
                    assert_eq!((other.tileset_id.as_ref(),other.metatile_id,
                        other.subtile_column,other.subtile_row),("kanto",0x0f,x as u8,z as u8),
                        "nearest border selection cannot tear a native tree block at {anchor:?}");
                }}
                checked+=1;
            }
            assert!(checked>0,"the production Route20 fixture must cover the affected border");
        }
        assert_eq!(app.world().resource::<BevyRuntimeShell>().shell.snapshot().unwrap().state_checksum,before);
    }
}

#[test]
#[ignore = "requires an external production content pack"]
fn coastal_swimmers_use_compiled_trainer_roles_without_advancing_the_rival_scene() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let data=loaded.pack().data();
    let variables=BTreeMap::from([("SPRITE_OLIVINE_RIVAL".into(),"SPRITE_RIVAL".into())]);
    let original=variables.clone();let icons=BTreeMap::new();let mut count=0;
    for (name,expected) in [("Route40",2),("Route41",5)] {
        let module=&data.maps[name];
        let mut found=0;
        for object in &module.objects {
            if object.sprite!="SPRITE_OLIVINE_RIVAL" {continue;}
            assert_eq!(module.trainer_scripts[&object.script].trainer_class,"SWIMMERM");
            let before=object.clone();
            assert_eq!(visible_map_object_model_source_from_data(data,name,object,&object.sprite,
                "rival",&variables,&icons),"swimmer_guy");
            let replaced=BTreeMap::from([("SPRITE_OLIVINE_RIVAL".into(),"SPRITE_ROCKET".into())]);
            assert_eq!(visible_map_object_model_source_from_data(data,name,object,&object.sprite,
                "rocket",&replaced,&icons),"rocket");
            assert_eq!(*object,before);found+=1;
        }
        assert_eq!(found,expected);count+=found;
    }
    assert_eq!(count,7);assert_eq!(variables,original);
}

#[test]
#[ignore = "requires an external production content pack"]
// Verify the actual classic player entity after map objects have populated
// the same image store. A source sheet alone cannot prove sprite association.
fn classic_player_keeps_human_art_when_item_balls_share_the_map() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .expect("set CRYSTAL_RENDER_TEST_PACK to the external production pack"));
    for (map_name, tile_x, tile_y) in [
        ("SilverCaveRoom2", 15, 18),
        ("SilverCaveRoom2", 14, 18),
        ("NewBarkTown", 13, 6),
    ] {
        let root = AssetRoot::new(pack.parent().unwrap());
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
        let runtime_shell = initialize_bevy_runtime_shell(root, runtime,
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier, map_name: map_name.into(), tile_x, tile_y,
            }, BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()), ..Default::default()
            }).unwrap();
        assert_eq!(runtime_shell.shell.snapshot().unwrap().overworld.mode, MovementMode::Normal);
        let mut app = App::new();
        app.insert_resource(runtime_shell)
            .insert_resource(RenderedViewport::default())
            .insert_resource(RenderedTilesetArt::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Update, (render_playfield, sync_visible_player_sprite).chain());
        for _ in 0..3 { app.update(); }
        let mut players = app.world_mut().query_filtered::<(&Sprite, &PlayerSpriteFrames), With<PlayerMarker>>();
        let (sprite, frames) = players.single(app.world()).unwrap();
        assert_eq!(frames.source_id.as_ref(), "chris", "{map_name}");
        assert_eq!(sprite.image, frames.standing, "{map_name}");
        let actual = app.world().resource::<Assets<Image>>().get(&sprite.image).unwrap().data.clone();
        let root = app.world().resource::<BevyRuntimeShell>().asset_root.clone();
        let art_key = app.world().resource::<RenderedTilesetArt>().sprite_cache.keys()
            .find(|key| key.sprite_id == "chris").unwrap().clone();
        let mut reference_images = Assets::<Image>::default();
        let reference = load_sprite_art(&root, "chris", art_key.palette_id,
            &art_key.time_of_day, &mut reference_images).unwrap();
        assert_eq!(actual, reference_images.get(&reference.down.standing.handle).unwrap().data,
            "{map_name}: player entity received another actor's image");
        assert!(actual.unwrap().chunks_exact(4).filter(|p| p[3] != 0).count() > 60);
    }
}
