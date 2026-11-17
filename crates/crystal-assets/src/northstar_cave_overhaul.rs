//! Additive cave revision, guarded by each installed room's exact preimage.

use anyhow::{Context, Result, ensure};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::northstar_cave_design::{MANIFEST, Revision};
use crate::{CompiledGamePack, MapModule};

/// Installs eight cave revisions on a parent whose cave programs are unchanged.
///
/// Whole-pack identity is deliberately not fixed: reviewed Gym changes may be
/// installed first. Existing cave actors, warps, story programs, item receipts,
/// encounters, palettes and map dimensions survive. New actors append at the end.
/// The caller owns pack identity derivation and bounded save-position migration.
///
/// # Errors
/// Rejects source/preimage drift, inaccessible terrain, inactive regional teams,
/// invalid text, or replacement of any retained cave identity/program.
/// A rejected installation leaves the input pack unchanged.
pub(crate) fn install(pack: &mut CompiledGamePack, design: &str, story: &str) -> Result<()> {
    let revision: Revision = serde_json::from_str(design)?;
    revision.validate()?;
    ensure!(
        !pack.report.manifests.iter().any(|id| id == MANIFEST),
        "cave revision already installed"
    );
    for cave in &revision.rooms {
        let room = pack.data.maps.get(&cave.id).context("installed cave")?;
        ensure!(
            preimage(room)? == cave.preimage_sha256
                && [room.attributes.width, room.attributes.height] == cave.dimensions_metatiles
                && room.attributes.tileset_name == cave.tileset,
            "installed cave differs from reviewed preimage: {}",
            cave.id
        );
    }
    let mut revised = pack.clone();
    for cave in &revision.rooms {
        let original = pack.data.maps.get(&cave.id).context("installed cave")?;
        let mut room = original.clone();
        room.blocks = cave.blocks()?;
        validate_native_art(&revised, &room)?;
        for (id, at) in &cave.actor_positions {
            let actor = room
                .objects
                .iter_mut()
                .find(|actor| actor.object_identifier.as_deref() == Some(id))
                .context("retained cave actor")?;
            actor.x = at[0];
            actor.y = at[1];
        }
        for (id, sight) in &cave.retained_sightlines {
            let actor = room
                .objects
                .iter_mut()
                .find(|actor| actor.object_identifier.as_deref() == Some(id))
                .context("retained cave sightline actor")?;
            ensure!(
                actor.object_type == "OBJECTTYPE_TRAINER",
                "retained cave battle actor"
            );
            let direction = sight.direction.to_ascii_uppercase();
            actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{direction}");
            actor.sightline_direction_override = Some(direction);
            actor.radius = sight.radius;
        }
        crate::northstar_cave_encounters::install(&mut revised, &mut room, cave, story)?;
        crate::northstar_cave_geometry::validate(&revised, &room, cave)
            .with_context(|| format!("cave geometry {}", cave.id))?;
        ensure!(
            room.events.warps == original.events.warps
                && room.events.coord_events == original.events.coord_events
                && room.objects[..original.objects.len()]
                    .iter()
                    .zip(&original.objects)
                    .all(
                        |(after, before)| after.object_identifier == before.object_identifier
                            && after.script == before.script
                            && after.event_flag == before.event_flag
                            && after.object_type == before.object_type
                    ),
            "preserve cave entrances, story events and saved actor identities"
        );
        for (label, program) in &original.scripts {
            // Only reviewed plain text may replace an existing program.
            if room.scripts.get(label) != Some(program) {
                ensure!(
                    original.script_text_bodies.contains_key(label)
                        && program
                            .as_array()
                            .is_some_and(|commands| commands.iter().all(|command| matches!(
                                command["command"].as_str(),
                                Some("text" | "line" | "para" | "cont" | "done" | "prompt")
                            ))),
                    "retained cave program changed: {label}"
                );
            }
        }
        crate::northstar_story_revision::publish(&mut revised, room)?;
    }
    revised.report.manifests.push(MANIFEST.into());
    // Browser save filenames include the composed manifest ID. Give these
    // eight rooms a distinct slot from the earlier five-cave review build.
    revised
        .report
        .manifests
        .push("northstar-cave-trails-v1".into());
    *pack = revised;
    Ok(())
}

