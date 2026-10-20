//! Real pack maps below the summit, using the ordinary authored terrain mesher.
//! No gameplay session, duplicate city layout, source export, or second dispatcher.
use crate::{
    VOXEL_RENDER_LAYER, VoxelMaterial, VoxelViewSettings, VoxelViewStatus,
    mesh::{TerrainImageSamples, TerrainMeshData},
};
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
};
use crystal_render_api::VisualWorldFrame;
use std::sync::Arc;

#[derive(Clone)]
pub struct SummitBackdropMap {
    pub frame: Arc<VisualWorldFrame>,
    /// Placement from the same source-map connection graph as the active world.
    pub origin_source: IVec2,
}
#[derive(Resource, Default)]
pub struct SummitBackdrop {
    pub revision: Option<u64>,
    pub tower_core: IVec2,
    pub maps: Vec<SummitBackdropMap>,
    /// One real exterior border metatile, meshed once and shared outside maps.
    pub border: Option<Arc<VisualWorldFrame>>,
}
/// Exact source-cell coverage. Border instances never replace a real map cell.
fn border_origins(backdrop: &SummitBackdrop) -> Vec<IVec2> {
    if backdrop.maps.is_empty() {
        return Vec::new();
    }
    let low = backdrop
        .maps
        .iter()
        .map(|m| m.origin_source)
        .fold(IVec2::splat(i32::MAX), IVec2::min)
        - IVec2::splat(16);
    let high = backdrop
        .maps
        .iter()
        .map(|m| m.origin_source + m.frame.grid_size.as_ivec2())
        .fold(IVec2::splat(i32::MIN), IVec2::max)
        + IVec2::splat(16);
    let mut result = Vec::new();
    for y in (low.y.div_euclid(4) * 4..high.y).step_by(4) {
        for x in (low.x.div_euclid(4) * 4..high.x).step_by(4) {
            let at = IVec2::new(x, y);
            if backdrop.maps.iter().any(|m| {
                let end = m.origin_source + m.frame.grid_size.as_ivec2();
                x < end.x && x + 4 > m.origin_source.x && y < end.y && y + 4 > m.origin_source.y
            }) {
                continue;
            }
            result.push(at);
        }
    }
    result
}
#[derive(Resource, Default)]
pub struct SummitBackdropStatus {
    pub ready: bool,
    pub maps: Vec<String>,
    pub error: Option<String>,
}
#[derive(Component)]
struct SummitVista;
#[derive(Default)]
struct Build {
    revision: Option<u64>,
    profile_revision: u64,
    attachment: Option<(Vec2, f32)>,
    root: Option<Entity>,
    next: usize,
    task: Option<
        Task<
            Result<
                (TerrainMeshData, Vec<crate::occluder_fade::PreparedGroup>),
                crate::TerrainMeshError,
            >,
        >,
    >,
    solid: Option<Handle<VoxelMaterial>>,
    water: Vec<(Handle<VoxelMaterial>, usize)>,
}
pub(crate) fn enabled(frame: &VisualWorldFrame) -> bool {
    frame.active
        && frame.map_id.as_ref() == "TinTowerRoof"
        && frame
            .tiles
            .first()
            .is_some_and(|t| t.source.tileset_id.as_ref() == "tower")
}
pub(crate) fn world_units(frame: &VisualWorldFrame) -> f32 {
    (frame.tile_size.max_element() / 8.).max(1.)
}
const SCALE: f32 = 1.0;
const DROP: f32 = 320.;
fn origin(frame: &VisualWorldFrame, tower_core: IVec2, attachment: Option<(Vec2, f32)>) -> Vec3 {
    let map_zero =
        -(frame.grid_origin.as_vec2() + frame.grid_size.as_vec2() * 0.5) * frame.tile_size;
    let roof_center = frame
        .source_map_size_core_tiles
        .unwrap_or(UVec2::new(20, 18))
        .as_vec2()
        * frame.tile_size;
    let (anchor, drop) = attachment.unwrap_or((
        (tower_core.as_vec2() + Vec2::splat(0.5)) * frame.tile_size * 2.,
        DROP * world_units(frame),
    ));
    let tower = anchor * SCALE;
    Vec3::new(
        map_zero.x + roof_center.x - tower.x + frame.center.x,
        -drop,
        map_zero.y + roof_center.y - tower.y - frame.center.y,
    )
}
pub(crate) fn tower_attachment(
    source: &SummitBackdropMap,
    terrain: &TerrainMeshData,
    tower_core: IVec2,
) -> Option<(Vec2, f32)> {
    if source.frame.map_id.as_ref() != "EcruteakCity" {
        return None;
    }
    let half = source.frame.grid_size.as_vec2() * source.frame.tile_size * 0.5;
    let door = (tower_core.as_vec2() + Vec2::splat(0.5)) * source.frame.tile_size * 2. - half;
    let building = terrain.rigid_buildings.iter().min_by(|a, b| {
        a.doorway
            .distance_squared(door)
            .total_cmp(&b.doorway.distance_squared(door))
    })?;
    if building.doorway.distance(door) > source.frame.tile_size.max_element() * 1.5 {
        return None;
    }
    let points = terrain.solid.positions.get(building.vertices.clone())?;
    if points.is_empty() {
        return None;
    }
    let low = points
        .iter()
        .map(|p| Vec3::from_array(*p))
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let high = points
        .iter()
        .map(|p| Vec3::from_array(*p))
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    // Center the playable summit on the actual tower footprint and keep the
    // existing exterior roof immediately underneath it, including terrace lift.
    Some((
        Vec2::new((low.x + high.x) * 0.5, (low.z + high.z) * 0.5) + half,
        high.y * SCALE + source.frame.tile_size.y * 0.5,
    ))
}
pub(super) fn register(app: &mut App) {
    app.init_resource::<SummitBackdrop>()
        .init_resource::<SummitBackdropStatus>()
        .add_systems(Update, sync.after(crate::sync_voxel_view))
        .add_systems(Update, summit_fog.after(crate::sync_voxel_atmosphere));
}
fn summit_fog(
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    clock: Option<Res<crystal_render_api::VisualWorldClock>>,
    mut cameras: Query<&mut bevy::pbr::DistanceFog, With<crate::VoxelWorldCamera>>,
) {
    if !enabled(&frame) || !settings.enabled {
        return;
    }
    let second = clock.and_then(|c| c.seconds_of_day()).unwrap_or(43200);
    for mut fog in &mut cameras {
        fog.color = crate::day_cycle::DayCycle::at_seconds(second).horizon;
        fog.falloff = bevy::pbr::FogFalloff::Linear {
            start: 1600. * world_units(&frame),
            end: 5000. * world_units(&frame),
        };
    }
}
fn spawn_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    root: Entity,
    mesh: crate::mesh::SurfaceMeshData,
    material: Handle<VoxelMaterial>,
    translations: &[Vec3],
) {
    if mesh.indices.is_empty() {
        return;
    }
    let mesh = meshes.add(mesh.into_mesh());
    for &translation in translations {
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(translation),
                bevy::camera::visibility::RenderLayers::layer(VOXEL_RENDER_LAYER),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        commands.entity(root).add_child(entity);
    }
}
fn publish_map(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<VoxelMaterial>,
    root: Entity,
    source: &SummitBackdropMap,
    mut terrain: TerrainMeshData,
    solid: Handle<VoxelMaterial>,
    groves: Vec<crate::occluder_fade::PreparedGroup>,
    border_positions: Option<&[IVec2]>,
) -> Handle<VoxelMaterial> {
    let frame = &source.frame;
    if border_positions.is_some() {
        // The fifth source column supplies real lawn paint to the native grove
        // builder. It is a sample, never an extra visible strip over a neighbor.
        let edge = (4. - frame.grid_size.x as f32 * 0.5) * frame.tile_size.x;
        for surface in [
            &mut terrain.solid,
            &mut terrain.textured,
            &mut terrain.animated_solid,
            &mut terrain.animated_textured,
        ] {
            surface.indices = surface
                .indices
                .chunks_exact(3)
                .filter(|tri| {
                    !tri.iter().all(|&i| {
                        let p = surface.positions[i as usize];
                        p[0] >= edge - 0.01 && p[1] <= frame.tile_size.y * 0.1
                    })
                })
                .flatten()
                .copied()
                .collect();
        }
    }
    let shift =
        (source.origin_source.as_vec2() + frame.grid_size.as_vec2() * 0.5) * frame.tile_size;
    let translations = border_positions
        .map(|origins| {
            origins
                .iter()
                .map(|p| {
                    let at = (p.as_vec2() + frame.grid_size.as_vec2() * 0.5) * frame.tile_size;
                    Vec3::new(at.x, 0., at.y)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![Vec3::new(shift.x, 0., shift.y)]);
    let mut material = crate::textured_terrain_material(frame.map_texture.clone());
    material.extension.cave_reveal = Vec4::new(0., 0., 2., 0.);
    let water = crate::water_surface::PreparedImages::for_frame(frame);
    material.extension.water_enabled = if water.mask.is_some() {
        crate::cave_water::registration(frame)
    } else {
        Vec4::ZERO
    };
    material.extension.water_mask = water.mask.map(|i| images.add(i));
    material.extension.water_shore = water.shore.map(|i| images.add(i));
    material.extension.water_bounds = crate::water_surface::bounds(frame);
    material.extension.water_style = crate::water_surface::style(frame);
    material.extension.water_contact = crate::water_surface::contact(frame);
    if let Some(palette) = crate::cave_water::native_palette(frame, images) {
        material.extension.water_deep = palette[0];
        material.extension.water_foam = palette[1];
    }
    let textured = materials.add(material);
    // Actual connected maps own their own complete rectangles. Repeating one
    // map's apron across its neighbors would cover their water and lower ground.
    terrain.background = None;
    for surface in std::mem::take(&mut terrain.scenery) {
        let mut material = crate::textured_terrain_material(images.add(surface.texture.image()));
        material.extension.cave_reveal = Vec4::new(0., 0., 2., 0.);
        material.base.alpha_mode = surface.texture.alpha_mode();
        material.base.double_sided = surface.texture.double_sided;
        if surface.texture.double_sided {
            material.base.cull_mode = None;
        }
        let handle = materials.add(material);
        spawn_mesh(commands, meshes, root, surface.mesh, handle, &translations);
    }
    for instance in std::mem::take(&mut terrain.tree_instances) {
        let mesh = meshes.add(instance.mesh.into_mesh());
        for &translation in &translations {
            for at in &instance.origins {
                let entity = commands
                    .spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(textured.clone()),
                        Transform::from_translation(translation + Vec3::from_array(*at)),
                        bevy::camera::visibility::RenderLayers::layer(VOXEL_RENDER_LAYER),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();
                commands.entity(root).add_child(entity);
            }
        }
    }
    for &translation in &translations {
        let grove_root = commands
            .spawn((
                Transform::from_translation(translation),
                Visibility::Inherited,
            ))
            .id();
        commands.entity(root).add_child(grove_root);
        for leaf in crate::occluder_fade::spawn_groups(
            commands,
            meshes,
            materials,
            grove_root,
            groves.clone(),
            solid.clone(),
            None,
        ) {
            commands
                .entity(leaf)
                .insert((NotShadowCaster, NotShadowReceiver));
        }
    }
    spawn_mesh(
        commands,
        meshes,
        root,
        terrain.textured,
        textured.clone(),
        &translations,
    );
    spawn_mesh(
        commands,
        meshes,
        root,
        terrain.animated_textured,
        textured.clone(),
        &translations,
    );
    spawn_mesh(
        commands,
        meshes,
        root,
        terrain.solid,
        solid.clone(),
        &translations,
    );
    spawn_mesh(
        commands,
        meshes,
        root,
        terrain.animated_solid,
        solid,
        &translations,
    );
    textured
}
fn sync(
    mut commands: Commands,
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    backdrop: Res<SummitBackdrop>,
    terrain_cache: Res<crate::TerrainRevisionCache>,
    profiles: Res<crate::live_profiles::LiveProfiles>,
    mut status: ResMut<SummitBackdropStatus>,
    mut world_status: ResMut<VoxelViewStatus>,
    mut build: Local<Build>,
    mut roots: Query<(&mut Transform, &mut Visibility), With<SummitVista>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let visible = enabled(&frame) && settings.enabled;
    for (mut transform, mut visibility) in &mut roots {
        visibility.set_if_neq(if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        if visible {
            let next=origin(&frame, backdrop.tower_core, build.attachment);
            if transform.translation != next { transform.translation = next; }
        }
    }
    if !visible {
        if frame.active && frame.map_id.as_ref() != "TinTowerRoof" {
            if let Some(root) = build.root.take() {
                commands.entity(root).despawn();
            }
            *build = Build::default();
            *status = SummitBackdropStatus::default();
        }
        return;
    }
    if backdrop.revision.is_none() || backdrop.maps.is_empty() {
        world_status.terrain_pending = true;
        world_status.active_frames = 0;
        return;
    }
    if build.revision != backdrop.revision || build.profile_revision != profiles.revision {
        // The active scene already prepares these exact images. Wait for its
        // publication instead of decoding PNGs on the frame thread and making
        // a second pair of GPU textures for the panorama.
        let (Some(stone), Some(bedrock)) =
            (&terrain_cache.painted_stone, &terrain_cache.painted_bedrock)
        else {
            world_status.terrain_pending = true;
            world_status.active_frames = 0;
            return;
        };
        if let Some(root) = build.root.take() {
            commands.entity(root).despawn();
        }
        let root = commands
            .spawn((
                SummitVista,
                Transform::from_translation(origin(&frame, backdrop.tower_core, build.attachment))
                    .with_scale(Vec3::splat(SCALE)),
                Visibility::Visible,
            ))
            .id();
        let mut material = crate::solid_terrain_material();
        material.extension.cave_reveal = Vec4::new(0., 0., 2., 0.);
        material.extension.painted_stone = Some(stone.clone());
        material.extension.painted_bedrock = Some(bedrock.clone());
        build.solid = Some(materials.add(material));
        build.root = Some(root);
        build.attachment = None;
        build.water.clear();
        build.next = 0;
        build.task = None;
        build.revision = backdrop.revision;
        build.profile_revision = profiles.revision;
        *status = SummitBackdropStatus::default();
    }
    if let Some(result) = build
        .task
        .as_mut()
        .and_then(|task| future::block_on(future::poll_once(task)))
    {
        build.task = None;
        match result {
            Ok((terrain, groves)) => {
                let is_border = build.next == backdrop.maps.len();
                let border_source = backdrop.border.as_ref().map(|frame| SummitBackdropMap {
                    frame: frame.clone(),
                    origin_source: IVec2::ZERO,
                });
                let source = if is_border {
                    border_source.as_ref().unwrap()
                } else {
                    &backdrop.maps[build.next]
                };
                if std::env::var_os("CRYSTAL_SUMMIT_TRACE").is_some() {
                    eprintln!(
                        "summit publish {} cells={} triangles={} authored={}",
                        source.frame.map_id,
                        source.frame.tiles.len(),
                        (terrain.solid.indices.len() + terrain.textured.indices.len()) / 3,
                        terrain
                            .authored_cells
                            .iter()
                            .filter(|p| p.is_some())
                            .count()
                    );
                }
                if let Some(attachment) = tower_attachment(source, &terrain, backdrop.tower_core) {
                    build.attachment = Some(attachment);
                    commands.entity(build.root.unwrap()).insert(
                        Transform::from_translation(origin(
                            &frame,
                            backdrop.tower_core,
                            build.attachment,
                        ))
                        .with_scale(Vec3::splat(SCALE)),
                    );
                    if std::env::var_os("CRYSTAL_SUMMIT_TRACE").is_some() {
                        eprintln!(
                            "summit attached to actual tower model: source_center={:?} vertical_drop={}",
                            attachment.0, attachment.1
                        );
                    }
                }
                let positions = is_border.then(|| border_origins(&backdrop));
                let handle = publish_map(
                    &mut commands,
                    &mut meshes,
                    &mut images,
                    &mut materials,
                    build.root.unwrap(),
                    source,
                    terrain,
                    build.solid.as_ref().unwrap().clone(),
                    groves,
                    positions.as_deref(),
                );
                let index = build.next;
                if !is_border {
                    build.water.push((handle, index));
                    status.maps.push(source.frame.map_id.to_string());
                }
                build.next += 1;
            }
            Err(error) => {
                status.error = Some(format!("source panorama mesh failed: {error:?}"));
            }
        }
    }
    for (handle, index) in &build.water {
        if let Some(mut material) = materials.get_mut(handle) {
            let map_zero = origin(&frame, backdrop.tower_core, build.attachment)
                + Vec3::new(
                    backdrop.maps[*index].origin_source.x as f32
                        * backdrop.maps[*index].frame.tile_size.x
                        * SCALE,
                    0.,
                    backdrop.maps[*index].origin_source.y as f32
                        * backdrop.maps[*index].frame.tile_size.y
                        * SCALE,
                );
            let kind = material.extension.water_enabled.x;
            material.extension.water_enabled = Vec4::new(
                kind,
                backdrop.maps[*index].frame.tile_size.x * SCALE,
                -map_zero.x,
                -map_zero.z,
            );
        }
    }
    if status.error.is_some() {
        world_status.terrain_pending = true;
        world_status.active_frames = 0;
        return;
    }
    status.ready = build.next == backdrop.maps.len() + usize::from(backdrop.border.is_some());
    if !status.ready {
        world_status.terrain_pending = true;
        world_status.active_frames = 0;
        if build.task.is_none() {
            let source = if build.next == backdrop.maps.len() {
                backdrop.border.as_ref().unwrap().clone()
            } else {
                backdrop.maps[build.next].frame.clone()
            };
            if std::env::var_os("CRYSTAL_SUMMIT_TRACE").is_some() {
                eprintln!(
                    "summit building {} of {}: {}",
                    build.next + 1,
                    backdrop.maps.len(),
                    source.map_id
                );
            }
            let samples = TerrainImageSamples::capture(&source, &images);
            let document = profiles.document.clone();
            let library = terrain_cache.forest_library.clone();
            build.task = Some(AsyncComputeTaskPool::get().spawn(async move {
                let built = crate::mesh::begin_distant_terrain_mesh_with_profiles(
                    &source, &samples, &document,
                )?
                .into_streamed();
                let mut groves = Vec::new();
                if let Some(stream) = built.streamed_groves {
                    for i in 0..stream.len() {
                        groves.extend(stream.prepare(i, &library));
                    }
                }
                Ok((built.terrain, groves))
            }));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn panorama_waits_for_shared_paint_and_reuses_the_active_image_handles() {
        use crystal_render_api::{VisualTile,VisualTileSource};
        AsyncComputeTaskPool::get_or_init(||bevy::tasks::TaskPoolBuilder::new().num_threads(1).build());
        let mut app=App::new();
        app.insert_resource(VisualWorldFrame {
            active:true,map_id:"TinTowerRoof".into(),
            tiles:vec![VisualTile {column:0,row:0,texture:Handle::default(),animation_frames:None,priority:false,
                source:VisualTileSource {tileset_id:"tower".into(),metatile_id:0,tile_index:0,subtile_column:0,subtile_row:0}}],
            ..default()
        }).insert_resource(VoxelViewSettings {enabled:true,..default()})
            .insert_resource(SummitBackdrop {revision:Some(1),
                maps:vec![SummitBackdropMap {frame:Arc::new(VisualWorldFrame::default()),origin_source:IVec2::ZERO}],
                ..default()})
            .init_resource::<crate::TerrainRevisionCache>()
            .init_resource::<crate::live_profiles::LiveProfiles>()
            .init_resource::<SummitBackdropStatus>()
            .init_resource::<VoxelViewStatus>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .add_systems(Update,sync);
        app.update();
        assert!(app.world().resource::<VoxelViewStatus>().terrain_pending);
        assert_eq!(app.world().resource::<Assets<Image>>().len(),0);
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(),0,
            "pending paint must not create a fallback material or decode on the frame thread");
        let (stone,bedrock)={
            let mut images=app.world_mut().resource_mut::<Assets<Image>>();
            (images.add(Image::default()),images.add(Image::default()))
        };
        {
            let mut cache=app.world_mut().resource_mut::<crate::TerrainRevisionCache>();
            cache.painted_stone=Some(stone.clone());cache.painted_bedrock=Some(bedrock.clone());
        }
        app.update();
        assert_eq!(app.world().resource::<Assets<Image>>().len(),2,
            "the panorama must allocate no duplicate painted images");
        let materials=app.world().resource::<Assets<VoxelMaterial>>();
        assert_eq!(materials.len(),1);
        let material=materials.iter().next().unwrap().1;
        assert_eq!(material.extension.painted_stone.as_ref(),Some(&stone));
        assert_eq!(material.extension.painted_bedrock.as_ref(),Some(&bedrock));
    }

    #[test]
    fn authored_border_fills_every_gap_without_covering_any_map_cell() {
        let maps = [
            (IVec2::ZERO, UVec2::new(8, 8)),
            (IVec2::new(8, 4), UVec2::new(12, 4)),
            (IVec2::new(-4, 8), UVec2::new(8, 8)),
        ]
        .into_iter()
        .map(|(at, size)| SummitBackdropMap {
            origin_source: at,
            frame: Arc::new(VisualWorldFrame {
                grid_size: size,
                ..default()
            }),
        })
        .collect();
        let backdrop = SummitBackdrop { maps, ..default() };
        let origins = border_origins(&backdrop);
        let unique = origins
            .iter()
            .map(|p| (p.x, p.y))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), origins.len());
        for y in -16..32 {
            for x in -20..36 {
                let in_map = backdrop.maps.iter().any(|m| {
                    let end = m.origin_source + m.frame.grid_size.as_ivec2();
                    x >= m.origin_source.x && x < end.x && y >= m.origin_source.y && y < end.y
                });
                let instances = origins
                    .iter()
                    .filter(|p| x >= p.x && x < p.x + 4 && y >= p.y && y < p.y + 4)
                    .count();
                assert_eq!(instances, usize::from(!in_map), "source cell {x},{y}");
            }
        }
        assert!(border_origins(&SummitBackdrop::default()).is_empty());
    }
    #[test]
    fn summit_draw_distance_includes_the_real_routes_without_changing_the_camera_pose() {
        use crystal_render_api::{VisualTile, VisualTileSource};
        let mut frame = VisualWorldFrame {
            active: true,
            map_id: "TinTower9F".into(),
            tile_size: Vec2::splat(32.),
            viewport_size: Vec2::new(640., 576.),
            tiles: vec![VisualTile {
                column: 0,
                row: 0,
                texture: Handle::default(),
                animation_frames: None,
                priority: false,
                source: VisualTileSource {
                    tileset_id: "tower".into(),
                    metatile_id: 9,
                    tile_index: 2,
                    subtile_column: 0,
                    subtile_row: 0,
                },
            }],
            ..default()
        };
        let controls = crate::VoxelCameraControls::default();
        let ordinary = controls.world_pose(&frame, &[]);
        frame.map_id = "TinTowerRoof".into();
        let summit = controls.world_pose(&frame, &[]);
        assert_eq!(summit.far, 24000.);
        assert!(ordinary.far < summit.far);
        assert_eq!(
            (
                summit.eye,
                summit.target,
                summit.near,
                summit.vertical_fov_radians
            ),
            (
                ordinary.eye,
                ordinary.target,
                ordinary.near,
                ordinary.vertical_fov_radians
            )
        );
    }
    #[test]
    fn summit_haze_tracks_the_clock_and_clears_in_other_rooms() {
        use bevy::{
            anti_alias::fxaa::Fxaa,
            pbr::{DistanceFog, FogFalloff},
        };
        use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldClock};
        let mut app = App::new();
        app.insert_resource(VisualWorldFrame {
            active: true,
            map_id: "TinTowerRoof".into(),
            tiles: vec![VisualTile {
                column: 0,
                row: 0,
                texture: Handle::default(),
                animation_frames: None,
                priority: false,
                source: VisualTileSource {
                    tileset_id: "tower".into(),
                    metatile_id: 9,
                    tile_index: 2,
                    subtile_column: 0,
                    subtile_row: 0,
                },
            }],
            ..default()
        })
        .insert_resource(VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .insert_resource(VisualWorldClock {
            minute_of_day: Some(720),
            second_of_minute: 0,
        })
        .init_resource::<crate::live_profiles::LiveProfiles>()
        .add_systems(Update, (crate::sync_voxel_atmosphere, summit_fog).chain());
        let camera = app
            .world_mut()
            .spawn((
                crate::VoxelWorldCamera,
                Camera::default(),
                DistanceFog::default(),
                Fxaa::default(),
            ))
            .id();
        app.update();
        let day = app.world().get::<DistanceFog>(camera).unwrap().color;
        assert!(matches!(
            app.world().get::<DistanceFog>(camera).unwrap().falloff,
            FogFalloff::Linear {
                start: 1600.,
                end: 5000.
            }
        ));
        app.world_mut()
            .resource_mut::<VisualWorldClock>()
            .minute_of_day = Some(0);
        app.update();
        let night = app.world().get::<DistanceFog>(camera).unwrap().color;
        assert!(
            night.to_linear().to_vec4().truncate().length()
                < day.to_linear().to_vec4().truncate().length()
        );
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "TinTower9F".into();
        app.update();
        assert_eq!(
            app.world().get::<DistanceFog>(camera).unwrap().color,
            Color::NONE
        );
    }
    #[test]
    fn city_anchor_stays_at_the_real_tower_door_when_the_roof_grid_moves() {
        let mut frame = VisualWorldFrame {
            grid_size: UVec2::new(80, 76),
            tile_size: Vec2::splat(8.),
            ..default()
        };
        let door = IVec2::new(37, 7);
        let a = origin(&frame, door, None);
        frame.grid_origin += IVec2::new(3, -7);
        assert_eq!(origin(&frame, door, None) - a, Vec3::new(-24., 0., 56.));
        let shifted = origin(&frame, door, None);
        frame.center = Vec2::new(2., 3.);
        assert_eq!(
            origin(&frame, door, None) - shifted,
            Vec3::new(2., 0., -3.),
            "background must share the active world's sub-tile scroll"
        );
        let mut scaled = frame.clone();
        scaled.center = Vec2::ZERO;
        scaled.grid_origin = IVec2::ZERO;
        let base = origin(&scaled, door, None);
        scaled.tile_size *= 4.;
        assert!(
            (origin(&scaled, door, None) - base * 4.).length() < 0.001,
            "the city and rooftop must use the same source tile scale"
        );
        assert_eq!(a.y, -DROP);
        let door_world = a + Vec3::new((37.5) * 16. * SCALE, 0., 7.5 * 16. * SCALE);
        assert_eq!(door_world, Vec3::new(-160., -DROP, -160.));
    }
}
