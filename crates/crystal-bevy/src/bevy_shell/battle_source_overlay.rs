//! Fixed native HUD overlays for original source OAM. Fragment coordinates,
//! never per-object billboard dimensions, determine the sampled LCD pixel.
use super::*;
use bevy::{
    asset::load_internal_asset,
    reflect::TypePath,
    render::{render_resource::{AsBindGroup, ShaderRef, ShaderType}, view::RenderLayers},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use crystal_voxel_view::BattleSourceProjection;

const SOURCE_SLOTS: usize = 10;
const SOURCE_SHADER: Handle<Shader> = Handle::weak_from_u128(0x94aa7509_6eae_448a_a8a5_72ee8f7f3c29);

#[derive(Resource, Default)]
pub(super) struct ImmersiveBattleSourceOverlayEnabled {
    pub enabled: bool,
    // The earlier layer-sync system also parks source sprites when the arena
    // is active without a source frame. Restoring an overlay must honor that.
    pub park_classic: bool,
}

#[derive(Component)]
pub(super) struct SourceOverlaySlot(usize);

// The original Sprite remains untouched, including parenting, UV crop, size,
// texture and transform. Only its draw layer is parked while this slot draws.
#[derive(Component)]
pub(super) struct ImmersiveBattleSourceObjectLayout(Option<RenderLayers>);

#[derive(Clone, Debug, Default, PartialEq, ShaderType)]
struct SourceUniform {
    view_to_source: [Vec4; 3],
    source_rect: Vec4,
    uv_rect: Vec4,
    // Physical fragment coordinates are shared by all ten slots. The mesh's
    // local UV is intentionally not used to recover a screen coordinate.
    viewport: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub(super) struct SourceMaterial {
    #[uniform(0)]
    source: SourceUniform,
    #[texture(1)]
    texture: Handle<Image>,
}
impl Material2d for SourceMaterial {
    fn fragment_shader() -> ShaderRef { SOURCE_SHADER.into() }
}

pub(super) fn install(app: &mut App) {
    if app.world().contains_resource::<AssetServer>() {
        app.add_plugins(Material2dPlugin::<SourceMaterial>::default());
        if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render_app.add_systems(bevy::render::Render,
                source_images_ready.in_set(bevy::render::RenderSet::PrepareAssets)
                    .after(bevy::render::render_asset::prepare_assets::<bevy::render::texture::GpuImage>)
                    .before(bevy::render::render_asset::prepare_assets::<bevy::sprite::PreparedMaterial2d<SourceMaterial>>));
        }
    } else {
        app.init_resource::<Assets<SourceMaterial>>();
    }
    app.init_resource::<Assets<Shader>>()
        .init_resource::<ImmersiveBattleSourceOverlayEnabled>()
        .add_systems(Startup, setup);
    load_internal_asset!(app, SOURCE_SHADER, "battle_source_overlay.wgsl", Shader::from_wgsl);
}

fn source_images_ready() {}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SourceMaterial>>) {
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    for slot in 0..SOURCE_SLOTS {
        commands.spawn((MaterialMesh2dBundle {
            mesh: quad.clone().into(),
            material: materials.add(SourceMaterial { source: SourceUniform::default(), texture: Handle::default() }),
            visibility: Visibility::Hidden,
            ..default()
        }, SourceOverlaySlot(slot), RenderLayers::layer(0)));
    }
}

struct SlotFrame {
    material: SourceMaterial,
    transform: Transform,
}

enum SlotProjection {
    Offscreen,
    Visible(SlotFrame),
}

