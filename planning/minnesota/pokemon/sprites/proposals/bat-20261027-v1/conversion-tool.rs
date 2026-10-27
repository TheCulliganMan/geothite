#![allow(dead_code)]
#[path = "../../../../../../crates/crystal-assets/src/minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;
#[path = "../../../../../../crates/crystal-assets/src/minnesota_sprite_sheet.rs"]
mod minnesota_sprite_sheet;
use minnesota_sprite_sheet::{
    ExteriorContour, MinnesotaSheet, SpriteConversionOptions, SpriteCrop, SpriteSampling,
    SpriteView,
};
fn main() -> anyhow::Result<()> {
    let output =
        std::path::Path::new("planning/minnesota/pokemon/sprites/proposals/bat-20261027-v1");
    let sheet =
        MinnesotaSheet::from_png(&std::fs::read(output.join("front-back.imagegen-v1.png"))?)?;
    let recipes = [
        (
            "echolet-front",
            SpriteView::Front,
            [100, 80, 480, 520],
            [40, 40],
            [32767, 30524, 23920, 0],
        ),
        (
            "echolet-back",
            SpriteView::Back,
            [690, 90, 550, 490],
            [46, 47],
            [32767, 30524, 23920, 0],
        ),
        (
            "sonatlas-front",
            SpriteView::Front,
            [0, 620, 710, 590],
            [54, 52],
            [32767, 31607, 19848, 0],
        ),
        (
            "sonatlas-back",
            SpriteView::Back,
            [710, 620, 540, 590],
            [46, 47],
            [32767, 31607, 19848, 0],
        ),
    ];
    let mut records = Vec::new();
    let mut board = vec![[255u8; 3]; 120 * 120];
    for (name, view, crop, fit, palette) in recipes {
        let frame = sheet.candidate_with_conversion(
            SpriteCrop {
                x: crop[0],
                y: crop[1],
                width: crop[2],
                height: crop[3],
                fit_width: fit[0],
                fit_height: fit[1],
            },
            palette,
            view,
            SpriteConversionOptions {
                minimum_white_samples: 4,
                minimum_black_samples: 12,
                exterior_contour: ExteriorContour::Solid,
                sampling: SpriteSampling::AreaVote,
            },
        )?;
        anyhow::ensure!(
            frame.indices().iter().all(|index| *index <= 3),
            "native four-ink frame"
        );
        anyhow::ensure!(palette.iter().all(|word| *word <= 32767), "RGB555 words");
        let bytes = frame.bitplanes();
        anyhow::ensure!(
            bytes.len() == view.canvas() * view.canvas() / 4,
            "native 2bpp length"
        );
        std::fs::write(
            output.join(format!("{name}.native.png")),
            frame.png(palette, 1)?,
        )?;
        std::fs::write(
            output.join(format!("{name}.zoom8.png")),
            frame.png(palette, 8)?,
        )?;
        std::fs::write(output.join(format!("{name}.2bpp")), bytes)?;
        std::fs::write(
            output.join(format!("{name}.indices.json")),
            serde_json::to_vec_pretty(&frame)?,
        )?;
        let (ox, oy) = match name {
            "echolet-front" => (4, 4),
            "echolet-back" => (68, 12),
            "sonatlas-front" => (4, 64),
            _ => (68, 72),
        };
        let colors = minnesota_pokemon_pixel_art::colored(frame.indices(), palette);
        for y in 0..view.canvas() {
            for x in 0..view.canvas() {
                board[(oy + y) * 120 + ox + x] = colors[y * view.canvas() + x];
            }
        }
        records.push(serde_json::json!({"name":name,"view":view,"crop":crop,"fit":fit,"palette_rgb555":palette,"canvas":frame.size,"status":"unregistered native proposal, visual review pending"}));
    }
    let mut zoom = vec![[255u8; 3]; 480 * 480];
    for y in 0..480 {
        for x in 0..480 {
            zoom[y * 480 + x] = board[(y / 4) * 120 + x / 4];
        }
    }
    std::fs::write(
        output.join("native-review-board.png"),
        minnesota_pokemon_pixel_art::png_rgb(480, 480, &zoom)?,
    )?;
    std::fs::write(
        output.join("conversion-v1.json"),
        serde_json::to_vec_pretty(&records)?,
    )?;
    Ok(())
}
