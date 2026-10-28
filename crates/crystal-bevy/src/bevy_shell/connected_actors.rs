//! Read-only neighboring map actors inside the retained outdoor window.

use std::collections::VecDeque;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use bevy::prelude::{Assets, IVec2, Image, Vec2};
use crystal_core::state::EventFlagMemory;
use crystal_core::world::session::OverworldSession;
use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame};

use super::{
    BevyRuntimeShell, Direction, RenderedTilesetArt, RuntimeShellSnapshot, TilePosition,
    object_event_initial_facing, resolve_visible_object_palette,
    resolve_visible_object_sprite_asset_id, sprite_frame_for_art,
    visible_effective_map_time_of_day, visible_map_object_model_palette,
    visible_map_object_model_source_from_data, visual_actor_intersects_grid, visual_facing,
};

/// Last presented source poses; these never become saved game objects.
#[derive(Default)]
pub(super) struct History {
    maps: VecDeque<(Arc<str>, Vec<VisualActor>)>,
    frame: Option<u64>,
    flags: Option<EventFlagMemory>,
}

impl History {
    fn remember(&mut self, window: &VisualWorldFrame, actors: &[VisualActor], frame: u64) {
        if self.frame.is_some_and(|previous| frame < previous) {
            self.maps.clear();
        }
        self.frame = Some(frame);
        if let Some(index) = self.maps.iter().position(|(map, _)| *map == window.map_id) {
            self.maps.remove(index);
        }
        let poses = actors
            .iter()
            .filter(|actor| matches!(actor.id, VisualActorId::Object(slot) if slot < 16))
            .map(|actor| {
                let mut actor = actor.clone();
                actor.center = (actor.center - window.center) * Vec2::new(1., -1.)
                    / window.tile_size
                    + window.grid_origin.as_vec2()
                    + window.grid_size.as_vec2() * 0.5;
                actor
            })
            .collect();
        self.maps.push_back((Arc::clone(&window.map_id), poses));
        while self.maps.len() > 4 {
            self.maps.pop_front();
        }
    }
}

