//! Read-only compiled movement-geometry audit; never starts a game session.
use anyhow::{Context, Result, ensure};
use crystal_assets::CompiledGamePack;
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, VecDeque};

fn distances(pack: &CompiledGamePack, start: (i16, i16)) -> Result<BTreeMap<(i16, i16), usize>> {
    let m = &pack.data().maps["MnR09Cave1F"];
    let map = OverworldMapData::from_attributes(&m.id, &m.attributes, m.blocks.clone());
    let collision = pack.data().tileset_collision("cave")?;
    let occupied = m
        .objects
        .iter()
        .map(|o| OccupiedTile {
            tile: TilePosition::new(o.x as i16, o.y as i16),
            object_identifier: o.object_identifier.clone(),
        })
        .collect::<Vec<_>>();
    let mut seen = BTreeMap::from([(start, 0)]);
    let mut todo = VecDeque::from([start]);
    while let Some(at) = todo.pop_front() {
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let mut player = PlayerMovementState::new(TilePosition::new(at.0, at.1));
            let options = StepOptions {
                force_step_after_turn: true,
                ..StepOptions::default()
            };
            let target = match attempt_ledge_jump_with_occupied_tiles(
                &mut player,
                direction,
                &map,
                &collision,
                options,
                &occupied,
            ) {
                LedgeJumpOutcome::Jumped { to, .. } => Some(to),
                _ => match attempt_step_with_occupied_tiles(
                    &mut player,
                    direction,
                    &map,
                    &collision,
                    options,
                    &occupied,
                ) {
                    StepOutcome::Moved { to, .. } => Some(to),
                    _ => None,
                },
            };
            if let Some(to) = target {
                let next = (to.x, to.y);
                // Warp apertures are terminal destinations, never walking shortcuts.
                if m.events
                    .warps
                    .iter()
                    .any(|w| (w.x as i16, w.y as i16) == next)
                {
                    continue;
                }
                let distance = seen[&at] + 1;
                if let Entry::Vacant(entry) = seen.entry(next) {
                    entry.insert(distance);
                    todo.push_back(next);
                }
            }
        }
    }
    Ok(seen)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 2, "usage: mine_return_geometry BEFORE AFTER");
    let before = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let after = crystal_assets::read_verified_compiled_game_pack(&args[1])?;
    let old = distances(&before, (19, 8))?;
    let new = distances(&after, (19, 8))?;
    let exit = (13, 18);
    let a = old.get(&exit).context("old exit")?;
    let b = new.get(&exit).context("new exit")?;
    ensure!(b < a, "return drift must shorten Nell-to-mouth walk");
    let reachable = distances(&after, exit)?;
    ensure!(
        old.keys().all(|p| reachable.contains_key(p)),
        "retained mine destinations disconnected"
    );
    for at in reachable.keys() {
        ensure!(
            distances(&after, *at)?.contains_key(&exit),
            "trapped mine cell {at:?}"
        );
    }
    let m = &after.data().maps["MnR09Cave1F"];
    let map = OverworldMapData::from_attributes(&m.id, &m.attributes, m.blocks.clone());
    let collision = after.data().tileset_collision("cave")?;
    for x in [16, 17] {
        let mut player = PlayerMovementState::new(TilePosition::new(x, 12));
        ensure!(
            matches!(attempt_ledge_jump_with_occupied_tiles(&mut player,Direction::Down,&map,&collision,StepOptions::default(),&[]),LedgeJumpOutcome::Jumped{to,..} if to==TilePosition::new(x,14)),
            "drop geometry"
        );
        ensure!(
            matches!(
                attempt_step_with_occupied_tiles(
                    &mut player,
                    Direction::Up,
                    &map,
                    &collision,
                    StepOptions {
                        force_step_after_turn: true,
                        ..StepOptions::default()
                    },
                    &[]
                ),
                StepOutcome::Blocked { .. }
            ),
            "reverse drop"
        );
    }
    println!(
        "Nell approach to mouth arrival: {a} -> {b} edges; {} reachable cells all return; both native drops and blocked reverses pass; no field moves enabled.",
        reachable.len()
    );
    Ok(())
}
