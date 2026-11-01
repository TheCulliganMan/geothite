//! Author a depot interception using ordinary native movement programs.
use std::collections::{BTreeMap, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::map::{CoordEvent, MapScene, MapScriptSectionCommand};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_story_revision::publish;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnStPaulUnionDepot")
        .context("current depot")?
        .clone();
    ensure!(room.scenes.scenes.is_empty(), "depot scene table occupied");
    let mut entry = vec![
        c("checkevent", &["EVENT_MN_COURIER_DEFEATED"]),
        c("iftrue", &["MnOverhaulDepotEnd"]),
        c("checkflag", &["ENGINE_PLAINBADGE"]),
        c("iffalse", &["MnOverhaulDepotEnd"]),
        c("checkevent", &["EVENT_MN_DISPATCH_NOTE"]),
        c("iffalse", &["MnOverhaulDepotEnd"]),
        c("readvar", &["VAR_XCOORD"]),
    ];
    let mut starts = room
        .events
        .warps
        .iter()
        .map(|w| [w.x, w.y])
        .collect::<Vec<_>>();
    starts.extend(
        room.events
            .warps
            .iter()
            .filter(|warp| warp.y == 23)
            .map(|warp| [warp.x, 22]),
    );
    starts.extend([[17, 10], [19, 16], [20, 15], [19, 17]]);
    starts.sort_unstable();
    starts.dedup();
    let mut by_x = BTreeMap::<u16, Vec<[u16; 2]>>::new();
    for tile in starts {
        by_x.entry(tile[0]).or_default().push(tile);
    }
    for (x, tiles) in &by_x {
        let label = format!("MnOverhaulDepotX{x}");
        entry.push(c("ifequal", &[&x.to_string(), &label]));
        let mut branch = vec![c("readvar", &["VAR_YCOORD"])];
        for tile in tiles {
            let target = format!("MnOverhaulDepotAt{}x{}", tile[0], tile[1]);
            branch.push(c("ifequal", &[&tile[1].to_string(), &target]));
            let movement = format!("{target}Walk");
            let mut steps = route(pack, &room, *tile, [19, 16])?
                .iter()
                .map(|direction| c("step", &[direction]))
                .collect::<Vec<_>>();
            steps.push(c("step_end", &[]));
            room.scripts.insert(movement.clone(), json!(steps));
            room.scripts.insert(
                target,
                json!([
                    c("applymovement", &["PLAYER", &movement]),
                    c("turnobject", &["PLAYER", "RIGHT"]),
                    c("setlasttalked", &["MN_ST_PAUL_DEPOT_COURIER"]),
                    c("faceobject", &["MN_ST_PAUL_DEPOT_COURIER", "PLAYER"]),
                    c(
                        "showemote",
                        &["EMOTE_SHOCK", "MN_ST_PAUL_DEPOT_COURIER", "15"]
                    ),
                    c("sjump", &["MN_ST_PAUL_DEPOT_COURIER"])
                ]),
            );
        }
        branch.push(c("end", &[]));
        room.scripts.insert(label, json!(branch));
    }
    entry.push(c("end", &[]));
    room.scripts
        .insert("MnOverhaulDepotEntry".into(), json!(entry));
    room.scripts
        .insert("MnOverhaulDepotEnd".into(), json!([c("end", &[])]));
    room.scenes.scenes.push(MapScene {
        scene_id: "SCENE_MN_OVERHAUL_DEPOT".into(),
        script_name: Some("MnOverhaulDepotEntry".into()),
    });
    room.map_script_section_commands.insert(
        1,
        MapScriptSectionCommand {
            command: "scene_script".into(),
            args: vec![
                "MnOverhaulDepotEntry".into(),
                "SCENE_MN_OVERHAUL_DEPOT".into(),
            ],
            command_index: 1,
        },
    );
    for (index, row) in room.map_script_section_commands.iter_mut().enumerate() {
        row.command_index = index;
    }
    for [x, y] in [[19, 16], [20, 15], [19, 17]] {
        room.events.coord_events.push(CoordEvent {
            x,
            y,
            scene_id: "-1".into(),
            script_name: "MnOverhaulDepotEntry".into(),
        });
    }
    pack.data
        .story_event_script_constants
        .maps
        .entry(room.id.clone())
        .or_default()
        .insert("SCENE_MN_OVERHAUL_DEPOT".into(), 0);
    publish(pack, room)
}

// Resolve paths once during authoring. The game executes only applymovement;
// it does not gain an independent movement or story dispatcher.
fn route(
    pack: &CompiledGamePack,
    room: &MapModule,
    start: [u16; 2],
    target: [u16; 2],
) -> Result<Vec<&'static str>> {
    let mut queue = VecDeque::from([start]);
    let mut paths = BTreeMap::from([(start, Vec::new())]);
    while let Some(tile) = queue.pop_front() {
        if tile == target {
            return paths.remove(&tile).context("authored depot route");
        }
        for (dx, dy, direction) in [
            (0, -1, "UP"),
            (-1, 0, "LEFT"),
            (1, 0, "RIGHT"),
            (0, 1, "DOWN"),
        ] {
            let (Ok(x), Ok(y)) = (
                u16::try_from(i32::from(tile[0]) + dx),
                u16::try_from(i32::from(tile[1]) + dy),
            ) else {
                continue;
            };
            let next = [x, y];
            if paths.contains_key(&next)
                || x >= room.attributes.width * 2
                || y >= room.attributes.height * 2
                || room.objects.iter().any(|o| [o.x, o.y] == next)
            {
                continue;
            }
            let collision = crate::authored_route_modpack::collision(
                pack,
                &room.blocks,
                room.attributes.width,
                next,
                &room.attributes.tileset_name,
            )?;
            if !matches!(collision, "FLOOR" | "WARP_CARPET_DOWN") {
                continue;
            }
            let mut path = paths[&tile].clone();
            path.push(direction);
            paths.insert(next, path);
            queue.push_back(next);
        }
    }
    anyhow::bail!("no safe depot approach from {start:?}")
}
