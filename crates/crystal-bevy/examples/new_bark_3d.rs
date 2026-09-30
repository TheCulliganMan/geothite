//! Local, playable connected Johto art slice using the production controller.
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
    let mut map = "NewBarkTown".to_string();
    let mut tile_x = None;
    let mut tile_y = None;
    let mut screenshot = None;
    let mut record = None;
    let mut walk = None;
    let mut measure = None;
    let mut seconds = 30;
    let mut zoom = 0;
    let mut orbit = -1;
    while let Some(flag) = args.next() {
        if !flag.starts_with('-') && screenshot.is_none() {
            screenshot = Some(PathBuf::from(flag));
            continue;
        }
        let value = args
            .next()
            .with_context(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--map" => map = value,
            "--x" => tile_x = Some(value.parse::<i16>()?),
            "--y" => tile_y = Some(value.parse::<i16>()?),
            "--screenshot" => screenshot = Some(PathBuf::from(value)),
            "--record" => record = Some(PathBuf::from(value)),
            "--walk" => walk = Some(value),
            "--measure" => measure = Some(PathBuf::from(value)),
            "--seconds" => seconds = value.parse::<u32>()?,
            "--zoom" => zoom = value.parse::<u8>()?,
            "--orbit" => orbit = value.parse::<i8>()?,
            _ => anyhow::bail!("unknown option {flag}"),
        }
    }
    anyhow::ensure!(
        [screenshot.is_some(), record.is_some(), measure.is_some()]
            .into_iter()
            .filter(|value| *value)
            .count()
            <= 1,
        "use only one of --screenshot, --record or --measure"
    );
    let default_position = match map.as_str() {
        "NewBarkTown" => (13, 6),
        "Route29" => (50, 9),
        "CherrygroveCity" => (29, 4),
        "Route30" => (7, 40),
        "Route31" => (14, 9),
        "VioletCity" => (18, 18),
        _ => anyhow::bail!(
            "choose NewBarkTown, Route29, CherrygroveCity, Route30, Route31 or VioletCity"
        ),
    };
    let pack = pack
        .canonicalize()
        .context("find compatible local Crystalpack")?;
    let root = AssetRoot::new(pack.parent().context("pack directory")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let (width, height) = runtime
        .data()
        .saved_map_tile_bounds(&map)
        .with_context(|| format!("missing preview map {map}"))?;
    let (tile_x, tile_y) = (
        tile_x.unwrap_or(default_position.0),
        tile_y.unwrap_or(default_position.1),
    );
    anyhow::ensure!(
        tile_x >= 0 && tile_y >= 0 && (tile_x as u16) < width && (tile_y as u16) < height,
        "preview coordinate is outside {map} ({width} by {height})"
    );
    let spawn_identifier = runtime.title_new_game_spawn_identifier()?;
    crystal_bevy::run_bevy_shell(
        root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: map.clone(),
            tile_x,
            tile_y,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            voxel_view_enabled: Some(true),
            voxel_camera: Some((zoom, orbit)),
            window_title: Some(format!(
                "Geothite | {map} 3D | Q/E orbit, PgUp/PgDn zoom, F3 2D/3D"
            )),
            render_test_hour: Some(16),
            render_test_screenshot: screenshot,
            render_test_walk: walk,
            #[cfg(not(target_arch = "wasm32"))]
            render_test_record: record.map(|path| (path, seconds)),
            #[cfg(not(target_arch = "wasm32"))]
            render_test_measure: measure.map(|path| (path, seconds)),
            quick_save_path: None,
            ..Default::default()
        },
    )
}