fn slot_frame(projection: BattleSourceProjection, object: &VisualBattleSourceObject,
    canvas: &crystal_render_api::VisualBattleCanvas, mode: crystal_render_api::BattleFlashMode,
    depth: f32) -> Option<SlotProjection> {
    let source_rect = Rect::from_center_size(object.center, object.size);
    let crop = object.uv_rect;
    if !crop.min.is_finite() || !crop.max.is_finite() || crop.size().min_element() <= 0.0
        || crop.min.cmplt(Vec2::ZERO).any() || crop.max.cmpgt(Vec2::ONE).any() {
        return None;
    }
    if canvas.physical_size.min_element() == 0 { return None; }
    projection.coverage(source_rect)?;
    let Some(coverage) = projection.raster_coverage(source_rect, canvas.physical_size) else {
        // A valid projected object outside this viewport is invisible. Never
        // resurrect its unprojected classic Sprite in the middle of the arena.
        return Some(SlotProjection::Offscreen);
    };
    let center = (coverage.center() - Vec2::splat(0.5)) * canvas.size;
    Some(SlotProjection::Visible(SlotFrame {
        material: SourceMaterial {
            source: SourceUniform {
                view_to_source: projection.view_to_source.to_cols_array_2d().map(|v| Vec3::from_array(v).extend(0.0)),
                source_rect: Vec4::new(source_rect.min.x, source_rect.min.y, source_rect.max.x, source_rect.max.y),
                uv_rect: Vec4::new(crop.min.x, crop.min.y, crop.max.x, crop.max.y),
                viewport: canvas.physical_size.as_vec2().extend(0.0).extend(0.0),
            },
            texture: if mode == crystal_render_api::BattleFlashMode::Reduced {
                object.neutral_texture.clone()
            } else { object.texture.clone() },
        },
        transform: Transform::from_xyz(center.x, -center.y, depth)
            .with_scale((coverage.size() * canvas.size).extend(1.0)),
    }))
}

