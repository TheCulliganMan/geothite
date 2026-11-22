use std::path::PathBuf;

use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_runtime::CrystalRuntime;

use super::PaintedRenderer;
use crate::{RuntimeTextRenderer, ViewportSize};

#[test]
#[ignore = "requires CRYSTAL_PORTRAIT_TEST_PACK, CRYSTAL_PORTRAIT_TEST_SAVE and CRYSTAL_PORTRAIT_TEST_ID"]
fn painted_trainer_portrait_uses_resolved_pack_pixels_in_desktop_and_mobile_layouts() -> Result<()>
{
    let pack =
        PathBuf::from(std::env::var_os("CRYSTAL_PORTRAIT_TEST_PACK").context("portrait pack")?);
    let save =
        PathBuf::from(std::env::var_os("CRYSTAL_PORTRAIT_TEST_SAVE").context("battle save")?);
    let expected = std::env::var("CRYSTAL_PORTRAIT_TEST_ID")?;
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let files = loaded.pack().runtime_files().clone();
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let mut controller = VisibleShellController::load_save(root, runtime, save, None)?;
    let mut snapshot = controller.snapshot()?;
    let battle = snapshot.battle.as_mut().context("saved active battle")?;
    assert_eq!(battle.trainer_portrait.as_deref(), Some(expected.as_str()));
    // The save contains authoritative battle state, not an in-flight printer.
    // Stage only the presentation flag; production controller phase ownership
    // is exercised separately by original_npc_controller_talk_battle_turn_and_save.
    battle.trainer_portrait_visible = true;
    let path = format!("gfx/trainers/{expected}.png");
    let expected_pixels =
        image::load_from_memory(files.get(&path).context("installed portrait")?)?.to_rgba8();
    let text = RuntimeTextRenderer::new(ViewportSize {
        width: 20,
        height: 18,
    })
    .render(&snapshot);
    for (columns, rows, touchscreen) in [(80, 40, false), (40, 60, true), (96, 30, true)] {
        let mut painter = PaintedRenderer::from_pack_assets(&files);
        let buffer = painter.draw(&snapshot, &text, columns, rows, None, false, touchscreen);
        let sprite = painter
            .sprites
            .get(&(path.clone(), false))
            .and_then(Option::as_ref)
            .context("TUI loaded the exact trainer override")?;
        assert_eq!(sprite.pixels, expected_pixels.as_raw().as_slice());
        assert_eq!((sprite.width, sprite.size), (56, 56));
        assert!(
            !painter
                .sprites
                .keys()
                .any(|(path, _)| path.starts_with("gfx/pokemon/")),
            "trainer introduction must not substitute the enemy Pokemon"
        );
        assert!(
            buffer.content.iter().any(|cell| cell.symbol() != " "
                && matches!(cell.fg, ratatui::style::Color::Rgb(_, _, _)))
        );
        if columns == 40 {
            assert!(
                (5_u16..40).any(|y| (2_u16..20)
                    .any(|x| matches!(buffer[(x, y)].symbol(), "." | ":" | "+" | "#" | "@"))),
                "narrow single-trainer art uses the available left half"
            );
        }
    }
    Ok(())
}
