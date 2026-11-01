//! The faithful walking event bends existing native blades; no sprite or mesh rebuild.
use super::*;
use bevy::render::render_resource::ShaderType;
use crystal_render_api::{VisualEffectId, VisualPlayerTraversal};

#[derive(Clone, Copy, Debug, Default, Reflect, ShaderType, PartialEq)]
pub(super) struct GrassUniform {
    pub contact: Vec4,
    /// X: zero selects native height; retained battles remove LCD display zoom.
    /// Y: blossom animation clock, shared by forward and shadow vertex passes.
    pub height_scale: Vec4,
}

#[derive(Resource, Default)]
struct Rustle {
    map: std::sync::Arc<str>,
    present: bool,
    endpoint: Option<Vec2>,
    center: Vec2,
    age: f32,
    radius: f32,
}

pub(super) fn is_rustle(actor: &VisualActor) -> bool {
    actor.id == VisualActorId::Effect(VisualEffectId::GrassRustle)
}

pub(super) fn retained_height_scale(tile_height: f32) -> Vec4 {
    Vec4::new(SOURCE_TILE_HEIGHT / tile_height, 0.0, 0.0, 0.0)
}

/// Retain the exact terrain finish on a material owned only by rooted grass.
/// A walking spring must not invalidate buildings, trees or actors.
pub(super) fn material(
    cached: &mut Option<Handle<VoxelMaterial>>,
    source: &Handle<VoxelMaterial>,
    materials: &mut Assets<VoxelMaterial>,
) -> Handle<VoxelMaterial> {
    let material = materials.get(source).expect("published solid terrain material").clone();
    if let Some(handle) = cached {
        *materials.get_mut(&*handle).expect("retained grass material") = material;
        return handle.clone();
    }
    let handle = materials.add(material);
    *cached = Some(handle.clone());
    handle
}

pub(super) fn register(app: &mut App) {
    app.init_resource::<Rustle>().add_systems(
        Update,
        sync.after(sync_voxel_view)
            .in_set(WorldRenderSet::RenderSync),
    );
}

impl Rustle {
    fn advance(
        &mut self,
        frame: &VisualWorldFrame,
        active: bool,
        traversal: Option<&VisualPlayerTraversal>,
        dt: f32,
    ) -> GrassUniform {
        if !active || !frame.active || self.map != frame.map_id {
            *self = Self {
                map: frame.map_id.clone(),
                ..default()
            };
        }
        if !active || !frame.active {
            return default();
        }
        self.age += dt;
        // Published coordinates move with the LCD crop. Keep the contact in
        // source-map space so the recovering patch stays planted while scrolling.
        let offset = frame.center
            - Vec2::new(
                frame.grid_origin.x as f32 * frame.tile_size.x,
                -(frame.grid_origin.y as f32) * frame.tile_size.y,
            );
        let endpoint = traversal
            .filter(|t| t.map_id == frame.map_id)
            .and_then(|t| t.step.map(|s| (s.to - offset).round()));
        let present = frame.actors.iter().any(is_rustle);
        if present {
            if !self.present || (endpoint.is_some() && endpoint != self.endpoint) {
                self.age = 0.0;
            }
            if let Some(player) = frame.actors.iter().find(|a| a.id == VisualActorId::Player) {
                let foot = footing::actor_ground_point(frame, player);
                self.center = foot - offset;
                self.radius = frame.tile_size.min_element() * 1.5;
            }
        }
        self.present = present;
        self.endpoint = endpoint;
        // A quick outward push followed by a smaller rebound. Finish the spring
        // even after the source's six-frame card expires, then stop all updates.
        let phase = (self.age / 0.28).clamp(0.0, 1.0);
        let spring = (phase * std::f32::consts::TAU).sin() * (1.0 - phase);
        GrassUniform {
            height_scale: Vec4::ZERO,
            contact: if self.radius > 0.0 && phase < 1.0 {
                Vec4::new(
                    self.center.x + offset.x,
                    -self.center.y - offset.y,
                    self.radius,
                    spring,
                )
            } else {
                Vec4::ZERO
            },
        }
    }
}

