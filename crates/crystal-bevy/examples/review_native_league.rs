//! One offscreen native session: initial legal party, earned League, Hall and Continue.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::{BevyShellConfig, BevyShellStart};
use crystal_runtime::CrystalRuntime;
use std::path::PathBuf;
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: review_native_league PACK OUTPUT_DIRECTORY")?,
    )
    .canonicalize()?;
    let directory = PathBuf::from(args.next().context("missing OUTPUT_DIRECTORY")?);
    anyhow::ensure!(args.next().is_none(), "unexpected arguments");
    std::fs::create_dir_all(&directory)?;
    let directory = directory.canonicalize()?;
    anyhow::ensure!(
        std::env::var("CRYSTAL_RENDER_TEST_HEADLESS").as_deref() == Ok("1")
            && std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TARGET").as_deref() == Ok("image")
            && std::env::var("CRYSTAL_RENDER_TEST_WALK_INPUT").as_deref() == Ok("synthetic"),
        "native League review requires explicit headless image capture and synthetic keyboard input"
    );
    let save = directory.join("earned-champion.save.json");
    anyhow::ensure!(
        !save.exists(),
        "use a fresh output directory; preserve previous earned saves"
    );
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack)?,
    )?;
    let entry = runtime.data().maps["IndigoPlateauPokecenter1F"]
        .events
        .warps
        .iter()
        .find(|warp| warp.target_map_constant == "WILLS_ROOM")
        .and_then(crystal_runtime::core::world::session::warp_tile_position_checked)
        .context("compiled Indigo Plateau warp to Will")?;
    use crystal_runtime::core::world::{
        map::{Direction, TilePosition},
        movement::StepOutcome,
    };
    let mut approach = None;
    for (dx, dy, direction) in [
        (0, 1, Direction::Up),
        (-1, 0, Direction::Right),
        (1, 0, Direction::Left),
        (0, -1, Direction::Down),
    ] {
        let tile = TilePosition::new(entry.x + dx, entry.y + dy);
        let Ok(mut probe) = runtime
            .data()
            .overworld_session("IndigoPlateauPokecenter1F", tile, 0)
        else {
            continue;
        };
        if probe.objects.iter().enumerate().any(|(index, object)| {
            probe
                .object_runtime_tile_checked(index, object)
                .is_ok_and(|position| position == tile)
        }) {
            continue;
        }
        let mut step = probe.step_and_check_warp_checked(direction, Default::default())?;
        if matches!(step.outcome, StepOutcome::Turned { .. }) {
            step = probe.step_and_check_warp_checked(direction, Default::default())?;
        }
        if step.warp.is_some() && probe.player.tile == entry {
            approach = Some(tile);
            break;
        }
    }
    let start = approach.context("compiled Will warp must have an ordinary walkable approach")?;
    println!("League fixture: compiled Will entrance {entry:?}, validated approach {start:?}");
    let spawn_identifier = runtime.title_new_game_spawn_identifier()?;
    crystal_bevy::run_bevy_shell(
        root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "IndigoPlateauPokecenter1F".into(),
            tile_x: start.x,
            tile_y: start.y,
        },
        BevyShellConfig {
            quick_save_path: Some(save),
            smoke_player_name: Some("CHRIS".into()),
            voxel_view_enabled: Some(true),
            render_test_league: true,
            render_test_walk: Some("A".into()),
            render_test_screenshot: Some(directory.join("continued-home-3d.png")),
            render_test_second_screenshot: Some(directory.join("continued-home-2d.png")),
            render_test_hour: Some(12),
            ..Default::default()
        },
    )
}
