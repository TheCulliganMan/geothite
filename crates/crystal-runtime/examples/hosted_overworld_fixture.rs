//! Verification-only multiplayer saves. Production never imports these fixtures.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::models::pokemon::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: hosted_overworld_fixture PACK OUTPUT")?,
    );
    let output = PathBuf::from(args.next().context("missing output directory")?);
    std::fs::create_dir_all(&output)?;
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack)?,
    )?;
    let spawn = runtime.title_new_game_spawn_identifier()?;
    let mut players = Vec::new();
    for (index, (name, x, y, species)) in [
        ("ALICE", 13, 6, "CHARMANDER"),
        ("BOB", 13, 7, "SQUIRTLE"),
        ("CAROL", 14, 7, "BULBASAUR"),
    ]
    .into_iter()
    .enumerate()
    {
        let id = 41001 + index as u64;
        let mut game = RuntimeGameShell::new_game_at_runtime_tile(
            root.clone(),
            runtime.clone(),
            spawn.clone(),
            "NewBarkTown",
            x,
            y,
        )?;
        game.set_trainer_identity(name, id as u16)?;
        game.set_player_gender((index % 2) as u8)?;
        game.add_party_pokemon(
            species,
            5,
            None,
            None,
            name,
            id as u16,
            Dv::from_non_hp(10, 10, 10, 10),
        )?;
        game.set_script_flag_for_smoke("EVENT_GOT_A_POKEMON_FROM_ELM")?;
        let filename = format!("{id}.crystalsave");
        game.save(output.join(&filename))?;
        players
            .push(serde_json::json!({"id":id.to_string(),"name":name,"save":filename,"x":x,"y":y}));
    }
    std::fs::write(
        output.join("fixtures.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "modpack_id": runtime.modpack().id(), "map": "NewBarkTown", "players":players,
        }))?,
    )?;
    Ok(())
}
