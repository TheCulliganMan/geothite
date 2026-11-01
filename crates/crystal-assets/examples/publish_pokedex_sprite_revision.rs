//! Refresh selected native sprites while preserving an existing static release UI.
//! Usage: publish_pokedex_sprite_revision BASE_RELEASE NEW_RELEASE SPECIES_ID...

#[allow(dead_code)]
#[path = "../src/minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;
#[allow(dead_code)]
#[path = "../src/minnesota_sprite_sheet.rs"]
mod minnesota_sprite_sheet;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

fn read(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path)?).with_context(|| path.display().to_string())
}

fn copy_release(source: &Path, output: &Path, old_base: &str, new_base: &str) -> Result<()> {
    fs::create_dir_all(output)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = output.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_release(&entry.path(), &target, old_base, new_base)?;
        } else {
            let bytes = fs::read(entry.path())?;
            let text = matches!(
                entry.path().extension().and_then(|value| value.to_str()),
                Some("html" | "json" | "js" | "css" | "svg")
            );
            if text {
                fs::write(
                    target,
                    String::from_utf8(bytes)?.replace(old_base, new_base),
                )?;
            } else {
                fs::write(target, bytes)?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        arguments.len() >= 3,
        "expected base release, new release and species IDs"
    );
    let base = Path::new(&arguments[0]);
    let output = Path::new(&arguments[1]);
    ensure!(!output.exists(), "new release must not already exist");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "release output must remain ignored"
    );
    let release_name = |path: &Path| -> Result<String> {
        Ok(path
            .file_name()
            .and_then(|value| value.to_str())
            .context("release name")?
            .to_owned())
    };
    let old_base = format!("/pokedex/releases/{}", release_name(base)?);
    let new_base = format!("/pokedex/releases/{}", release_name(output)?);
    let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = project.join("planning/minnesota/pokemon");
    let registry = read(&root.join("sprite-generation.json"))?;
    let profiles = read(&root.join("species-profiles.json"))?;
    copy_release(base, output, &old_base, &new_base)?;
    for id in &arguments[2..] {
        let profile = profiles["species"]
            .as_array()
            .context("profiles")?
            .iter()
            .find(|profile| profile["id"].as_str() == Some(id))
            .context("unknown species")?;
        let mut palette = [32767, 0, 0, 0];
        for index in 0..2 {
            let color = &profile["shiny_rgb5"][index];
            let channel = |channel: usize| -> Result<u16> {
                let value = color[channel].as_u64().context("shiny RGB5 channel")?;
                ensure!(value < 32, "invalid RGB5 channel");
                Ok(u16::try_from(value)?)
            };
            palette[index + 1] = channel(0)? | channel(1)? << 5 | channel(2)? << 10;
        }
        let mut views = 0;
        for family in registry["families"]
            .as_array()
            .context("registered families")?
        {
            let manifest = read(&root.join(family["manifest"].as_str().context("manifest path")?))?;
            for frame in manifest["sprites"].as_array().context("sprites")? {
                if frame["species_id"].as_str() != Some(id) {
                    continue;
                }
                let view = frame["view"].as_str().context("view")?;
                let stem = format!("{}.{}", frame["id"].as_str().context("frame ID")?, view);
                let directory = project
                    .join("target/minnesota-pokemon/native")
                    .join(family["id"].as_str().context("family ID")?);
                let name = format!("{}-{view}", id.to_ascii_lowercase().replace('_', "-"));
                fs::copy(
                    directory.join(format!("{stem}.png")),
                    output.join("sprites").join(format!("{name}.png")),
                )?;
                let indices = read(&directory.join(format!("{stem}.json")))?;
                let rows = indices["rows"]
                    .as_array()
                    .context("indexed rows")?
                    .iter()
                    .map(|row| Ok(row.as_str().context("indexed row")?.replace('.', "0")))
                    .collect::<Result<Vec<_>>>()?;
                fs::write(
                    output.join("sprites").join(format!("{name}-shiny.png")),
                    minnesota_sprite_sheet::indexed_rows_png(&rows, palette)?,
                )?;
                views += 1;
            }
        }
        ensure!(views == 2, "species must have exactly two reviewed views");
    }
    println!(
        "Refreshed {} species in {}",
        arguments.len() - 2,
        output.display()
    );
    Ok(())
}
