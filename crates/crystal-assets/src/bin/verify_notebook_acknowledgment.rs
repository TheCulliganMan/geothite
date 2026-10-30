//! Verify notebook progress in supplied unmodified native saves.
use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::read_loaded_verified_compiled_game_pack;
use crystal_core::save::read_save_game_bytes_for_modpack;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let pack = read_loaded_verified_compiled_game_pack(
        args.next()
            .context("PACK BEFORE PAGE1 PAGE2 COMPLETE REPEAT CONTINUED")?,
    )?;
    let identity = pack.save_modpack_identity()?;
    let content_identity = pack.pack_identity()?;
    let paths = args.map(PathBuf::from).collect::<Vec<_>>();
    ensure!(paths.len() == 6, "six actual native phase saves required");
    let saves = paths
        .iter()
        .map(|path| {
            read_save_game_bytes_for_modpack(
                &std::fs::read(path)?,
                path.display().to_string(),
                &identity,
                &content_identity.content_hash,
            )
            .map_err(anyhow::Error::from)
        })
        .collect::<Result<Vec<_>>>()?;
    let before = saves[0].state();
    let baseline = before
        .flags
        .active_event_flags()
        .cloned()
        .collect::<BTreeSet<_>>();
    const NOTE: &str = "EVENT_MN_PRAIRIE_NOTEBOOK";
    ensure!(!baseline.contains(NOTE), "initial notebook must be unread");
    for (index, save) in saves.iter().enumerate() {
        let state = save.state();
        ensure!(
            state.flags.is_event_flag_set(NOTE)? == (index >= 3),
            "notebook acknowledgment phase {index}"
        );
        ensure!(
            state.badges == before.badges
                && state.money == before.money
                && state.bag == before.bag
                && state.storage == before.storage,
            "notebook must grant no badge, money, item or Pokemon in phase {index}"
        );
        let mut flags = state
            .flags
            .active_event_flags()
            .cloned()
            .collect::<BTreeSet<_>>();
        flags.remove(NOTE);
        ensure!(
            flags == baseline,
            "only the notebook event may be added in phase {index}"
        );
    }
    println!(
        "{}",
        serde_json::json!({"status":"PASS", "native_saves_verified":6,
        "supplied_precompletion_saves_unread":true, "complete_repeat_continue_flag_persist":true,
        "only_notebook_event_added":true, "party_bag_money_badges_preserved":true})
    );
    Ok(())
}
