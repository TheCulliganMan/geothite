//! Native stream banks, a healer's circuit and an optional cross-cleft waterway.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;
use crate::northstar_cave_geometry::resting_tiles;
use crate::northstar_regional_design::apply_rooms;

const ROOMS: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/wayfinding-v01/small-water-caves.json");

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    install_banks(pack)?;
    apply_rooms(pack, ROOMS)?;
    check_castorock_gate(pack)?;
    check_shelter_and_cleft(pack)
}

// Retain the 16px strip of standing room on an existing bank while extending
// water into the other half. Each quadrant keeps its original art and collision.
fn install_banks(pack: &mut CompiledGamePack) -> Result<()> {
    let layout = pack
        .runtime_files
        .get_mut("data/tilesets/cave_metatiles.bin")
        .context("native cave art")?;
    let tileset = pack.data.tilesets.get_mut("cave").context("cave tileset")?;
    for (id, sources) in [
        (0x41, [0x02, 0x3d, 0x02, 0x0f]),
        (0x42, [0x3e, 0x02, 0x0f, 0x02]),
    ] {
        let key = format!("{id:02x}");
        ensure!(
            layout.len() == id * 16 && !tileset.collision.contains_key(&key),
            "occupied cave bank art {key}"
        );
        let mut glyphs = [0; 16];
        let mut collisions = Vec::with_capacity(4);
        for (q, source) in sources.into_iter().enumerate() {
            let source_key = format!("{source:02x}");
            let terrain = tileset
                .collision
                .get(&source_key)
                .context("original bank collision")?;
            collisions.push(terrain.get(q).context("native bank quadrant")?.clone());
            let corner = (q / 2) * 8 + (q % 2) * 2;
            for i in [corner, corner + 1, corner + 4, corner + 5] {
                glyphs[i] = layout[source * 16 + i];
            }
        }
        layout.extend(glyphs);
        tileset.collision.insert(key, collisions);
    }
    pack.runtime_files.insert(
        "data/tilesets/cave.json".into(),
        serde_json::to_vec(&tileset.collision)?,
    );
    Ok(())
}

fn check_castorock_gate(pack: &CompiledGamePack) -> Result<()> {
    let cave = &pack.data.maps["MnR03Cave1F"];
    let arrival = [7, 8];
    let pool_clue = [3, 5];
    let dry = resting_tiles(pack, cave, arrival, false)?;
    let water = resting_tiles(pack, cave, arrival, true)?;
    ensure!(
        dry.contains(&pool_clue)
            && dry.contains(&[5, 10])
            && kind(pack, cave, [6, 10])? == "WALL"
            && !dry.contains(&[4, 11])
            && water.contains(&[4, 11])
            && !water.contains(&[19, 4])
            && !water.contains(&[21, 10]),
        "stream bank must offer a dry discovery without bypassing the Strength boulder"
    );

    let mut pushed = cave.clone();
    let boulder = pushed
        .objects
        .iter_mut()
        .find(|o| o.script == "MN_R03_JAW_BOULDER")
        .context("Castorock Strength boulder")?;
    ensure!(
        [boulder.x, boulder.y] == [13, 6],
        "unmoved Strength boulder"
    );
    boulder.x = 15;
    let east = resting_tiles(pack, &pushed, [14, 6], false)?;
    ensure!(
        [arrival, [19, 4], [23, 12], [21, 10]]
            .iter()
            .all(|p| east.contains(p)),
        "pushed gate must retain both observations, creature approach and return"
    );
    Ok(())
}

fn check_shelter_and_cleft(pack: &CompiledGamePack) -> Result<()> {
    let shelter = &pack.data.maps["MnR13Cave1F"];
    let dry = resting_tiles(pack, shelter, [7, 14], false)?;
    ensure!(
        [[9, 7], [13, 6], [15, 8], [14, 11], [5, 4]]
            .iter()
            .all(|p| dry.contains(p))
            && kind(pack, shelter, [8, 8])? == "WATER"
            && kind(pack, shelter, [9, 8])? == "FLOOR",
        "dry bank, healer, cache and supply circuit must connect"
    );

    let cleft = &pack.data.maps["MnR20Cave1F"];
    let dry = resting_tiles(pack, cleft, [5, 20], false)?;
    let water = resting_tiles(pack, cleft, [8, 17], true)?;
    ensure!(
        dry.contains(&[23, 18])
            && dry.contains(&[8, 17])
            && [[10, 19], [12, 19], [11, 18], [11, 20]]
                .iter()
                .all(|p| !dry.contains(p))
            && [[9, 17], [12, 19], [16, 15], [23, 18]]
                .iter()
                .all(|p| water.contains(p))
            && kind(pack, cleft, [9, 17])? == "WATER",
        "both dry mouths and the Surf-only seep cache must keep their approaches"
    );
    Ok(())
}