pub(super) fn sync_immersive_battle_source_object_layout(
    layout: Res<crystal_voxel_view::BattleSceneLayout>,
    mut commands: Commands,
    status: Res<crystal_voxel_view::BattleViewStatus>,
    enabled: Res<ImmersiveBattleSourceOverlayEnabled>,
    frame: Res<VisualBattleFrame>,
    canvas: Res<crystal_render_api::VisualBattleCanvas>,
    mode: Res<crystal_render_api::BattleFlashMode>,
    mut materials: ResMut<Assets<SourceMaterial>>,
    cameras: Query<&Transform, (With<MainCameraMarker>, Without<SourceOverlaySlot>)>,
    objects: Query<(Entity, &ImmersiveBattleSourceObject, &Transform,
        Option<&RenderLayers>, Option<&ImmersiveBattleSourceObjectLayout>), Without<SourceOverlaySlot>>,
    mut overlays: Query<(&SourceOverlaySlot, &Handle<SourceMaterial>, &mut Transform, &mut Visibility), Without<ImmersiveBattleSourceObject>>,
) {
    let projection = (enabled.enabled && status.active)
        .then(|| BattleSourceProjection::new(&layout, canvas.size)).flatten();
    let mut slots: [Option<SlotFrame>; SOURCE_SLOTS] = std::array::from_fn(|_| None);
    // WOBBLE_SCREEN can move the native camera. Follow it for coverage only;
    // the common screen-space inverse projection never receives BG shake.
    let camera_offset = cameras.get_single().map_or(Vec2::ZERO, |camera| camera.translation.truncate());
    let available: [bool; SOURCE_SLOTS] = std::array::from_fn(|slot| overlays.iter()
        .any(|(candidate, material, _, _)| candidate.0 == slot && materials.contains(material.id())));
    for (entity, slot, transform, layers, stored) in &objects {
        let projected = projection.filter(|_| slot.0 < SOURCE_SLOTS && available[slot.0])
            .and_then(|projection| frame.source.as_ref().and_then(|source| source.objects.iter()
                .find(|object| object.slot == slot.0)).and_then(|object|
                    slot_frame(projection, object, &canvas, *mode, transform.translation.z)));
        if let Some(projected) = projected {
            if stored.is_none() {
                commands.entity(entity).insert(ImmersiveBattleSourceObjectLayout(layers.cloned()));
            }
            set_immersive_hidden_layer(&mut commands, entity, layers, true);
            slots[slot.0] = match projected {
                SlotProjection::Visible(frame) => Some(frame),
                SlotProjection::Offscreen => None,
            };
        } else if let Some(stored) = stored {
            if enabled.park_classic {
                set_immersive_hidden_layer(&mut commands, entity, layers, true);
            } else if let Some(original) = &stored.0 { commands.entity(entity).insert(original.clone()); }
            else { commands.entity(entity).remove::<RenderLayers>(); }
            commands.entity(entity).remove::<ImmersiveBattleSourceObjectLayout>();
        }
    }
    for (slot, handle, mut transform, mut visibility) in &mut overlays {
        let Some(current) = slots[slot.0].as_ref() else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        if materials.get(handle).is_some_and(|material|
            material.source != current.material.source || material.texture != current.material.texture) {
            *materials.get_mut(handle).unwrap() = current.material.clone();
        }
        let mut pose = current.transform;
        pose.translation += camera_offset.extend(0.0);
        transform.set_if_neq(pose);
        visibility.set_if_neq(Visibility::Visible);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projected_oam_cache_is_fixed_and_palette_modes_only_change_texture() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<crystal_voxel_view::BattleSceneLayout>()
            .init_resource::<crystal_voxel_view::BattleViewStatus>()
            .init_resource::<crystal_render_api::VisualBattleCanvas>()
            .init_resource::<crystal_render_api::BattleFlashMode>()
            .init_resource::<VisualBattleFrame>();
        install(&mut app);
        app.add_systems(Update, sync_immersive_battle_source_object_layout);
        app.update();
        let ids = {
            let world = app.world_mut();
            world.query_filtered::<Entity, With<SourceOverlaySlot>>().iter(world).collect::<Vec<_>>()
        };
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        let object = VisualBattleSourceObject {
            slot: 2, object_id: Arc::from("TEST"),
            texture: Handle::weak_from_u128(100), neutral_texture: Handle::weak_from_u128(101),
            center: Vec2::new(124.0, 32.0), size: Vec2::splat(8.0),
            uv_rect: Rect::from_corners(Vec2::ZERO, Vec2::ONE),
        };
        let canvas = crystal_render_api::VisualBattleCanvas { size: Vec2::new(800.0,600.0), physical_size: UVec2::new(1600,1200) };
        let projection = BattleSourceProjection::new(&crystal_voxel_view::BattleSceneLayout::default(), canvas.size).unwrap();
        let SlotProjection::Visible(full) = slot_frame(projection, &object, &canvas, crystal_render_api::BattleFlashMode::Full, 3.898).unwrap()
            else { panic!("interior source object must project into the viewport") };
        let SlotProjection::Visible(reduced) = slot_frame(projection, &object, &canvas, crystal_render_api::BattleFlashMode::Reduced, 3.898).unwrap()
            else { panic!("interior source object must project into the viewport") };
        assert_eq!(full.transform, reduced.transform);
        assert_eq!(full.material.source, reduced.material.source);
        assert_eq!(full.material.texture, object.texture);
        assert_eq!(reduced.material.texture, object.neutral_texture);
        let source = VisualBattleSourceFrame {
            frame: 58, bgp: 0xe4, battler_bgps: [0xe4; 2],
            battler_palettes: [[[1.0; 4]; 4]; 2],
            battler_textures: [Handle::default(), Handle::default()],
            battler_offsets: [Vec2::ZERO; 2], screen_offset: Vec2::new(4.0, -3.0),
            line_x_offsets: Some([6; 95]), line_y_offsets: Some([-5; 95]),
            objects: vec![object.clone()], battler_rows: [None; 2],
        };
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source.clone());
        app.world_mut().resource_mut::<ImmersiveBattleSourceOverlayEnabled>().enabled = true;
        let camera = app.world_mut().spawn((Transform::default(), MainCameraMarker)).id();
        let parent = app.world_mut().spawn(SpatialBundle::default()).id();
        let original = Transform::from_xyz(17.0, 23.0, 3.898);
        let original_texture = Handle::<Image>::weak_from_u128(102);
        let sprite = app.world_mut().spawn((SpriteBundle {
            transform: original, texture: original_texture.clone(),
            sprite: Sprite { rect: Some(Rect::new(1.0, 2.0, 7.0, 8.0)), custom_size: Some(Vec2::splat(32.0)), ..default() },
            ..default()
        }, ImmersiveBattleSourceObject(2))).id();
        app.world_mut().entity_mut(sprite).set_parent(parent);
        for iteration in 0..64 {
            let active = iteration % 4 != 0;
            let mode = if iteration % 2 == 0 { crystal_render_api::BattleFlashMode::Full }
                else { crystal_render_api::BattleFlashMode::Reduced };
            app.world_mut().resource_mut::<crystal_voxel_view::BattleViewStatus>().active = active;
            *app.world_mut().resource_mut::<crystal_render_api::BattleFlashMode>() = mode;
            *app.world_mut().resource_mut::<crystal_render_api::VisualBattleCanvas>() =
                if iteration % 3 == 0 { canvas } else {
                    crystal_render_api::VisualBattleCanvas { size: Vec2::new(600.0, 900.0), physical_size: UVec2::new(900, 1350) }
                };
            let camera_offset = Vec2::new(iteration as f32 - 32.0, 19.0 - iteration as f32);
            app.world_mut().get_mut::<Transform>(camera).unwrap().translation = camera_offset.extend(0.0);
            app.update();
            let current_canvas = *app.world().resource::<crystal_render_api::VisualBattleCanvas>();
            let current_projection = BattleSourceProjection::new(&crystal_voxel_view::BattleSceneLayout::default(), current_canvas.size).unwrap();
            let SlotProjection::Visible(mut expected) = slot_frame(current_projection, &object, &current_canvas, mode, original.translation.z).unwrap()
                else { panic!("interior source object must remain visible") };
            expected.transform.translation += camera_offset.extend(0.0);
            assert_eq!(app.world().get::<Transform>(sprite), Some(&original));
            assert_eq!(app.world().get::<Handle<Image>>(sprite), Some(&original_texture));
            assert_eq!(app.world().get::<Parent>(sprite).unwrap().get(), parent);
            assert_eq!(app.world().get::<ImmersiveBattleSourceObjectLayout>(sprite).is_some(), active);
            let world = app.world_mut();
            let mut visible = 0;
            for (slot, handle, pose, visibility) in world.query::<(&SourceOverlaySlot, &Handle<SourceMaterial>, &Transform, &Visibility)>().iter(world) {
                if *visibility == Visibility::Visible {
                    visible += 1;
                    assert_eq!(slot.0, 2);
                    assert_eq!(*pose, expected.transform, "camera movement must not become another source offset");
                    let material = world.resource::<Assets<SourceMaterial>>().get(handle).unwrap();
                    assert_eq!(material.texture, if mode == crystal_render_api::BattleFlashMode::Full {
                        object.texture.clone()
                    } else { object.neutral_texture.clone() });
                }
            }
            assert_eq!(visible, usize::from(active));
            assert_eq!(world.resource::<Assets<SourceMaterial>>().len(), SOURCE_SLOTS);
            assert_eq!(world.resource::<Assets<Mesh>>().len(), mesh_count);
            assert_eq!(world.resource::<VisualBattleFrame>().source.as_ref(), Some(&source));
        }
        // Fully offscreen projection still owns (and hides) its classic
        // Sprite. It must not pop back into the unprojected LCD layout.
        *app.world_mut().resource_mut::<crystal_render_api::VisualBattleCanvas>() =
            crystal_render_api::VisualBattleCanvas { size: Vec2::new(600.0, 900.0), physical_size: UVec2::new(600, 900) };
        app.world_mut().resource_mut::<VisualBattleFrame>().source.as_mut().unwrap().objects[0].center = Vec2::new(4.0, 4.0);
        app.world_mut().resource_mut::<crystal_voxel_view::BattleViewStatus>().active = true;
        app.update();
        assert!(app.world().get::<ImmersiveBattleSourceObjectLayout>(sprite).is_some());
        {
            let world = app.world_mut();
            assert!(world.query_filtered::<&Visibility, With<SourceOverlaySlot>>().iter(world).all(|v| *v == Visibility::Hidden));
        }
        // Source retirement removes the native entity. No overlay may retain
        // its previous visibility, even while the arena remains active.
        app.world_mut().entity_mut(sprite).despawn();
        app.world_mut().resource_mut::<VisualBattleFrame>().source.as_mut().unwrap().objects.clear();
        app.world_mut().resource_mut::<crystal_voxel_view::BattleViewStatus>().active = true;
        app.update();
        let world = app.world_mut();
        let after = world.query_filtered::<Entity, With<SourceOverlaySlot>>().iter(world).collect::<Vec<_>>();
        assert_eq!(ids, after);
        assert_eq!(ids.len(), SOURCE_SLOTS);
        assert!(world.query_filtered::<&Visibility, With<SourceOverlaySlot>>().iter(world).all(|v| *v == Visibility::Hidden));
        assert_eq!(world.resource::<Assets<SourceMaterial>>().len(), SOURCE_SLOTS);
        assert_eq!(mesh_count, 1);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), mesh_count);
    }
}