pub(super) fn append(
    runtime: &BevyRuntimeShell,
    window: &VisualWorldFrame,
    orbit: f32,
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    actors: &mut Vec<VisualActor>,
    history: &mut History,
) -> Result<()> {
    let Some((_, snapshot)) = runtime.cached_snapshot.as_ref() else {
        return Ok(());
    };
    if snapshot.overworld.map_name != window.map_id.as_ref() {
        return Ok(());
    }
    let data = runtime.shell.runtime().data();
    let state = runtime.shell.session().state();
    // Entering a town can update its Fly and scene flags. Invalidate only
    // rosters whose own resident eligibility changed; unrelated flags must
    // not remove people on the outgoing side of the seam.
    if history.flags.as_ref() != Some(&state.flags) {
        if let Some(previous) = &history.flags {
            history.maps.retain(|(id, _)| {
                snapshot
                    .maps
                    .iter()
                    .find(|map| map.map_name == id.as_ref())
                    .is_some_and(|map| {
                        map.objects.iter().all(|object| {
                            previous.event_flags.get(&object.event_flag)
                                == state.flags.event_flags.get(&object.event_flag)
                        })
                    })
            });
        }
        history.flags = Some(state.flags.clone());
    }
    // Use the saved global cursor to invalidate poses after a save rollback.
    history.remember(window, actors, state.frame_counter);
    for region in window.source_regions.iter().skip(1) {
        let start = window.grid_origin - IVec2::splat(4);
        let end = window.grid_origin + window.grid_size.as_ivec2() + IVec2::splat(4);
        if (region.origin + region.size.as_ivec2()).cmple(start).any()
            || region.origin.cmpge(end).any()
        {
            continue;
        }
        let Some((map_index, map)) = snapshot
            .maps
            .iter()
            .enumerate()
            .find(|(_, map)| map.map_name == region.map_id.as_ref())
        else {
            continue;
        };
        if map.objects.is_empty() {
            continue;
        }
        if let Some((_, poses)) = history
            .maps
            .iter()
            .find(|(id, _)| id.as_ref() == region.map_id.as_ref())
        {
            // Keep the actual loaded-roster outcome when leaving a map. Core
            // saves one object image, so reinitializing this view would lose
            // its callback visibility and last presented movement pose.
            for pose in poses {
                let VisualActorId::Object(slot) = pose.id else {
                    continue;
                };
                let mut actor = pose.clone();
                actor.id = identifier(map_index, slot as usize)?;
                actor.center = window.center
                    + (region.origin.as_vec2() + pose.center
                        - window.grid_origin.as_vec2()
                        - window.grid_size.as_vec2() * 0.5)
                        * window.tile_size
                        * Vec2::new(1., -1.);
                if visual_actor_intersects_grid(&actor, window.center, window.grid_size) {
                    actors.push(actor);
                }
            }
            continue;
        }
        let module = data.map_module(&map.map_name)?;
        // An unvisited object callback has no presented outcome yet. Never
        // speculate its script execution, visibility changes or random input.
        if !state.map_object_overrides.contains_key(&map.map_name)
            && module
                .map_script_section_commands
                .iter()
                .any(|command| command.args.iter().any(|arg| arg == "MAPCALLBACK_OBJECTS"))
        {
            continue;
        }
        let mut roster = OverworldSession::with_events_and_objects(
            data.overworld_map(&map.map_name)?,
            module.events.clone(),
            module.objects.clone(),
            data.tileset_collision(&module.attributes.tileset_name)?,
            TilePosition::new(0, 0),
        );
        roster.frame = runtime.shell.session().overworld().frame;
        roster.set_time(state.time.registers.hours, state.time.time_of_day);
        roster.sync_event_flag_memory(&state.flags);
        data.apply_saved_overworld_overrides(&mut roster, state)?;
        let period = match snapshot.progression.time.time_of_day {
            crystal_core::world::encounters::TimeOfDay::Morning => "morn",
            crystal_core::world::encounters::TimeOfDay::Day => "day",
            crystal_core::world::encounters::TimeOfDay::Night => "nite",
        };
        let period = visible_effective_map_time_of_day(map, period, false);
        for (index, object) in roster.objects.iter().enumerate() {
            if !roster.is_object_visible(object) {
                continue;
            }
            let tile = roster
                .object_runtime_tile_checked(index, object)
                .context("neighbor actor coordinates")?;
            let point = region.origin + IVec2::new(i32::from(tile.x) * 2, i32::from(tile.y) * 2);
            let local = point - window.grid_origin;
            if local.cmplt(IVec2::splat(-4)).any()
                || local
                    .cmpge(window.grid_size.as_ivec2() + IVec2::splat(4))
                    .any()
            {
                continue;
            }
            let Some(sprite) = sprite_source(snapshot, &object.sprite) else {
                continue;
            };
            let sprite_id = resolve_visible_object_sprite_asset_id(
                &runtime.asset_root,
                sprite,
                &snapshot.script_events.variable_sprites,
                &snapshot.presentation.menu_icons,
            );
            let direction = object
                .object_identifier
                .as_ref()
                .and_then(|id| roster.object_facings.get(id).copied())
                .or_else(|| object_event_initial_facing(&object.spritemovedata))
                .context("neighbor actor facing")?;
            let side = match direction {
                Direction::Down => 0,
                Direction::Left => 1,
                Direction::Up => 2,
                Direction::Right => 3,
            };
            let view = match (side + (orbit / 2.).round().rem_euclid(4.) as usize) % 4 {
                0 => Direction::Down,
                1 => Direction::Left,
                2 => Direction::Up,
                _ => Direction::Right,
            };
            let palette = resolve_visible_object_palette(
                &object.sprite,
                object.pal,
                &snapshot.presentation.sprite_palette_defaults,
            );
            let picture = sprite_frame_for_art(
                art,
                &runtime.asset_root,
                &sprite_id,
                palette,
                period,
                view,
                false,
                images,
            )
            .context("neighbor actor source artwork")?;
            let source = visible_map_object_model_source_from_data(
                data,
                &map.map_name,
                object,
                sprite,
                &sprite_id,
                &snapshot.script_events.variable_sprites,
                &snapshot.presentation.menu_icons,
            );
            let model_palette = visible_map_object_model_palette(
                &runtime.asset_root,
                &map.map_name,
                object,
                sprite,
                &source,
                &snapshot.script_events.variable_sprites,
            )?;
            let corner = window.center
                + (local.as_vec2() - window.grid_size.as_vec2() * 0.5)
                    * window.tile_size
                    * Vec2::new(1., -1.);
            let actor = VisualActor {
                id: identifier(map_index, index)?,
                source_id: source.into(),
                model_palette,
                texture: picture.handle,
                size: picture.size,
                center: corner + picture.size * Vec2::new(0.5, -0.5),
                flip_x: false,
                above_priority: false,
                facing: Some(visual_facing(direction)),
            };
            if visual_actor_intersects_grid(&actor, window.center, window.grid_size) {
                actors.push(actor);
            }
        }
    }
    Ok(())
}

