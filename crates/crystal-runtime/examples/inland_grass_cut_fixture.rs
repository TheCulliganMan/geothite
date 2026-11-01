//! Create explicit local grass-Cut QA fixtures or inspect actual played saves.
//! No gameplay inputs run here. Party, move, badge, position and supplies are
//! pre-input setup, not earned campaign progress; inspection never writes.
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::models::LearnedMove;
use crystal_core::systems::script_flags::{
    BADGE_ENGINE_FLAGS, ScriptFlagCommand, apply_script_flag_mutation,
};
use crystal_core::systems::time::GameDate;
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(
        args.len() >= 4,
        "fixture PACK SAVE MAP X Y FACING CUT HIVE [HOUR], or inspect PACK SAVE"
    );
    let pack = PathBuf::from(&args[2]);
    let root = AssetRoot::new(pack.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack).expect("verified candidate"),
    )
    .expect("native candidate runtime");
    match args[1].as_str() {
        "fixture" => {
            anyhow::ensure!(
                matches!(args.len(), 10 | 11),
                "fixture PACK SAVE MAP X Y FACING CUT HIVE [HOUR]"
            );
            anyhow::ensure!(
                matches!(args[8].as_str(), "0" | "1") && matches!(args[9].as_str(), "0" | "1"),
                "CUT and HIVE must be 0 or 1"
            );
            let path = PathBuf::from(&args[3]);
            assert!(!path.exists(), "preserve prior fixture");
            let mut spawn = runtime
                .data()
                .runtime_spawn_point(runtime.title_new_game_spawn_identifier().expect("HOME"))
                .expect("native source spawn")
                .clone();
            spawn.map_name = args[4].clone();
            spawn.tile_x = args[5].parse().expect("initial x");
            spawn.tile_y = args[6].parse().expect("initial y");
            anyhow::ensure!(
                spawn.tile_x >= 0 && spawn.tile_y >= 0,
                "nonnegative initial tile required"
            );
            spawn.metatile_x = spawn.tile_x / 2;
            spawn.metatile_y = spawn.tile_y / 2;
            spawn.subtile_x = spawn.tile_x % 2;
            spawn.subtile_y = spawn.tile_y % 2;
            let (mut state, mut overworld) = runtime
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
                .expect("explicit initial native grass approach");
            let mut carrier = runtime
                .data()
                .create_pokemon("VOYAGEAR", 60, Default::default())
                .expect("active Minnesota Cut carrier");
            assert!(
                carrier
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|name| name == "CUT")
            );
            assert!(!carrier.moves.iter().any(|known| known.name == "CUT"));
            if args[8] == "1" {
                carrier.moves = vec![LearnedMove {
                    name: "CUT".into(),
                    current_pp: runtime.data().moves["CUT"].pp,
                    pp_ups: 0,
                }];
            }
            carrier.item = Some("SMOKE_BALL".into());
            assert!(state.storage.party.add_pokemon(carrier));
            state.sync_party_from_storage();
            for (index, flag) in BADGE_ENGINE_FLAGS.iter().enumerate() {
                let granted = index == 1 && args[9] == "1";
                apply_script_flag_mutation(
                    &mut state,
                    ScriptFlagCommand {
                        command: if granted { "setflag" } else { "clearflag" }.into(),
                        flag_id: (*flag).into(),
                        source_script: "ThawGrassCutInitialFixture".into(),
                        command_index: index,
                    },
                )
                .expect("consistent initial badge fixture");
            }
            overworld.set_player_facing(match args[7].as_str() {
                "left" => Direction::Left,
                "right" => Direction::Right,
                "up" => Direction::Up,
                "down" => Direction::Down,
                _ => anyhow::bail!("facing must be left, right, up or down"),
            });
            state.overworld =
                crystal_core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
            state.player_name = "CHRIS".into();
            state.repel_steps_remaining = 250;
            state.active_repel_item = Some("MAX_REPEL".into());
            state.money = 10000;
            if let Some(hour) = args.get(10) {
                let hour: u8 = hour.parse()?;
                anyhow::ensure!(hour < 24, "initial hour must be in 0..24");
                state
                    .time
                    .update_from_datetime(GameDate::new(2000, 1, 1), hour, 0, 0);
                overworld.set_time(hour, state.time.time_of_day);
            }
            runtime
                .save_game(&path, state)
                .expect("native identity-bound pre-input save");
            println!("{}", path.display());
        }
        "inspect" => {
            anyhow::ensure!(args.len() == 4, "inspect PACK SAVE");
            let state = runtime
                .load_save(&args[3])
                .expect("read actual native save");
            let overrides = state
                .map_block_overrides
                .iter()
                .flat_map(|(map, blocks)| {
                    blocks.iter().map(
                        move |(&(x, y), &block)| json!({"map":map,"metatile":[x,y],"block":block}),
                    )
                })
                .collect::<Vec<_>>();
            let party = state.storage.party.pokemon.iter().flatten().map(|mon|
                json!({"species":mon.species.id,"moves":mon.moves.iter().map(|m| json!({"name":m.name,"current_pp":m.current_pp,"pp_ups":m.pp_ups})).collect::<Vec<_>>(),"hp":mon.hp}))
                .collect::<Vec<_>>();
            let pose = match &state.overworld {
                crystal_core::state::OverworldMemory::Active {
                    map_name,
                    tile,
                    facing,
                    mode,
                } => json!({"map":map_name,"position":{"x":tile.x,"y":tile.y},
                        "facing":format!("{facing:?}"),"mode":format!("{mode:?}")}),
                _ => json!(null),
            };
            println!(
                "{}",
                json!({"overworld":pose,"overrides":overrides,"party":party,
                "badges":{"johto":state.badges.johto,"kanto":state.badges.kanto},"money":state.money,
                "time":state.time})
            );
        }
        _ => anyhow::bail!("mode must be fixture or inspect"),
    }
    Ok(())
}
