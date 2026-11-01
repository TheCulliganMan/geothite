//! Fair visitors use bounded rectangular walks through ordinary native floor.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
pub(crate) struct Patrol {
    id: String,
    size_core: [u16; 2],
    #[serde(default)]
    behavior: Behavior,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Behavior {
    #[default]
    Run,
    Walk,
    Rotate,
}

pub(crate) fn install(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    patrols: &[Patrol],
) -> Result<()> {
    let mut routes = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for patrol in patrols {
        ensure!(
            ids.insert(&patrol.id),
            "duplicate fair patrol {}",
            patrol.id
        );
        let index = map
            .objects
            .iter()
            .position(|actor| actor.object_identifier.as_deref() == Some(patrol.id.as_str()))
            .context("fair patrol actor")?;
        let actor = &map.objects[index];
        ensure!(
            actor.object_type == "OBJECTTYPE_SCRIPT",
            "patrol must be a non-trainer"
        );
        if matches!(patrol.behavior, Behavior::Rotate) {
            ensure!(
                patrol.size_core == [0, 0],
                "rotating visitor stays in place"
            );
            map.objects[index].spritemovedata = "SPRITEMOVEDATA_SPINCLOCKWISE".into();
            continue;
        }
        let [width, height] = patrol.size_core;
        ensure!(
            (1..=15).contains(&width) && (1..=15).contains(&height),
            "fair patrol extent"
        );
        let right = actor
            .x
            .checked_add(width)
            .context("patrol width overflow")?;
        let bottom = actor
            .y
            .checked_add(height)
            .context("patrol height overflow")?;
        for y in actor.y..=bottom {
            for x in actor.x..=right {
                if x != actor.x && x != right && y != actor.y && y != bottom {
                    continue;
                }
                ensure!(
                    kind(pack, map, [x, y])? == "FLOOR"
                        && routes.insert([x, y])
                        && !map
                            .objects
                            .iter()
                            .enumerate()
                            .any(|(other, a)| other != index && [a.x, a.y] == [x, y])
                        && !map.events.warps.iter().any(|w| [w.x, w.y] == [x, y]),
                    "fair patrol {} crosses a blocked or occupied tile at {x},{y}",
                    patrol.id
                );
            }
        }
        let actor = &mut map.objects[index];
        actor.spritemovedata = match patrol.behavior {
            Behavior::Walk => "SPRITEMOVEDATA_PATROL_CLOCKWISE_SLOW",
            Behavior::Run => "SPRITEMOVEDATA_PATROL_CLOCKWISE",
            Behavior::Rotate => unreachable!("rotating visitors were handled above"),
        }
        .into();
        actor.move_range_x = width;
        actor.move_range_y = height;
    }
    Ok(())
}
