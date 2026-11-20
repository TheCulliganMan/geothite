//! Keep the urgent lighthouse handoff at the door and the partner offer with Tamarack.
use anyhow::{Result, ensure};
use serde_json::json;

use crate::MapModule;
use crate::grand_marais_modpack::authoring::{command, refresh_event_section};

const WELCOME: &str = "MnGrandMaraisWelcome";
pub(super) const DOORWAY: &str = "MnMaraisDoorwayPart";
const DONE: &str = "MnGrandMaraisStationNoop";

pub(super) fn apply(room: &mut MapModule) -> Result<()> {
    let [tamarack] = room.objects.as_mut_slice() else {
        anyhow::bail!("expected one professor in the Grand Marais field station");
    };
    ensure!(
        (tamarack.x, tamarack.y) == (5, 3)
            && tamarack.script == WELCOME
            && tamarack.sprite == "SPRITE_SCIENTIST",
        "unexpected field station professor"
    );
    tamarack.sprite = "SPRITE_MN_PROF_TAMARACK".into();

    let lanes = &mut room.events.coord_events;
    ensure!(lanes.len() == 2, "expected two field station doorway lanes");
    for (lane, x) in lanes.iter_mut().zip([2, 3]) {
        ensure!(
            (lane.x, lane.y) == (x, 6)
                && lane.scene_id == "SCENE_MN_FIELD_STATION"
                && lane.script_name == WELCOME,
            "unexpected field station doorway at column {x}"
        );
        lane.script_name = DOORWAY.into();
    }
    ensure!(
        room.scripts.get(DONE) == Some(&json!([command("end", &[])]))
            && !room.scripts.contains_key(DOORWAY),
        "unexpected field station doorway scripts"
    );
    room.scripts.insert(
        DOORWAY.into(),
        json!([
            command("checkevent", &["EVENT_MN_GRAND_MARAIS_SIGNAL"]),
            command("iftrue", &[DONE]),
            command("checkitem", &["MACHINE_PART"]),
            command("iftrue", &[DONE]),
            command("sjump", &["MnRewardMachinePart"]),
        ]),
    );
    refresh_event_section(room);
    Ok(())
}
