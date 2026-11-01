//! Restore the original nurse program and native compact Center interiors.

use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-native-center-restoration-v1";

fn is_center(name: &str, room: &MapModule) -> bool {
    name.starts_with("Mn")
        && matches!(
            room.attributes.tileset_name.as_str(),
            "pokecenter" | "pokecom_center"
        )
        && room
            .objects
            .iter()
            .any(|object| object.sprite == "SPRITE_NURSE")
}

fn restore_nurses(room: &mut MapModule) -> Result<usize> {
    let nurses: Vec<_> = room
        .objects
        .iter()
        .filter(|object| object.sprite == "SPRITE_NURSE")
        .map(|object| object.script.clone())
        .collect();
    for label in &nurses {
        ensure!(
            room.scripts.contains_key(label),
            "missing nurse program {label}"
        );
        room.scripts.insert(
            label.clone(),
            serde_json::json!([authoring::command("jumpstd", &["PokecenterNurseScript"])]),
        );
    }
    Ok(nurses.len())
}

/// Build an unselected candidate using the original pack's native nurse and
/// Center layout. Existing regional actors, story programs and services remain.
/// This creates a new pack identity; existing saves need an explicit, verified
/// coordinate migration before they can be used with the resized rooms.
///
/// # Errors
/// Rejects a missing native template, ambiguous stair event, missing nurse
/// scripts, invalid map metadata or an invalid resulting pack.
pub fn restore_minnesota_center_nurses(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|item| item == MANIFEST),
        "Center restoration already installed"
    );
    let native = base
        .data
        .maps
        .get("CherrygrovePokecenter1F")
        .context("native Center template")?;
    ensure!(
        (native.attributes.width, native.attributes.height) == (5, 4),
        "native Center template dimensions"
    );
    let native_blocks = native.blocks.clone();
    let native_stair = native
        .events
        .warps
        .iter()
        .find(|warp| warp.x == 0 && warp.y == 7 && warp.target_map_constant == "POKECENTER_2F")
        .context("native Center second-floor stair")?
        .clone();
    let mut pack = base.clone();
    let names: Vec<_> = pack
        .data
        .maps
        .iter()
        .filter(|(name, room)| is_center(name, room))
        .map(|(name, _)| name.clone())
        .collect();
    let mut nurse_count = 0;
    let mut compact_count = 0;
    for name in names {
        let mut room = pack
            .data
            .maps
            .get(&name)
            .context("regional Center")?
            .clone();
        let count = restore_nurses(&mut room)?;
        if count == 0 {
            continue;
        }
        nurse_count += count;
        if (room.attributes.width, room.attributes.height) != (5, 4) {
            resize_center(&mut room)?;
        }
        {
            room.blocks.clone_from(&native_blocks);
            let stairs: Vec<_> = room
                .events
                .warps
                .iter()
                .filter(|warp| warp.x == 0 && warp.y == 7)
                .collect();
            if let Some(stair) = stairs.first() {
                ensure!(
                    stairs.len() == 1
                        && stair.target_map_constant == native_stair.target_map_constant
                        && stair.target_warp_id == native_stair.target_warp_id,
                    "conflicting Center stair in {name}"
                );
            } else {
                let mut stair = native_stair.clone();
                stair.index = u16::try_from(room.events.warps.len() + 1)?;
                room.events.warps.push(stair);
            }
            compact_count += 1;
        }
        authoring::finish(&pack, &mut room)?;
        let map_names = pack.data.map_name_by_constant_from_attributes()?;
        room.script_map_commands =
            crate::parse_script_map_commands(&room.id, &room.scripts, &map_names)?;
        crate::authored_town_publication::replace(&mut pack, room)?;
        for spawn in pack
            .data
            .runtime_spawn_points
            .values_mut()
            .filter(|spawn| spawn.map_name == name)
        {
            spawn.tile_x = 3;
            spawn.tile_y = 5;
            spawn.metatile_x = 1;
            spawn.metatile_y = 2;
            spawn.subtile_x = 1;
            spawn.subtile_y = 1;
        }
    }
    ensure!(
        nurse_count > 0 && compact_count > 0,
        "regional Center nurses and compact rooms required"
    );
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    ensure!(
        pack.compiled_audio == base.compiled_audio && pack.audio_manifest == base.audio_manifest,
        "preserve audio programs and PCM contracts"
    );
    Ok(pack)
}

