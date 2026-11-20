//! Attach outdoor wayfinding to the printed surface the player can see.
#[path = "northstar_sign_plaques.rs"]
mod plaques;

use anyhow::{Context, Result, ensure};

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule, authored_town_publication};

const PARENT_HASHES: [&str; 2] = [
    "2130c429085c8608a7feb0e8438f25aa7b1b226446b2e6148c4aea54562841f0",
    "0c778f3fbdc475c524aede7427261093fc9728cca4bb1ca26746ce16a07a5963",
];

fn move_read(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    script: &str,
    old: [u16; 2],
    printed: [u16; 2],
) -> Result<()> {
    ensure!(
        kind(pack, map, old)? == "FLOOR" && kind(pack, map, printed)? == "WALL",
        "unexpected ground or sign for {script}"
    );
    ensure!(
        !map.events.bg_events.iter().any(|e| [e.x, e.y] == printed),
        "another interaction occupies {script}'s board"
    );
    let event = map
        .events
        .bg_events
        .iter_mut()
        .find(|e| e.script == script)
        .context("missing trail sign")?;
    ensure!(
        [event.x, event.y] == old && event.event_type == "BGEVENT_READ",
        "unexpected reading square for {script}"
    );
    [event.x, event.y] = printed;

    // Keep the source order and indices of all other native map commands.
    let original = [
        old[0].to_string(),
        old[1].to_string(),
        "BGEVENT_READ".into(),
        script.into(),
    ];
    let wire = map
        .map_event_section_commands
        .iter_mut()
        .find(|r| r.command == "bg_event" && r.args == original)
        .with_context(|| format!("missing native bg_event for {script}"))?;
    wire.args[0] = printed[0].to_string();
    wire.args[1] = printed[1].to_string();
    Ok(())
}

/// Aligns five outdoor reads with visible boards and adds two local sign plaques.
///
/// # Errors
/// Returns an error for an unexpected edition, occupied reading surface, event
/// mismatch or invalid compiled content.
pub fn revise_trail_signs(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-research-choices-v1")
            && PARENT_HASHES.contains(&pack.identity.content_hash.as_str()),
        "expected a Research Choices edition"
    );

    type SignMove = (&'static str, [u16; 2], [u16; 2]);
    let rooms: &[(&str, &[SignMove])] = &[
        (
            "MnDetroitLakesApproach",
            &[("MN_COHESION_DETROIT_SIGN", [8, 13], [8, 12])],
        ),
        (
            "MnPreservePineEntry",
            &[("MN_COHESION_PRESERVE_NOTICE", [13, 11], [12, 10])],
        ),
        (
            "MnEly",
            &[
                ("MnElyTrailSign", [20, 3], [21, 3]),
                ("MnElyCampSign", [14, 21], [15, 21]),
            ],
        ),
        (
            "MnMinneapolis",
            &[("MN_MP_ARRIVALINFO_SIGN", [40, 60], [40, 58])],
        ),
    ];
    for &(name, signs) in rooms {
        let mut room = pack.data.maps.get(name).context(name)?.clone();
        if name == "MnMinneapolis" {
            // West shoulder of the Arrival Desk, beside the south approach.
            // Only the upper-left quadrant gains the sign and wall collision.
            let index = 29 * usize::from(room.attributes.width) + 20;
            let block = room
                .blocks
                .get_mut(index)
                .context("Arrival Desk shoulder")?;
            ensure!(
                room.attributes.tileset_name == "mn_minneapolis_city" && *block == 0x01,
                "unexpected Arrival Desk shoulder"
            );
            *block = 0x45;
        }
        for &(script, old, printed) in signs {
            move_read(&pack, &mut room, script, old, printed)?;
        }
        authored_town_publication::replace(&mut pack, room)?;
    }
    plaques::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-trail-signs-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
