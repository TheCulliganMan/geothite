//! Prove authored Cut gates using the supplied native block replacements.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_design::RouteDesign;
use crate::authored_route_modpack::collision;

pub(crate) struct CutReachability {
    pub after: BTreeSet<[u16; 2]>,
    pub after_blocks: Vec<u16>,
    pub gated_actor_ids: BTreeSet<String>,
}

pub(crate) fn validate(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
    before: &BTreeSet<[u16; 2]>,
) -> Result<Option<CutReachability>> {
    let width = design.dimensions_metatiles[0];
    let declared_trees = design
        .cut_gates
        .iter()
        .map(|gate| gate.tile)
        .chain(design.shortcut.iter().map(|shortcut| shortcut.tile))
        .collect::<BTreeSet<_>>();
    for y in 0..design.dimensions_metatiles[1] * 2 {
        for x in 0..width * 2 {
            if collision(pack, blocks, width, [x, y], tileset)? == "CUT_TREE" {
                ensure!(
                    declared_trees.contains(&[x, y]),
                    "undeclared decorative Cut tree at {x},{y}"
                );
            }
        }
    }
    if design.cut_gates.is_empty() {
        return Ok(None);
    }
    let actors = design
        .trainers
        .iter()
        .map(|a| (a.id.as_str(), a.tile))
        .chain(design.residents.iter().map(|a| (a.id.as_str(), a.tile)))
        .chain(design.items.iter().map(|a| (a.id.as_str(), a.tile)))
        .collect::<BTreeMap<_, _>>();
    let occupied = actors.values().copied().collect::<BTreeSet<_>>();
    let replacements = pack
        .data()
        .field_moves
        .cut
        .replacements
        .get(tileset)
        .context("authored Cut tileset has no inherited native replacement rules")?;
    let mut after_blocks = blocks.to_vec();
    let mut ids = BTreeSet::new();
    let mut indices = BTreeSet::new();
    let mut gated_actor_ids = BTreeSet::new();
    for gate in &design.cut_gates {
        ensure!(
            !gate.id.is_empty() && ids.insert(gate.id.as_str()),
            "duplicate or empty Cut gate identity"
        );
        ensure!(
            collision(pack, blocks, width, gate.tile, tileset)? == "CUT_TREE",
            "{}: no native Cut tree",
            gate.id
        );
        ensure!(
            before.contains(&gate.approach),
            "{}: Cut approach is inaccessible before cutting",
            gate.id
        );
        ensure!(
            gate.tile[0].abs_diff(gate.approach[0]) + gate.tile[1].abs_diff(gate.approach[1]) == 1,
            "{}: Cut approach is not adjacent",
            gate.id
        );
        ensure!(
            !gate.targets.is_empty(),
            "{}: Cut gate has no named gameplay target",
            gate.id
        );
        let index =
            usize::from(gate.tile[1] / 2) * usize::from(width) + usize::from(gate.tile[0] / 2);
        ensure!(
            indices.insert(index),
            "multiple Cut gates share one source metatile"
        );
        let source = blocks[index];
        let native = replacements
            .get(&source)
            .context("Cut gate lacks a native source block replacement")?;
        let declared = u16::from_str_radix(&gate.replacement_block, 16)
            .context("Cut replacement must be a hexadecimal source block")?;
        ensure!(
            native.variant == "tree" && native.replacement_block_id == declared,
            "{}: declared Cut replacement differs from the supplied native rule",
            gate.id
        );
        after_blocks[index] = declared;
        for id in &gate.gated_actor_ids {
            ensure!(
                actors.contains_key(id.as_str()),
                "{}: unknown gated actor {id}",
                gate.id
            );
            ensure!(
                gated_actor_ids.insert(id.clone()),
                "actor {id} belongs to multiple Cut gates"
            );
        }
    }
    let after = walkable_region(pack, design, &after_blocks, tileset, &occupied)?;
    for gate in &design.cut_gates {
        // Opening every other declared tree must not provide a free alternate
        // entrance to this gate's target or reward pocket.
        let index =
            usize::from(gate.tile[1] / 2) * usize::from(width) + usize::from(gate.tile[0] / 2);
        let mut closed_blocks = after_blocks.clone();
        closed_blocks[index] = blocks[index];
        let closed = walkable_region(pack, design, &closed_blocks, tileset, &occupied)?;
        // A reward must remain sealed after later Surf access too. Do not use
        // current NPC bodies as permanent barriers: trainer approach scenes
        // can move them away from their authored standing tile.
        let closed_surf = crate::authored_route_surface::reachable_with_surf(
            pack,
            design,
            &closed_blocks,
            tileset,
            &BTreeSet::new(),
        )?;
        for target in &gate.targets {
            ensure!(
                !closed_surf.contains(target),
                "{}: target {target:?} has a Surf bypass around Cut",
                gate.id
            );
            ensure!(
                !closed.contains(target) && after.contains(target),
                "{}: target {target:?} is bypassable or remains blocked after Cut",
                gate.id
            );
        }
        for id in &gate.gated_actor_ids {
            let tile = actors[id.as_str()];
            ensure!(
                !has_approach(tile, &closed) && has_approach(tile, &after),
                "{}: actor {id} is not actually gated by Cut",
                gate.id
            );
            ensure!(
                !has_approach(tile, &closed_surf),
                "{}: actor {id} has a Surf bypass around Cut",
                gate.id
            );
        }
    }
    Ok(Some(CutReachability {
        after,
        after_blocks,
        gated_actor_ids,
    }))
}

