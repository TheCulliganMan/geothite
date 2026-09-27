//! cargo run -p crystal-mapgen --example render_proof -- <pret> <source-v2.json> <output>
use anyhow::{Context, Result};
use crystal_mapgen::*;
use image::{
    RgbaImage,
    imageops::{self, FilterType},
};
use std::{collections::BTreeMap, path::PathBuf};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pret = PathBuf::from(args.next().context("external pret directory")?);
    let source = PathBuf::from(args.next().context("normalized schema-2 source")?);
    let output = PathBuf::from(args.next().context("ignored output directory")?);
    std::fs::create_dir_all(&output)?;
    let art = SourceArtPreview::load(&pret)?;
    let atlas = art.atlas()?;
    annotate_atlas(&atlas).save(output.join("atlas.png"))?;
    let source: MapSource = serde_json::from_slice(&std::fs::read(source)?)?;
    let start = std::time::Instant::now();
    let grid = generate_grid(source, 96, 96)?;
    let second = generate_grid(grid.source.clone(), 96, 96)?;
    anyhow::ensure!(grid == second, "nondeterministic scene");
    println!(
        "Minneapolis generation + determinism check: {:?}",
        start.elapsed()
    );
    let mut grids = vec![("minneapolis", grid)];
    for name in [
        "urban",
        "residential",
        "waterfront",
        "woodland",
        "meadow",
        "rocky",
    ] {
        grids.push((name, generate_grid(fixture(name), 64, 64)?));
    }
    let mut views = RgbaImage::new(480 * 3, 432 * 2);
    let mut html = String::from(
        "<!doctype html><meta charset='utf-8'><title>Geothite scenery proof</title><style>body{background:#141a20;color:#e7edf3;font:16px system-ui;max-width:1280px;margin:40px auto;padding:0 20px}h1{font-size:32px}p{max-width:900px;line-height:1.6}.row{display:flex;flex-wrap:wrap;gap:20px}figure{margin:0 0 24px}img{image-rendering:pixelated;max-width:100%;border:1px solid #445}figcaption{padding:10px 0;color:#b5c6d9}a{color:#9ac8ff}.map{width:768px}code{color:#b4dfc4}</style><h1>Geothite: fresh scenery renders</h1><p>Generator v2, current Rust scene planner and production atlas builder. Original tile art loaded directly from the external pret checkout. These are static terrain renders, without characters, UI or gameplay simulation. Minneapolis uses the archived one-mile OSM snapshot with unknown descriptive tags kept unknown; it is not a fresh seven-cell region. The other six scenes are explicitly synthetic land-cover fixtures.</p>",
    );
    for (number, (name, grid)) in grids.iter().enumerate() {
        let audit = audit_grid(grid);
        println!(
            "{name}: {} structures, audit={}, {:?}",
            audit.structures, audit.passed, audit.errors
        );
        std::fs::write(
            output.join(format!("{name}-grid.json")),
            serde_json::to_vec_pretty(grid)?,
        )?;
        let day = art.render(grid, "day")?;
        let night = art.render(grid, "night")?;
        day.save(output.join(format!("{name}-day.png")))?;
        night.save(output.join(format!("{name}-night.png")))?;
        let scene = grid.scene.as_ref().unwrap();
        let center = scene
            .structures
            .first()
            .map(|s| (s.origin.0 * 32 + 32, s.origin.1 * 32 + 48))
            .or_else(|| {
                scene
                    .destinations
                    .first()
                    .map(|d| (d.origin.0 * 32 + 80, d.origin.1 * 32 + 64))
            })
            .unwrap_or((grid.width * 16, grid.height * 16));
        let walk = crop(&day, center, 160, 144, 3);
        let walk_night = crop(&night, center, 160, 144, 3);
        walk.save(output.join(format!("{name}-walk-day.png")))?;
        walk_night.save(output.join(format!("{name}-walk-night.png")))?;
        let wide = crop(&day, center, 320, 240, 2);
        wide.save(output.join(format!("{name}-detail.png")))?;
        if number > 0 {
            imageops::overlay(
                &mut views,
                &walk,
                ((number - 1) % 3 * 480) as i64,
                ((number - 1) / 3 * 432) as i64,
            );
        }
        html += &format!(
            "<h2>{name}</h2><p>{} structures. Scene audit: {}. Walking view is 160×144 source pixels enlarged 3× without smoothing.</p><div class='row'><figure><a href='{name}-day.png'><img class='map' src='{name}-day.png'></a><figcaption>Full generated map · click for native pixels</figcaption></figure><div><figure><img src='{name}-walk-day.png'><figcaption>Walking scale · day</figcaption></figure><figure><img src='{name}-walk-night.png'><figcaption>Same location · night</figcaption></figure></div></div>",
            audit.structures,
            if audit.passed { "passed" } else { "FAILED" }
        );
    }
    views.save(output.join("six-family-walking-views.png"))?;
    html += "<p>Art source: pret/pokecrystal. Geography: © OpenStreetMap contributors. No game artwork or generated images are checked into Git.</p>";
    std::fs::write(output.join("index.html"), html)?;
    Ok(())
}
fn crop(image: &RgbaImage, center: (u16, u16), w: u32, h: u32, scale: u32) -> RgbaImage {
    let x = u32::from(center.0)
        .saturating_sub(w / 2)
        .min(image.width() - w);
    let y = u32::from(center.1)
        .saturating_sub(h / 2)
        .min(image.height() - h);
    imageops::resize(
        &imageops::crop_imm(image, x, y, w, h).to_image(),
        w * scale,
        h * scale,
        FilterType::Nearest,
    )
}
fn area(kind: FeatureKind, w: f64, s: f64, e: f64, n: f64) -> Feature {
    Feature {
        details: Default::default(),
        kind,
        name: None,
        area: true,
        bridge: false,
        points: vec![
            Coordinate { lat: s, lon: w },
            Coordinate { lat: n, lon: w },
            Coordinate { lat: n, lon: e },
            Coordinate { lat: s, lon: e },
            Coordinate { lat: s, lon: w },
        ],
    }
}
fn fixture(name: &str) -> MapSource {
    let mut cover = area(FeatureKind::Park, 0., 0., 1., 1.);
    cover.details.tags = BTreeMap::from([(
        if matches!(name, "urban" | "residential" | "meadow") {
            "landuse"
        } else {
            "natural"
        }
        .into(),
        match name {
            "urban" => "commercial",
            "residential" => "residential",
            "woodland" => "wood",
            "waterfront" => "wetland",
            "rocky" => "bare_rock",
            _ => "meadow",
        }
        .into(),
    )]);
    let mut features = vec![cover];
    for lat in [0.3, 0.5, 0.7] {
        features.push(Feature {
            details: Default::default(),
            kind: FeatureKind::Street,
            name: Some("Fixture Avenue".into()),
            area: false,
            bridge: false,
            points: vec![Coordinate { lat, lon: 0.02 }, Coordinate { lat, lon: 0.98 }],
        });
    }
    if matches!(name, "urban" | "residential") {
        for lat in [0.34, 0.55, 0.75] {
            for lon in [0.08, 0.22, 0.38, 0.55, 0.70, 0.88] {
                let mut f = area(
                    FeatureKind::Building,
                    lon - 0.016,
                    lat - 0.016,
                    lon + 0.016,
                    lat + 0.016,
                );
                f.details.osm_id = Some(format!("fixture/{lat}/{lon}"));
                features.push(f);
            }
        }
    }
    if name == "waterfront" {
        features.push(area(FeatureKind::Water, 0.58, 0.05, 0.98, 0.95));
    }
    MapSource {
        schema_version: SOURCE_SCHEMA_VERSION,
        center: Coordinate { lat: 0.5, lon: 0.5 },
        bounds: BoundingBox {
            south: 0.,
            west: 0.,
            north: 1.,
            east: 1.,
        },
        attribution: format!("Synthetic {name} fixture"),
        features,
        h3: None,
    }
}

