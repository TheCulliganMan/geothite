//! Original native floorplans with explicit active exits and passive arrivals.
use std::collections::BTreeMap;

use anyhow::Result;
use crystal_core::map::BackgroundEvent;
use serde_json::json;

use crate::ely_league_scripts::{self as scripts, Member};
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

pub(crate) const MAPS: [(&str, &str, &str); 9] = [
    (
        "MnElyPokecenter1F",
        "MN_ELY_POKECENTER_1F",
        "GoldenrodPokecenter1F",
    ),
    ("MnElyMart", "MN_ELY_MART", "CherrygroveMart"),
    ("MnLeagueLobby", "MN_LEAGUE_LOBBY", "IlexForestAzaleaGate"),
    ("MnLeagueRoom1", "MN_LEAGUE_ROOM_1", "KogasRoom"),
    ("MnLeagueRoom2", "MN_LEAGUE_ROOM_2", "BrunosRoom"),
    ("MnLeagueRoom3", "MN_LEAGUE_ROOM_3", "WillsRoom"),
    ("MnLeagueRoom4", "MN_LEAGUE_ROOM_4", "KarensRoom"),
    ("MnLeagueChampion", "MN_LEAGUE_CHAMPION", "LancesRoom"),
    ("MnLeagueHallOfFame", "MN_LEAGUE_HALL_OF_FAME", "HallOfFame"),
];

pub(crate) fn finish(pack: &CompiledGamePack, map: &mut MapModule) -> Result<()> {
    authoring::finish(pack, map)?;
    let name = &map.id;
    let body = &map.scripts;
    map.script_runtime_commands = crate::parse_script_runtime_commands(name, body)?;
    map.script_object_commands = crate::parse_script_object_commands(name, body)?;
    map.script_variable_commands = crate::parse_script_variable_commands(name, body)?;
    map.script_movements = crate::parse_script_movements(name, body, &map.script_object_commands)?;
    map.script_block_changes = crate::parse_script_block_changes(name, body)?;
    map.trainer_scripts = crate::parse_trainer_scripts(name, body)?;
    map.scripted_trainer_battles = crate::parse_scripted_trainer_battles(name, body)?;
    let mut names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<BTreeMap<_, _>>();
    names.insert("MN_ELY".into(), "MnEly".into());
    for (id, constant, _) in MAPS {
        names.insert(constant.into(), id.into());
    }
    map.script_map_commands = crate::parse_script_map_commands(name, body, &names)?;
    Ok(())
}

pub(crate) fn build(
    pack: &CompiledGamePack,
    members: &[Member],
) -> Result<Vec<(MapModule, [u16; 2])>> {
    let mut result = Vec::new();
    for (i, (name, constant, template)) in MAPS.iter().enumerate() {
        let mut map = authoring::blank(pack, template, name, constant)?;
        let start = match i {
            0 => {
                // This public clinic has no link-club floor. Replace the old
                // upstairs stair art with the same native room floor.
                map.blocks[15] = 0x04;
                for tile in [(3, 7), (4, 7), (3, 5)] {
                    authoring::warp(&mut map, tile, "MN_ELY", 5);
                }
                map.scripts.insert(
                    "MnElyNurse".into(),
                    json!([
                        c("blackoutmod", &["MN_ELY_POKECENTER_1F"]),
                        c("jumpstd", &["PokecenterNurseScript"])
                    ]),
                );
                authoring::npc(pack, &mut map, "MnElyNurse", "SPRITE_NURSE", 3, 1)?;
                map.scripts.insert(
                    "MnElyPC".into(),
                    json!([
                        c("opentext", &[]),
                        c("special", &["PokemonCenterPC"]),
                        c("closetext", &[]),
                        c("end", &[])
                    ]),
                );
                map.events.bg_events.push(BackgroundEvent {
                    x: 8,
                    y: 0,
                    event_type: "BGEVENT_READ".into(),
                    script: "MnElyPC".into(),
                });
                scripts::reset_on_entry(&mut map);
                [3, 5]
            }
            1 => {
                for tile in [(2, 7), (3, 7), (2, 5)] {
                    authoring::warp(&mut map, tile, "MN_ELY", 6);
                }
                map.scripts.insert(
                    "MnElyClerk".into(),
                    json!([
                        c("faceplayer", &[]),
                        c("opentext", &[]),
                        c("pokemart", &["MARTTYPE_STANDARD", "MART_MN_ELY"]),
                        c("closetext", &[]),
                        c("end", &[])
                    ]),
                );
                authoring::npc(pack, &mut map, "MnElyClerk", "SPRITE_CLERK", 1, 3)?;
                [2, 5]
            }
            2 => {
                for (tile, target) in [
                    ((0, 4), 7),
                    ((0, 5), 8),
                    ((9, 4), 7),
                    ((9, 5), 8),
                    ((1, 4), 7),
                    ((1, 5), 8),
                ] {
                    authoring::warp(&mut map, tile, "MN_ELY", target);
                }
                scripts::registration(pack, &mut map)?;
                [1, 4]
            }
            3..=7 => {
                let member = &members[i - 3];
                let previous = if i == 3 {
                    "MN_LEAGUE_LOBBY"
                } else {
                    MAPS[i - 1].1
                };
                let bottom = map.attributes.height * 2 - 1;
                for lane in 0..2 {
                    authoring::warp(
                        &mut map,
                        (4 + lane, bottom),
                        previous,
                        if i == 3 {
                            5 + lane as i16
                        } else {
                            7 + lane as i16
                        },
                    );
                }
                let top = if i == 7 { 1 } else { 2 };
                for lane in 0..2 {
                    authoring::warp(
                        &mut map,
                        (4 + lane, top),
                        MAPS[i + 1].1,
                        if i == 7 {
                            3 + lane as i16
                        } else {
                            5 + lane as i16
                        },
                    );
                }
                for lane in 0..2 {
                    authoring::warp(
                        &mut map,
                        (4 + lane, bottom - 1),
                        previous,
                        if i == 3 {
                            5 + lane as i16
                        } else {
                            7 + lane as i16
                        },
                    );
                }
                for lane in 0..2 {
                    authoring::warp(
                        &mut map,
                        (4 + lane, if i == 7 { 2 } else { 4 }),
                        MAPS[i + 1].1,
                        if i == 7 {
                            3 + lane as i16
                        } else {
                            5 + lane as i16
                        },
                    );
                }
                scripts::battle(pack, &mut map, member, i - 2)?;
                [4, bottom - 1]
            }
            _ => {
                map.attributes.music = Some("MUSIC_INDIGO_PLATEAU".into());
                for lane in 0..2 {
                    authoring::warp(
                        &mut map,
                        (4 + lane, 13),
                        "MN_LEAGUE_CHAMPION",
                        7 + lane as i16,
                    );
                }
                for lane in 0..2 {
                    authoring::warp(
                        &mut map,
                        (4 + lane, 12),
                        "MN_LEAGUE_CHAMPION",
                        7 + lane as i16,
                    );
                }
                scripts::hall(pack, &mut map)?;
                [4, 12]
            }
        };
        finish(pack, &mut map)?;
        result.push((map, start));
    }
    Ok(result)
}
