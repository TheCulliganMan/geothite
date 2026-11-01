//! Update the website's location guide from the actual installed regional pack.
use std::path::Path;

use anyhow::{Result, ensure};

#[path = "../src/bin/export_minnesota_pokedex_web/locations.rs"]
mod locations;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3,
        "usage: publish_pokedex_locations PACK BASELINE_RELEASE NEW_RELEASE"
    );
    let pack = crystal_assets::read_verified_compiled_game_pack(Path::new(&args[0]))?;
    locations::publish(&pack, Path::new(&args[1]), Path::new(&args[2]))
}
