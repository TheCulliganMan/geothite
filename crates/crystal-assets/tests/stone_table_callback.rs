use crystal_assets::read_verified_compiled_game_pack;
use crystal_core::{state::GameState, world::map::TilePosition};

#[test]
#[ignore = "requires an explicit external CRYSTAL_RENDER_TEST_PACK"]
fn compiled_pack_installs_stone_table_callback_on_map_entry() {
    let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external game pack");
    let data = read_verified_compiled_game_pack(pack)
        .expect("load explicit external compiled pack")
        .data()
        .clone();
    let map_name = "BlackthornGym2F";
    let map = data
        .overworld_map(map_name)
        .expect("assemble Blackthorn Gym second floor");
    let (width, height) = map.checked_tile_bounds().expect("map bounds");
    let mut session = None;
    'candidate: for y in 0..height {
        for x in 0..width {
            if let Ok(candidate) =
                data.overworld_session(map_name, TilePosition::new(x as i16, y as i16), 0)
            {
                session = Some(candidate);
                break 'candidate;
            }
        }
    }
    let mut session = session.expect("reachable Blackthorn Gym tile");
    let mut state = GameState::default();
    data.apply_map_setup_callbacks(&mut state, &mut session, map_name, "MAPSETUP_WARP")
        .expect("execute command-queue callback");
    assert!(state.script_runtime.command_queue.is_empty());
    assert_eq!(state.script_runtime.stone_table_entries.len(), 3);
    assert_eq!(state.script_runtime.stone_table_entries[0].warp, 5);
    assert_eq!(
        state.script_runtime.stone_table_entries[0].object_event,
        "BLACKTHORNGYM2F_BOULDER1"
    );
    assert_eq!(
        state.script_runtime.stone_table_entries[0].script,
        ".Boulder1@BlackthornGym2FSetUpStoneTableCallback"
    );
    for (index, entry) in state.script_runtime.stone_table_entries.iter().enumerate() {
        data.validate_saved_stone_table_entry_command(
            &format!("script_runtime.stone_table_entries[{index}].source_script"),
            entry,
        )
        .expect("installed stone table must pass save-reference validation");
    }
}