pub(crate) fn validate_shortcut(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
    before: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    let Some(shortcut) = &design.shortcut else {
        return Ok(());
    };
    let (Some(from), Some(to), Some(minimum)) = (
        shortcut.compare_from,
        shortcut.compare_to,
        shortcut.minimum_saved_steps,
    ) else {
        anyhow::bail!("Cut shortcut requires explicit comparison endpoints and saved distance");
    };
    ensure!(
        minimum >= 9 && before.contains(&from) && before.contains(&to),
        "Cut shortcut comparison must join existing reachable paths with a useful saving"
    );
    let width = design.dimensions_metatiles[0];
    let index =
        usize::from(shortcut.tile[1] / 2) * usize::from(width) + usize::from(shortcut.tile[0] / 2);
    let native = pack
        .data()
        .field_moves
        .cut
        .replacements
        .get(tileset)
        .and_then(|rules| rules.get(&blocks[index]))
        .context("Cut shortcut lacks its native replacement")?;
    ensure!(
        native.variant == "tree",
        "Cut shortcut is not a native tree"
    );
    let mut opened = blocks.to_vec();
    opened[index] = native.replacement_block_id;
    for surf in [false, true] {
        let distance = |terrain: &[u16]| {
            crate::authored_route_surface::shortest_path_distance(
                pack,
                design,
                terrain,
                tileset,
                &BTreeSet::new(),
                from,
                to,
                surf,
            )
        };
        let closed =
            distance(blocks)?.context("Cut shortcut has no ordinary return before cutting")?;
        let after =
            distance(&opened)?.context("Cut shortcut remains disconnected after cutting")?;
        ensure!(
            closed
                .checked_sub(after)
                .is_some_and(|saved| saved >= u32::from(minimum)),
            "Cut shortcut saving is bypassed or false (Surf={surf}, before={closed}, after={after})"
        );
    }
    for id in &shortcut.useful_target_actor_ids {
        let tile = design
            .trainers
            .iter()
            .map(|a| (&a.id, a.tile))
            .chain(design.residents.iter().map(|a| (&a.id, a.tile)))
            .chain(design.items.iter().map(|a| (&a.id, a.tile)))
            .find_map(|(actor, tile)| (actor == id).then_some(tile))
            .with_context(|| format!("Cut shortcut names unknown actor {id}"))?;
        let approaches = [
            [tile[0].saturating_sub(1), tile[1]],
            [tile[0] + 1, tile[1]],
            [tile[0], tile[1].saturating_sub(1)],
            [tile[0], tile[1] + 1],
        ]
        .into_iter()
        .filter(|point| before.contains(point))
        .collect::<Vec<_>>();
        for surf in [false, true] {
            let best = |terrain: &[u16]| -> Result<Option<u32>> {
                let mut best = None;
                for &approach in &approaches {
                    if let Some(distance) = crate::authored_route_surface::shortest_path_distance(
                        pack,
                        design,
                        terrain,
                        tileset,
                        &BTreeSet::new(),
                        from,
                        approach,
                        surf,
                    )? {
                        best = Some(best.map_or(distance, |prior: u32| prior.min(distance)));
                    }
                }
                Ok(best)
            };
            let closed = best(blocks)?
                .with_context(|| format!("Cut shortcut activity {id} has no ordinary approach"))?;
            let after = best(&opened)?
                .with_context(|| format!("Cut shortcut activity {id} remains blocked"))?;
            ensure!(
                closed
                    .checked_sub(after)
                    .is_some_and(|saved| saved >= u32::from(minimum)),
                "Cut shortcut does not improve the named activity {id} (Surf={surf}, before={closed}, after={after})"
            );
        }
    }
    Ok(())
}

fn has_approach([x, y]: [u16; 2], seen: &BTreeSet<[u16; 2]>) -> bool {
    [
        [x.saturating_sub(1), y],
        [x + 1, y],
        [x, y.saturating_sub(1)],
        [x, y + 1],
    ]
    .iter()
    .any(|p| seen.contains(p) && p[0] / 64 == x / 64 && p[1] / 48 == y / 48)
}

fn walkable_region(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<BTreeSet<[u16; 2]>> {
    let [width, height] = design.dimensions_metatiles;
    let start = design.endpoints[0].gate_metatile.map(|v| v * 2);
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        for [mut nx, mut ny] in [
            [i32::from(x) - 1, i32::from(y)],
            [i32::from(x) + 1, i32::from(y)],
            [i32::from(x), i32::from(y) - 1],
            [i32::from(x), i32::from(y) + 1],
        ] {
            let dx = nx - i32::from(x);
            let dy = ny - i32::from(y);
            // Match the ordinary route graph: ice can carry the player straight
            // to land, but does not allow a turn at a transient sliding tile.
            while (0..i32::from(width) * 2).contains(&nx)
                && (0..i32::from(height) * 2).contains(&ny)
                && collision(
                    pack,
                    blocks,
                    width,
                    [u16::try_from(nx)?, u16::try_from(ny)?],
                    tileset,
                )? == "ICE"
            {
                nx += dx;
                ny += dy;
            }
            if !(0..i32::from(width) * 2).contains(&nx) || !(0..i32::from(height) * 2).contains(&ny)
            {
                continue;
            }
            let point = [u16::try_from(nx)?, u16::try_from(ny)?];
            if seen.contains(&point) || occupied.contains(&point) {
                continue;
            }
            let kind = collision(pack, blocks, width, point, tileset)?;
            if matches!(kind, "FLOOR" | "LADDER" | "DOOR")
                || (kind == "TALL_GRASS"
                    && (!design.required_grass_crossings.is_empty()
                        || !design.battle_or_grass_choices.is_empty()))
            {
                seen.insert(point);
                queue.push_back(point);
            }
        }
    }
    Ok(seen)
}
