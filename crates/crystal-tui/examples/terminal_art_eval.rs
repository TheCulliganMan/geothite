//! Verification-only rendered frames. Read external packs/saves; never modify
//! them. Outputs belong under ignored target/, not in the shipped content.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_runtime::CrystalRuntime;
use geothite::{PaintedRenderer, RuntimeTextRenderer};
use ratatui::style::Color;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("terminal_art_eval PACK OUTPUT [SAVE…]")?,
    );
    let output = PathBuf::from(args.next().context("missing output directory")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let files = loaded.pack().runtime_files().clone();
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let saves: Vec<_> = args.map(PathBuf::from).collect();
    let mut games = vec![(
        "room".to_owned(),
        VisibleShellController::new_game(root.clone(), runtime.clone(), "CHRIS", None)?,
    )];
    for save in saves {
        let name = save
            .file_stem()
            .context("save name")?
            .to_string_lossy()
            .to_string();
        games.push((
            name,
            VisibleShellController::load_save(root.clone(), runtime.clone(), save, None)?,
        ));
    }
    std::fs::create_dir_all(&output)?;
    for (name, mut game) in games {
        let snapshot = game.presentation_snapshot()?;
        // Fixture names are not proof of the visible phase (a "battle" save
        // can start on the grass before the encounter). Label actual state.
        let name = format!("{}-{:?}-{name}", snapshot.overworld.map_name, snapshot.phase);
        let checksum = snapshot.state_checksum.clone();
        let text = RuntimeTextRenderer::default().render(&snapshot);
        for (cols, rows) in [(80, 24), (120, 40), (160, 48)] {
            let mut painter = PaintedRenderer::from_pack_assets(&files);
            let coarse = painter.draw(&snapshot, &text, cols, rows, None, false, false);
            let before_viewport = painter.viewport;
            let fine = painter.draw_native(&snapshot, &text, cols, rows, None, false);
            let bounds = painter.scene_bounds();
            if let [_, _, width, height] = bounds.as_slice() {
                let image = painter
                    .circle_image(u32::from(*width) * 12, u32::from(*height) * 24)
                    .context("shared circle field")?;
                image.save(output.join(format!("{name}-{cols}x{rows}-canvas.png")))?;
            }
            assert_eq!(
                painter.viewport, before_viewport,
                "glyph encoding must not change the camera"
            );
            let cells = |b: &ratatui::buffer::Buffer| {
                b.content
                    .iter()
                    .map(|c| {
                        let color = |color| match color {
                            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
                            _ => "#e8e6da".into(),
                        };
                        serde_json::json!([c.symbol(), color(c.fg), color(c.bg)])
                    })
                    .collect::<Vec<_>>()
            };
            std::fs::write(
                output.join(format!("{name}-{cols}x{rows}.json")),
                serde_json::to_vec(&serde_json::json!({
                    "cols": cols, "rows": rows, "map": snapshot.overworld.map_name,
                    "phase": format!("{:?}", snapshot.phase),
                    "tile": [snapshot.overworld.tile.x, snapshot.overworld.tile.y],
                    "before": cells(&coarse), "after": cells(&fine),
                }))?,
            )?;
        }
        assert_eq!(
            game.presentation_snapshot()?.state_checksum,
            checksum,
            "painting cannot advance gameplay"
        );
    }
    Ok(())
}
