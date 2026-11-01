//! Explicit location fixture followed by normal controller inputs.
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::authored_route_fixture::indoor_walk;
use super::r10_gallery::{continue_save, map, press, tile};

#[test]
#[ignore = "requires the unpublished Driftless passage review pack"]
fn driftless_passage_controller_crosses_both_ways_and_resumes() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_DRIFTLESS_REVIEW_PACK").expect("review pack"))
            .canonicalize()
            .expect("existing review pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified review pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("home"),
            map_name: "MnR01C00R00".into(),
            tile_x: 18,
            tile_y: 18,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("local position fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    c.set_runtime_journal_enabled(false);
    assert_eq!(tile(&mut c), (18, 18));
    for _ in 0..6 {
        if map(&mut c) == "MnR01Cave1F" {
            break;
        }
        press(&mut c, GameButton::Up);
    }
    assert_eq!(map(&mut c), "MnR01Cave1F");
    assert_eq!(tile(&mut c), (9, 11));
    indoor_walk(&mut c, &runtime, (9, 10));
    indoor_walk(&mut c, &runtime, (9, 8));
    indoor_walk(&mut c, &runtime, (14, 8));
    indoor_walk(&mut c, &runtime, (31, 12));
    indoor_walk(&mut c, &runtime, (31, 13));
    assert_eq!(map(&mut c), "MnR01C00R00");
    assert_eq!(tile(&mut c), (40, 20));
    c.wait_frames(60).expect("passive return");
    assert_eq!(map(&mut c), "MnR01C00R00");
    c = continue_save(root, runtime.clone(), &mut c);
    assert_eq!(tile(&mut c), (40, 20));
    for _ in 0..6 {
        if map(&mut c) == "MnR01Cave1F" {
            break;
        }
        press(&mut c, GameButton::Up);
    }
    assert_eq!(map(&mut c), "MnR01Cave1F");
    assert_eq!(tile(&mut c), (31, 13));
    indoor_walk(&mut c, &runtime, (31, 12));
    indoor_walk(&mut c, &runtime, (14, 8));
    indoor_walk(&mut c, &runtime, (9, 8));
    indoor_walk(&mut c, &runtime, (9, 10));
    indoor_walk(&mut c, &runtime, (9, 11));
    assert_eq!(map(&mut c), "MnR01C00R00");
    assert_eq!(tile(&mut c), (18, 18));
}

#[test]
#[ignore = "requires the unpublished Driftless passage review pack"]
fn voyageurs_cleft_controller_exits_and_reenters_aligned_mouths() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_DRIFTLESS_REVIEW_PACK").expect("review pack"))
            .canonicalize()
            .expect("existing review pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified review pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("home"),
            map_name: "MnR20Cave1F".into(),
            tile_x: 23,
            tile_y: 18,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("local position fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    c.set_runtime_journal_enabled(false);
    indoor_walk(&mut c, &runtime, (23, 19));
    assert_eq!(map(&mut c), "MnR20C01R00");
    assert_eq!(tile(&mut c), (12, 32));
    for _ in 0..6 {
        if map(&mut c) == "MnR20Cave1F" {
            break;
        }
        press(&mut c, GameButton::Up);
    }
    assert_eq!(map(&mut c), "MnR20Cave1F");
    assert_eq!(tile(&mut c), (23, 19));
    indoor_walk(&mut c, &runtime, (23, 18));
    indoor_walk(&mut c, &runtime, (5, 20));
    indoor_walk(&mut c, &runtime, (5, 21));
    c.wait_frames(30).expect("settle the western cave exit");
    assert_eq!(map(&mut c), "MnR20C00R00");
    assert_eq!(tile(&mut c), (60, 34));
    for _ in 0..6 {
        if map(&mut c) == "MnR20Cave1F" {
            break;
        }
        press(&mut c, GameButton::Up);
    }
    assert_eq!(map(&mut c), "MnR20Cave1F");
    assert_eq!(tile(&mut c), (5, 21));
}
