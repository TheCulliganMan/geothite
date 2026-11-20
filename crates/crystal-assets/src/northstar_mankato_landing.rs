//! Keep the whole lower quarry ledge clear for the hop back to the entrance.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::refresh_event_section;

const GYM: &str = "MnMankatoGym";
const GUIDE: &str = "MnMankatoGymGuide";
const PARENTS: [&str; 2] = [
    "a200a721ccd6057f14842d950cf957f263d6b3fbebddd3aecc080abdcc974668",
    "2ad46453c417efdea3cbc1c8fa33ac4a141cb9fe423769408d424689e505b4c9",
];

/// Clears the lower terrace landings in the two Leech Lake Trainers editions.
///
/// # Errors
/// Returns an error if the parent, gym, statue or guide differs from the
/// authored layout, or if the resulting game pack fails runtime verification.
pub fn revise_quarry_landings(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        PARENTS.contains(&pack.identity.content_hash.as_str())
            && pack.report.manifests.last().map(String::as_str)
                == Some("northstar-leech-lake-trainers-v1"),
        "quarry landings require a matching Leech Lake Trainers archive"
    );
    let mut room = pack.data.maps.get(GYM).context("Mankato Gym")?.clone();
    ensure!(
        room.attributes.tileset_name == "mn_gym_2"
            && (room.attributes.width, room.attributes.height) == (8, 10),
        "unexpected quarry dimensions or tileset"
    );
    // The west statue is flush against the wall. Place its eastern partner
    // against the opposite wall and continue the plank floor underneath the lip.
    let apron = room.blocks.get_mut(64..72).context("entrance apron")?;
    ensure!(
        apron == [0x05, 0x12, 0x12, 0x12, 0x12, 0x12, 0x06, 0x09],
        "unexpected entrance floor or statues"
    );
    apron[6] = 0x12;
    apron[7] = 0x06;
    let guide = room
        .objects
        .iter_mut()
        .find(|object| object.script == GUIDE)
        .context("Mankato Gym guide")?;
    ensure!(
        (guide.x, guide.y) == (9, 16) && guide.spritemovedata == "SPRITEMOVEDATA_STANDING_DOWN",
        "unexpected guide position or facing"
    );
    guide.y = 17;
    guide.spritemovedata = "SPRITEMOVEDATA_STANDING_LEFT".into();
    refresh_event_section(&mut room);
    authored_town_publication::replace(&mut pack, room)?;

    pack.report
        .manifests
        .push("northstar-quarry-landings-v1".into());
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
