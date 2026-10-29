//! Route-house publication, fishermen's directions and native preservation.
use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::{build_minnesota_fishing_rods, read_verified_compiled_game_pack};

#[test]
#[ignore = "requires CRYSTAL_FISHING_RODS_BASE with completed Minnesota campaign"]
fn fishing_rods_route_houses_preserve_battles_and_advertise_gurus() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_FISHING_RODS_BASE").expect("Minnesota parent"),
    ))
    .expect("verified parent");
    let result = build_minnesota_fishing_rods(&base).expect("route guru houses");
    assert!(build_minnesota_fishing_rods(&result).is_err());
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.runtime_files() == result.runtime_files());
    assert_eq!(base.data().fishing, result.data().fishing);
    assert_eq!(base.data().items, result.data().items);
    assert_eq!(base.data().maps.len() + 3, result.data().maps.len());
    let mut fishermen = 0;
    for (id, old) in &base.data().maps {
        let new = &result.data().maps[id];
        assert_eq!(old.attributes, new.attributes, "route identity {id}");
        assert_eq!(old.objects, new.objects, "existing actors {id}");
        assert_eq!(
            old.trainer_scripts, new.trainer_scripts,
            "battle/defeat/after callback {id}"
        );
        assert_eq!(old.script_runtime_commands, new.script_runtime_commands);
        assert_eq!(old.script_item_grants, new.script_item_grants);
        assert_eq!(old.script_flag_commands, new.script_flag_commands);
        assert_eq!(old.script_control_commands, new.script_control_commands);
        assert_eq!(old.events.coord_events, new.events.coord_events);
        for (label, program) in &old.scripts {
            if &new.scripts[label] == program {
                continue;
            }
            let commands = program.as_array().expect("native text program");
            assert_eq!(commands.last().expect("terminator")["command"], "done");
            assert_eq!(
                &new.scripts[label].as_array().expect("expanded text")[..commands.len() - 1],
                &commands[..commands.len() - 1],
                "preserved original dialogue {id}/{label}"
            );
            assert!(
                serde_json::to_string(&new.scripts[label])
                    .expect("fisher text")
                    .contains("GURU"),
                "directions {id}/{label}"
            );
        }
        for trainer in old
            .trainer_scripts
            .values()
            .filter(|t| id.starts_with("Mn") && t.trainer_class == "FISHER")
        {
            fishermen += 1;
            assert!(
                serde_json::to_string(&new.scripts[&trainer.win_text])
                    .expect("victory text")
                    .contains("GURU"),
                "post-battle guru direction {}",
                trainer.trainer_id
            );
        }
        let footprint = match id.as_str() {
            "MnR01C00R00" => Some([12, 15]),
            "MnR03C00R00" => Some([15, 13]),
            "MnR06C00R00" => Some([18, 18]),
            _ => None,
        };
        if let Some([x, y]) = footprint {
            let mut expected = old.blocks.clone();
            for (dx, dy, block) in [
                (0, 0, 0x18),
                (1, 0, 0x19),
                (0, 1, 0x16),
                (1, 1, 0x1e),
                (0, 2, 0x01),
                (1, 2, 0x45),
            ] {
                expected[(y + dy) * usize::from(old.attributes.width) + x + dx] = block;
            }
            assert_eq!(expected, new.blocks, "six explicit facade/porch cells {id}");
            assert_eq!(
                &new.events.warps[..old.events.warps.len()],
                old.events.warps
            );
            assert_eq!(new.events.warps.len(), old.events.warps.len() + 2);
        } else {
            assert_eq!(old.blocks, new.blocks);
            assert_eq!(old.events, new.events);
        }
    }
    assert!(fishermen >= 20, "whole-region fishermen coverage");
    // A guru is useful before water traversal only if the main route reaches
    // the porch on foot. Check both ends of each chapter's through corridor.
    for (route, porch, vertical) in [
        ("MnR01C00R00", (25_u16, 34_u16), false),
        ("MnR03C00R00", (31, 30), true),
        ("MnR06C00R00", (37, 40), true),
    ] {
        let map = &result.data().maps[route];
        let width = map.attributes.width * 2;
        let height = map.attributes.height * 2;
        let collision = &result.data().tilesets[&map.attributes.tileset_name].collision;
        let mut seen = BTreeSet::from([porch]);
        let mut queue = VecDeque::from([porch]);
        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy) in [(-1_i32, 0_i32), (1, 0), (0, -1), (0, 1)] {
                let (nx, ny) = (i32::from(x) + dx, i32::from(y) + dy);
                if nx < 0 || ny < 0 || nx >= i32::from(width) || ny >= i32::from(height) {
                    continue;
                }
                let point = (nx as u16, ny as u16);
                let (x, y) = point;
                let block =
                    map.blocks[usize::from(y / 2) * usize::from(width / 2) + usize::from(x / 2)];
                let kind = &collision[&format!("{block:02x}")][usize::from((y % 2) * 2 + x % 2)];
                if matches!(
                    kind.as_str(),
                    "FLOOR" | "GRASS" | "TALL_GRASS" | "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK"
                ) && !map.objects.iter().any(|actor| (actor.x, actor.y) == point)
                    && seen.insert(point)
                {
                    queue.push_back(point);
                }
            }
        }
        assert!(
            seen.iter()
                .any(|&(x, y)| if vertical { y == 0 } else { x == 0 }),
            "{route}: first main corridor edge reaches guru without an HM"
        );
        assert!(
            seen.iter().any(|&(x, y)| if vertical {
                y == height - 1
            } else {
                x == width - 1
            }),
            "{route}: second main corridor edge reaches guru without an HM"
        );
    }
    for (room, rod) in [
        ("MnDriftlessFishingGuruHouse", "OLD_ROD"),
        ("MnRiverValleyFishingGuruHouse", "GOOD_ROD"),
        ("MnPineCountryFishingGuruHouse", "SUPER_ROD"),
    ] {
        let room = &result.data().maps[room];
        assert_eq!(
            room.events.warps.len(),
            3,
            "both carpet exits plus safe arrival"
        );
        assert_eq!(room.objects.len(), 1);
        assert!(
            room.script_item_grants
                .iter()
                .any(|grant| grant.item_id == rod)
        );
        assert!(
            room.script_control_commands
                .iter()
                .filter(|command| command.target_label.is_some())
                .all(|command| command.resolved_target_script.is_some())
        );
    }
}
