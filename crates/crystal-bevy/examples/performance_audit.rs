//! One isolated offscreen scenario per process. Orchestrate with tools/performance_audit.py.
use std::path::PathBuf;

use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::performance_audit::{AuditFixture, AuditRequest, AuditScenario};
use crystal_bevy::{BevyShellConfig, BevyShellStart};
use crystal_runtime::CrystalRuntime;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("expected PACK SCENARIO.json OUTPUT_DIR [--cpu-only]")?,
    )
    .canonicalize()?;
    let recipe = args.next().context("missing scenario file or --catalog")?;
    if recipe == "--catalog" {
        let root = AssetRoot::new(pack.parent().context("pack directory")?);
        let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
        for name in args {
            let map = runtime
                .data()
                .maps
                .get(&name)
                .with_context(|| format!("unknown map {name}"))?;
            println!(
                "{}",
                serde_json::json!({"map":name,"attributes":map.attributes,"warps":map.events.warps})
            );
        }
        return Ok(());
    }
    let scenario_path = PathBuf::from(recipe);
    let directory = PathBuf::from(args.next().context("missing output directory")?);
    let gpu = match args.next().as_deref() {
        None => true,
        Some("--cpu-only") => false,
        Some(value) => anyhow::bail!("unknown argument {value}"),
    };
    anyhow::ensure!(args.next().is_none(), "unexpected argument");
    let scenario: AuditScenario = serde_json::from_slice(&std::fs::read(scenario_path)?)?;
    scenario.validate()?;
    anyhow::ensure!(
        std::env::var("CRYSTAL_RENDER_TEST_HEADLESS").as_deref() == Ok("1")
            && std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TARGET").as_deref() == Ok("image"),
        "audits require CRYSTAL_RENDER_TEST_HEADLESS=1 and CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image"
    );
    std::fs::create_dir_all(&directory)?;
    anyhow::ensure!(
        std::fs::read_dir(&directory)?.next().is_none(),
        "audit directory must be empty"
    );
    #[cfg(feature = "operation-trace")]
    let _trace = crystal_bevy::operation_trace::start();
    let asset_root = AssetRoot::new(pack.parent().context("pack directory")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded)?;
    let metadata = serde_json::json!({
        "schema":1,"scenario":scenario,"pack_identity":runtime.pack_identity(),
        "os":std::env::consts::OS,"arch":std::env::consts::ARCH,
        "debug_assertions":cfg!(debug_assertions),"operation_trace":cfg!(feature="operation-trace"),
        "gpu_instrumentation":gpu,"viewport":[960,864],"hour":12,
        "target":"offscreen_image","pipeline_compilation":"production asynchronous",
        "simulation":"production real-time cadence","cache_policy":"fresh process; OS/driver caches retained"
    });
    std::fs::write(
        directory.join("metadata.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    let fixture = scenario.fixture;
    crystal_bevy::run_bevy_shell(
        asset_root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier()?,
            map_name: scenario.map.clone(),
            tile_x: scenario.tile[0],
            tile_y: scenario.tile[1],
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            voxel_view_enabled: Some(true),
            render_test_window_size: Some((960, 864)),
            render_test_hour: Some(12),
            render_test_battle: fixture == AuditFixture::Battle,
            render_test_surf_encounter: fixture == AuditFixture::SurfShore,
            performance_audit: Some(AuditRequest {
                scenario,
                directory: directory.clone(),
                gpu,
            }),
            ..Default::default()
        },
    )?;
    anyhow::ensure!(
        directory.join("audit.json").is_file(),
        "scenario ended without a report"
    );
    Ok(())
}
