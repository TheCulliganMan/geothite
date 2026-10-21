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
use std::{collections::VecDeque, sync::Arc};

mod tower_fit;
pub(crate) use tower_fit::{TowerAttachment, prepare as tower_attachment};

#[derive(Clone)]
pub struct SummitBackdropMap {
    pub frame: Arc<VisualWorldFrame>,
    /// Placement from the same source-map connection graph as the active world.
    pub origin_source: IVec2,
}
#[derive(Resource, Default)]
pub struct SummitBackdrop {
    pub revision: Option<u64>,
    /// Source maps arrive progressively, before their geometry is complete.
    pub source_pending: bool,
    pub animation_frame: u64,
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
#[derive(Resource, Default)]
struct Build {
    revision: Option<u64>,
    profile_revision: u64,
    attachment: Option<TowerAttachment>,
    root: Option<Entity>,
    next: usize,
    tasks: VecDeque<Task<Result<PreparedMap, crate::TerrainMeshError>>>,
    scheduled: usize,
    publication: Option<MapPublication>,
    solid: Option<Handle<VoxelMaterial>>,
    water: Vec<(Handle<VoxelMaterial>, usize)>,
}
/// These upper floors provide lead time for the full roof panorama.
pub fn summit_preload_interior(map: &str) -> bool {
    // This async pool is a CPU worker on native. Do not move whole-map meshing
    // onto a browser's frame thread as speculative work.
    #[cfg(not(target_arch = "wasm32"))]
    { matches!(map, "TinTower7F" | "TinTower8F" | "TinTower9F") }
    #[cfg(target_arch = "wasm32")]
    { let _ = map; false }
}

/// Give the foreground scene and its initial GPU preparation priority.
pub fn summit_preload_foreground_ready(status: &VoxelViewStatus) -> bool {
    status.active && !status.terrain_pending && !status.profiles_pending && status.active_frames >= 30
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
const DROP: f32 = 320.;
#[cfg(test)]
fn origin(frame: &VisualWorldFrame, tower_core: IVec2, attachment: Option<TowerAttachment>) -> Vec3 {
    tower_fit::transform(frame, tower_core, attachment).translation
}
pub(super) fn register(app: &mut App) {
    app.init_resource::<SummitBackdrop>()
        .init_resource::<SummitBackdropStatus>()
        .init_resource::<Build>()
        .add_systems(Update, sync.after(crate::sync_voxel_view))
        .add_systems(Update, summit_fog.after(crate::sync_voxel_atmosphere));
}
fn summit_fog(
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    clock: Option<Res<crystal_render_api::VisualWorldClock>>,
    build: Option<Res<Build>>,
    mut cameras: Query<(&mut bevy::pbr::DistanceFog, Option<&mut Projection>), With<crate::VoxelWorldCamera>>,
) {
    if !enabled(&frame) || !settings.enabled {
        return;
    }
    let second = clock.and_then(|c| c.seconds_of_day()).unwrap_or(43200);
    let scale = tower_fit::transform(&frame, IVec2::ZERO,
        build.as_ref().and_then(|b| b.attachment)).scale.x;
    for (mut fog, projection) in &mut cameras {
        fog.color = crate::day_cycle::DayCycle::at_seconds(second).horizon;
        fog.falloff = bevy::pbr::FogFalloff::Linear {
            start: 1600. * world_units(&frame) * scale,
            end: 5000. * world_units(&frame) * scale,
        };
        if let Some(mut projection) = projection
            && let Projection::Perspective(p) = &mut *projection {
            p.far = p.far.max(6000. * world_units(&frame) * scale);
        }
    }
}
// Bound retained CPU outputs while allowing native preparation to overlap.
#[cfg(not(target_arch = "wasm32"))]
const MAPS_IN_FLIGHT: usize = 2;
#[cfg(target_arch = "wasm32")]
const MAPS_IN_FLIGHT: usize = 1;
const PRELOAD_PLACEMENTS_PER_FRAME: usize = 8;
const ENTRY_PLACEMENTS_PER_FRAME: usize = 128;

struct MapPublication {
    root: Entity,
    translations: Vec<Vec3>,
    next: usize,
    surfaces: Vec<(Handle<Mesh>, Handle<VoxelMaterial>)>,
    trees: Vec<(Handle<Mesh>, Vec<[f32; 3]>)>,
    groves: Vec<crate::occluder_fade::PreparedGroup>,
    solid: Handle<VoxelMaterial>,
    textured: Handle<VoxelMaterial>,
}
impl MapPublication {
    // Bound repeated border placement without duplicating its uploaded assets.
    fn advance(&mut self, commands: &mut Commands, meshes: &mut Assets<Mesh>,
        materials: &mut Assets<VoxelMaterial>, budget: usize) -> bool {
        let end = (self.next + budget).min(self.translations.len());
        for &translation in &self.translations[self.next..end] {
            for (mesh, material) in &self.surfaces {
                self.spawn_draw(commands, mesh.clone(), material.clone(), translation);
            }
            for (mesh, origins) in &self.trees {
                for at in origins {
                    self.spawn_draw(commands, mesh.clone(), self.textured.clone(),
                        translation + Vec3::from_array(*at));
                }
            }
            let root = commands.spawn((Transform::from_translation(translation),
                Visibility::Inherited)).id();
            commands.entity(self.root).add_child(root);
            for leaf in crate::occluder_fade::spawn_region_groups(commands, meshes, materials,
                root, self.groves.clone(), self.solid.clone()) {
                commands.entity(leaf).insert((NotShadowCaster, NotShadowReceiver));
            }
        }
        self.next = end;
        end == self.translations.len()
    }
    fn spawn_draw(&self, commands: &mut Commands, mesh: Handle<Mesh>,
        material: Handle<VoxelMaterial>, translation: Vec3) {
        let entity = commands.spawn((Mesh3d(mesh), MeshMaterial3d(material),
            Transform::from_translation(translation),
            bevy::camera::visibility::RenderLayers::layer(VOXEL_RENDER_LAYER),
            NotShadowCaster, NotShadowReceiver)).id();
        commands.entity(self.root).add_child(entity);
    }
}
fn prepare_surface(meshes: &mut Assets<Mesh>,
    surfaces: &mut Vec<(Handle<Mesh>, Handle<VoxelMaterial>)>,
    mesh: crate::mesh::SurfaceMeshData, material: Handle<VoxelMaterial>) {
    if !mesh.indices.is_empty() { surfaces.push((meshes.add(mesh.into_mesh()), material)); }
}
// Static surfaces and water images arrive ready for GPU publication from the
// existing terrain worker, without frame-thread partitioning or shoreline scans.
struct PreparedMap {
    terrain: TerrainMeshData,
    groves: Vec<crate::occluder_fade::PreparedGroup>,
    textured: Vec<Mesh>,
    solid: Vec<Mesh>,
    attachment: Option<TowerAttachment>,
    source_triangles: usize,
    water: crate::water_surface::PreparedImages,
}

fn prepare_static_surface(mesh: crate::mesh::SurfaceMeshData, tile_size: Vec2) -> Vec<Mesh> {
    #[cfg(not(target_arch = "wasm32"))]
    let batches = crate::terrain_batches::partition(mesh, tile_size);
    #[cfg(target_arch = "wasm32")]
    let batches = { let _ = tile_size; vec![mesh] };
    batches.into_iter().filter(|batch| !batch.indices.is_empty())
        .map(crate::mesh::SurfaceMeshData::into_mesh).collect()
}

fn prepare_map_geometry(
    frame: Arc<VisualWorldFrame>,
    mut terrain: TerrainMeshData,
    groves: Vec<crate::occluder_fade::PreparedGroup>,
    is_border: bool,
    tower_core: IVec2,
) -> PreparedMap {
    let source_triangles = (terrain.solid.indices.len() + terrain.textured.indices.len()) / 3;
    let attachment = tower_attachment(&frame, &mut terrain, tower_core);
    if is_border {
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
    let textured = prepare_static_surface(std::mem::take(&mut terrain.textured), frame.tile_size);
    let solid = prepare_static_surface(std::mem::take(&mut terrain.solid), frame.tile_size);
    let water = crate::water_surface::PreparedImages::for_frame(&frame);
    PreparedMap { terrain, groves, textured, solid, attachment, source_triangles, water }
}
fn prepare_map(
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<VoxelMaterial>,
    root: Entity,
    source: &SummitBackdropMap,
    prepared: PreparedMap,
    solid: Handle<VoxelMaterial>,
    border_positions: Option<&[IVec2]>,
) -> MapPublication {
    let frame = &source.frame;
    let PreparedMap { mut terrain, groves, textured: prepared_textured, solid: prepared_solid, water, .. } = prepared;
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
    let mut surfaces = Vec::new();
    for surface in std::mem::take(&mut terrain.scenery) {
        let mut material = crate::textured_terrain_material(images.add(surface.texture.image()));
        material.extension.cave_reveal = Vec4::new(0., 0., 2., 0.);
        material.base.alpha_mode = surface.texture.alpha_mode();
        material.base.double_sided = surface.texture.double_sided;
        if surface.texture.double_sided {
            material.base.cull_mode = None;
        }
        let handle = materials.add(material);
        prepare_surface(meshes, &mut surfaces, surface.mesh, handle);
    }
    let trees = std::mem::take(&mut terrain.tree_instances).into_iter()
        .map(|instance| (meshes.add(instance.mesh.into_mesh()), instance.origins)).collect();
    for mesh in prepared_textured { surfaces.push((meshes.add(mesh), textured.clone())); }
    prepare_surface(meshes, &mut surfaces, terrain.animated_textured, textured.clone());
    for mesh in prepared_solid { surfaces.push((meshes.add(mesh), solid.clone())); }
    prepare_surface(meshes, &mut surfaces, terrain.animated_solid, solid.clone());
    MapPublication {root, translations, next:0, surfaces, trees, groves, solid, textured}
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
    mut build: ResMut<Build>,
    mut roots: Query<(&mut Transform, &mut Visibility), With<SummitVista>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let visible = enabled(&frame) && settings.enabled;
    let related = frame.map_id.as_ref() == "TinTowerRoof" || summit_preload_interior(&frame.map_id);
    let preparing = frame.active && settings.enabled
        && (enabled(&frame) || summit_preload_interior(&frame.map_id));
    for (mut transform, mut visibility) in &mut roots {
        visibility.set_if_neq(if visible && status.ready {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        if visible {
            let next=tower_fit::transform(&frame, backdrop.tower_core, build.attachment);
            transform.set_if_neq(next);
        }
    }
    if !preparing {
        if frame.active && !related {
            if let Some(root) = build.root.take() {
                commands.entity(root).despawn();
            }
            *build = Build::default();
            *status = SummitBackdropStatus::default();
        }
        return;
    }
    if !visible && !summit_preload_foreground_ready(&world_status) { return; }
    if backdrop.revision.is_none() || backdrop.maps.is_empty() {
        if visible {
            world_status.terrain_pending = true;
            world_status.active_frames = 0;
        }
        return;
    }
    if build.revision != backdrop.revision || build.profile_revision != profiles.revision {
        // The active scene already prepares these exact images. Wait for its
        // publication instead of decoding PNGs on the frame thread and making
        // a second pair of GPU textures for the panorama.
        let (Some(stone), Some(bedrock)) =
            (&terrain_cache.painted_stone, &terrain_cache.painted_bedrock)
        else {
            if visible {
                world_status.terrain_pending = true;
                world_status.active_frames = 0;
            }
            return;
        };
        if let Some(root) = build.root.take() {
            commands.entity(root).despawn();
        }
        let root = commands
            .spawn((
                SummitVista,
                tower_fit::transform(&frame, backdrop.tower_core, None),
                Visibility::Hidden,
            ))
            .id();
        #[cfg(not(target_arch = "wasm32"))]
        commands.entity(root).insert(crate::pipeline_preload::HiddenDestination);
        let mut material = crate::solid_terrain_material();
        material.extension.cave_reveal = Vec4::new(0., 0., 2., 0.);
        material.extension.painted_stone = Some(stone.clone());
        material.extension.painted_bedrock = Some(bedrock.clone());
        build.solid = Some(materials.add(material));
        build.root = Some(root);
        build.attachment = None;
        build.water.clear();
        build.next = 0;
        build.tasks.clear();
        build.scheduled = 0;
        build.publication = None;
        build.revision = backdrop.revision;
        build.profile_revision = profiles.revision;
        *status = SummitBackdropStatus::default();
    }
    let completed = if build.publication.is_none() {
        build.tasks.front_mut().and_then(|task| future::block_on(future::poll_once(task)))
    } else { None };
    if let Some(result) = completed {
        build.tasks.pop_front();
        match result {
            Ok(prepared) => {
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
                        prepared.source_triangles,
                        prepared.terrain
                            .authored_cells
                            .iter()
                            .filter(|p| p.is_some())
                            .count()
                    );
                }
                if let Some(attachment) = prepared.attachment {
                    build.attachment = Some(attachment);
                    commands.entity(build.root.unwrap()).insert(
                        tower_fit::transform(
                            &frame,
                            backdrop.tower_core,
                            build.attachment,
                        ),
                    );
                    if std::env::var_os("CRYSTAL_SUMMIT_TRACE").is_some() {
                        eprintln!(
                            "summit attached to actual tower eaves: {:?}", attachment
                        );
                    }
                }
                let positions = is_border.then(|| border_origins(&backdrop));
                let publication = prepare_map(
                    &mut meshes,
                    &mut images,
                    &mut materials,
                    build.root.unwrap(),
                    source,
                    prepared,
                    build.solid.as_ref().unwrap().clone(),
                    positions.as_deref(),
                );
                let index = build.next;
                if !is_border {
                    build.water.push((publication.textured.clone(), index));
                    status.maps.push(source.frame.map_id.to_string());
                }
                build.publication = Some(publication);
            }
            Err(error) => {
                // Later results belong to later source indices. Never consume
                // them under the failed map's unchanged publication index.
                build.tasks.clear();
                status.error = Some(format!("source panorama mesh failed: {error:?}"));
            }
        }
    }
    if let Some(publication) = &mut build.publication {
        if publication.advance(&mut commands, &mut meshes, &mut materials,
            if visible { ENTRY_PLACEMENTS_PER_FRAME } else { PRELOAD_PLACEMENTS_PER_FRAME }) {
            build.publication = None;
            build.next += 1;
        } else {
            if visible { world_status.terrain_pending = true; world_status.active_frames = 0; }
            return;
        }
    }
    if visible {
    let vista=tower_fit::transform(&frame, backdrop.tower_core, build.attachment);
    let scale=vista.scale.x;
    for (handle, index) in &build.water {
        if let Some(current) = materials.get(handle).map(|m|m.extension.water_enabled) {
            let map_zero = vista.translation
                + Vec3::new(
                    backdrop.maps[*index].origin_source.x as f32
                        * backdrop.maps[*index].frame.tile_size.x
                        * scale,
                    0.,
                    backdrop.maps[*index].origin_source.y as f32
                        * backdrop.maps[*index].frame.tile_size.y
                        * scale,
                );
            let next = Vec4::new(
                current.x,
                backdrop.maps[*index].frame.tile_size.x * scale,
                -map_zero.x,
                -map_zero.z,
            );
            if current != next {
                materials.get_mut(handle).unwrap().extension.water_enabled = next;
            }
        }
    }
    }
    if status.error.is_some() {
        if visible {
            world_status.terrain_pending = true;
            world_status.active_frames = 0;
        }
        return;
    }
    status.ready = !backdrop.source_pending
        && build.next == backdrop.maps.len() + usize::from(backdrop.border.is_some());
    // This scene holds entry readiness until it is complete. Avoid rendering
    // its growing geometry behind that loading transition. Hidden materials
    // use the existing staged-destination pipeline warmup before the reveal.
    for (_, mut visibility) in &mut roots {
        visibility.set_if_neq(if visible && status.ready {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    if !status.ready {
        if visible {
            world_status.terrain_pending = true;
            world_status.active_frames = 0;
        }
        while build.tasks.len() < MAPS_IN_FLIGHT {
            let index = build.scheduled;
            let total = backdrop.maps.len() + usize::from(backdrop.border.is_some());
            if index >= total || (backdrop.source_pending && index == backdrop.maps.len()) { break; }
            let source = if index == backdrop.maps.len() {
                backdrop.border.as_ref().unwrap().clone()
            } else {
                backdrop.maps[index].frame.clone()
            };
            if std::env::var_os("CRYSTAL_SUMMIT_TRACE").is_some() {
                eprintln!(
                    "summit building {} of {}: {}",
                    index + 1,
                    backdrop.maps.len(),
                    source.map_id
                );
            }
            let samples = TerrainImageSamples::capture(&source, &images);
            let document = profiles.document.clone();
            let library = terrain_cache.forest_library.clone();
            let is_border = index == backdrop.maps.len();
            let tower_core = backdrop.tower_core;
            build.tasks.push_back(AsyncComputeTaskPool::get().spawn(async move {
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
                Ok(prepare_map_geometry(source, built.terrain, groves, is_border, tower_core))
            }));
            build.scheduled += 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progressive_border_places_every_instance_with_one_shared_asset_set() {
        for (count,budget) in [(19_usize,PRELOAD_PLACEMENTS_PER_FRAME),(300,ENTRY_PLACEMENTS_PER_FRAME)] {
        let mut world = World::new();
        let root = world.spawn((Transform::default(), Visibility::Hidden)).id();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<VoxelMaterial>::default();
        let mesh = meshes.add(Mesh::from(Cuboid::default()));
        let material = materials.add(crate::solid_terrain_material());
        let translations = (0..count).map(|i|Vec3::new(i as f32 * 32.,0.,64.)).collect::<Vec<_>>();
        let groves = crate::shared_groves::prepare([-4.,4.,-4.,4.],0.,8.,[0,0],[false;4]);
        assert!(!groves.is_empty(),"exercise actual shared source groves");
        let mut publication = MapPublication {root, translations:translations.clone(), next:0,
            surfaces:vec![(mesh.clone(),material.clone())], trees:Vec::new(), groves,
            solid:material.clone(),textured:material};
        let mut counts = Vec::new();
        let mut done = false;
        for step in 0..3 {
            let mut queue = bevy::ecs::world::CommandQueue::default();
            { let mut commands = Commands::new(&mut queue,&world);
                done = publication.advance(&mut commands,&mut meshes,&mut materials,budget); }
            queue.apply(&mut world);
            counts.push(meshes.len());
            let count = world.query::<(&Mesh3d,&Transform)>().iter(&world)
                .filter(|(m,_)|m.0==mesh).count();
            assert!(count <= (step+1)*budget,"per-update placement budget");
            assert_eq!(done,step==2,"partial border must not report readiness");
        }
        assert!(done);
        assert!(counts.windows(2).all(|pair|pair[0]==pair[1]),
            "later placements must reuse the uploaded geometry, including groves");
        let mut actual = world.query::<(&Mesh3d,&Transform)>().iter(&world)
            .filter(|(m,_)|m.0==mesh).map(|(_,t)|t.translation).collect::<Vec<_>>();
        actual.sort_by(|a,b|a.x.total_cmp(&b.x));
        assert_eq!(actual,translations,"no omitted, repeated, or displaced source instances");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn panorama_waits_for_shared_paint_and_reuses_the_active_image_handles() {
        use crystal_render_api::{VisualTile,VisualTileSource};
        AsyncComputeTaskPool::get_or_init(||bevy::tasks::TaskPoolBuilder::new().num_threads(1).build());
        for map in ["TinTowerRoof","TinTower7F","TinTower8F","TinTower9F"] {
        let mut app=App::new();
        app.insert_resource(VisualWorldFrame {
            active:true,map_id:map.into(),
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
            .init_resource::<Build>()
            .insert_resource(VoxelViewStatus {active:true,active_frames:123,..default()})
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .add_systems(Update,sync);
        app.update();
        assert_eq!(app.world().resource::<VoxelViewStatus>().terrain_pending,map=="TinTowerRoof");
        if map!="TinTowerRoof" {assert_eq!(app.world().resource::<VoxelViewStatus>().active_frames,123);}
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
        let root=app.world().resource::<Build>().root.unwrap();
        assert!(!app.world().resource::<SummitBackdropStatus>().ready);
        assert_eq!(*app.world().get::<Visibility>(root).unwrap(),Visibility::Hidden,
            "sharing paint does not make the unfinished panorama visible");
        assert!(app.world().get::<crate::pipeline_preload::HiddenDestination>(root).is_some());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn cached_panorama_app() -> (App,Entity,Handle<Mesh>,Handle<VoxelMaterial>) {
        use bevy::asset::AssetPlugin;
        use crystal_render_api::{VisualTile,VisualTileSource};
        AsyncComputeTaskPool::get_or_init(||bevy::tasks::TaskPoolBuilder::new().num_threads(1).build());
        let mut app=App::new();
        app.add_plugins((MinimalPlugins,AssetPlugin::default())).init_asset::<VoxelMaterial>();
        app.insert_resource(VisualWorldFrame {
            active:true,map_id:"TinTower7F".into(),tile_size:Vec2::splat(8.),grid_size:UVec2::splat(16),
            source_map_size_core_tiles:Some(UVec2::splat(8)),
            tiles:vec![VisualTile {column:0,row:0,texture:Handle::default(),animation_frames:None,priority:false,
                source:VisualTileSource {tileset_id:"tower".into(),metatile_id:0,tile_index:0,subtile_column:0,subtile_row:0}}],
            ..default()
        }).insert_resource(VoxelViewSettings {enabled:true,..default()})
            .insert_resource(SummitBackdrop {revision:Some(1),
                maps:vec![SummitBackdropMap {origin_source:IVec2::ZERO,
                    frame:Arc::new(VisualWorldFrame {tile_size:Vec2::splat(8.),..default()})}],..default()})
            .init_resource::<crate::TerrainRevisionCache>()
            .init_resource::<crate::live_profiles::LiveProfiles>()
            .insert_resource(SummitBackdropStatus {ready:true,maps:vec!["cached".into()],error:None})
            .insert_resource(VoxelViewStatus {active:true,active_frames:123,..default()})
            .init_resource::<Assets<Mesh>>().init_resource::<Assets<Image>>()
            .add_systems(Update,sync);
        let (stone,bedrock)={let mut images=app.world_mut().resource_mut::<Assets<Image>>();
            (images.add(Image::default()),images.add(Image::default()))};
        {let mut cache=app.world_mut().resource_mut::<crate::TerrainRevisionCache>();
            cache.painted_stone=Some(stone);cache.painted_bedrock=Some(bedrock);}
        let material=app.world_mut().resource_mut::<Assets<VoxelMaterial>>().add(crate::solid_terrain_material());
        let mesh=app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::default()));
        let root=app.world_mut().spawn((SummitVista,Transform::default(),Visibility::Visible)).id();
        let child=app.world_mut().spawn((Mesh3d(mesh.clone()),MeshMaterial3d(material.clone()))).id();
        app.world_mut().entity_mut(root).add_child(child);
        app.insert_resource(Build {revision:Some(1),root:Some(root),next:1,scheduled:1,
            solid:Some(material.clone()),water:vec![(material.clone(),0)],..default()});
        (app,root,mesh,material)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_prefetch_is_bounded_and_unrelated_maps_cancel_every_queued_task() {
        let (mut app,root,_,_)=cached_panorama_app();
        let source=app.world().resource::<SummitBackdrop>().maps[0].clone();
        app.world_mut().resource_mut::<SummitBackdrop>().maps=vec![source;5];
        {let mut build=app.world_mut().resource_mut::<Build>();build.next=0;build.scheduled=0;}
        app.world_mut().resource_mut::<SummitBackdropStatus>().ready=false;
        app.update();
        let build=app.world().resource::<Build>();
        assert_eq!(build.scheduled,MAPS_IN_FLIGHT);
        assert_eq!(build.tasks.len(),MAPS_IN_FLIGHT);
        assert_eq!(build.next,0,"prefetch must not claim publication");
        assert!(!app.world().resource::<VoxelViewStatus>().terrain_pending);
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id="NewBarkTown".into();
        app.update();app.update();
        let build=app.world().resource::<Build>();
        assert!(build.tasks.is_empty());assert_eq!(build.scheduled,0);assert!(build.root.is_none());
        assert!(app.world().get_entity(root).is_err());
        assert!(app.world().resource::<SummitBackdropStatus>().maps.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn prefetched_results_publish_in_source_order_after_current_publication() {
        use std::sync::atomic::{AtomicBool,Ordering};
        let (mut app,root,_,material)=cached_panorama_app();
        let frames=["first","second","third"].map(|name|Arc::new(VisualWorldFrame {
            map_id:name.into(),tile_size:Vec2::splat(8.),grid_size:UVec2::ONE,..default()
        }));
        app.world_mut().resource_mut::<SummitBackdrop>().maps=frames.iter().map(|frame|
            SummitBackdropMap {frame:frame.clone(),origin_source:IVec2::ZERO}).collect();
        {let mut status=app.world_mut().resource_mut::<SummitBackdropStatus>();
            status.ready=false;status.maps=vec!["first".into()];}
        let gate=Arc::new(AtomicBool::new(false));
        let first_gate=gate.clone();let first=frames[1].clone();let second=frames[2].clone();
        let pool=AsyncComputeTaskPool::get();
        let pending_first=pool.spawn(async move {
            while !first_gate.load(Ordering::Acquire) {future::yield_now().await;}
            Ok(prepare_map_geometry(first,TerrainMeshData::default(),Vec::new(),false,IVec2::ZERO))
        });
        let completed_second=pool.spawn(async move {
            let prepared=prepare_map_geometry(second,TerrainMeshData::default(),Vec::new(),false,IVec2::ZERO);
            gate.store(true,Ordering::Release);Ok(prepared)
        });
        {let mut build=app.world_mut().resource_mut::<Build>();
            build.next=0;build.scheduled=3;build.tasks=VecDeque::from([pending_first,completed_second]);
            build.publication=Some(MapPublication {root,translations:Vec::new(),next:0,
                surfaces:Vec::new(),trees:Vec::new(),groves:Vec::new(),
                solid:material.clone(),textured:material});}
        app.update();
        assert_eq!(app.world().resource::<Build>().next,1);
        assert_eq!(app.world().resource::<Build>().tasks.len(),2,
            "queued results cannot overtake the current publication");
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        while !app.world().resource::<SummitBackdropStatus>().ready {
            assert!(std::time::Instant::now()<deadline,"ordered worker results must settle");
            app.update();std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(app.world().resource::<SummitBackdropStatus>().maps,["first","second","third"]);
        assert_eq!(app.world().resource::<Build>().next,3);
        assert!(app.world().resource::<Build>().tasks.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_map_discards_prefetched_success_without_publishing_under_wrong_source() {
        let (mut app,_,_,_)=cached_panorama_app();
        let frames=["first","second"].map(|name|Arc::new(VisualWorldFrame {
            map_id:name.into(),tile_size:Vec2::splat(8.),grid_size:UVec2::ONE,..default()
        }));
        app.world_mut().resource_mut::<SummitBackdrop>().maps=frames.iter().map(|frame|
            SummitBackdropMap {frame:frame.clone(),origin_source:IVec2::ZERO}).collect();
        {let mut status=app.world_mut().resource_mut::<SummitBackdropStatus>();
            status.ready=false;status.maps.clear();}
        let second=frames[1].clone();let pool=AsyncComputeTaskPool::get();
        let failed=pool.spawn(async {Err(crate::TerrainMeshError::InactiveFrame)});
        let later=pool.spawn(async move {
            Ok(prepare_map_geometry(second,TerrainMeshData::default(),Vec::new(),false,IVec2::ZERO))
        });
        {let mut build=app.world_mut().resource_mut::<Build>();
            build.next=0;build.scheduled=2;build.tasks=VecDeque::from([failed,later]);}
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        while app.world().resource::<SummitBackdropStatus>().error.is_none() {
            assert!(std::time::Instant::now()<deadline,"failed worker result must settle");
            app.update();std::thread::sleep(std::time::Duration::from_millis(1));
        }
        for _ in 0..16 {app.update();}
        let build=app.world().resource::<Build>();
        assert!(build.tasks.is_empty(),"failure cancels every prefetched result");
        assert!(build.publication.is_none());assert_eq!(build.next,0);
        let status=app.world().resource::<SummitBackdropStatus>();
        assert!(status.maps.is_empty(),"later geometry cannot masquerade as the failed source");
        assert!(!status.ready);
        assert!(status.error.as_ref().unwrap().contains("InactiveFrame"));
        assert!(!app.world().resource::<VoxelViewStatus>().terrain_pending,
            "a failed hidden preload must leave the foreground room playable");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn hidden_upper_floor_cache_reuses_geometry_and_never_blocks_the_room() {
        use bevy::asset::AssetEvent;
        let (mut app,root,mesh,material)=cached_panorama_app();
        app.update();
        app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>().clear();
        for map in ["TinTower7F","TinTower8F","TinTower9F"] {
            app.world_mut().resource_mut::<VisualWorldFrame>().map_id=map.into();
            for _ in 0..16 {
                app.update();
                let events=app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>().drain().collect::<Vec<_>>();
                assert!(!events.iter().any(|event|event.is_modified(&material)));
                assert!(!app.world().resource::<VoxelViewStatus>().terrain_pending);
                assert_eq!(app.world().resource::<VoxelViewStatus>().active_frames,123);
            }
            assert_eq!(app.world().resource::<Build>().root,Some(root));
            assert!(app.world().resource::<Build>().tasks.is_empty());
            assert_eq!(*app.world().get::<Visibility>(root).unwrap(),Visibility::Hidden);
        }
        app.world_mut().resource_mut::<SummitBackdrop>().source_pending=true;
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id="TinTowerRoof".into();
        app.update();
        assert!(!app.world().resource::<SummitBackdropStatus>().ready);
        assert!(app.world().resource::<VoxelViewStatus>().terrain_pending);
        assert_eq!(*app.world().get::<Visibility>(root).unwrap(),Visibility::Hidden,
            "unfinished geometry must not consume roof rendering while entry waits");
        assert!(app.world().resource::<Build>().tasks.is_empty(),"wait for the next source map, not a missing border");
        app.world_mut().resource_mut::<SummitBackdrop>().source_pending=false;
        app.world_mut().resource_mut::<VoxelViewStatus>().terrain_pending=false;
        app.update();
        assert!(app.world().resource::<SummitBackdropStatus>().ready);
        assert_eq!(*app.world().get::<Visibility>(root).unwrap(),Visibility::Visible);
        assert_eq!(app.world().resource::<Build>().root,Some(root));
        assert_eq!(app.world().resource::<Build>().solid.as_ref(),Some(&material));
        assert!(app.world().resource::<Assets<Mesh>>().get(&mesh).is_some());
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(),1);
        app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>().clear();
        for _ in 0..64 {
            app.update();
            let events=app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>().drain().collect::<Vec<_>>();
            assert!(!events.iter().any(|event|event.is_modified(&material)),"unchanged water registration must retain its material");
        }
        app.world_mut().resource_mut::<VisualWorldFrame>().grid_origin.x+=4;
        app.update();
        let events=app.world_mut().resource_mut::<Messages<AssetEvent<VoxelMaterial>>>().drain().collect::<Vec<_>>();
        assert!(events.iter().any(|event|event.is_modified(&material)),"real anchor changes still update water coordinates");
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id="TinTower6F".into();
        app.update();
        assert!(app.world().get_entity(root).is_err());
        assert!(app.world().resource::<Build>().root.is_none());
        assert!(app.world().resource::<SummitBackdropStatus>().maps.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn cached_panorama_revalidates_source_and_profile_revisions_before_reuse() {
        for profile in [false,true] {
            let (mut app,root,_,_)=cached_panorama_app();
            if profile {app.world_mut().resource_mut::<crate::live_profiles::LiveProfiles>().revision+=1;}
            else {app.world_mut().resource_mut::<SummitBackdrop>().revision=Some(2);}
            app.update();
            assert!(app.world().get_entity(root).is_err());
            let new_root=app.world().resource::<Build>().root.unwrap();
            assert_ne!(new_root,root);
            assert_eq!(*app.world().get::<Visibility>(new_root).unwrap(),Visibility::Hidden);
            assert!(!app.world().resource::<VoxelViewStatus>().terrain_pending);
            assert!(!app.world().resource::<SummitBackdropStatus>().ready);
        }
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
        let door_world = a + Vec3::new((37.5) * 16., 0., 7.5 * 16.);
        assert_eq!(door_world, Vec3::new(-160., -DROP, -160.));
    }
}
