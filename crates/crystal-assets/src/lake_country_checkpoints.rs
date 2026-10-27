//! Independent two-city and Wind Badge checkpoints; every return is free.
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde_json::json;

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    base: &CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
) -> Result<()> {
    for (city, x, root, ids, flags, lines) in [
        (
            "MnMinneapolis",
            32,
            "MnMPNorthCheck",
            [
                "MN_MP_NORTH_WARDEN_WEST".into(),
                "MN_MP_NORTH_WARDEN_EAST".into(),
            ],
            vec!["ENGINE_PLAINBADGE", "ENGINE_FOGBADGE"],
            vec![
                "Earn the CAPITOL".into(),
                "BADGE in ST PAUL.".into(),
                "Then return here.".into(),
                "ST CLOUD is north.".into(),
            ],
        ),
        (
            "MnStCloud",
            24,
            "MnSCWindCheck",
            [
                "MN_SC_NORTH_WARDEN_WEST".into(),
                "MN_SC_NORTH_WARDEN_EAST".into(),
            ],
            vec!["ENGINE_STORMBADGE"],
            vec![
                "Earn SAVEN's".into(),
                "WIND BADGE first.".into(),
                "Then BRAINERD".into(),
                "lies to the north.".into(),
            ],
        ),
    ] {
        let map = maps.get_mut(city).context("north checkpoint city")?;
        checkpoint(
            base,
            map,
            &Checkpoint {
                identifiers: &ids,
                bodies: [[x, 2], [x + 1, 2]],
                inside: [[x, 3], [x + 1, 3]],
                outside: [[x, 1], [x + 1, 1]],
                root,
                incoming_facing: "DOWN",
                outward: "north",
                required_event: "",
                badge_flags: &flags,
                badge_lines: &lines,
                evidence_lines: &[],
            },
        )?;
        for event in &mut map.events.coord_events {
            if event.script_name.starts_with(root) {
                event.scene_id = "-1".into();
            }
        }
        if city == "MnMinneapolis" {
            let c = authoring::command;
            map.scripts.insert(
                root.into(),
                json!([
                    c("checkflag", &["ENGINE_PLAINBADGE"]),
                    c("iffalse", &["MnMPNorthNeedCurrent"]),
                    c("checkflag", &["ENGINE_FOGBADGE"]),
                    c("iffalse", &["MnMPNorthCheckBadges"]),
                    c("disappear", &[&ids[0]]),
                    c("disappear", &[&ids[1]]),
                    c("end", &[])
                ]),
            );
            authoring::text(
                map,
                "MnMPNorthCurrentText",
                &["Earn AVRIN's", "CURRENT BADGE.", "ST PAUL is east."],
            );
            map.scripts.insert(
                "MnMPNorthNeedCurrent".into(),
                json!([
                    c("opentext", &[]),
                    c("writetext", &["MnMPNorthCurrentText"]),
                    c("waitbutton", &[]),
                    c("closetext", &[]),
                    c("applymovement", &["PLAYER", "MnMPNorthCheckOneStep"]),
                    c("end", &[])
                ]),
            );
        }
    }
    Ok(())
}
