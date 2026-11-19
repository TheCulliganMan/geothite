//! Read the full face of each prairie landmark and approach Kensigard from any side.
use anyhow::{Result, ensure};
use crystal_core::map::CoordEvent;

use crate::CompiledGamePack;
use crate::authored_dialogue::install_canonical;
use crate::authored_town_geometry::kind;
use crate::authored_town_publication::replace;
use crate::northstar_lair_authoring::read;
use crate::northstar_rival_arc::map;

#[cfg(test)]
#[path = "northstar_field_margin_reads_tests.rs"]
mod tests;

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut field = map(pack, "MnFieldMargin")?;
    let mut road = map(pack, "MnR16C01R00")?;
    for label in [
        "MnKensigardText_caretaker_intro",
        "MnKensigardText_caretaker_partial",
        "MnKensigardText_caretaker_comparison",
        "MnKensigardText_claim_corrected",
    ] {
        install_canonical(&mut road, label)?;
    }
    road.script_text_bodies = crate::parse_script_text_bodies(&road.id, &road.scripts)?;
    replace(pack, road)?;

    // Both round stones have four solid quarters. The exposed soil has a
    // narrow, solid west face and a walkable east half.
    ensure!(
        field.events.bg_events.len() == 4,
        "unexpected field objects"
    );
    for (label, original, faces) in [
        ("MnKensigardEdge", [8, 7], &[[8, 6], [9, 6], [9, 7]][..]),
        (
            "MnKensigardBoundary",
            [9, 10],
            &[[8, 10], [8, 11], [9, 11]][..],
        ),
        ("MnKensigardSoil", [12, 8], &[[12, 9]][..]),
        ("MnKensigardGrooves", [14, 22], &[[14, 23]][..]),
    ] {
        ensure!(
            field.events.bg_events.iter().any(|e| [e.x, e.y] == original
                && e.script == label
                && e.event_type == "BGEVENT_READ")
                && kind(pack, &field, original)? == "WALL",
            "field landmark moved: {label}"
        );
        for &at in faces {
            ensure!(
                kind(pack, &field, at)? == "WALL"
                    && !field.events.bg_events.iter().any(|e| [e.x, e.y] == at),
                "field landmark has a different face at {at:?}"
            );
            read(&mut field, at, label);
        }
    }

    let reveal = "MnKensigardReveal";
    ensure!(
        matches!(field.events.coord_events.as_slice(), [e] if [e.x, e.y] == [13, 6]
            && e.scene_id == "-1" && e.script_name == reveal)
            && field
                .objects
                .iter()
                .any(|a| [a.x, a.y] == [13, 5] && a.script == "MN_MARGIN_KENSIGARD"),
        "unexpected Kensigard reveal"
    );
    for at in [[12, 5], [13, 4], [14, 5]] {
        ensure!(
            kind(pack, &field, at)? == "FLOOR",
            "obstructed Kensigard approach"
        );
        field.events.coord_events.push(CoordEvent {
            x: at[0],
            y: at[1],
            scene_id: "-1".into(),
            script_name: reveal.into(),
        });
    }
    crate::grand_marais_modpack::authoring::refresh_event_section(&mut field);
    replace(pack, field)
}