fn resize_center(room: &mut MapModule) -> Result<()> {
    ensure!(
        room.events.warps.len() == 3,
        "unexpected existing Center warps: {}",
        room.id
    );
    ensure!(
        room.objects
            .iter()
            .filter(|object| object.sprite == "SPRITE_NURSE")
            .count()
            == 1,
        "one nurse required in {}",
        room.id
    );
    if room.id != "MnRochesterPokecenter1F" {
        ensure!(
            room.events.coord_events.is_empty(),
            "custom Center scene requires an authored relocation: {}",
            room.id
        );
        for body in room
            .scripts
            .values()
            .filter_map(serde_json::Value::as_array)
        {
            ensure!(
                !body.iter().any(|command| matches!(
                    command["command"].as_str(),
                    Some("applymovement" | "moveobject")
                )),
                "scripted Center actor movement requires an authored relocation: {}",
                room.id
            );
        }
    }
    let positions = [(2, 3), (8, 6), (1, 6), (6, 3), (8, 3), (1, 5), (6, 4)];
    let mut ordinary = 0usize;
    for object in &mut room.objects {
        let position = if object.sprite == "SPRITE_NURSE" {
            (3, 1)
        } else if room.id == "MnRochesterPokecenter1F" && object.script == "MN_ARC_RC_REED" {
            (5, 4)
        } else {
            let position = *positions
                .get(ordinary)
                .context("too many Center actors for the native room")?;
            ordinary += 1;
            position
        };
        object.x = position.0;
        object.y = position.1;
    }
    for (index, warp) in room.events.warps.iter_mut().enumerate() {
        let (x, y) = [(3, 7), (4, 7), (3, 5)][index];
        warp.x = x;
        warp.y = y;
    }
    let mut other_background = 0usize;
    for bg in &mut room.events.bg_events {
        let (x, y) = if bg.script.to_ascii_uppercase().contains("PC") {
            (9, 1)
        } else {
            let position = *[(0, 2), (1, 2)]
                .get(other_background)
                .context("too many non-PC Center signs for native counters")?;
            other_background += 1;
            position
        };
        bg.x = x;
        bg.y = y;
    }
    room.attributes.width = 5;
    room.attributes.height = 4;
    if room.id == "MnRochesterPokecenter1F" {
        relocate_rochester_rival(room)?;
    }
    Ok(())
}

