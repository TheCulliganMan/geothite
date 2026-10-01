//! Disposable native battle preview using the real production controller.
//! Starts at battle commands after the controller completes the normal intro.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::{BevyShellConfig, BevyShellStart};
use crystal_runtime::CrystalRuntime;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1).peekable();
    let pack = args
        .next_if(|arg| !arg.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("content-packs/core-modular.browser.crystalpack"));
    let mut screenshot = None;
    let mut second = None;
    let mut record = None;
    let mut measure = None;
    let mut record_on_move = false;
    let mut seconds = 20;
    let mut live = false;
    let mut enabled = true;
    let mut shadow_ball = false;
    let mut psychic = false;
    let mut hyper_beam = false;
    let mut surf = false;
    let mut size_comparison = false;
    let mut pidgeotto = false;
    let mut window_size = None;
    let mut reduced_flashes = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--live" => live = true,
            "--record-on-move" => record_on_move = true,
            "--classic" => enabled = false,
            "--shadow-ball" => shadow_ball = true,
            "--psychic" => psychic = true,
            "--hyper-beam" => hyper_beam = true,
            "--surf" => surf = true,
            "--size-comparison" => size_comparison = true,
            "--pidgeotto" => pidgeotto = true,
            "--size" => {
                let value = args.next().context("--size WIDTHxHEIGHT")?;
                let (width, height) = value
                    .split_once('x')
                    .context("--size expects WIDTHxHEIGHT")?;
                window_size = Some((width.parse::<u32>()?, height.parse::<u32>()?));
            }
            "--reduced-flashes" => reduced_flashes = true,
            "--screenshot" => {
                screenshot = Some(PathBuf::from(args.next().context("--screenshot path")?))
            }
            "--second" => second = Some(PathBuf::from(args.next().context("--second path")?)),
            "--record" => record = Some(PathBuf::from(args.next().context("--record directory")?)),
            "--measure" => {
                measure = Some(PathBuf::from(args.next().context("--measure directory")?))
            }
            "--seconds" => seconds = args.next().context("--seconds duration")?.parse::<u32>()?,
            _ => anyhow::bail!("unknown option {flag}"),
        }
    }
    anyhow::ensure!(
        [
            shadow_ball,
            psychic,
            hyper_beam,
            surf,
            size_comparison,
            pidgeotto
        ]
        .into_iter()
        .filter(|active| *active)
        .count()
            <= 1,
        "choose only one of --shadow-ball, --psychic, --hyper-beam --surf, --size-comparison or --pidgeotto"
    );
    anyhow::ensure!(
        [screenshot.is_some(), record.is_some(), measure.is_some()]
            .into_iter()
            .filter(|active| *active)
            .count()
            <= 1,
        "choose screenshot, recording or measurement"
    );
    anyhow::ensure!(
        second.is_none() || screenshot.is_some(),
        "--second needs --screenshot"
    );
    anyhow::ensure!(
        !record_on_move || record.is_some(),
        "--record-on-move needs --record"
    );
    let pack = pack
        .canonicalize()
        .context("find compatible external Crystalpack")?;
    let root = AssetRoot::new(pack.parent().context("pack directory")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let spawn_identifier = runtime.title_new_game_spawn_identifier()?;
    let (map_name, tile_x, tile_y) = if size_comparison {
        let map_name = "UnionCave1F";
        let (width, height) = runtime
            .data()
            .saved_map_tile_bounds(map_name)
            .context("find size comparison map bounds")?;
        let (x, y) = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x as i16, y as i16)))
            .find(|&(x, y)| {
                runtime
                    .data()
                    .overworld_session(
                        map_name,
                        crystal_runtime::core::world::map::TilePosition::new(x, y),
                        0,
                    )
                    .is_ok()
            })
            .context("find a walkable size comparison preview tile")?;
        (map_name, x, y)
    } else {
        ("Route36", 20, 8)
    };
    runtime
        .data()
        .overworld_session(
            map_name,
            crystal_runtime::core::world::map::TilePosition::new(tile_x, tile_y),
            0,
        )
        .context("validate the disposable battle preview location")?;
    crystal_bevy::run_bevy_shell(
        root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: map_name.into(),
            tile_x,
            tile_y,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            voxel_view_enabled: Some(enabled),
            window_title: Some(
                "Geothite | 3D battle | Arrows / Z confirm / X cancel / F3 view / F4 flashes"
                    .into(),
            ),
            render_test_battle: true,
            render_test_shadow_ball: shadow_ball,
            render_test_psychic: psychic,
            render_test_hyper_beam: hyper_beam,
            render_test_surf: surf,
            render_test_size_comparison: size_comparison,
            render_test_pidgeotto: pidgeotto,
            render_test_window_size: window_size,
            battle_reduced_flashes: reduced_flashes,
            render_test_hour: Some(16),
            render_test_screenshot: screenshot,
            render_test_second_screenshot: second,
            render_test_live: live,
            #[cfg(not(target_arch = "wasm32"))]
            render_test_record: record.map(|path| (path, seconds)),
            #[cfg(not(target_arch = "wasm32"))]
            render_test_record_on_move: record_on_move,
            #[cfg(not(target_arch = "wasm32"))]
            render_test_measure: measure.map(|path| (path, seconds)),
            quick_save_path: None,
            ..Default::default()
        },
    )
}
