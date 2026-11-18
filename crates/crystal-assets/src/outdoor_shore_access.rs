//! Verify the installed dock edges with the same movement, Surf and rod functions as the game.
use anyhow::{Result, ensure};
use crystal_core::models::LearnedMove;
use crystal_core::random::ReplayDivider;
use crystal_core::state::GameState;
use crystal_core::systems::field_moves::{FieldMoveError, apply_surf_field_move};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{MovementMode, StepOptions, StepOutcome, attempt_step};
use crystal_core::world::session::OverworldSession;

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

fn step(session: &mut OverworldSession, facing: Direction) -> StepOutcome {
    attempt_step(
        &mut session.player,
        facing,
        &session.map,
        &session.tileset,
        StepOptions {
            force_step_after_turn: true,
            ..Default::default()
        },
    )
}

fn reverse(direction: Direction) -> Direction {
    match direction {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

pub(super) fn surf_partner(pack: &CompiledGamePack) -> Result<GameState> {
    let mut state = GameState::reset_wram_for_new_game();
    let mut pokemon = pack
        .data
        .create_pokemon("GILLPUP", 20, Default::default())?;
    pokemon.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: pack.data.moves["SURF"].pp,
        pp_ups: 0,
    }];
    state.storage.party.pokemon[0] = Some(pokemon);
    Ok(state)
}

pub(super) fn verify_dock_edge(
    pack: &CompiledGamePack,
    state: &GameState,
    name: &str,
    deck: TilePosition,
    facing: Direction,
) -> Result<()> {
    let room = &pack.data.maps[name];
    ensure!(
        kind(pack, room, [u16::try_from(deck.x)?, u16::try_from(deck.y)?])? == "FLOOR",
        "unwalkable casting position on {name} at {deck:?}"
    );
    let mut session = pack.data.overworld_session(name, deck, 0)?;
    let water = deck.moved(facing);
    ensure!(
        matches!(step(&mut session, facing), StepOutcome::Blocked { at, .. } if at == water)
            && session.player.tile == deck,
        "{name} lets a walker step into water at {water:?}"
    );
    let mut fisher = state.clone();
    ensure!(
        fisher
            .bag
            .add_item(pack.data.item("OLD_ROD")?, 1)
            .map_err(anyhow::Error::msg)?,
        "could not equip the Old Rod"
    );
    let rod = pack.data.field_fishing_rod(&fisher, "OLD_ROD")?;
    let cast = pack.data.cast_fishing_rod_in_session_with_divider(
        &mut fisher,
        &session,
        &rod,
        &mut ReplayDivider::new([251; 512]),
    )?;
    ensure!(
        cast.checked_water_target == water,
        "{name} rod misses water at {water:?}"
    );
    let mut swimmer = state.clone();
    let rule = &pack.data.field_moves.surf;
    ensure!(
        rule.badge.region == "johto" && rule.badge.index < swimmer.badges.johto.len(),
        "invalid Surf badge"
    );
    swimmer.badges.johto[rule.badge.index] = true;
    let launch = apply_surf_field_move(
        &pack.data.field_moves,
        &swimmer,
        &swimmer.storage,
        &session.map,
        &session.tileset,
        &mut session.player,
        0,
    )?;
    ensure!(
        launch.to_tile == water && session.player.mode == MovementMode::Surf,
        "{name} Surf missed water from {deck:?}"
    );
    ensure!(
        matches!(step(&mut session, reverse(facing)), StepOutcome::Moved { to, .. } if to == deck)
            && session.player.mode == MovementMode::Normal,
        "{name} cannot land back on the dock at {deck:?}"
    );
    Ok(())
}

