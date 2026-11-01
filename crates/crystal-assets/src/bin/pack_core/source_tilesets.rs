use super::radio_text_export::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

fn integer(value: &str) -> Result<u8> {
    if let Some(value) = value.strip_prefix('$') {
        Ok(u8::from_str_radix(value, 16)?)
    } else {
        Ok(value.parse()?)
    }
}
fn palette(source: &str) -> Result<Vec<u8>> {
    let labels = [
        "GRAY", "RED", "GREEN", "WATER", "YELLOW", "BROWN", "ROOF", "TEXT",
    ];
    let mut result = Vec::new();
    let mut repeat: Option<(usize, Vec<u8>)> = None;
    for line in source
        .lines()
        .map(code_part)
        .filter(|line| !line.is_empty())
    {
        if let Some(count) = line.strip_prefix("rept ") {
            ensure!(repeat.is_none(), "nested palette repeat");
            repeat = Some((count.parse()?, Vec::new()));
            continue;
        }
        if line == "endr" {
            let (count, body) = repeat.take().context("palette repeat terminator")?;
            for _ in 0..count {
                result.extend_from_slice(&body);
            }
            continue;
        }
        let mut values = Vec::new();
        if let Some(args) = line.strip_prefix("tilepal ") {
            let args = source_arguments(args)?;
            ensure!(args.len() == 9, "tilepal must contain 8 palettes");
            let bank = integer(&args[0])?;
            ensure!(bank <= 1, "invalid tile bank");
            for label in &args[1..] {
                let value = labels
                    .iter()
                    .position(|value| value == label)
                    .with_context(|| format!("unknown palette {label}"))?;
                values.push((bank << 3) | u8::try_from(value)?);
            }
        } else if let Some(args) = line.strip_prefix("db ") {
            for arg in source_arguments(args)? {
                let value = integer(&arg)?;
                values.extend([value & 15, value >> 4]);
            }
        } else {
            anyhow::bail!("unsupported palette source {line}");
        }
        if let Some((_, body)) = repeat.as_mut() {
            body.extend(values);
        } else {
            result.extend(values);
        }
    }
    ensure!(repeat.is_none(), "unterminated palette repeat");
    ensure!(!result.is_empty(), "source palette map is empty");
    Ok(result)
}
fn copy_presentation(source: &Path, target: &Path) -> Result<()> {
    for entry in std::fs::read_dir(source)? {
        let path = entry?.path();
        let destination = target.join(path.file_name().context("source asset filename")?);
        if path.is_dir() {
            copy_presentation(&path, &destination)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| {
                matches!(
                    ext,
                    "png"
                        | "2bpp"
                        | "1bpp"
                        | "gbcpal"
                        | "pal"
                        | "tilemap"
                        | "rle"
                        | "bin"
                        | "attrmap"
                        | "dimensions"
                )
            })
        {
            std::fs::create_dir_all(destination.parent().context("asset output directory")?)?;
            let bytes = std::fs::read(&path)?;
            if std::fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
                std::fs::write(&destination, bytes)?;
            }
        }
    }
    Ok(())
}
pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let assets = root.join("apps/web/assets");
    let data = assets.join("data");
    let output = data.join("content-packs/core-modular/tilesets");
    std::fs::create_dir_all(&output)?;
    let presentation = data.join("tilesets");
    std::fs::create_dir_all(&presentation)?;
    let mut catalog = BTreeMap::new();
    let referenced = super::source_map_geometry::read(&vendor)?
        .into_values()
        .map(|map| map.attributes.tileset_name)
        .collect::<std::collections::BTreeSet<_>>();
    for entry in std::fs::read_dir(vendor.join("data/tilesets"))? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("tileset filename")?;
        let Some(id) = name.strip_suffix("_collision.asm") else {
            continue;
        };
        if !referenced.contains(id) {
            continue;
        }
        let mut collision = BTreeMap::new();
        for line in std::fs::read_to_string(&path)?.lines().map(code_part) {
            if let Some(args) = line.strip_prefix("tilecoll ") {
                let args = source_arguments(args)?;
                ensure!(args.len() == 4, "invalid tile collision");
                collision.insert(format!("{:02x}", collision.len()), args);
            }
        }
        ensure!(!collision.is_empty(), "empty tileset {id}");
        let palettes = palette(&std::fs::read_to_string(
            vendor
                .join("gfx/tilesets")
                .join(format!("{id}_palette_map.asm")),
        )?)
        .with_context(|| format!("source palette {id}"))?;
        let value = json!({"collision":collision,"palette_map":palettes});
        let _: crystal_assets::TilesetDefinition = serde_json::from_value(value.clone())?;
        std::fs::write(
            output.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&json!({id:value}))?,
        )?;
        std::fs::copy(
            vendor
                .join("data/tilesets")
                .join(format!("{id}_metatiles.bin")),
            presentation.join(format!("{id}_metatiles.bin")),
        )?;
        catalog.insert(id.to_owned(), value);
    }
    std::fs::write(
        data.join("tilesets.json"),
        serde_json::to_vec_pretty(&catalog)?,
    )?;
    copy_presentation(&vendor.join("gfx"), &assets.join("gfx"))?;
    Ok(())
}
