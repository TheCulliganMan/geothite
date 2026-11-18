//! Fixed, terrain-aware sight battles on the numbered Minnesota roads.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::TilePosition;
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::authored_town_publication;
use crate::{CompiledGamePack, MapModule};

type Tile = [u16; 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Road {
    entries: Vec<Tile>,
    cut: Option<CutGate>,
    trainers: BTreeMap<String, Sightline>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CutGate {
    tile: Tile,
    trainer: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sightline {
    facing: String,
    range: u16,
}

fn ground(kind: &str) -> bool {
    matches!(kind, "FLOOR" | "TALL_GRASS")
}

fn offset([x, y]: Tile, dx: i16, dy: i16) -> Option<Tile> {
    Some([x.checked_add_signed(dx)?, y.checked_add_signed(dy)?])
}

fn reachable(road: &BTreeSet<Tile>, entries: &[Tile], blocked: &[Tile]) -> BTreeSet<Tile> {
    let mut reached = BTreeSet::new();
    let mut todo = VecDeque::from_iter(entries.iter().copied());
    while let Some(tile) = todo.pop_front() {
        if !road.contains(&tile) || blocked.contains(&tile) || !reached.insert(tile) {
            continue;
        }
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            if let Some(next) = offset(tile, dx, dy) {
                todo.push_back(next);
            }
        }
    }
    reached
}

fn check_entry(pack: &CompiledGamePack, room: &MapModule, tile: Tile) -> Result<()> {
    let [x, y] = tile;
    let entry = TilePosition::new(i16::try_from(x)?, i16::try_from(y)?);
    for edge in &room.attributes.connections {
        let on_edge = match edge.direction.as_str() {
            "north" => y == 0,
            "south" => y + 1 == room.attributes.height * 2,
            "west" => x == 0,
            "east" => x + 1 == room.attributes.width * 2,
            _ => false,
        };
        if on_edge {
            let neighbor = pack
                .data
                .maps
                .get(&edge.target_map)
                .context("road neighbor")?;
            let adjacent = crate::connection_destination_tile(
                entry,
                &edge.direction,
                edge.offset,
                &neighbor.attributes,
            )?;
            let at = [u16::try_from(adjacent.x)?, u16::try_from(adjacent.y)?];
            ensure!(
                ground(kind(pack, neighbor, at)?)
                    && !neighbor.objects.iter().any(|a| [a.x, a.y] == at),
                "blocked neighboring edge"
            );
            return Ok(());
        }
    }
    anyhow::bail!("{} has no connected edge at {tile:?}", room.id)
}

fn install_road(pack: &CompiledGamePack, room: &mut MapModule, design: &Road) -> Result<()> {
    let actors: BTreeSet<_> = room.objects.iter().map(|a| [a.x, a.y]).collect();
    let mut road = BTreeSet::new();
    for y in 0..room.attributes.height * 2 {
        for x in 0..room.attributes.width * 2 {
            let tile = [x, y];
            if ground(kind(pack, room, tile)?) && !actors.contains(&tile) {
                road.insert(tile);
            }
        }
    }
    for &entry in &design.entries {
        ensure!(road.contains(&entry), "blocked road entry at {entry:?}");
        check_entry(pack, room, entry)?;
    }
    let before_cut = reachable(&road, &design.entries, &[]);
    let mut opened = road.clone();
    // The three secluded challenges begin on the far side of existing Cut gates.
    // Change only the geometry proof. The playable tree and its flags stay native.
    if let Some(gate) = &design.cut {
        let [x, y] = gate.tile;
        ensure!(
            design.trainers.contains_key(&gate.trainer)
                && kind(pack, room, gate.tile)? == "CUT_TREE"
                && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .into_iter()
                    .filter_map(|(dx, dy)| offset(gate.tile, dx, dy))
                    .any(|p| before_cut.contains(&p)),
            "unreachable Cut tree at {:?}",
            gate.tile
        );
        let block = room.blocks
            [usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2)];
        let tileset = &room.attributes.tileset_name;
        let cut = pack
            .data
            .field_moves
            .cut
            .replacements
            .get(tileset)
            .and_then(|rules| rules.get(&block))
            .context("native Cut replacement")?;
        let collision =
            &pack.data.tilesets[tileset].collision[&format!("{:02x}", cut.replacement_block_id)];
        ensure!(
            cut.variant == "tree" && ground(&collision[usize::from(y % 2 * 2 + x % 2)]),
            "Cut does not open the gate"
        );
        opened.insert(gate.tile);
    }

    let mut reserved: BTreeSet<_> = design.entries.iter().copied().collect();
    reserved.extend(room.events.bg_events.iter().map(|e| [e.x, e.y]));
    reserved.extend(room.events.coord_events.iter().map(|e| [e.x, e.y]));
    reserved.extend(room.events.warps.iter().map(|e| [e.x, e.y]));

    for actor in room
        .objects
        .iter_mut()
        .filter(|a| a.object_type == "OBJECTTYPE_TRAINER")
    {
        let id = &actor.script;
        let line = design
            .trainers
            .get(id)
            .with_context(|| format!("missing lane for {id}"))?;
        let (dx, dy) = match line.facing.as_str() {
            "UP" => (0, -1),
            "DOWN" => (0, 1),
            "LEFT" => (-1, 0),
            "RIGHT" => (1, 0),
            _ => anyhow::bail!("unknown facing for {id}"),
        };
        ensure!(
            actor.object_identifier.as_ref() == Some(id)
                && actor.sightline_direction_override.is_none()
                && actor.spritemovedata.starts_with("SPRITEMOVEDATA_STANDING_")
                && actor.sprite_has_facings
                && actor.radius > 0
                && (1..=4).contains(&line.range)
                && room.trainer_scripts.contains_key(id),
            "expected ordinary standing road trainer {id}"
        );
        let gated = design.cut.as_ref().is_some_and(|cut| cut.trainer == *id);
        let mut lane = Vec::new();
        let mut at = [actor.x, actor.y];
        for _ in 0..line.range {
            at = offset(at, dx, dy).context("sight extends outside road")?;
            ensure!(
                road.contains(&at) && reserved.insert(at) && before_cut.contains(&at) != gated,
                "{id} cannot cover {at:?} from its road or Cut pocket"
            );
            lane.push(at);
        }
        // Require a lateral entry that can be reached without first crossing this
        // trainer's ray. A stranded ray inside an enclosed island cannot pass.
        let approach = reachable(if gated { &opened } else { &road }, &design.entries, &lane);
        ensure!(
            lane.iter().any(|&at| [(-dy, dx), (dy, -dx)]
                .into_iter()
                .filter_map(|(a, b)| offset(at, a, b))
                .any(|p| approach.contains(&p))),
            "no first crossing for {id}"
        );

        let row = room
            .map_event_section_commands
            .iter_mut()
            .find(|r| r.command == "object_event" && r.args.get(11) == Some(id))
            .context("road trainer object_event")?;
        ensure!(
            row.args.get(3) == Some(&actor.spritemovedata)
                && row.args.get(10).and_then(|s| s.parse::<u16>().ok()) == Some(actor.radius),
            "compiled object and event disagree for {id}"
        );
        actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{}", line.facing);
        actor.sightline_direction_override = Some(line.facing.clone());
        actor.radius = line.range;
        row.args[3].clone_from(&actor.spritemovedata);
        row.args[10] = line.range.to_string();
    }
    ensure!(
        design.trainers.len()
            == room
                .objects
                .iter()
                .filter(|a| a.object_type == "OBJECTTYPE_TRAINER")
                .count(),
        "unexpected extra trainer on {}",
        room.id
    );
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack, directory: &Path) -> Result<()> {
    let roads: BTreeMap<String, Road> =
        serde_json::from_slice(&fs::read(directory.join("trainers.json"))?)?;
    let mut total = 0;
    for (name, design) in roads {
        let numbered = name.as_bytes();
        ensure!(
            numbered.len() == 11
                && numbered.starts_with(b"MnR")
                && numbered[5] == b'C'
                && numbered[8] == b'R'
                && [3, 4, 6, 7, 9, 10]
                    .iter()
                    .all(|&i| numbered[i].is_ascii_digit()),
            "expected a numbered road chunk, found {name}"
        );
        let mut room = pack.data.maps.get(&name).context("numbered road")?.clone();
        install_road(pack, &mut room, &design)
            .with_context(|| format!("trainer lanes on {name}"))?;
        total += design.trainers.len();
        authored_town_publication::replace(pack, room)?;
    }
    ensure!(total == 125, "expected all 125 numbered-road trainers");
    Ok(())
}
