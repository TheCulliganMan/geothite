//! Expand distant NPC presentation only when the shared snapshot needs a write.
//!
//! The gameplay object roster still owns movement, interaction, and RNG.
//! Distant actors retain their last loaded pose only in the presentation view.

use std::sync::Arc;

use anyhow::{Context, Result};
use crystal_runtime::RuntimeShellSnapshot;

use super::BevyRuntimeShell;

pub(super) fn expand_shared(
    snapshot: &mut Arc<RuntimeShellSnapshot>,
    runtime: &BevyRuntimeShell,
) -> Result<()> {
    let world = runtime.shell.session().overworld();
    if snapshot.battle.is_some() || snapshot.overworld.map_name != world.map.name {
        return Ok(());
    }
    if (0..world.objects.len()).any(|index| object_is_missing(snapshot, runtime, index)) {
        expand_fullscreen_object_presentation(Arc::make_mut(snapshot), runtime)?;
    }
    Ok(())
}

fn object_is_missing(
    snapshot: &RuntimeShellSnapshot,
    runtime: &BevyRuntimeShell,
    index: usize,
) -> bool {
    let world = runtime.shell.session().overworld();
    !snapshot.visible_object_slots.contains(&index)
        && !world.object_has_loaded_struct(index)
        && world.object_is_eligible_for_expanded_view(index)
        && !world.objects[index]
            .object_identifier
            .as_ref()
            .is_some_and(|id| world.invisible_object_struct_identifiers.contains(id))
}

pub(super) fn expand_fullscreen_object_presentation(
    snapshot: &mut RuntimeShellSnapshot,
    runtime: &BevyRuntimeShell,
) -> Result<()> {
    let world = runtime.shell.session().overworld();
    if snapshot.battle.is_some() || snapshot.overworld.map_name != world.map.name {
        return Ok(());
    }
    for (index, object) in world.objects.iter().enumerate() {
        if !object_is_missing(snapshot, runtime, index) {
            continue;
        }
        let mut tile = world
            .object_runtime_tile_checked(index, object)
            .context("resolve distant fullscreen NPC coordinates")?;
        if let Some(id) = object.object_identifier.as_ref() {
            let mut facing =
                world.object_facings.get(id).copied().with_context(|| {
                    format!("fullscreen NPC {id} has no source-initialized facing")
                })?;
            if let Some((last_tile, last_facing)) = world.unloaded_object_presentation.get(id) {
                tile = *last_tile;
                facing = *last_facing;
            }
            snapshot
                .visible_object_runtime_tiles
                .insert(id.clone(), tile);
            snapshot.visible_object_facings.insert(id.clone(), facing);
        }
        snapshot.visible_objects.push(object.clone());
        snapshot.visible_object_slots.push(index);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
    use crystal_runtime::CrystalRuntime;

    use super::super::{BevyShellConfig, BevyShellStart};

    #[test]
    fn shared_expansion_copies_only_when_distant_objects_are_missing() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.repository_root
                    .join("content-packs/text-tui.crystalpack")
            });
        let loaded = read_loaded_verified_compiled_game_pack(path).expect("load external pack");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)
            .expect("construct production runtime");
        for (map, x, y, needs_expansion) in [
            ("PlayersHouse2F", 3, 3, false),
            ("GoldenrodCity", 14, 8, true),
        ] {
            let shell = super::super::initialize_bevy_runtime_shell(
                root.clone(),
                runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier: 14,
                    map_name: map.to_string(),
                    tile_x: x,
                    tile_y: y,
                },
                BevyShellConfig::default(),
            )
            .expect("initialize field presentation");
            let original = Arc::new(shell.shell.presentation_snapshot().expect("field snapshot"));
            let mut expected = (*original).clone();
            super::expand_fullscreen_object_presentation(&mut expected, &shell)
                .expect("owned expansion");
            let mut expanded = Arc::clone(&original);
            super::expand_shared(&mut expanded, &shell).expect("shared expansion");
            assert_eq!(*expanded, expected);
            assert_eq!(Arc::ptr_eq(&original, &expanded), !needs_expansion);
            assert_eq!(
                *original,
                shell
                    .shell
                    .presentation_snapshot()
                    .expect("unchanged authority")
            );
            let retained = Arc::clone(&expanded);
            super::expand_shared(&mut expanded, &shell).expect("idempotent expansion");
            assert!(Arc::ptr_eq(&retained, &expanded));
        }
    }
}
