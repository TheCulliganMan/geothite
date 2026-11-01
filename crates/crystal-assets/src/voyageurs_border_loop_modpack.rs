//! Publish the free Voyageurs border route without replacing its source world.

use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;

type Tile = [u16; 2];

#[derive(Deserialize)]
struct BorderQuest {
    quest: String,
}

/// Compiles four clipped Voyageurs maps and their two-mouth granite cave.
///
/// Preserves existing maps, runtime files, source art, audio, HOME and recovery
/// spawns. Both protected outlets remain necessary walking crossings; native
/// collision closes unconnected Surf edges and rejects decorative Cut canopy.
/// SQ03 reporting and its once-only PP Up remain with the Falls authority.
/// This installs no admission, expedition counters or regional hub dispatcher.
///
/// # Errors
/// Returns an error for invalid authoring, changed SQ03/traversal contracts,
/// missing sources, occupied identifiers or failed native pack validation.
pub fn build_voyageurs_border_loop_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    let contract: BorderQuest = serde_json::from_str(dossier_json)?;
    ensure!(
        contract.quest == "SQ03",
        "Voyageurs border circuit must remain SQ03"
    );
    let pack = crate::authored_route_modpack::build_authored_route(
        base,
        layout_json,
        dossier_json,
        "MN_R20",
    )?;
    validate_border_route(&pack)?;
    Ok(pack)
}

fn collision(pack: &CompiledGamePack, [x, y]: Tile) -> Result<&str> {
    ensure!(x < 128 && y < 64, "Voyageurs anchor outside route");
    let name = format!("MnR20C{:02}R{:02}", x / 64, y / 48);
    let module = pack.data.maps.get(&name).context("Voyageurs native map")?;
    let index =
        usize::from(y % 48 / 2) * usize::from(module.attributes.width) + usize::from(x % 64 / 2);
    let block = module
        .blocks
        .get(index)
        .context("clipped Voyageurs block")?;
    pack.data
        .tilesets
        .get(&module.attributes.tileset_name)
        .context("native Voyageurs art")?
        .collision
        .get(&format!("{block:02x}"))
        .and_then(|quadrants| quadrants.get(usize::from(y % 2) * 2 + usize::from(x % 2)))
        .map(String::as_str)
        .context("native Voyageurs collision quadrant")
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

fn validate_border_route(pack: &CompiledGamePack) -> Result<()> {
    let mut occupied = BTreeSet::new();
    for row in 0..2 {
        for column in 0..2 {
            let name = format!("MnR20C{column:02}R{row:02}");
            let module = pack.data.maps.get(&name).context("published border map")?;
            let expected_height = if row == 0 { 24 } else { 8 };
            ensure!(
                module.attributes.width == 32
                    && module.attributes.height == expected_height
                    && module.attributes.tileset_name == "mn_headwaters",
                "Voyageurs clipped map dimensions or native rail art changed"
            );
            for object in &module.objects {
                occupied.insert([object.x + column * 64, object.y + row * 48]);
            }
        }
    }
    for name in [
        "MnR20C00R00",
        "MnR20C01R00",
        "MnR20C00R01",
        "MnR20C01R01",
        "MnR20Cave1F",
    ] {
        let module = pack.data.maps.get(name).context("border authority map")?;
        ensure!(
            module.script_shop_commands.is_empty() && module.script_economy_commands.is_empty(),
            "free Voyageurs travel may not install admission or purchase commands"
        );
        ensure!(
            !module.script_flag_commands.iter().any(|command| {
                command.command == "setevent"
                    && matches!(
                        command.flag_id.as_str(),
                        "EVENT_MN_BORDER_REPORT" | "EVENT_MN_BORDER_CIRCUIT_REWARDED"
                    )
            }) && !module
                .script_item_grants
                .iter()
                .any(|grant| grant.item_id == "PP_UP")
                && !module
                    .script_field_pickups
                    .iter()
                    .any(|pickup| pickup.item_id.as_deref() == Some("PP_UP")),
            "Voyageurs route may not replace the Falls circuit reporting authority"
        );
    }
    let mut walking = BTreeSet::new();
    let mut ports = BTreeSet::new();
    for y in 0..64 {
        for x in 0..128 {
            let tile = [x, y];
            let kind = collision(pack, tile)?;
            ensure!(kind != "CUT_TREE", "Voyageurs has incidental Cut canopy");
            if matches!(kind, "FLOOR" | "TALL_GRASS" | "LADDER") {
                if !occupied.contains(&tile) {
                    walking.insert(tile);
                }
                if x == 0 || x == 127 || y == 0 || y == 63 {
                    ports.insert(tile);
                }
            }
            if x == 0 || x == 127 || y == 0 || y == 63 {
                ensure!(
                    !matches!(kind, "WATER" | "WATERFALL" | "WHIRLPOOL"),
                    "Voyageurs has an unconnected outer Surf edge at {tile:?}"
                );
            }
        }
    }
    ensure!(
        ports == BTreeSet::from([[0, 44], [0, 45], [127, 36], [127, 37]]),
        "Voyageurs exterior walking ports changed"
    );
    let start = [2, 44];
    let end = [124, 36];
    ensure!(
        walking.contains(&start) && reachable(start, &walking).contains(&end),
        "free Voyageurs walking route is disconnected"
    );
    for (name, x0, x1, north_y) in [
        ("western outlet bridge", 38, 50, 40),
        ("eastern outlet bridge", 90, 100, 34),
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
    for y in 24..28 {
        ensure!(
            collision(pack, [19, y])? == "WALL",
            "Voyageurs lake pier has an open end"
        );
    }
    ensure!(
        collision(pack, [18, 24])? == "WALL" && collision(pack, [18, 27])? == "WALL",
        "Voyageurs lake pier has an open corner"
    );
    Ok(())
}
