//! The final Fair release binds its authored layout to the two actual Lakeside archives.
use crate::{CompiledGamePack, LoadedCompiledGamePack, MapModule};
use anyhow::{Context, Result, ensure};
use crystal_core::world::session::object_tile_position_checked;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const POLICY: &str = "data/northstar-save-compatibility.json";
const MARKER: &str = "northstar-fair-release-v1";

/// Seals the selected Fair artwork and save migration into a fresh release pack.
///
/// # Errors
/// Rejects other source editions, a changed object index, or a missing saved reference.
pub fn build_northstar_fair_release(
    mut next: CompiledGamePack,
    previous: &LoadedCompiledGamePack,
    public: &LoadedCompiledGamePack,
) -> Result<CompiledGamePack> {
    let old = previous.pack();
    let raw = previous.save_modpack_identity()?;
    let hosted = public.save_modpack_identity()?;
    ensure!(
        next.identity.content_hash
            == "ebeb151d58fb7c716dc58cdbcde0c4fc196de0441e6634b553312e6ccd889f86",
        "expected the selected Fair Patrols Intro artwork"
    );
    ensure!(
        raw.hash() == "e56ded3d55b8364e8756a4a02eb530fb74f52c03951c1c01d962441f7e2193d9"
            && hosted.hash() == "ad775429c71fc48aabbc8e1e2db40d3594a209dbc81e8b959fed1b73f5486426",
        "expected the released Lakeside archives"
    );
    let inherited: Value = serde_json::from_slice(
        old.runtime_files
            .get(POLICY)
            .context("Lakeside save policy")?,
    )?;
    ensure!(
        inherited["schema_version"] == 8
            && old.runtime_files.get(POLICY) == public.pack().runtime_files.get(POLICY),
        "mismatched Lakeside policy"
    );
    ensure!(
        old.data
            .pokemon
            .keys()
            .all(|id| next.data.pokemon.contains_key(id))
            && old
                .data
                .items
                .keys()
                .all(|id| next.data.items.contains_key(id))
            && old
                .data
                .moves
                .keys()
                .all(|id| next.data.moves.contains_key(id)),
        "saved catalog identifiers removed"
    );
    let mut layouts = BTreeMap::new();
    for (name, before) in &old.data.maps {
        let after = next.data.maps.get(name).context("saved map removed")?;
        ensure!(
            before.attributes.width == after.attributes.width
                && before.attributes.height == after.attributes.height,
            "resized map {name} needs a coordinate migration"
        );
        let before_set = old
            .data
            .tileset_collision(&before.attributes.tileset_name)?;
        let after_set = next
            .data
            .tileset_collision(&after.attributes.tileset_name)?;
        let mut rebind = Vec::new();
        let mut cells = Vec::new();
        let width = usize::from(after.attributes.width);
        for (i, (&a, &b)) in before.blocks.iter().zip(&after.blocks).enumerate() {
            let (x, y) = (i % width, i / width);
            let old_types = before_set
                .metatiles
                .get(usize::from(a))
                .context("old block")?
                .collision;
            let new_types = after_set
                .metatiles
                .get(usize::from(b))
                .context("new block")?
                .collision;
            if a != b || old_types != new_types {
                rebind.push([x, y]);
            }
            for (q, (&prior, &new)) in old_types.iter().zip(&new_types).enumerate() {
                if prior != new {
                    cells.push([x * 2 + q % 2, y * 2 + q / 2, usize::from(prior)]);
                }
            }
        }
        let mut actors = Vec::new();
        for (i, actor) in before.objects.iter().enumerate() {
            let new = after.objects.get(i).context("saved object removed")?;
            ensure!(
                actor.object_identifier == new.object_identifier,
                "changed save object index {name} {i}"
            );
            if (
                actor.x,
                actor.y,
                &actor.spritemovedata,
                actor.move_range_x,
                actor.move_range_y,
            ) != (
                new.x,
                new.y,
                &new.spritemovedata,
                new.move_range_x,
                new.move_range_y,
            ) {
                actors.push(i + 1);
            }
        }
        let events = |room: &MapModule| -> BTreeSet<[u16; 2]> {
            room.events
                .warps
                .iter()
                .map(|e| [e.x, e.y])
                .chain(room.events.coord_events.iter().map(|e| [e.x, e.y]))
                .collect()
        };
        let triggers: Vec<_> = events(after).difference(&events(before)).copied().collect();
        let additions: Vec<_> = after
            .objects
            .iter()
            .skip(before.objects.len())
            .filter_map(object_tile_position_checked)
            .map(|p| [p.x, p.y])
            .collect();
        if !rebind.is_empty()
            || !actors.is_empty()
            || !triggers.is_empty()
            || !additions.is_empty()
            || before.script_block_changes != after.script_block_changes
        {
            let changes: BTreeSet<_> = before
                .script_block_changes
                .iter()
                .map(|c| [c.x / 2, c.y / 2, c.block_id])
                .collect();
            let kept: BTreeSet<_> = after
                .script_block_changes
                .iter()
                .map(|c| [c.x / 2, c.y / 2, c.block_id])
                .collect();
            let removed: Vec<_> = changes.difference(&kept).copied().collect();
            let mut replacements = Vec::new();
            for &[x, y, id] in &removed {
                // An earned gate or switch can change its tile artwork. Require every
                // producer of the saved value to agree on the same replacement.
                let mut candidates = BTreeSet::new();
                let complete = before
                    .script_block_changes
                    .iter()
                    .filter(|c| [c.x / 2, c.y / 2, c.block_id] == [x, y, id])
                    .all(|old| {
                        let found: Vec<_> = after
                            .script_block_changes
                            .iter()
                            .filter(|new| {
                                new.source_script == old.source_script
                                    && new.x == old.x
                                    && new.y == old.y
                            })
                            .map(|new| new.block_id)
                            .collect();
                        candidates.extend(found.iter().copied());
                        !found.is_empty()
                    });
                if complete && candidates.len() == 1 {
                    replacements.push([
                        x,
                        y,
                        id,
                        *candidates.first().context("missing replacement")?,
                    ]);
                }
            }
            layouts.insert(name, json!({"blocks":rebind, "prior_tiles":cells, "actors":actors,"triggers":triggers, "removed_changes":removed, "replacements":replacements}));
        }
    }
    next.report.manifests.push(MARKER.into());
    let id = next.runtime_modpack_id()?;
    let policy = json!({"schema_version":9, "target_manifest":MARKER, "target_modpack_ids":[id,format!("{id}+realtime-clock+player-customization")],
        "sources":[{"modpack_id":raw.id(),"modpack_hash":raw.hash(),"content_hash":previous.pack_content_hash()?},{"modpack_id":hosted.id(),"modpack_hash":hosted.hash(),"content_hash":public.pack_content_hash()?}],
        "inherited":inherited, "layouts":layouts});
    next.runtime_files
        .insert(POLICY.into(), serde_json::to_vec(&policy)?);
    next.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        next.format_version,
        &next.data,
        &next.audio_manifest,
        &next.runtime_files,
        &next.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&next)?;
    Ok(next)
}