pub(crate) fn validate_native_art(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let tileset = &room.attributes.tileset_name;
    let definition = pack
        .data
        .tilesets
        .get(tileset)
        .context("native cave tileset")?;
    let files = pack.runtime_files();
    let layout = files
        .get(&format!("data/tilesets/{tileset}_metatiles.bin"))
        .context("native cave metatiles")?;
    let graphics = files
        .get(&format!("gfx/tilesets/{tileset}.2bpp"))
        .context("native cave glyphs")?;
    ensure!(graphics.len() % 16 == 0, "complete native cave glyph data");
    for block in room
        .blocks
        .iter()
        .copied()
        .chain([u16::from(room.attributes.border_block)])
    {
        let offset = usize::from(block) * 16;
        let glyphs = layout
            .get(offset..offset + 16)
            .context("defined cave metatile")?;
        ensure!(
            glyphs.iter().any(|&glyph| glyph != u8::MAX),
            "undefined all-FF cave metatile"
        );
        ensure!(
            definition.collision.contains_key(&format!("{block:02x}")),
            "defined native cave collision"
        );
        for &glyph in glyphs {
            let palette = definition
                .palette_map
                .get(usize::from(glyph))
                .context("defined native cave glyph palette")?;
            let physical = if palette & 8 == 0 {
                usize::from(glyph)
            } else {
                usize::from(glyph & 127) + graphics.len() / 32
            };
            ensure!(
                graphics.get(physical * 16..(physical + 1) * 16).is_some(),
                "cave glyph references an absent physical graphics bank"
            );
        }
    }
    Ok(())
}

pub(crate) fn preimage(room: &MapModule) -> Result<String> {
    // Value normalizes nested object keys before hashing, independent of Rust
    // struct field declaration order or the pack's CBOR map ordering.
    let value = json!({
        "blocks":room.blocks, "objects":room.objects,
        "events":room.events, "scripts":room.scripts,
    });
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(&value)?)))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::northstar_cave_design::{MAPS, TRAINERS};

    #[test]
    #[ignore = "requires external CRYSTAL_CAVE_PARENT; static pack data only"]
    fn cave_overlay_preserves_receipts_and_rejects_changed_preimages() {
        let base = crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_CAVE_PARENT").expect("external unchanged cave parent"),
        ))
        .expect("verified cave parent");
        let source =
            include_str!("../../../planning/minnesota/caves/continuation-20261117/rooms.json");
        let story = include_str!("../../../planning/minnesota/story.json");
        let revision: crate::northstar_cave_design::Revision =
            serde_json::from_str(source).expect("cave contracts");
        let mut revised = base.clone();
        super::install(&mut revised, source, story)
            .expect("native cave geometry and event contracts");
        assert!(
            revised
                .runtime_modpack_id()
                .expect("eight-cave save slot")
                .split('+')
                .any(|part| part == "northstar-cave-trails-v1")
        );
        for (name, value) in &base.data.story_event_script_constants.global {
            assert_eq!(
                revised.data.story_event_script_constants.global.get(name),
                Some(value),
                "retained saved event/party identifier {name}"
            );
        }
        for (name, map) in &base.data.maps {
            if !MAPS.contains(&name.as_str()) {
                assert_eq!(
                    revised.data.maps.get(name),
                    Some(map),
                    "unrelated room {name}"
                );
            } else {
                let after = &revised.data.maps[name];
                let design = revision
                    .rooms
                    .iter()
                    .find(|room| room.id == *name)
                    .expect("authored cave");
                assert_eq!(map.events.warps, after.events.warps);
                assert_eq!(map.attributes, after.attributes);
                for (before, retained) in map.objects.iter().zip(&after.objects) {
                    let mut expected = before.clone();
                    if let Some(id) = before.object_identifier.as_deref() {
                        if let Some(at) = design.actor_positions.get(id) {
                            [expected.x, expected.y] = *at;
                        }
                        if let Some(sight) = design.retained_sightlines.get(id) {
                            let direction = sight.direction.to_ascii_uppercase();
                            expected.spritemovedata =
                                format!("SPRITEMOVEDATA_STANDING_{direction}");
                            expected.sightline_direction_override = Some(direction);
                            expected.radius = sight.radius;
                        }
                    }
                    assert_eq!(
                        &expected, retained,
                        "only declared old actor adjustments survive"
                    );
                }
                for pickup in &map.script_field_pickups {
                    assert!(
                        after.script_field_pickups.contains(pickup),
                        "existing pickup receipt survives"
                    );
                }
            }
        }
        assert_eq!(
            revised.data.trainers.trainers.len(),
            base.data.trainers.trainers.len() + TRAINERS.len()
        );
        assert_eq!(revised.data.wild_encounters, base.data.wild_encounters);
        assert_eq!(revised.runtime_files, base.runtime_files);
        assert_eq!(revised.audio_manifest, base.audio_manifest);
        let mut drifted = base.clone();
        drifted
            .data
            .maps
            .get_mut(MAPS[0])
            .expect("source cave")
            .blocks[0] ^= 1;
        let before = drifted.data.clone();
        assert!(super::install(&mut drifted, source, story).is_err());
        assert_eq!(
            before, drifted.data,
            "failed preimage check cannot partially install content"
        );
    }
}