fn annotate_atlas(atlas: &RgbaImage) -> RgbaImage {
    let glyphs: [[u8; 5]; 16] = [
        [7, 5, 5, 5, 7],
        [2, 6, 2, 2, 7],
        [7, 1, 7, 4, 7],
        [7, 1, 7, 1, 7],
        [5, 5, 7, 1, 1],
        [7, 4, 7, 1, 7],
        [7, 4, 7, 5, 7],
        [7, 1, 1, 1, 1],
        [7, 5, 7, 5, 7],
        [7, 5, 7, 1, 7],
        [2, 5, 7, 5, 5],
        [6, 5, 6, 5, 6],
        [7, 4, 4, 4, 7],
        [6, 5, 5, 5, 6],
        [7, 4, 6, 4, 7],
        [7, 4, 6, 4, 4],
    ];
    let mut out = RgbaImage::from_pixel(
        16 * 68,
        atlas.height() / 32 * 82,
        image::Rgba([30, 34, 40, 255]),
    );
    for i in 0..atlas.width() / 32 * atlas.height() / 32 {
        let x = (i % 16) * 68;
        let y = (i / 16) * 82;
        let tile = imageops::resize(
            &imageops::crop_imm(atlas, i % 16 * 32, i / 16 * 32, 32, 32).to_image(),
            64,
            64,
            FilterType::Nearest,
        );
        imageops::overlay(&mut out, &tile, x as i64, y as i64);
        for (digit, value) in [(0, i / 16), (1, i % 16)] {
            for (gy, bits) in glyphs[value as usize].iter().enumerate() {
                for gx in 0..3 {
                    if bits & (1 << (2 - gx)) != 0 {
                        for dy in 0..2 {
                            for dx in 0..2 {
                                out.put_pixel(
                                    x + 22 + digit * 8 + gx * 2 + dx,
                                    y + 66 + gy as u32 * 2 + dy,
                                    image::Rgba([255, 255, 255, 255]),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    out
}
