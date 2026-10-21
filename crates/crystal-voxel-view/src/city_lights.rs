//! Clock-driven exterior luminaires. Named model materials own emission;
//! finite window rectangles illuminate nearby geometry with diffuse area light.
use crate::{
    exterior_models::{Kind, Model},
    mesh::SurfaceMeshData,
};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

pub(crate) const MAX_LIGHTS: usize = 32;
pub(crate) const UV_BASE: f32 = 1248.;
pub(crate) const SOURCE_UV_BASE: f32 = 1280.;

pub(crate) fn enabled(map: &str) -> bool {
    crate::cave_water::outdoor(map)
        && map.strip_prefix("Route").is_none_or(|route| {
            !route.is_empty() && route.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// Local RTC only: fade on during dusk, off during dawn, continuous at midnight.
pub(crate) fn gain(seconds: u32) -> f32 {
    let hour = (seconds % 86400) as f32 / 3600.;
    let smooth = |a: f32, b: f32, v: f32| {
        let t = ((v - a) / (b - a)).clamp(0., 1.);
        t * t * (3. - 2. * t)
    };
    if hour < 12. {
        1. - smooth(4., 6., hour)
    } else {
        smooth(17., 19., hour)
    }
}

/// Tag only authenticated glass, marquee bulbs, station clock and shop letters.
/// Geometry, authored daytime color, frames and gameplay stay untouched.
pub(crate) fn mark(mesh: &mut SurfaceMeshData, first: usize, model: &Model, kind: Kind) {
    let center = (Vec3::from_array(model.min) + Vec3::from_array(model.max)) * 0.5;
    for (vertices, material) in model.material_ranges() {
        for i in vertices {
            let role = match (kind, material) {
                (Kind::ModernArcade | Kind::KantoArcade, "yellow" | "coral") => 2.,
                (Kind::ModernArcade | Kind::KantoArcade, "glass_light")
                    if model.surface.positions[i][1] > 2.4 =>
                {
                    2.
                }
                (Kind::RadioTower, "coral") => 3.,
                (
                    Kind::RadioTower
                    | Kind::ModernStation
                    | Kind::ModernDepartment
                    | Kind::KantoStation
                    | Kind::KantoDepartment
                    | Kind::Silph,
                    "cream",
                ) => 1.,
                (Kind::Lighthouse, "yellow")
                    if (7.8..9.2).contains(&model.surface.positions[i][1]) =>
                {
                    0.
                }
                (_, "glass" | "glass_light" | "glass_dark") => 0.,
                _ => continue,
            };
            let n = Vec3::from_array(model.surface.normals[i]);
            let outward = n.dot(
                (Vec3::from_array(model.surface.positions[i]) - center) * Vec3::new(1., 0., 1.),
            ) > 0.;
            // The reflection strips emit with the pane but never own a second
            // spill source. Inward glass faces must not illuminate the building.
            let role = if role == 0. && (!outward || material == "glass_light") {
                0.25
            } else {
                role
            };
            // Distinct facade ownership survives batching, cutaway and workers.
            let base =
                if role <= 0.25 && mesh.uvs[first + i] == crate::scenery_finish::SOURCE_PAINT_UV {
                    SOURCE_UV_BASE
                } else {
                    UV_BASE
                };
            mesh.uvs[first + i] = [first as f32 / 16_777_216., base + role];
        }
    }
}

/// Preserve the native glass optics and source paint while adding emission.
pub(crate) fn mark_window(
    mesh: &mut SurfaceMeshData,
    vertices: std::ops::Range<usize>,
    owner: usize,
    source_paint: bool,
) {
    for i in vertices {
        if crate::scenery_finish::town_glass_coordinates(mesh.uvs[i]).is_some() {
            continue;
        }
        let source = source_paint || mesh.uvs[i] == crate::scenery_finish::SOURCE_PAINT_UV;
        let outward = mesh.normals[i][2] > 0.9;
        mesh.uvs[i] = [
            owner as f32 / 16_777_216.,
            (if source { SOURCE_UV_BASE } else { UV_BASE }) + if outward { 0. } else { 0.25 },
        ];
    }
}

pub(crate) fn source_window_tag(uv: [f32; 2]) -> bool {
    (SOURCE_UV_BASE..=SOURCE_UV_BASE + 0.25).contains(&uv[1])
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Emitter {
    pub position: Vec4,
    pub direction: Vec4,
}

/// Recover each separate, outward-facing pane from its authored triangles.
/// Frames and reflection strips never expand or join the light rectangle.
pub(crate) fn collect(mesh: &SurfaceMeshData) -> Vec<Emitter> {
    let mut selected = HashMap::new();
    let mut vertices = Vec::new();
    let mut sides = Vec::new();
    for i in 0..mesh.positions.len() {
        let native = crate::scenery_finish::town_glass_coordinates(mesh.uvs[i]).is_some();
        let source_glass = crate::scenery_finish::source_glass_coordinates(mesh.uvs[i]);
        let native = native || (source_glass.is_some_and(|(_, lit)| lit)
            && mesh.normals[i][2] > 0.9);
        if mesh.uvs[i][1] != UV_BASE && mesh.uvs[i][1] != SOURCE_UV_BASE && !native {
            continue;
        }
        let n = Vec3::from_array(mesh.normals[i]);
        let side = if n.x > 0.9 {
            0
        } else if n.x < -0.9 {
            1
        } else if n.z > 0.9 {
            2
        } else if n.z < -0.9 {
            3
        } else {
            continue;
        };
        selected.insert(i, vertices.len());
        vertices.push(i);
        sides.push(side);
    }
    let mut parent: Vec<_> = (0..vertices.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    fn join(parent: &mut [usize], a: usize, b: usize) {
        let a = root(parent, a);
        let b = root(parent, b);
        parent[b] = a;
    }
    let mut welded = HashMap::new();
    for (local, &i) in vertices.iter().enumerate() {
        let key = (
            mesh.positions[i].map(|v| if v == 0. { 0 } else { v.to_bits() }),
            sides[local],
        );
        if let Some(&previous) = welded.get(&key) {
            join(&mut parent, previous, local);
        } else {
            welded.insert(key, local);
        }
    }
    for triangle in mesh.indices.chunks_exact(3) {
        if let (Some(&a), Some(&b), Some(&c)) = (
            selected.get(&(triangle[0] as usize)),
            selected.get(&(triangle[1] as usize)),
            selected.get(&(triangle[2] as usize)),
        ) {
            if sides[a] == sides[b] && sides[a] == sides[c] {
                join(&mut parent, a, b);
                join(&mut parent, a, c);
            }
        }
    }
    let mut panes = BTreeMap::<usize, (Vec3, Vec3, usize)>::new();
    for (local, &i) in vertices.iter().enumerate() {
        let p = Vec3::from_array(mesh.positions[i]);
        let entry = panes
            .entry(root(&mut parent, local))
            .or_insert((p, p, sides[local]));
        entry.0 = entry.0.min(p);
        entry.1 = entry.1.max(p);
    }
    panes
        .into_values()
        .filter_map(|(low, high, side)| {
            let normal = [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z][side];
            let width = (high - low).dot(Vec3::new(normal.z.abs(), 0., normal.x.abs()));
            let height = high.y - low.y;
            if width < 1. || height < 2. {
                return None;
            }
            Some(Emitter {
                position: ((low + high) * 0.5 + normal * 0.5).extend(width * 0.5),
                direction: normal.extend(height * 0.5),
            })
        })
        .collect()
}

pub(crate) fn nearest(
    emitters: &[Emitter],
    player: Vec3,
) -> ([Vec4; MAX_LIGHTS], [Vec4; MAX_LIGHTS]) {
    let mut sorted: Vec<_> = emitters.iter().enumerate().collect();
    sorted.sort_by(|(ai, a), (bi, b)| {
        a.position
            .truncate()
            .distance_squared(player)
            .total_cmp(&b.position.truncate().distance_squared(player))
            .then(ai.cmp(bi))
    });
    let mut positions = [Vec4::ZERO; MAX_LIGHTS];
    let mut directions = positions;
    for (i, (_, emitter)) in sorted.into_iter().take(MAX_LIGHTS).enumerate() {
        positions[i] = emitter.position;
        directions[i] = emitter.direction;
    }
    (positions, directions)
}

// Preserve holes and emitter order, but never send the shader through unused
// trailing slots. A zero-width emitter cannot contribute to the existing loop.
fn emitter_span(positions: &[Vec4; MAX_LIGHTS]) -> usize {
    positions.iter().rposition(|p| p.w > 0.).map_or(0, |i| i + 1)
}

pub(crate) fn sync(
    frame: Res<crystal_render_api::VisualWorldFrame>,
    clock: Option<Res<crystal_render_api::VisualWorldClock>>,
    status: Res<crate::VoxelViewStatus>,
    cache: Res<crate::TerrainRevisionCache>,
    mut materials: ResMut<Assets<crate::VoxelMaterial>>,
) {
    if !frame.is_changed()
        && !cache.is_changed()
        && !status.is_changed()
        && !materials.is_changed()
        && !clock.as_ref().is_some_and(|clock| clock.is_changed())
    {
        return;
    }
    let active = status.active
        && enabled(&frame.map_id)
        && !crate::interior_cutaway::is_cave(&frame)
        && !frame
            .tiles
            .first()
            .is_some_and(|tile| crate::interior::has_back_wall(&tile.source.tileset_id))
        && cache
            .built_frame
            .as_ref()
            .is_some_and(|built| built.map_id == frame.map_id);
    let seconds = clock.as_ref().and_then(|v| v.seconds_of_day());
    let strength = if active {
        seconds.map_or(0., gain)
    } else {
        0.
    };
    // All palette materials consume world positions. Use the exact retained
    // terrain transform, shared with rendering, so camera scrolling cannot
    // slide the light pools relative to the authored facade.
    let offset =
        crate::retained_terrain_transform(&frame, &cache).map_or(Vec3::ZERO, |t| t.translation);
    let player = frame
        .actors
        .iter()
        .find(|a| a.id == crystal_render_api::VisualActorId::Player)
        .map_or(Vec3::ZERO, |a| {
            crate::visual_point_to_voxel(crate::footing::actor_ground_point(&frame, a), 0.)
        });
    let (positions, directions) = if strength > 0. {
        nearest(&cache.city_emitters, player - offset)
    } else {
        ([Vec4::ZERO; MAX_LIGHTS], [Vec4::ZERO; MAX_LIGHTS])
    };
    let positions = positions.map(|p| {
        if p.w > 0. {
            (p.truncate() + offset).extend(p.w)
        } else {
            p
        }
    });
    let night = Vec4::new(strength, emitter_span(&positions) as f32, 0., 0.);
    // Summit materials use the existing isolated-town lighting mode. Their
    // authored windows still emit at night even though the foreground atlas
    // belongs to a tower room. No rooftop-local spill rectangles belong below.
    let panorama_night = Vec4::new(if status.active && crate::summit_vista::enabled(&frame) {
        seconds.map_or(0., gain)
    } else { 0. }, 0., 0., 0.);
    let no_windows = [Vec4::ZERO; MAX_LIGHTS];
    let changed: Vec<_> = materials
        .iter()
        .filter_map(|(id, m)| {
            let panorama = m.extension.cave_reveal.z > 1.5;
            let (night, windows, planes) = if panorama {
                (panorama_night, &no_windows, &no_windows)
            } else { (night, &positions, &directions) };
            (m.extension.city_night != night
                || m.extension.city_windows != *windows
                || m.extension.city_window_planes != *planes)
                .then_some((id, panorama))
        })
        .collect();
    for (id, panorama) in changed {
        if let Some(mut m) = materials.get_mut(id) {
            m.extension.city_night = if panorama { panorama_night } else { night };
            m.extension.city_windows = if panorama { no_windows } else { positions };
            m.extension.city_window_planes = if panorama { no_windows } else { directions };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summit_windows_follow_the_clock_without_foreground_light_spill_or_idle_material_churn() {
        use bevy::asset::{AssetEvent, AssetPlugin};
        use crystal_render_api::{VisualWorldClock, VisualWorldFrame, VisualTile, VisualTileSource};
        let frame = VisualWorldFrame { active: true, map_id: "TinTowerRoof".into(),
            tile_size: Vec2::splat(32.), tiles: vec![VisualTile { column: 0, row: 0,
                texture: Handle::default(), priority: false, animation_frames: None,
                source: VisualTileSource { tileset_id: "tower".into(), metatile_id: 2,
                    tile_index: 2, subtile_column: 0, subtile_row: 0 } }], ..default() };
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default())).init_asset::<crate::VoxelMaterial>();
        app.insert_resource(frame.clone())
            .insert_resource(VisualWorldClock { minute_of_day: Some(22*60), second_of_minute: 0 })
            .insert_resource(crate::VoxelViewStatus { active: true, ..default() })
            .insert_resource(crate::TerrainRevisionCache { built_frame: Some(frame),
                city_emitters: vec![Emitter { position: Vec4::new(1., 16., 2., 64.), direction: Vec4::Z }],
                ..default() })
            .init_resource::<Assets<crate::VoxelMaterial>>()
            .add_systems(Update, sync);
        let mut distant = crate::solid_terrain_material();
        distant.extension.cave_reveal.z = 2.;
        let distant = app.world_mut().resource_mut::<Assets<crate::VoxelMaterial>>().add(distant);
        let foreground = app.world_mut().resource_mut::<Assets<crate::VoxelMaterial>>()
            .add(crate::solid_terrain_material());
        for (hour, expected) in [(22,1.), (5,0.5), (12,0.), (22,1.)] {
            app.world_mut().resource_mut::<VisualWorldClock>().minute_of_day = Some(hour*60);
            app.update();
            let materials = app.world().resource::<Assets<crate::VoxelMaterial>>();
            let m = materials.get(&distant).unwrap();
            assert_eq!(m.extension.city_night, Vec4::new(expected, 0., 0., 0.));
            assert_eq!(m.extension.city_windows, [Vec4::ZERO; MAX_LIGHTS]);
            assert_eq!(m.extension.city_window_planes, [Vec4::ZERO; MAX_LIGHTS]);
            assert_eq!(materials.get(&foreground).unwrap().extension.city_night, Vec4::ZERO);
        }
        app.world_mut().resource_mut::<Messages<AssetEvent<crate::VoxelMaterial>>>().clear();
        app.update();
        assert!(!app.world_mut().resource_mut::<Messages<AssetEvent<crate::VoxelMaterial>>>().drain()
            .any(|event| matches!(event, AssetEvent::Modified { .. })));
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "TinTower9F".into();
        app.update();
        assert_eq!(app.world().resource::<Assets<crate::VoxelMaterial>>().get(&distant).unwrap()
            .extension.city_night, Vec4::ZERO);
    }
    #[test]
    fn shader_emitter_span_keeps_holes_and_every_original_contributor() {
        for last in 0..MAX_LIGHTS {
            let mut positions = [Vec4::ZERO; MAX_LIGHTS];
            positions[last] = Vec4::new(1., 2., 3., 4.);
            positions[0].w = if last > 0 { -1. } else { 4. };
            assert_eq!(emitter_span(&positions), last + 1);
            let original: Vec<_> = positions.iter().enumerate().filter(|(_, p)| p.w > 0.).collect();
            let bounded: Vec<_> = positions[..emitter_span(&positions)].iter().enumerate()
                .filter(|(_, p)| p.w > 0.).collect();
            assert_eq!(original, bounded);
        }
        assert_eq!(emitter_span(&[Vec4::ZERO; MAX_LIGHTS]), 0);
    }

    #[test]
    fn city_lights_update_idle_clock_anchor_all_materials_and_clear_on_exit() {
        use crate::{TerrainCacheKey, TerrainRevisionCache, VoxelViewStatus};
        use crystal_render_api::{VisualWorldClock, VisualWorldFrame};
        let frame = VisualWorldFrame {
            map_id: "GoldenrodCity".into(),
            center: Vec2::new(64., -32.),
            tile_size: Vec2::splat(32.),
            ..default()
        };
        let mut app = App::new();
        app.insert_resource(Assets::<crate::VoxelMaterial>::default())
            .insert_resource(frame.clone())
            .insert_resource(VisualWorldClock {
                minute_of_day: Some(0),
                second_of_minute: 0,
            })
            .insert_resource(VoxelViewStatus {
                active: true,
                ..default()
            })
            .insert_resource(TerrainRevisionCache {
                key: Some(TerrainCacheKey::from_frame(&frame)),
                built_frame: Some(frame.clone()),
                city_emitters: vec![Emitter {
                    position: Vec4::new(1., 16., 2., 64.),
                    direction: Vec4::Z,
                }],
                ..default()
            })
            .add_systems(Update, sync);
        let handles: Vec<_> = (0..2)
            .map(|_| {
                app.world_mut()
                    .resource_mut::<Assets<crate::VoxelMaterial>>()
                    .add(crate::solid_terrain_material())
            })
            .collect();
        app.update();
        for h in &handles {
            let m = app
                .world()
                .resource::<Assets<crate::VoxelMaterial>>()
                .get(h)
                .unwrap();
            assert_eq!(m.extension.city_night, Vec4::new(1., 1., 0., 0.));
        }
        assert_eq!(
            app.world()
                .resource::<Assets<crate::VoxelMaterial>>()
                .get(&handles[0])
                .unwrap()
                .extension
                .city_windows[0],
            Vec4::new(65., 16., 34., 64.)
        );
        // Deferred building/actor parts can create materials after the map and
        // clock have settled. They must inherit the same night state at once.
        app.update();
        let late = app.world_mut().resource_mut::<Assets<crate::VoxelMaterial>>()
            .add(crate::solid_terrain_material());
        app.update();
        let material = app.world().resource::<Assets<crate::VoxelMaterial>>()
            .get(&late).unwrap();
        assert_eq!(material.extension.city_night, Vec4::new(1., 1., 0., 0.));
        assert_eq!(material.extension.city_windows[0], Vec4::new(65., 16., 34., 64.));
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .grid_origin
            .x += 1;
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<crate::VoxelMaterial>>()
                .get(&handles[0])
                .unwrap()
                .extension
                .city_windows[0],
            Vec4::new(33., 16., 34., 64.)
        );
        app.world_mut()
            .resource_mut::<VisualWorldClock>()
            .minute_of_day = Some(12 * 60);
        app.update();
        for h in &handles {
            let m = app
                .world()
                .resource::<Assets<crate::VoxelMaterial>>()
                .get(h)
                .unwrap();
            assert_eq!(m.extension.city_night, Vec4::ZERO);
            assert_eq!(m.extension.city_windows, [Vec4::ZERO; MAX_LIGHTS]);
        }
        app.world_mut()
            .resource_mut::<VisualWorldClock>()
            .minute_of_day = Some(0);
        // No receiving lights must not disable authored night pane emission.
        app.world_mut().resource_mut::<TerrainRevisionCache>().city_emitters.clear();
        app.update();
        for handle in &handles {
            let material = app.world().resource::<Assets<crate::VoxelMaterial>>().get(handle).unwrap();
            assert_eq!(material.extension.city_night, Vec4::new(1., 0., 0., 0.));
            assert_eq!(material.extension.city_windows, [Vec4::ZERO; MAX_LIGHTS]);
        }
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "GoldenrodDeptStore1F".into();
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<crate::VoxelMaterial>>()
                .get(&handles[0])
                .unwrap()
                .extension
                .city_night,
            Vec4::ZERO
        );
    }

    #[test]
    fn city_lights_recover_separate_window_rectangles_without_lighting_the_wall_between() {
        let mut mesh = SurfaceMeshData::default();
        for x in [0., 32.] {
            let first = mesh.positions.len() as u32;
            mesh.positions.extend([
                [x, 0., 0.],
                [x + 8., 0., 0.],
                [x + 8., 16., 0.],
                [x, 16., 0.],
            ]);
            mesh.normals.extend([[0., 0., 1.]; 4]);
            mesh.uvs.extend([[0., UV_BASE]; 4]);
            mesh.colors.extend([[0.2, 0.4, 0.5, 1.]; 4]);
            mesh.indices
                .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        }
        let windows = collect(&mesh);
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].position, Vec4::new(4., 8., 0.5, 4.));
        assert_eq!(windows[1].position, Vec4::new(36., 8., 0.5, 4.));
        assert_eq!(windows[0].direction, Vec4::new(0., 0., 1., 8.));
    }

    #[test]
    fn city_lights_follow_dusk_dawn_and_midnight_without_jumps() {
        for hour in [0, 1, 3, 19, 20, 23, 24] {
            assert_eq!(gain(hour * 3600), 1.);
        }
        for hour in [6, 9, 12, 15, 17] {
            assert_eq!(gain(hour * 3600), 0.);
        }
        assert_eq!(gain(5 * 3600), 0.5);
        assert_eq!(gain(18 * 3600), 0.5);
        for s in 0..86400 {
            assert!((gain(s + 1) - gain(s)).abs() < 0.0003);
        }
        for map in [
            "GoldenrodCity",
            "EcruteakCity",
            "NewBarkTown",
            "PalletTown",
            "CinnabarIsland",
            "SaffronCity",
        ] {
            assert!(enabled(map));
        }
        assert!(!enabled("GoldenrodDeptStore1F"));
        assert!(!enabled("Route32Pokecenter1F"));
    }
    #[test]
    fn city_lights_cover_all_johto_building_kits_and_preserve_native_panes() {
        use crate::new_bark_models::{ModelKind::*, model};
        for kind in [
            House,
            PlayerHouse,
            Lab,
            Pokecenter,
            Mart,
            RouteGate,
            TraditionalHouse,
            VioletGym,
            SproutTower,
        ] {
            let m = model(kind);
            let mut mesh = SurfaceMeshData::default();
            m.append_fitted(&mut mesh, [0., 96., 0., 96.], 0., 16., None);
            m.light_fitted_source_paint(&mut mesh, 0);
            let before = mesh.clone();
            m.mark_city_lights(&mut mesh, 0);
            assert_eq!(mesh.positions, before.positions);
            assert_eq!(mesh.colors, before.colors);
            assert!(!collect(&mesh).is_empty(), "{kind:?}");
            for (i, uv) in before.uvs.iter().enumerate() {
                if crate::scenery_finish::town_glass_coordinates(*uv).is_some() {
                    assert_eq!(mesh.uvs[i], *uv, "daytime pane optics");
                }
            }
        }
    }

    #[test]
    fn radio_tower_name_is_five_solid_glyphs_in_front_of_its_panel_with_a_separate_light_role() {
        let m=crate::exterior_models::model(Kind::RadioTower);
        let panel=m.surface.positions.iter().enumerate().filter(|(i,_)|m.vertex_material(*i)=="violet_dark")
            .map(|(_,p)|*p).collect::<Vec<_>>();
        let front=panel.iter().map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max);
        let bounds=[panel.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min),
            panel.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max),
            panel.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min),
            panel.iter().map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max)];
        let mut spans=Vec::new();
        for tri in m.surface.indices.chunks_exact(3) {
            if m.vertex_material(tri[0]as usize)!="cream" {continue;}
            let p=tri.iter().map(|i|m.surface.positions[*i as usize]).collect::<Vec<_>>();
            assert!(p.iter().all(|p|p[2]>front && p[0]>bounds[0] && p[0]<bounds[1] && p[1]>bounds[2] && p[1]<bounds[3]),
                "name strokes sit entirely in front of and within the existing sign");
            spans.push([p.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min),
                p.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max)]);
        }
        spans.sort_by(|a,b|a[0].total_cmp(&b[0]));
        let mut groups=Vec::<[f32;2]>::new();
        for span in spans {
            if let Some(last)=groups.last_mut() && span[0]<=last[1]+0.00001 {last[1]=last[1].max(span[1]);}
            else {groups.push(span);}
        }
        assert_eq!(groups.len(),5,"five separate readable RADIO glyphs");
        let mut mesh=SurfaceMeshData::default();m.append_fitted(&mut mesh,[0.,64.,0.,96.],0.,16.,Some(24.));
        let before=mesh.clone();mark(&mut mesh,0,m,Kind::RadioTower);
        assert_eq!(mesh.positions,before.positions);assert_eq!(mesh.colors,before.colors);
        assert_eq!(mesh.normals,before.normals);assert_eq!(mesh.indices,before.indices);
        for i in 0..mesh.positions.len() {
            if m.vertex_material(i)=="cream" {assert_eq!(mesh.uvs[i][1],UV_BASE+1.);}
            if m.vertex_material(i)=="coral" {assert_eq!(mesh.uvs[i][1],UV_BASE+3.);}
        }
    }

    #[test]
    fn city_lights_use_named_glass_and_preserve_day_geometry() {
        for &kind in crate::exterior_models::ALL {
            let m = crate::exterior_models::model(kind);
            let mut mesh = SurfaceMeshData::default();
            m.append_fitted(&mut mesh, [0., 96., 0., 96.], 0., 16., None);
            let before = mesh.clone();
            mark(&mut mesh, 0, m, kind);
            assert_eq!(mesh.positions, before.positions);
            assert_eq!(mesh.normals, before.normals);
            assert_eq!(mesh.indices, before.indices);
            assert_eq!(mesh.colors, before.colors);
            if m.material_ranges().any(|(_, name)| name == "glass") {
                assert!(!collect(&mesh).is_empty(), "{kind:?}");
            }
            for i in 0..mesh.positions.len() {
                if m.vertex_material(i) == "glass" {
                    assert!((UV_BASE..=UV_BASE + 0.25).contains(&mesh.uvs[i][1]));
                }
                if m.vertex_material(i).starts_with("stone") {
                    assert_eq!(mesh.uvs[i], before.uvs[i]);
                }
            }
        }
    }
}
