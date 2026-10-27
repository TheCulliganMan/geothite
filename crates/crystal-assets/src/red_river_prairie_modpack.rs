//! Compile the reviewed Red River Prairie through the native route publisher.

use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

type Tile = [u16; 2];

/// Compiles the two Red River Prairie maps from explicit terrain and events.
///
/// Retains the supplied maps, runtime files, original art, audio, HOME and
/// recovery spawns. The two necessary protected crossings and complete water
/// boundary are checked against the resulting native collision, in addition to
/// the shared grass, trainer-choice, ledge and event validation. This does not
/// bind the Moorhead/Detroit Lakes hubs or complete the earned SQ02 journey.
///
/// # Errors
/// Returns an error for invalid authoring, missing native sources, occupied
/// identifiers, changed traversal contracts or failed native pack validation.
pub fn build_red_river_prairie_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    let pack = crate::authored_route_modpack::build_authored_route(
        base,
        layout_json,
        dossier_json,
        "MN_R16",
    )?;
    validate_crossings_and_boundary(&pack)?;
    Ok(pack)
}

fn collision(pack: &CompiledGamePack, [x, y]: Tile) -> Result<&str> {
    ensure!(x < 96 && y < 48, "Red River Prairie anchor outside route");
    let name = format!("MnR16C{:02}R00", x / 64);
    let module = pack.data.maps.get(&name).context("Red River Prairie map")?;
    let index = usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x % 64 / 2);
    let block = module.blocks.get(index).context("clipped prairie block")?;
    let quadrants = pack
        .data
        .tilesets
        .get(&module.attributes.tileset_name)
        .context("native prairie tileset")?
        .collision
        .get(&format!("{block:02x}"))
        .context("native prairie collision")?;
    quadrants
        .get(usize::from(y % 2) * 2 + usize::from(x % 2))
        .map(String::as_str)
        .context("native prairie quadrant")
}

fn reachable(start: Tile, allowed: &BTreeSet<Tile>) -> BTreeSet<Tile> {
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        for tile in [
            [x.saturating_sub(1), y],
            [x + 1, y],
            [x, y.saturating_sub(1)],
            [x, y + 1],
        ] {
            if allowed.contains(&tile) && seen.insert(tile) {
                queue.push_back(tile);
            }
        }
    }
    seen
}

fn validate_crossings_and_boundary(pack: &CompiledGamePack) -> Result<()> {
    let mut occupied = BTreeSet::new();
    for (column, width) in [(0, 32), (1, 16)] {
        let name = format!("MnR16C{column:02}R00");
        let module = pack.data.maps.get(&name).context("published prairie map")?;
        ensure!(
            module.attributes.width == width
                && module.attributes.height == 24
                && module.attributes.tileset_name == "mn_headwaters",
            "Red River Prairie clipped map dimensions or art changed"
        );
        for object in &module.objects {
            occupied.insert([object.x + column * 64, object.y]);
        }
    }
    let expected_ports = BTreeSet::from([[0, 24], [0, 25], [95, 24], [95, 25]]);
    let mut ports = BTreeSet::new();
    let mut walking = BTreeSet::new();
    for y in 0..48 {
        for x in 0..96 {
            let tile = [x, y];
            let kind = collision(pack, tile)?;
            ensure!(
                kind != "CUT_TREE",
                "Red River Prairie has incidental Cut canopy"
            );
            if matches!(kind, "FLOOR" | "TALL_GRASS" | "LADDER") {
                if !occupied.contains(&tile) {
                    walking.insert(tile);
                }
                if x == 0 || x == 95 || y == 0 || y == 47 {
                    ports.insert(tile);
                }
            }
            if x == 0 || x == 95 || y == 0 || y == 47 {
                ensure!(
                    !matches!(kind, "WATER" | "WATERFALL" | "WHIRLPOOL"),
                    "Red River Prairie has an unconnected outer Surf edge at {tile:?}"
                );
            }
        }
    }
    ensure!(
        ports == expected_ports,
        "Red River Prairie exterior walking ports changed"
    );
    let start = [92, 24];
    let end = [2, 24];
    ensure!(
        walking.contains(&start) && reachable(start, &walking).contains(&end),
        "Red River Prairie free walking road is disconnected"
    );
    for (name, x0, x1, north_y) in [
        ("Red River bridge", 10, 28, 22),
        ("drainage bridge", 60, 68, 28),
    ] {
        let mut without_crossing = walking.clone();
        for x in x0..x1 {
            for (offset, expected) in [(0, "WALL"), (1, "FLOOR"), (2, "FLOOR"), (3, "WALL")] {
                let tile = [x, north_y + offset];
                ensure!(
                    collision(pack, tile)? == expected,
                    "{name}: native rail/deck changed"
                );
                without_crossing.remove(&tile);
            }
        }
        ensure!(
            !reachable(start, &without_crossing).contains(&end),
            "{name} is no longer a necessary walking crossing"
        );
    }
    for y in 14..18 {
        ensure!(
            collision(pack, [19, y])? == "WALL",
            "river pier has an open end"
        );
    }
    ensure!(
        collision(pack, [18, 14])? == "WALL" && collision(pack, [18, 17])? == "WALL",
        "river pier has an open corner"
    );
    Ok(())
}
