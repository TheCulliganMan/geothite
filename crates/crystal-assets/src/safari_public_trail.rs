//! Retire the obsolete basecamp barricade when the real permit court is active.

use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, SafariZone};

const OLD_GUARDS: [&str; 2] = [
    "MN_BOUNDARY_PENDING_GATE_WEST",
    "MN_BOUNDARY_PENDING_GATE_EAST",
];

pub(crate) fn install(pack: &mut CompiledGamePack, zone: &SafariZone) -> Result<()> {
    let map = pack
        .data
        .maps
        .get_mut(&zone.admission_map)
        .context("Safari basecamp missing")?;
    // Story gates were appended after the original basecamp residents. The
    // obsolete officers occupy slots 3/4, not the end of the current roster.
    // The real admission ranger stays at slot 2; later actors retain their
    // identifiers and relative order. Reject numeric script references rather
    // than silently redirecting a shifted native object index.
    let obsolete = map
        .objects
        .get(3..5)
        .context("Safari basecamp guard roster missing")?;
    let guards = obsolete.iter().zip(OLD_GUARDS).zip([32, 33]);
    for ((object, id), x) in guards {
        ensure!(
            object.script == id
                && object.object_identifier.as_deref() == Some(id)
                && object.x == x
                && object.y == 6
                && object.event_flag == "-1"
                && object.sprite == "SPRITE_OFFICER"
                && object.spritemovedata == "SPRITEMOVEDATA_STANDING_DOWN"
                && object.move_range_x == 0
                && object.move_range_y == 0,
            "Safari obsolete basecamp guard anchor changed: {id}"
        );
    }
    for command in &map.script_object_commands {
        for id in [&command.object_id, &command.target_object_id]
            .into_iter()
            .flatten()
        {
            ensure!(
                !OLD_GUARDS.contains(&id.as_str()),
                "a live basecamp script still references obsolete Safari guard {id}"
            );
            ensure!(
                id.parse::<u16>().is_err(),
                "basecamp script {} uses numeric object index {id}",
                command.source_script
            );
        }
    }
    map.objects.drain(3..5);
    authoring::refresh_event_section(map);
    // Some source epochs also retain the raw header in the split catalog.
    // Never create a second header: update only an already existing mirror.
    if let Some(header) = pack
        .data
        .map_scripts
        .get_mut(&format!("{}_MapEvents", map.id))
    {
        let rows = header
            .as_array_mut()
            .context("Safari basecamp event mirror is not an array")?;
        let before = rows.len();
        rows.retain(|row| {
            !(row["command"] == "object_event"
                && row["args"][11]
                    .as_str()
                    .is_some_and(|id| OLD_GUARDS.contains(&id)))
        });
        ensure!(
            before - rows.len() == 2,
            "Safari basecamp event mirror guards changed"
        );
    }
    Ok(())
}
