//! Publish the authored R10 carrier ridge and its exact native inward corner.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    add_r10_carrier_corner_art, read_verified_compiled_game_pack, revise_authored_route_terrain,
};
use sha2::{Digest, Sha256};

fn validate_corner_recipe(source: &str) -> Result<()> {
    let layout: serde_json::Value = serde_json::from_str(source)?;
    let mut recipes = layout
        .get("derived_blocks")
        .and_then(serde_json::Value::as_array)
        .context("R10 native art recipes")?
        .iter()
        .filter(|recipe| recipe.get("id").and_then(serde_json::Value::as_str) == Some("85"));
    let recipe = recipes.next().context("R10 inward corner85 recipe")?;
    let expected = serde_json::json!({
        "id": "85", "source_tileset": "johto", "source_block": "71",
        "tile_quadrant_sources": [
            {"source_block": "71", "source_quadrant": 0},
            {"source_block": "69", "source_quadrant": 1},
            {"source_block": "71", "source_quadrant": 2},
            {"source_block": "71", "source_quadrant": 3}
        ],
        "collision": ["FLOOR", "WALL", "FLOOR", "FLOOR"],
        "required_typed_extension": "r10_carrier_corner_art::add_r10_carrier_corner_art"
    });
    ensure!(
        recipes.next().is_none() && recipe == &expected,
        "R10 source recipe85 does not match the reviewed v2 floor ending"
    );
    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().context(
        "usage: revise_r10_carrier_plateau PACK SHA256 BEFORE AFTER DOSSIER ART_OUTPUT OUTPUT",
    )?);
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("parent SHA256 must be UTF-8"))?;
    let previous = std::fs::read_to_string(args.next().context("exact before layout")?)?;
    let revised = std::fs::read_to_string(args.next().context("revised layout")?)?;
    let dossier = std::fs::read_to_string(args.next().context("native route dossier")?)?;
    let art_output = PathBuf::from(args.next().context("fresh art-stage pack")?);
    let output = PathBuf::from(args.next().context("fresh terrain pack")?);
    ensure!(
        args.next().is_none() && art_output != output && !art_output.exists() && !output.exists(),
        "two fresh distinct outputs required"
    );
    validate_corner_recipe(&previous)?;
    validate_corner_recipe(&revised)?;
    let hash = || -> Result<String> { Ok(format!("{:x}", Sha256::digest(std::fs::read(&input)?))) };
    ensure!(hash()? == expected, "R10 parent SHA256 differs");
    let base = read_verified_compiled_game_pack(&input)?;
    let art = add_r10_carrier_corner_art(&base)?;
    art.write_preserving_storage(&art_output)?;
    let art = read_verified_compiled_game_pack(&art_output)
        .context("written native R10 inward-corner readback")?;
    let terrain = revise_authored_route_terrain(&art, &previous, &revised, &dossier)?;
    ensure!(hash()? == expected, "R10 parent changed during publication");
    terrain.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("written R10 carrier ridge readback")?;
    println!("{}", output.display());
    Ok(())
}
