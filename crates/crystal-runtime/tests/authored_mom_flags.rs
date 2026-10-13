use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;

#[test]
fn external_pack_mom_pokegear_flags_retain_authoritative_command_indices() {
    let Some(path) = std::env::var_os("CRYSTAL_RENDER_TEST_PACK") else {
        return;
    };
    let path = PathBuf::from(path);
    assert!(path.is_file(), "explicit gameplay pack must exist");
    let assets = AssetRoot::new(path.parent().unwrap());
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &assets,
        read_loaded_verified_compiled_game_pack(&path).unwrap(),
    )
    .unwrap();
    for (index, flag) in [(5, "ENGINE_POKEGEAR"), (6, "ENGINE_PHONE_CARD")] {
        let command = runtime
            .data()
            .script_flag_command("PlayersHouse1F", "MeetMomScript", index)
            .expect("authored Mom flag mutation must remain in the compiled map catalog");
        assert_eq!(command.flag_id, flag);
        assert!(runtime.has_script_flag_command_at("PlayersHouse1F", "MeetMomScript", index));
    }
    let reward = runtime
        .compiled_script_commands("ReceiveItemScript")
        .unwrap();
    assert_eq!(
        reward
            .iter()
            .filter_map(|command| command.get("command").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        ["waitsfx", "farwritetext", "playsound", "waitsfx", "end"],
        "Mom's standard reward must retain its complete authored text and sound sequence"
    );
}