fn sync(
    frame: Res<VisualWorldFrame>,
    status: Res<VoxelViewStatus>,
    traversal: Option<Res<VisualPlayerTraversal>>,
    time: Res<Time>,
    mut rustle: ResMut<Rustle>,
    cache: Res<TerrainRevisionCache>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut previous_material: Local<Option<Handle<VoxelMaterial>>>,
) {
    let next = rustle.advance(
        &frame,
        status.active,
        traversal.as_deref(),
        time.delta_secs(),
    );
    if previous_material.as_ref() != cache.grass_material.as_ref() {
        // A promoted destination retains the previous room's material in its
        // hidden cache. Settle it once so that stale contact cannot keep the
        // conservative shadow-culling safety guard active indefinitely.
        if let Some(handle) = previous_material.take() {
            if materials.get(&handle).is_some_and(|m| m.extension.grass_rustle.contact != Vec4::ZERO) {
                if let Some(mut material) = materials.get_mut(&handle) {
                    material.extension.grass_rustle.contact = Vec4::ZERO;
                }
            }
        }
        *previous_material = cache.grass_material.clone();
    }
    if let Some(handle) = cache.grass_material.as_ref() {
        if materials.get(handle).is_some_and(|m| m.extension.grass_rustle.contact != next.contact) {
            if let Some(mut material) = materials.get_mut(handle) {
                material.extension.grass_rustle.contact = next.contact;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> VisualWorldFrame {
        let player = VisualActor {
            id: VisualActorId::Player,
            source_id: "player".into(),
            model_palette: None,
            texture: default(),
            center: Vec2::new(12.0, 24.0),
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
        };
        let mut effect = player.clone();
        effect.id = VisualActorId::Effect(VisualEffectId::GrassRustle);
        VisualWorldFrame {
            active: true,
            map_id: "Route29".into(),
            tile_size: Vec2::splat(8.0),
            actors: vec![player, effect],
            ..default()
        }
    }
    #[test]
    fn battle_height_normalization_survives_overworld_rustle_reset() {
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VoxelViewStatus>()
            .init_resource::<Time>()
            .init_resource::<Rustle>()
            .init_resource::<TerrainRevisionCache>()
            .init_resource::<Assets<VoxelMaterial>>()
            .add_systems(Update, sync);
        let mut material = voxel_material(StandardMaterial::default());
        material.extension.grass_rustle.contact = Vec4::ONE;
        material.extension.grass_rustle.height_scale = retained_height_scale(64.0);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(material);
        app.world_mut().resource_mut::<TerrainRevisionCache>().grass_material = Some(handle.clone());
        app.update();
        let materials = app.world().resource::<Assets<VoxelMaterial>>();
        let grass = materials.get(&handle).unwrap().extension.grass_rustle;
        assert_eq!(grass.contact, Vec4::ZERO);
        assert_eq!(grass.height_scale.x, 0.125);
    }

    #[test]
    fn walking_spring_never_modifies_static_actor_or_flower_materials() {
        use bevy::asset::{AssetEvent, AssetPlugin};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default())).init_asset::<VoxelMaterial>();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1. / 60.)));
        let (source, grass, actor, flower) = {
            let mut materials = app.world_mut().resource_mut::<Assets<VoxelMaterial>>();
            let source = materials.add(solid_terrain_material());
            let grass = material(&mut None, &source, &mut materials);
            let actor = materials.add(solid_terrain_material());
            let flower = materials.add(solid_terrain_material());
            (source, grass, actor, flower)
        };
        app.insert_resource(frame())
            .insert_resource(VoxelViewStatus { active: true, ..default() })
            .insert_resource(TerrainRevisionCache {
                solid_material: Some(source.clone()), grass_material: Some(grass.clone()),
                animated_solid_material: Some(flower.clone()), ..default()
            })
            .init_resource::<Rustle>()
            .add_systems(Update, sync);
        app.update();
        app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>().clear();
        let mut modified = 0;
        let mut positive = false;
        let mut rebound = false;
        for _ in 0..64 {
            app.update();
            let events = app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>()
                .drain().collect::<Vec<_>>();
            modified += usize::from(events.iter().any(|e| e.is_modified(&grass)));
            for handle in [&source, &actor, &flower] {
                assert!(!events.iter().any(|e| e.is_modified(handle)),
                    "walking must not invalidate a non-grass material");
            }
            let materials = app.world().resource::<Assets<VoxelMaterial>>();
            let contact = materials.get(&grass).unwrap().extension.grass_rustle.contact;
            positive |= contact.w > 0.0;
            rebound |= contact.w < 0.0;
            for handle in [&source, &actor, &flower] {
                assert_eq!(materials.get(handle).unwrap().extension.grass_rustle.contact, Vec4::ZERO);
            }
        }
        assert!(modified > 5 && positive && rebound);
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().get(&grass).unwrap()
            .extension.grass_rustle.contact, Vec4::ZERO);
    }

    #[test]
    fn destination_handoff_settles_previous_grass_without_touching_static_finish() {
        use bevy::asset::{AssetEvent, AssetPlugin};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default())).init_asset::<VoxelMaterial>();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1. / 60.)));
        let (source, grass, destination) = {
            let mut materials = app.world_mut().resource_mut::<Assets<VoxelMaterial>>();
            let source = materials.add(solid_terrain_material());
            let grass = material(&mut None, &source, &mut materials);
            let destination = material(&mut None, &source, &mut materials);
            (source, grass, destination)
        };
        app.insert_resource(frame())
            .insert_resource(VoxelViewStatus { active: true, ..default() })
            .insert_resource(TerrainRevisionCache { grass_material: Some(grass.clone()), ..default() })
            .init_resource::<Rustle>()
            .add_systems(Update, sync);
        app.update(); app.update();
        assert!(app.world().resource::<Assets<VoxelMaterial>>().get(&grass).unwrap()
            .extension.grass_rustle.contact.w > 0.0);
        app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>().clear();
        app.world_mut().resource_mut::<TerrainRevisionCache>().grass_material = Some(destination.clone());
        app.update();
        let materials = app.world().resource::<Assets<VoxelMaterial>>();
        assert_eq!(materials.get(&grass).unwrap().extension.grass_rustle.contact, Vec4::ZERO);
        assert!(materials.get(&destination).unwrap().extension.grass_rustle.contact.w > 0.0);
        let events = app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>()
            .drain().collect::<Vec<_>>();
        assert!(events.iter().any(|e| e.is_modified(&grass)));
        assert!(!events.iter().any(|e| e.is_modified(&source)));
        app.update();
        let events = app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>()
            .drain().collect::<Vec<_>>();
        assert!(!events.iter().any(|e| e.is_modified(&grass)), "hidden grass settles only once");
        app.world_mut().resource_mut::<TerrainRevisionCache>().grass_material = None;
        app.update();
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().get(&destination).unwrap()
            .extension.grass_rustle.contact, Vec4::ZERO);
    }

    #[test]
    fn native_grass_rustle_pushes_rebounds_and_settles_without_the_card() {
        let mut frame = frame();
        let mut state = Rustle::default();
        state.advance(&frame, true, None, 0.0);
        let push = state.advance(&frame, true, None, 0.06).contact;
        assert_eq!(push.truncate(), Vec3::new(12.0, -24.0, 12.0));
        assert!(push.w > 0.5);
        frame.actors.pop();
        let rebound = state.advance(&frame, true, None, 0.14).contact;
        assert!(rebound.w < 0.0);
        assert_eq!(state.advance(&frame, true, None, 0.1).contact, Vec4::ZERO);
        assert_eq!(state.advance(&frame, true, None, 1.0).contact, Vec4::ZERO);
    }
    #[test]
    fn native_grass_rustle_retriggers_for_consecutive_steps_and_tracks_scroll() {
        use crystal_render_api::VisualTraversalStep;
        let mut frame = frame();
        let mut traversal = VisualPlayerTraversal {
            map_id: frame.map_id.clone(),
            step: Some(VisualTraversalStep {
                from: Vec2::ZERO,
                to: Vec2::X * 16.0,
                progress: 0.5,
                ledge_jump: false,
            }),
        };
        let mut state = Rustle::default();
        state.advance(&frame, true, Some(&traversal), 0.0);
        assert!(
            state
                .advance(&frame, true, Some(&traversal), 0.06)
                .contact
                .w
                > 0.5
        );
        traversal.step.as_mut().unwrap().to.x += 16.0;
        frame.actors[0].center += Vec2::new(16.0, 8.0);
        let restarted = state.advance(&frame, true, Some(&traversal), 0.02).contact;
        assert_eq!(restarted, Vec4::new(28.0, -32.0, 12.0, 0.0));
        assert!(
            state
                .advance(&frame, true, Some(&traversal), 0.06)
                .contact
                .w
                > 0.5
        );
        frame.actors.pop();
        state.advance(&frame, true, Some(&traversal), 0.4);
        frame.actors.push(frame.actors[0].clone());
        frame.actors[1].id = VisualActorId::Effect(VisualEffectId::GrassRustle);
        assert_eq!(
            state
                .advance(&frame, true, Some(&traversal), 0.01)
                .contact
                .w,
            0.0
        );
    }

    #[test]
    fn native_grass_rustle_keeps_recovery_on_the_same_source_patch_during_scroll() {
        let mut frame = frame();
        let mut state = Rustle::default();
        state.advance(&frame, true, None, 0.0);
        frame.actors.pop();
        frame.center += Vec2::new(3.0, -5.0);
        let scrolled = state.advance(&frame, true, None, 0.06).contact;
        assert_eq!(scrolled.truncate(), Vec3::new(15.0, -19.0, 12.0));
        frame.grid_origin += IVec2::new(1, 2);
        let recropped = state.advance(&frame, true, None, 0.0).contact;
        assert_eq!(recropped.truncate(), Vec3::new(7.0, -35.0, 12.0));
        assert_eq!(scrolled.w, recropped.w);
    }

    #[test]
    fn native_grass_rustle_clears_on_map_changes_and_3d_disable() {
        let mut frame = frame();
        let mut state = Rustle::default();
        state.advance(&frame, true, None, 0.0);
        assert!(state.advance(&frame, true, None, 0.06).contact.w > 0.0);
        assert_eq!(state.advance(&frame, false, None, 0.01).contact, Vec4::ZERO);
        frame.actors.pop();
        frame.map_id = "NewBarkTown".into();
        assert_eq!(state.advance(&frame, true, None, 0.01).contact, Vec4::ZERO);
    }
}