fn relocate_rochester_rival(room: &mut MapModule) -> Result<()> {
    ensure!(
        room.scripts.contains_key("MnArcRCOffer")
            && room
                .objects
                .iter()
                .any(|object| object.script == "MN_ARC_RC_REED"),
        "Rochester rival scene is missing"
    );
    room.events
        .coord_events
        .retain(|event| !event.script_name.starts_with("MnArcRCEntry"));
    for x in (0u16..=9).filter(|x| *x != 5) {
        let script = format!("MnNativeCenterReedEntry{x}");
        let movement = format!("MnNativeCenterReedApproach{x}");
        let (direction, count, face, player_face) = if x < 5 {
            ("LEFT", usize::from(4 - x), "LEFT", "RIGHT")
        } else {
            ("RIGHT", usize::from(x - 6), "RIGHT", "LEFT")
        };
        crate::northstar_rival_arc::movement(room, &movement, &[(direction, count)]);
        let c = authoring::command;
        crate::northstar_rival_arc::put(
            room,
            &script,
            vec![
                c("checkevent", &["EVENT_MN_REED_ROCHESTER_MET"]),
                c("iftrue", &["MnArcRCEnd"]),
                c("checkevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
                c("iffalse", &["MnArcRCEnd"]),
                c("applymovement", &["MN_ARC_RC_REED", &movement]),
                c("turnobject", &["MN_ARC_RC_REED", face]),
                c("turnobject", &["PLAYER", player_face]),
                c("sjump", &["MnArcRCOffer"]),
            ],
        );
        room.events
            .coord_events
            .push(crystal_core::map::CoordEvent {
                x,
                y: 4,
                scene_id: "-1".into(),
                script_name: script,
            });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires the selected external Minnesota review pack"]
    fn native_nurses_and_compact_center_layouts_are_restored() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let parent = std::env::var_os("CRYSTAL_CENTER_PARENT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/minnesota-reviewed-maps-v11.crystalpack"));
        let base = crate::read_verified_compiled_game_pack(parent).expect("review parent pack");
        let output = super::restore_minnesota_center_nurses(&base).expect("restoration candidate");
        let native = &base.data.maps["CherrygrovePokecenter1F"];
        let mut count = 0;
        for (name, room) in &output.data.maps {
            if !super::is_center(name, room) {
                continue;
            }
            for nurse in room
                .objects
                .iter()
                .filter(|object| object.sprite == "SPRITE_NURSE")
            {
                assert_eq!(
                    room.scripts[&nurse.script],
                    serde_json::json!([{"command":"jumpstd", "args":["PokecenterNurseScript"]}])
                );
                count += 1;
            }
            assert_eq!(
                (room.attributes.width, room.attributes.height),
                (5, 4),
                "{name}"
            );
            assert_eq!(room.blocks, native.blocks, "native layout {name}");
            assert!(room.events.warps.iter().any(|warp| warp.x == 0
                && warp.y == 7
                && warp.target_map_constant == "POKECENTER_2F"));
        }
        assert!(super::is_center(
            "MnLeechLakeCareCabin",
            &base.data.maps["MnLeechLakeCareCabin"]
        ));
        assert!(count >= 19, "all regional nurses");
        assert_eq!(output.compiled_audio, base.compiled_audio);
        assert_eq!(output.audio_manifest, base.audio_manifest);
        for (name, before) in &base.data.maps {
            let after = &output.data.maps[name];
            if !super::is_center(name, before) {
                assert_eq!(before, after, "unrelated map {name}");
                continue;
            }
            for warp in &before.events.warps {
                let restored = after
                    .events
                    .warps
                    .iter()
                    .find(|item| item.index == warp.index)
                    .expect("existing warp index");
                assert_eq!(
                    (&restored.target_map_constant, restored.target_warp_id),
                    (&warp.target_map_constant, warp.target_warp_id),
                    "{name} warp {}",
                    warp.index
                );
            }
            for object in &before.objects {
                assert!(
                    after.objects.iter().any(|item| item.script == object.script
                        && item.sprite == object.sprite
                        && item.event_flag == object.event_flag),
                    "{name}: lost actor {}",
                    object.script
                );
                if object.sprite != "SPRITE_NURSE" {
                    assert_eq!(
                        before.scripts.get(&object.script),
                        after.scripts.get(&object.script),
                        "{name}: changed actor program {}",
                        object.script
                    );
                }
            }
            for background in &before.events.bg_events {
                assert!(
                    after
                        .events
                        .bg_events
                        .iter()
                        .any(|item| item.script == background.script),
                    "{name}: lost background service {}",
                    background.script
                );
            }
        }
        if let Some(path) = std::env::var_os("CRYSTAL_CENTER_CANDIDATE") {
            let path = std::path::PathBuf::from(path);
            assert!(!path.exists(), "candidate output must be fresh");
            assert!(
                std::process::Command::new("git")
                    .args(["check-ignore", "--quiet", "--no-index"])
                    .arg(&path)
                    .status()
                    .expect("git check-ignore")
                    .success(),
                "candidate must remain ignored"
            );
            output
                .write_preserving_storage(&path)
                .expect("write review candidate");
            assert_eq!(
                crate::read_verified_compiled_game_pack(path).expect("roundtrip"),
                output
            );
        }
    }
}