fn identifier(map: usize, slot: usize) -> Result<VisualActorId> {
    // Current-map slot IDs remain unchanged: encounter proofs consume them.
    // Neighbor identities occupy a separate, checked presentation namespace.
    // Authored regional cities exceed the original sixteen visible slots.
    // These are presentation identities, so reserve a full u16 for their
    // source index without changing the current map's authoritative slot IDs.
    ensure!(
        slot <= usize::from(u16::MAX),
        "neighbor object index exceeds presentation identity range"
    );
    let map = u32::try_from(map).context("neighbor map index")?;
    ensure!(
        map < 1 << 15,
        "neighbor map index exceeds presentation identity range"
    );
    let slot = u32::try_from(slot).context("neighbor object index")?;
    Ok(VisualActorId::Object(0x8000_0000 | map << 16 | slot))
}

fn sprite_source<'a>(snapshot: &'a RuntimeShellSnapshot, source: &'a str) -> Option<&'a str> {
    let resident = match source {
        "SPRITE_DAY_CARE_MON_1" => &snapshot.day_care.man,
        "SPRITE_DAY_CARE_MON_2" => &snapshot.day_care.lady,
        _ => return Some(source),
    };
    resident
        .pokemon
        .as_ref()
        .filter(|_| resident.active)
        .map(|pokemon| pokemon.species.id.as_str())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
    use crystal_runtime::CrystalRuntime;

    use super::super::{BevyShellConfig, BevyShellStart, initialize_bevy_runtime_shell};
    use super::*;

    #[test]
    fn expanded_neighbor_cities_keep_distinct_ids_beyond_sixteen_slots() {
        let mut ids = std::collections::HashSet::new();
        for map in [0, 1, 32767] {
            for slot in [0, 15, 16, 31, 255, 65535] {
                let id = identifier(map, slot).expect("authored city presentation identity");
                assert!(
                    ids.insert(id),
                    "neighbor objects must never share a rig identity"
                );
                assert_ne!(
                    id,
                    VisualActorId::Object(slot as u32),
                    "active-map IDs stay separate"
                );
            }
        }
        assert!(identifier(32768, 0).is_err());
        assert!(identifier(0, 65536).is_err());
    }

    #[test]
    fn connected_people_are_present_before_the_seam_and_respect_event_flags() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
        let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.repository_root
                    .join("content-packs/text-tui.crystalpack")
            });
        let loaded = read_loaded_verified_compiled_game_pack(pack).expect("verified external pack");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
        let mut shell = initialize_bevy_runtime_shell(
            root,
            runtime,
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: 14,
                map_name: "Route29".into(),
                tile_x: 2,
                tile_y: 8,
            },
            BevyShellConfig {
                render_test_hour: Some(12),
                ..Default::default()
            },
        )
        .expect("source-side controller");
        let snapshot = Arc::new(shell.shell.snapshot().expect("source snapshot"));
        let (map_index, town) = snapshot
            .maps
            .iter()
            .enumerate()
            .find(|(_, map)| map.map_name == "CherrygroveCity")
            .expect("connected town");
        let (slot, _) = town
            .objects
            .iter()
            .enumerate()
            .find(|(_, object)| object.sprite == "SPRITE_TEACHER")
            .expect("town teacher");
        let id = identifier(map_index, slot).expect("presentation identity");
        let (conditional_slot, conditional) = town
            .objects
            .iter()
            .enumerate()
            .find(|(_, object)| object.event_flag != "-1")
            .expect("conditional town resident");
        let conditional_id = identifier(map_index, conditional_slot).expect("conditional identity");
        let flag = conditional.event_flag.clone();
        let route = snapshot
            .maps
            .iter()
            .find(|map| map.map_name == "Route29")
            .expect("route");
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let window = super::super::destination_preload::source_frame(
            &snapshot,
            &shell,
            route,
            TilePosition::new(2, 8),
            "day",
            &mut art,
            &mut images,
        )
        .expect("connected source window");
        shell.cached_snapshot = Some((0, Arc::clone(&snapshot)));
        let before = shell
            .shell
            .state_checksum()
            .expect("authoritative checksum");
        let mut actors = Vec::new();
        let mut history = History::default();
        append(
            &shell,
            &window,
            0.,
            &mut art,
            &mut images,
            &mut actors,
            &mut history,
        )
        .expect("neighbor presentation");
        assert!(
            actors.iter().any(|actor| actor.id == id),
            "the teacher must be visible from Route 29 before map ownership changes"
        );
        assert_eq!(
            shell.shell.state_checksum().expect("unchanged authority"),
            before
        );
        assert!(
            actors.iter().all(
                |actor| matches!(actor.id, VisualActorId::Object(id) if id & 0x8000_0000 != 0)
            ),
            "neighbor IDs must not replace authoritative current-map object slots"
        );
        shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(&flag, false)
            .expect("visible resident event flag");
        actors.clear();
        append(
            &shell,
            &window,
            0.,
            &mut art,
            &mut images,
            &mut actors,
            &mut history,
        )
        .expect("visible conditional resident");
        assert!(actors.iter().any(|actor| actor.id == conditional_id));
        shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(&flag, true)
            .expect("hidden resident event flag");
        let hidden = shell
            .shell
            .state_checksum()
            .expect("hidden-object flag state");
        actors.clear();
        append(
            &shell,
            &window,
            0.,
            &mut art,
            &mut images,
            &mut actors,
            &mut history,
        )
        .expect("updated neighbor presentation");
        assert!(!actors.iter().any(|actor| actor.id == conditional_id));
        assert_eq!(
            shell
                .shell
                .state_checksum()
                .expect("unchanged hidden state"),
            hidden
        );

        // Arrival flags must not erase the outgoing presented roster.
        let source = shell.shell.session().overworld();
        let slot = 5;
        let tile = source
            .object_runtime_tile_checked(slot, &source.objects[slot])
            .expect("source resident position");
        let source_center = Vec2::new(f32::from(tile.x) * 2. + 1., f32::from(tile.y) * 2. + 1.);
        let pose = VisualActor {
            id: VisualActorId::Object(slot as u32),
            source_id: "cooltrainer_m".into(),
            model_palette: None,
            texture: Default::default(),
            center: window.center
                + (source_center - window.grid_origin.as_vec2() - window.grid_size.as_vec2() * 0.5)
                    * window.tile_size
                    * Vec2::new(1., -1.),
            size: Vec2::splat(64.),
            flip_x: false,
            above_priority: false,
            facing: Some(-Vec2::Y),
        };
        actors = vec![pose];
        append(
            &shell,
            &window,
            0.,
            &mut art,
            &mut images,
            &mut actors,
            &mut history,
        )
        .expect("remember outgoing presentation");
        let mut controller = super::super::VisibleShellController { shell };
        // The west entrance is at y=6/7; y=8 is a source tree border.
        for _ in 0..64 {
            if controller.shell.shell.session().overworld().player.tile.y <= 6 {
                break;
            }
            controller
                .press(super::super::GameButton::Up)
                .expect("reach west entrance");
        }
        for _ in 0..128 {
            controller
                .press(super::super::GameButton::Left)
                .expect("walk across seam");
            if controller.shell.shell.current_map_name() == "CherrygroveCity" {
                break;
            }
        }
        let after = Arc::new(controller.snapshot().expect("town snapshot"));
        assert_eq!(after.overworld.map_name, "CherrygroveCity");
        let town = after
            .maps
            .iter()
            .find(|map| map.map_name == "CherrygroveCity")
            .expect("town source map");
        let town_window = super::super::destination_preload::source_frame(
            &after,
            &controller.shell,
            town,
            after.overworld.tile,
            "day",
            &mut art,
            &mut images,
        )
        .expect("town window");
        controller.shell.cached_snapshot = Some((0, Arc::clone(&after)));
        actors.clear();
        append(
            &controller.shell,
            &town_window,
            0.,
            &mut art,
            &mut images,
            &mut actors,
            &mut history,
        )
        .expect("outgoing resident after crossing");
        let route_index = after
            .maps
            .iter()
            .position(|map| map.map_name == "Route29")
            .expect("route source identity");
        let id = identifier(route_index, slot).expect("outgoing presentation identity");
        let actor = actors
            .iter()
            .find(|actor| actor.id == id)
            .expect("the outgoing person must remain presented after the map changes");
        let region = town_window
            .source_regions
            .iter()
            .find(|region| region.map_id.as_ref() == "Route29")
            .expect("connected outgoing region");
        let expected = town_window.center
            + (region.origin.as_vec2() + source_center
                - town_window.grid_origin.as_vec2()
                - town_window.grid_size.as_vec2() * 0.5)
                * town_window.tile_size
                * Vec2::new(1., -1.);
        assert!(actor.center.abs_diff_eq(expected, 0.01));
    }
}
