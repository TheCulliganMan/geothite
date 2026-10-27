//! Core-tile checks for the expanded preserve's stone climbs and short drops.
use super::*;

pub(super) fn validate(pack: &CompiledGamePack, blocks: &[u16], anchors: &Value) -> Result<()> {
    let kind = |tile| collision(pack, blocks, 64, tile, "mn_boundary_safari");
    let mut ground = BTreeSet::new();
    for y in 0..112u16 {
        for x in 0..128u16 {
            if matches!(kind([x, y])?, "FLOOR" | "TALL_GRASS") {
                ground.insert([x, y]);
            }
        }
    }
    let door = point(&anchors["root_beer_lady"]["door"]["tile"])?;
    let passive = point(&anchors["root_beer_lady"]["door"]["return_tile"])?;
    ensure!(
        kind(door)? == "DOOR"
            && kind(passive)? == "FLOOR"
            && door[0] == passive[0]
            && door[1] + 1 == passive[1],
        "cabin needs real door and safe return"
    );
    let source: Source = serde_json::from_value(anchors.clone())?;
    for actor in source
        .retire_anchors
        .iter()
        .chain(&source.landmark_actors)
        .chain(&source.decorations)
    {
        ground.remove(&actor.tile);
    }
    let ledges = anchors["one_way_ledges"]
        .as_array()
        .context("expanded short ledges")?;
    ensure!(ledges.len() == 5, "retain five named short cliff returns");
    for ledge in ledges {
        let from = point(&ledge["hop_from"])?;
        let above = point(&ledge["upper_approach"])?;
        let landing = point(&ledge["landing"])?;
        let reverse = point(&ledge["reverse_blocked"])?;
        ensure!(
            ledge["direction"] == "south"
                && kind(from)? == "HOP_DOWN"
                && kind(reverse)? == "WALL"
                && ground.contains(&above)
                && ground.contains(&landing)
                && above == [from[0], from[1] - 1]
                && landing == [from[0], from[1] + 2]
                && reverse == [from[0], from[1] + 1],
            "invalid directional cliff hop"
        );
        for side in ledge["side_barriers"]
            .as_array()
            .context("ledge side barriers")?
        {
            ensure!(kind(point(side)?)? == "WALL", "short ledge side bypass");
        }
        ensure!(
            reached(landing, &ground).contains(&above),
            "short ledge lacks ordinary uphill return"
        );
    }
    let stairs = blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| **block == 0x84)
        .map(|(index, _)| {
            [
                u16::try_from(index % 64).expect("bounded preserve x"),
                u16::try_from(index / 64).expect("bounded preserve y"),
            ]
        })
        .collect::<Vec<_>>();
    ensure!(
        stairs.len() == 5,
        "retain five native cave-stair rock climbs"
    );
    for [x, y] in stairs {
        let above = [x * 2 + 1, y * 2 - 1];
        let below = [x * 2 + 1, y * 2 + 2];
        ensure!(
            ground.contains(&above)
                && ground.contains(&below)
                && kind([x * 2, y * 2 + 1])? == "WALL"
                && kind([x * 2 + 1, y * 2])? == "FLOOR"
                && kind([x * 2 + 1, y * 2 + 1])? == "FLOOR",
            "stone stair must join both rock tiers through native walking quadrants"
        );
        ensure!(
            reached(above, &ground).contains(&below),
            "stone stair walking path disconnected"
        );
    }
    Ok(())
}
