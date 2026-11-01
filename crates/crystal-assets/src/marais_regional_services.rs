//! A regional field station offers the local lamp part without a demo starter.
use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::collections::BTreeMap;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule, RuntimeSpawnPoint};

pub(crate) const PART: &str = "EVENT_MN_MARAIS_SPARE_PART_RECEIVED";

/// Registers only these two existing clinics; visiting their nurse selects one.
pub(crate) fn recoveries(
    pack: &mut CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
) -> Result<()> {
    for (name, script, tile) in [
        ("MnDuluthPokecenter1F", "MN_DULUTH_NURSE", [10, 13]),
        ("MnGrandMaraisPokecenter", "MnGrandMaraisNurse", [3, 5]),
    ] {
        ensure!(
            !pack
                .data
                .runtime_spawn_points
                .values()
                .any(|s| s.map_name == name),
            "unregistered regional clinic required"
        );
        let map = maps.get_mut(name).context("regional clinic")?;
        ensure!(
            crate::authored_town_geometry::kind(pack, map, tile)? == "FLOOR",
            "safe recovery floor"
        );
        let constant = map
            .attributes
            .map_constant
            .as_ref()
            .context("clinic constant")?;
        let meta = pack
            .data
            .runtime_map_metadata
            .get(constant)
            .context("clinic identity")?;
        let identifier = pack
            .data
            .runtime_spawn_points
            .values()
            .map(|s| s.identifier)
            .max()
            .context("spawn catalog")?
            .checked_add(1)
            .context("recovery identifier overflow")?;
        pack.data.runtime_spawn_points.insert(
            identifier.to_string(),
            RuntimeSpawnPoint {
                identifier,
                map_constant: constant.clone(),
                map_name: name.into(),
                group_name: meta.group_name.clone(),
                group_id: i16::try_from(meta.group_id)?,
                map_id: i16::try_from(meta.map_id)?,
                tile_x: tile[0] as i16,
                tile_y: tile[1] as i16,
                metatile_x: (tile[0] / 2) as i16,
                metatile_y: (tile[1] / 2) as i16,
                subtile_x: (tile[0] % 2) as i16,
                subtile_y: (tile[1] % 2) as i16,
            },
        );
        let body = map
            .scripts
            .get_mut(script)
            .and_then(serde_json::Value::as_array_mut)
            .context("regional nurse script")?;
        let insert_at = if name == "MnDuluthPokecenter1F" {
            body.iter()
                .position(|c| c["command"] == "special" && c["args"] == json!(["HealParty"]))
                .context("existing Duluth healing")?
                + 1
        } else {
            ensure!(
                *body == vec![authoring::command("jumpstd", &["PokecenterNurseScript"])],
                "native Marais nurse"
            );
            0
        };
        body.insert(insert_at, authoring::command("blackoutmod", &[constant]));
    }
    Ok(())
}

pub(crate) fn allocate(pack: &mut CompiledGamePack) -> Result<()> {
    let flags = &mut pack.data.story_event_script_constants.global;
    ensure!(!flags.contains_key(PART), "unused local spare-part flag");
    let id = flags
        .iter()
        .filter(|(k, _)| k.starts_with("EVENT_"))
        .map(|(_, v)| *v)
        .max()
        .context("event catalog")?
        .checked_add(1)
        .context("event overflow")?;
    u16::try_from(id)?;
    flags.insert(PART.into(), id);
    Ok(())
}

pub(crate) fn station(map: &mut MapModule) -> Result<()> {
    ensure!(
        map.id == "MnGrandMaraisFieldStation" && map.scripts.contains_key("MnGrandMaraisWelcome"),
        "installed field station"
    );
    // Retire only text that promises the removed demo creature/supply bundle.
    // The keeper reminder and acknowledged-repair conversation stay reachable.
    map.scripts.remove("MnGrandMaraisWelcomeText");
    map.scripts.remove("MnGrandMaraisPartnerText");
    map.events
        .coord_events
        .retain(|e| e.script_name != "MnGrandMaraisWelcome");
    let c = authoring::command;
    map.scripts.insert(
        "MnGrandMaraisWelcome".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[PART]),
            c("iftrue", &["MnMaraisPartThanks"]),
            c("opentext", &[]),
            c("writetext", &["MnMaraisPartOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnMaraisPartDeclined"]),
            c("giveitem", &["MACHINE_PART", "1"]),
            c("iffalse", &["MnMaraisPartFull"]),
            c("writetext", &["MnMaraisPartReceivedText"]),
            c("waitbutton", &[]),
            c("setevent", &[PART]),
            c("closetext", &[]),
            c("setscene", &["SCENE_MN_FIELD_READY"]),
            c("end", &[])
        ]),
    );
    for (root, label) in [
        ("MnMaraisPartDeclined", "MnMaraisPartDeclinedText"),
        ("MnMaraisPartFull", "MnMaraisPartFullText"),
    ] {
        map.scripts.insert(
            root.into(),
            json!([
                c("writetext", &[label]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("setscene", &["SCENE_MN_FIELD_READY"]),
                c("end", &[])
            ]),
        );
    }
    map.scripts.insert(
        "MnMaraisPartThanks".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MnMaraisPartThanksText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &["MnGrandMaraisWelcomeAfter"])
        ]),
    );
    for (label, lines) in [
        (
            "MnMaraisPartOfferText",
            vec![
                "The harbor called.",
                "GUNFLINT is west.",
                "Our keeper needs",
                "this spare part.",
                "Will you take it?",
            ],
        ),
        (
            "MnMaraisPartReceivedText",
            vec!["A spare lamp part.", "The keeper waits."],
        ),
        (
            "MnMaraisPartDeclinedText",
            vec!["It can wait here.", "The trail is open."],
        ),
        (
            "MnMaraisPartFullText",
            vec!["Make room first.", "I will keep it."],
        ),
        (
            "MnMaraisPartThanksText",
            vec![
                "The keeper is by",
                "the harbor lamp.",
                "The northbound",
                "relay is separate.",
            ],
        ),
    ] {
        authoring::text(map, label, &lines);
    }
    Ok(())
}