pub(super) fn verify(pack: &CompiledGamePack) -> Result<()> {
    use Direction::{Down, Left, Right, Up};
    let state = surf_partner(pack)?;
    for (name, first, last, y) in [("MnR01C00R00", 12, 19, 34), ("MnR15C00R00", 34, 41, 14)] {
        let room = &pack.data.maps[name];
        let mut edges = Vec::new();
        for x in first..=last {
            edges.extend([(x, y, Up), (x, y + 1, Down)]);
        }
        edges.extend([(first, y, Left), (first, y + 1, Left)]);
        for (x, y, facing) in edges {
            // A pickup occupies one board until collected. Test permanent open water from
            // the other boards and preserve the item's original approach.
            if !room
                .objects
                .iter()
                .any(|obj| i32::from(obj.x) == i32::from(x) && i32::from(obj.y) == i32::from(y))
            {
                verify_dock_edge(pack, &state, name, TilePosition::new(x, y), facing)?;
            }
        }
    }
    for y in 13..=19 {
        verify_dock_edge(pack, &state, "MnBrainerd", TilePosition::new(47, y), Left)?;
        if y != 16 {
            verify_dock_edge(pack, &state, "MnBrainerd", TilePosition::new(48, y), Right)?;
        }
    }
    for (name, x, y, facing) in [
        ("MnBemidji", 36, 35, Up),
        ("MnBemidji", 36, 36, Down),
        ("MnBemidji", 39, 35, Right),
        ("MnDetroitLakes", 22, 28, Up),
        ("MnDetroitLakes", 22, 29, Down),
        ("MnDetroitLakes", 17, 28, Left),
        ("MnDuluth", 60, 57, Down),
        ("MnDuluth", 61, 57, Down),
        ("MnBrainerd", 47, 13, Up),
        ("MnBrainerd", 48, 13, Up),
        ("MnR05C00R01", 61, 40, Right),
        ("MnR05C00R01", 56, 40, Up),
        ("MnR05C00R01", 56, 41, Down),
        ("MnR06C00R00", 30, 42, Left),
        ("MnR06C00R00", 32, 42, Up),
        ("MnR06C00R00", 32, 43, Down),
        ("MnR07C00R00", 41, 14, Right),
        ("MnR07C00R00", 39, 14, Up),
        ("MnR07C00R00", 39, 15, Down),
        ("MnR08C00R01", 58, 23, Left),
        ("MnR08C00R01", 60, 23, Up),
        ("MnR08C00R01", 60, 24, Down),
        ("MnR16C00R00", 19, 15, Right),
        ("MnR16C00R00", 15, 15, Up),
        ("MnR16C00R00", 15, 16, Down),
        ("MnR18C00R00", 31, 22, Right),
        ("MnR18C00R00", 24, 22, Up),
        ("MnR18C00R00", 24, 23, Down),
        ("MnR18C00R00", 21, 34, Right),
        ("MnR18C00R00", 18, 34, Up),
        ("MnR18C00R00", 18, 35, Down),
        ("MnR20C00R00", 19, 25, Right),
        ("MnR20C00R00", 15, 25, Up),
        ("MnR20C00R00", 15, 26, Down),
    ] {
        verify_dock_edge(pack, &state, name, TilePosition::new(x, y), facing)?;
    }
    // Test takeoff from actual walkable land. A high shelf ends at rock or a blocked bank.
    let mut swimmer = state.clone();
    swimmer.badges.johto[pack.data.field_moves.surf.badge.index] = true;
    for (name, x, y, facing) in [
        ("MnR03C00R00", 17, 24, Right),
        ("MnR03C00R00", 26, 36, Left),
        ("MnR05C00R00", 54, 17, Right),
    ] {
        let from = TilePosition::new(x, y);
        let mut session = pack.data.overworld_session(name, from, 0)?;
        session.player.facing = facing;
        let result = apply_surf_field_move(
            &pack.data.field_moves,
            &swimmer,
            &swimmer.storage,
            &session.map,
            &session.tileset,
            &mut session.player,
            0,
        );
        ensure!(
            matches!(
                result,
                Err(FieldMoveError::BlockedTarget { .. } | FieldMoveError::TargetNotWater { .. })
            ) && session.player.tile == from
                && session.player.mode == MovementMode::Normal,
            "{name} allows launching over the cliff at {from:?}"
        );
    }
    for (name, x, y, facing, lands) in [
        ("MnR03C00R00", 14, 7, Down, true),
        ("MnR03C00R00", 27, 43, Right, true),
        ("MnR05C00R00", 56, 17, Left, false),
        ("MnR10C00R00", 46, 30, Left, false),
        ("MnR10C00R00", 46, 31, Left, false),
        ("MnR10C00R00", 44, 32, Up, false),
        ("MnR10C00R00", 45, 32, Up, false),
        ("MnR10C00R00", 48, 30, Up, true),
        ("MnR11C00R02", 24, 24, Left, false),
        ("MnR11C00R02", 26, 22, Up, false),
        ("MnR11C00R02", 27, 22, Up, false),
        ("MnR11C00R02", 28, 22, Up, false),
        ("MnR11C00R02", 25, 24, Right, true),
        ("MnR11C00R03", 18, 12, Left, false),
        ("MnR11C00R03", 18, 12, Up, false),
        ("MnR11C00R03", 19, 12, Up, false),
        ("MnR11C00R03", 20, 12, Up, false),
        ("MnR11C00R03", 21, 12, Up, false),
        ("MnR11C00R03", 23, 12, Right, true),
        ("MnR15C00R00", 26, 18, Left, false),
        ("MnR15C00R00", 26, 20, Left, true),
    ] {
        let water = TilePosition::new(x, y);
        let mut session = pack.data.overworld_session(name, water, 0)?;
        session.player.mode = MovementMode::Surf;
        let shore = water.moved(facing);
        let result = step(&mut session, facing);
        if lands {
            ensure!(
                matches!(result, StepOutcome::Moved {to, ..} if to == shore)
                    && session.player.mode == MovementMode::Normal,
                "{name} lost low landing at {shore:?}"
            );
        } else {
            ensure!(
                matches!(result, StepOutcome::Blocked { .. })
                    && session.player.tile == water
                    && session.player.mode == MovementMode::Surf,
                "{name} climbable water wall at {shore:?}"
            );
        }
    }
    Ok(())
}
