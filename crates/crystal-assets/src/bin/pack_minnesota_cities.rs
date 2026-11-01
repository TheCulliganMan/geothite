//! Publish three authored cities around their already installed canonical gyms.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_bemidji_city_modpack, build_brainerd_city_modpack, build_duluth_city_modpack,
    read_verified_compiled_game_pack,
};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if paths.first().and_then(|path| path.to_str()) == Some("--city") {
        ensure!(
            paths.len() == 7,
            "usage: pack_minnesota_cities --city CITY BASE GYMS TOWN LAYOUT OUTPUT"
        );
        let city = paths[1].to_str().context("city must be UTF-8")?;
        return publish_city(city, &paths[2..]);
    }
    ensure!(
        paths.len() == 9,
        "usage: pack_minnesota_cities BASE GYMS DULUTH_TOWN DULUTH_LAYOUT BRAINERD_TOWN BRAINERD_LAYOUT BEMIDJI_TOWN BEMIDJI_LAYOUT OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[8].canonicalize().unwrap_or_else(|_| paths[8].clone()),
        "city output must differ from the completed base"
    );
    // Capture all source inputs before construction. An editor changing a file
    // later cannot mix a new gym program with an older city reference mid-build.
    let inputs = paths[1..8]
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .with_context(|| format!("read authored city input {}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let duluth = build_duluth_city_modpack(&base, &inputs[1], &inputs[2], &inputs[0])
        .context("publish native Duluth around canonical Gym8")?;
    let brainerd = build_brainerd_city_modpack(&duluth, &inputs[3], &inputs[4], &inputs[0])
        .context("publish native Brainerd around canonical Gym6")?;
    let bemidji = build_bemidji_city_modpack(&brainerd, &inputs[5], &inputs[6], &inputs[0])
        .context("publish native Bemidji around canonical Gym7")?;
    bemidji.write_preserving_storage(&paths[8])?;
    read_verified_compiled_game_pack(&paths[8])
        .context("strictly verify the written three-city pack")?;
    println!(
        "published native Duluth, Brainerd and Bemidji: {}",
        paths[8].display()
    );
    Ok(())
}

fn publish_city(city: &str, paths: &[PathBuf]) -> Result<()> {
    ensure!(
        paths[0].canonicalize()? != paths[4].canonicalize().unwrap_or_else(|_| paths[4].clone()),
        "city output must differ from the completed base"
    );
    let source = paths[1..4]
        .iter()
        .map(std::fs::read_to_string)
        .collect::<std::io::Result<Vec<_>>>()
        .context("read canonical gym and city sources")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let pack = match city {
        "bemidji" => build_bemidji_city_modpack(&base, &source[1], &source[2], &source[0])?,
        "brainerd" => build_brainerd_city_modpack(&base, &source[1], &source[2], &source[0])?,
        "duluth" => build_duluth_city_modpack(&base, &source[1], &source[2], &source[0])?,
        _ => anyhow::bail!("city must be bemidji, brainerd or duluth"),
    };
    pack.write_preserving_storage(&paths[4])?;
    read_verified_compiled_game_pack(&paths[4]).context("verify written city pack")?;
    println!("published native {city}: {}", paths[4].display());
    Ok(())
}
