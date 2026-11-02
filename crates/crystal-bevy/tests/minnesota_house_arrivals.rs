//! Exercise representative house doors using production controller inputs.
#![cfg(feature = "bevy-shell")]

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_HOUSE_CONTROLLER_PACK"]
fn house_doors_enter_and_return_through_controller() {
    let path = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_MINNESOTA_HOUSE_CONTROLLER_PACK")
            .expect("external repaired pack"),
    )
    .canonicalize()
    .expect("pack path");
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified pack"),
    )
    .expect("runtime");
    for name in [
        "MnBemidjiNorthHouse",
        "MnDuluthHarborHouse",
        "MnStPaulBoatCoop",
        "MnDetroitLakesCanoeCabin",
    ] {
        let room = runtime
            .data()
            .assemble_map_module_from_compiled_payloads(name)
            .expect("room");
        let constant = room.attributes.map_constant.as_deref().expect("constant");
        let outdoor_constant = &room.events.warps[0].target_map_constant;
        let outside = runtime
            .data()
            .maps
            .values()
            .find(|m| m.attributes.map_constant.as_deref() == Some(outdoor_constant.as_str()))
            .expect("outside");
        let return_warp = outside
            .events
            .warps
            .iter()
            .find(|warp| warp.index as i16 == room.events.warps[0].target_warp_id)
            .expect("outside return");
        let door = outside
            .events
            .warps
            .iter()
            .find(|warp| {
                warp.target_map_constant == constant
                    && warp.target_warp_id == 3
                    && warp.x == return_warp.x
                    && warp.y + 1 == return_warp.y
            })
            .expect("door");
        let (x, y) = (door.x, door.y + 1);
        let mut spawn = runtime
            .data()
            .runtime_spawn_point(runtime.title_new_game_spawn_identifier().expect("spawn id"))
            .expect("spawn")
            .clone();
        spawn.map_name = outside.id.clone();
        spawn.tile_x = i16::try_from(x).expect("x");
        spawn.tile_y = i16::try_from(y).expect("y");
        spawn.metatile_x = i16::try_from(x / 2).expect("metatile x");
        spawn.metatile_y = i16::try_from(y / 2).expect("metatile y");
        spawn.subtile_x = i16::try_from(x % 2).expect("subtile x");
        spawn.subtile_y = i16::try_from(y % 2).expect("subtile y");
        let (state, _) = runtime
            .data()
            .start_overworld_session_from_spawn(
                &spawn,
                &runtime
                    .data()
                    .audio_ids()
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            )
            .expect("door fixture");
        let save = std::env::temp_dir().join(format!(
            "geothite-house-{}-{name}.crystalsave",
            std::process::id()
        ));
        runtime.save_game(&save, state).expect("save fixture");
        let mut game =
            VisibleShellController::load_save(root.clone(), runtime.clone(), save.clone(), None)
                .expect("load fixture");
        std::fs::remove_file(save).expect("remove fixture");
        game.set_runtime_journal_enabled(false);
        for _ in 0..8 {
            game.press(GameButton::Up).expect("enter");
            game.wait_frames(16).expect("settle");
            if game.snapshot().expect("entry snapshot").overworld.map_name == name {
                break;
            }
        }
        let arrived = game.snapshot().expect("arrival").overworld;
        assert_eq!(arrived.map_name, name);
        assert_eq!(
            (arrived.tile.x, arrived.tile.y),
            (room.events.warps[2].x as i16, room.events.warps[2].y as i16)
        );
        for _ in 0..8 {
            game.press(GameButton::Down).expect("leave");
            game.wait_frames(16).expect("settle");
            if game.snapshot().expect("exit snapshot").overworld.map_name == outside.id {
                break;
            }
        }
        let returned = game.snapshot().expect("return").overworld;
        assert_eq!(returned.map_name, outside.id, "{name}");
        assert_eq!((returned.tile.x, returned.tile.y), (x as i16, y as i16));
    }
}
