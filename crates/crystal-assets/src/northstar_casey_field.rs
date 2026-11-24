//! Give the Northstar Champion a field sprite using Casey's installed portrait palette.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;
use crate::minnesota_native_npc;
use crate::minnesota_trainer_art;

const ROOM: &str = "MnLeagueChampion";
const SCRIPT: &str = "MnLeagueChampionChallenge";
const ORIGINAL: &str = "SPRITE_LANCE";
const SPRITE: &str = "SPRITE_MN_CASEY";
const STEM: &str = "mn_casey";
const POSES: &str = include_str!(
    "../../../planning/minnesota/npcs/portrait-cast-v01/champion/field-pixels-v1.json"
);

/// Installs Casey's native field bank and rebinds the Northstar Champion-room actor.
///
/// # Errors
/// Rejects stale field bindings, occupied sprite resources, missing Casey art,
/// or an invalid resulting pack.
pub fn build_northstar_casey_field(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let palette = base
        .runtime_files
        .get("gfx/trainers/mn_portrait_champion.gbcpal")
        .context("Casey's battle portrait palette")?;
    let (inks, remainder) = palette.as_chunks::<2>();
    let inks: [[u8; 2]; 4] = inks.try_into().context("four Casey portrait inks")?;
    let palette = inks.map(u16::from_le_bytes);
    ensure!(
        remainder.is_empty()
            && palette[0] == 32767
            && palette[3] == 0
            && palette[2] > 0
            && palette[2] < palette[1]
            && palette[1] < 32767,
        "invalid Casey portrait palette"
    );
    let art = minnesota_native_npc::compile_field(STEM, &palette, POSES)?;
    ensure!(
        !base.data.sprite_palette_defaults.contains_key(SPRITE)
            && art
                .keys()
                .all(|path| !base.runtime_files.contains_key(path)),
        "Casey's field sprite is already registered"
    );

    let mut room = base
        .data
        .maps
        .get(ROOM)
        .context("Northstar Champion room")?
        .clone();
    let actor = room
        .objects
        .iter_mut()
        .find(|actor| actor.script == SCRIPT && [actor.x, actor.y] == [5, 7])
        .context("Casey at (5, 7)")?;
    ensure!(
        actor.sprite == ORIGINAL && actor.sprite_has_facings,
        "unexpected Casey actor sprite"
    );
    actor.sprite = SPRITE.into();
    let event = room
        .map_event_section_commands
        .iter_mut()
        .find(|row| {
            row.command == "object_event"
                && row.args.get(11).is_some_and(|script| script == SCRIPT)
                && row
                    .args
                    .starts_with(&["5".into(), "7".into(), ORIGINAL.into()])
        })
        .context("original Casey object event")?;
    event.args[2] = SPRITE.into();
    minnesota_trainer_art::rebind_actor_event_source(&mut room, SCRIPT, [5, 7], SPRITE)?;

    let mut pack = base.clone();
    pack.data.sprite_palette_defaults.insert(SPRITE.into(), 0);
    pack.runtime_files.extend(art);
    authored_town_publication::replace(&mut pack, room)?;
    pack.report
        .manifests
        .push("northstar-casey-field-v1".into());
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
