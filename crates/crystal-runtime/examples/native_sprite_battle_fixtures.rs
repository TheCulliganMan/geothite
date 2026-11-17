//! Verification-only battle saves for reviewed native creature artwork.
//! Production never creates or imports these fixtures.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::battle::start::LinkBattleStart;
use crystal_core::models::pokemon::Dv;
use crystal_core::random::LinkBattleRandomState;
use crystal_core::state::LinkSerialConnectionStatus;
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(args.next().context(
        "usage: native_sprite_battle_fixtures PACK OUTPUT_DIRECTORY SPECIES,SPECIES,...",
    )?);
    let output = PathBuf::from(args.next().context("missing output directory")?);
    let names = args.next().context("missing species list")?;
    ensure!(args.next().is_none(), "unexpected fixture argument");
    let names: Vec<_> = names.split(',').collect();
    ensure!(names.len() >= 2, "select at least two species");
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack)?,
    )?;
    let spawn = runtime.title_new_game_spawn_identifier()?;
    std::fs::create_dir_all(&output)?;
    for (appearance, dvs) in [
        ("normal", Dv::from_non_hp(8, 9, 10, 11)),
        ("shiny", Dv::from_non_hp(10, 10, 10, 10)),
    ] {
        for (index, &player) in names.iter().enumerate() {
            let opponent = names[(index + 1) % names.len()];
            let mut shell = RuntimeGameShell::new_game(root.clone(), runtime.clone(), spawn)?;
            shell.add_party_pokemon(player, 30, None, None, "SPRITE_QA", 1, dvs)?;
            let enemy = runtime.data().create_pokemon(opponent, 30, dvs)?;
            let random = LinkBattleRandomState {
                seeds: [31, 127, 223, 54, 118, 77, 159, 229, 86, 213],
                count: 0,
            };
            let state = shell.session_mut().state_mut();
            state.link_session.link_mode = 3;
            state.link_session.serial_connection_status =
                LinkSerialConnectionStatus::UsingInternalClock;
            state.link_session.battle_random = Some(random.clone());
            shell.start_link_battle(&LinkBattleStart {
                opponent_player_id: 2,
                opponent_name: "SPRITE_QA".into(),
                enemy_party: vec![enemy],
                random_state: random,
            })?;
            let path = output.join(format!("{player}-{opponent}-{appearance}.crystalsave"));
            shell.save(&path)?;
            // Reopen through production validation rather than trusting the writer.
            RuntimeGameShell::resume_from_save(root.clone(), runtime.clone(), &path)?;
        }
    }
    println!(
        "validated {} battle saves; every selected species appears on both battle sides in normal and shiny palettes",
        names.len() * 2
    );
    Ok(())
}
