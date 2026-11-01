//! Read-only performance check against the external pack. No fullscreen logs.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_runtime::CrystalRuntime;
use geothite::{PaintedRenderer, RuntimeTextRenderer};
use image::{
    ImageEncoder,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};
use std::{path::PathBuf, time::Instant};

fn main() -> Result<()> {
    let pack = PathBuf::from(
        std::env::args()
            .nth(1)
            .context("terminal_canvas_bench PACK")?,
    );
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let files = loaded.pack().runtime_files().clone();
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let mut game = VisibleShellController::new_game(root, runtime, "CHRIS", None)?;
    let snapshot = game.presentation_snapshot()?;
    let text = RuntimeTextRenderer::default().render(&snapshot);
    for (cols, rows, cw, ch) in [(80, 24, 12, 24), (128, 48, 16, 32), (160, 60, 16, 32)] {
        let mut painter = PaintedRenderer::from_pack_assets(&files);
        painter.draw_native(&snapshot, &text, cols, rows, None, false);
        let mut prepare = 0.;
        let mut raster = 0.;
        let mut last = None;
        for _ in 0..30 {
            painter.advance_ink_by(1. / 30.);
            let start = Instant::now();
            painter.draw_native(&snapshot, &text, cols, rows, None, false);
            prepare += start.elapsed().as_secs_f64();
            let bounds = painter.scene_bounds();
            let (_, _, w, h) = (bounds[0], bounds[1], bounds[2], bounds[3]);
            let (w, h) = geothite::terminal_canvas_size(u32::from(w) * cw, u32::from(h) * ch);
            let start = Instant::now();
            last = painter.circle_image(w, h);
            raster += start.elapsed().as_secs_f64();
        }
        let image = last.context("Home art")?;
        println!(
            "{cols}x{rows}, {}x{} pixels: scene {:.2}ms, raster {:.2}ms",
            image.width(),
            image.height(),
            prepare * 1000. / 30.,
            raster * 1000. / 30.
        );
        for (compression, filter) in [
            (CompressionType::Default, FilterType::Adaptive),
            (CompressionType::Fast, FilterType::Adaptive),
            (CompressionType::Fast, FilterType::Sub),
            (CompressionType::Fast, FilterType::Up),
            (CompressionType::Fast, FilterType::NoFilter),
        ] {
            let start = Instant::now();
            let mut bytes = 0;
            for _ in 0..10 {
                let mut png = Vec::new();
                PngEncoder::new_with_quality(&mut png, compression, filter).write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    image::ExtendedColorType::Rgb8,
                )?;
                bytes = png.len();
            }
            println!(
                "  {compression:?}/{filter:?}: {:.2}ms, {bytes} bytes/frame ({:.2} MB/s @30fps with base64)",
                start.elapsed().as_secs_f64() * 100.,
                bytes as f64 * 40. / 1e6
            );
        }
    }
    Ok(())
}
