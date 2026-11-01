//! Geometry-level review of production poses. This supplements GPU/game captures;
//! it cannot establish that a pose looks good by merely detecting nonzero motion.
use super::*;
use bevy::prelude::Vec2;
use std::collections::HashMap;

fn posed(rig: &SpeciesRig, pose: &[Transform]) -> Vec<Vec3> {
    let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
    rig.skin_matrices_into(pose, &mut matrices).unwrap();
    (0..rig.neutral.positions.len())
        .map(|i| rig.skin_point(i, &matrices))
        .collect()
}

// Detect surfaces that meet in the neutral asset but are pulled apart by
// different skin ownership. Exact paint/normal splits must remain coincident.
fn seams(rig: &SpeciesRig) -> Vec<(usize, usize)> {
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    let epsilon = height * 0.0001;
    let mut cells: HashMap<([i32; 3], u16), Vec<usize>> = HashMap::new();
    let mut result = Vec::new();
    for (i, p) in rig.neutral.positions.iter().enumerate() {
        let cell = (p.map(|v| (v / epsilon).round() as i32), ladybird_flight::weld_domain(rig, i));
        if let Some(previous) = cells.get(&cell) {
            if let Some(&other) = previous.iter().find(|&&j| {
                (rig.joint_indices[i] != rig.joint_indices[j]
                    || rig.joint_weights[i] != rig.joint_weights[j])
                    && Vec3::from_array(*p).distance(Vec3::from_array(rig.neutral.positions[j]))
                        < epsilon
            }) {
                result.push((i, other));
            }
        }
        cells.entry(cell).or_default().push(i);
    }
    result
}

#[test]
fn pokemon_motion_roster_preserves_supports_and_connected_surfaces() {
    let names = crate::battle_species_models::supported_species()
        .map(str::to_owned)
        .chain((b'A'..=b'Z').map(|c| format!("UNOWN_{}", c as char)));
    for name in names {
        let rig = world_rig(crate::species_rig::for_species(&name).unwrap());
        for (i, joint) in rig.joints.iter().enumerate() {
            assert!(
                joint.parent.is_none_or(|parent| parent < i),
                "{name} hierarchy must spawn and evaluate parents before children"
            );
        }
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        let pairs = seams(rig);
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        for tick in 0..16 {
            rig.sample_locomotion(tick as f32 / 16., tick as f32 * TAU / 16., 1., &mut pose);
            if locomotion(&name) == Locomotion::Walk {
                gait::assert_supports(rig, &pose);
                let style = travel::style(rig.species.name());
                let flight = if style == travel::Style::ClusterHop { 0. } else {
                    style.hop_height(rig.species.name()) * travel::airborne(tick as f32 * TAU / 16.)
                };
                let roll_pivot = if style == travel::Style::Roll {
                    let pivot = (rig.neutral_bounds.0 + rig.neutral_bounds.1) * 0.5;
                    (pivot - Quat::from_rotation_x(tick as f32 * TAU / 16.) * pivot).y
                } else { 0. };
                assert!(
                    pose[0].translation.y - roll_pivot < height * (0.06 + flight),
                    "{name} phase {tick}/16 needs excessive floor correction: {}",
                    pose[0].translation.y / height
                );
            }
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                assert!(
                    rig.skin_point(v, &matrices).y >= -height * 0.001,
                    "{name} vertex {v} penetrates floor at phase {tick}"
                );
            }
            for &(a, b) in &pairs {
                let distance = rig
                    .skin_point(a, &matrices)
                    .distance(rig.skin_point(b, &matrices));
                assert!(
                    distance < height * 0.001,
                    "{name} connected surface split at phase {tick}: {}",
                    distance / height
                );
            }
        }
    }
}

fn contact_sheet(rig: &SpeciesRig, states: &[Vec<Vec3>], path: &std::path::Path) {
    use image::{Rgb, RgbImage};
    const CELL: u32 = 192;
    let mut image =
        RgbImage::from_pixel(CELL * states.len() as u32, CELL * 2, Rgb([233, 231, 221]));
    let (min, max) = rig.neutral_bounds;
    let center = (min + max) * 0.5;
    let extent = (max - min).max_element() * 1.55;
    for view in 0..2 {
        let camera = Quat::from_rotation_y(if view == 0 { 0.55 } else { 1.57 });
        let tilt = Quat::from_rotation_x(0.10);
        let rotate = tilt * camera;
        let light = Vec3::new(-0.35, 0.8, 0.5).normalize();
        let scale = CELL as f32 / extent;
        for (col, points) in states.iter().enumerate() {
            let mut depth = vec![f32::NEG_INFINITY; (CELL * CELL) as usize];
            let projected: Vec<_> = points
                .iter()
                .map(|&p| {
                    let p = rotate * (p - center);
                    Vec3::new(
                        CELL as f32 * 0.5 + p.x * scale,
                        CELL as f32 * 0.5 - p.y * scale,
                        p.z,
                    )
                })
                .collect();
            let floor = (rotate * (Vec3::new(0., 0., 0.) - center)).y;
            let line = (CELL as f32 * 0.5 - floor * scale).round() as u32;
            if line < CELL {
                for x in 0..CELL {
                    image.put_pixel(
                        col as u32 * CELL + x,
                        view * CELL + line,
                        Rgb([160, 162, 150]),
                    );
                }
            }
            for tri in rig.neutral.indices.chunks_exact(3) {
                let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                let [a, b, c] = ids.map(|i| projected[i]);
                let signed = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
                if signed.abs() < 0.00001 {
                    continue;
                }
                let x0 = a.x.min(b.x).min(c.x).floor().max(0.) as u32;
                let x1 = a.x.max(b.x).max(c.x).ceil().min(CELL as f32 - 1.).max(0.) as u32;
                let y0 = a.y.min(b.y).min(c.y).floor().max(0.) as u32;
                let y1 = a.y.max(b.y).max(c.y).ceil().min(CELL as f32 - 1.).max(0.) as u32;
                let normal = (points[ids[1]] - points[ids[0]])
                    .cross(points[ids[2]] - points[ids[0]])
                    .normalize_or_zero();
                let intensity = 0.48 + 0.52 * (rotate * normal).dot(light).max(0.);
                let color = rig.neutral.colors[ids[0]]
                    .map(|v| (v * intensity * 255.).clamp(0., 255.) as u8);
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let p = bevy::prelude::Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                        let v0 = b.truncate() - a.truncate();
                        let v1 = c.truncate() - a.truncate();
                        let v2 = p - a.truncate();
                        let u = (v2.x * v1.y - v2.y * v1.x) / signed;
                        let v = (v0.x * v2.y - v0.y * v2.x) / signed;
                        if u < 0. || v < 0. || u + v > 1. {
                            continue;
                        }
                        let z = a.z + u * (b.z - a.z) + v * (c.z - a.z);
                        let index = (y * CELL + x) as usize;
                        if z > depth[index] {
                            depth[index] = z;
                            image.put_pixel(
                                col as u32 * CELL + x,
                                view * CELL + y,
                                Rgb([color[0], color[1], color[2]]),
                            );
                        }
                    }
                }
            }
        }
    }
    image.save(path).unwrap();
}

#[test]
#[ignore = "explicit geometry review; writes only to the requested ignored directory"]
fn pokemon_motion_geometry_review() {
    let output = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_POKEMON_REVIEW_DIR").expect("review output directory"),
    );
    std::fs::create_dir_all(&output).unwrap();
    let filter = std::env::var("CRYSTAL_POKEMON_REVIEW_SPECIES").ok();
    let all_sheets = std::env::var("CRYSTAL_POKEMON_REVIEW_ALL").is_ok_and(|v| v == "1");
    let mut report = String::from(
        "species,mode,penetration_height,seam_gap_height,seam_pairs,root_lift_height\n",
    );
    let names = crate::battle_species_models::supported_species()
        .map(str::to_owned)
        .chain((b'A'..=b'Z').map(|c| format!("UNOWN_{}", c as char)));
    for name in names.filter(|name| {
        filter
            .as_ref()
            .is_none_or(|f| f.split(',').any(|s| s.eq_ignore_ascii_case(name)))
    }) {
        let canonical = crate::species_rig::for_species(&name).unwrap();
        let rig = world_rig(canonical);
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        let pairs = seams(rig);
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut states = Vec::new();
        let mut floor = 0.0_f32;
        let mut gap = 0.0_f32;
        let mut root_lift = 0.0_f32;
        for tick in 0..64 {
            let t = tick as f32 / 64.;
            rig.sample_locomotion(t, t * TAU, 1., &mut pose);
            root_lift = root_lift.max(pose[0].translation.y);
            let points = posed(rig, &pose);
            floor = floor.min(points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min));
            for &(a, b) in &pairs {
                gap = gap.max(points[a].distance(points[b]));
            }
            if tick % 8 == 0 {
                states.push(points);
            }
        }
        for clip in [Clip::Attack, Clip::Hit] {
            canonical
                .sample_into(
                    clip,
                    canonical.clip(clip).duration * 0.4,
                    Playback::Clamp,
                    &mut pose[..canonical.joints.len()],
                )
                .unwrap();
            battle_float(canonical, &mut pose[..canonical.joints.len()]);
            states.push(posed(canonical, &pose[..canonical.joints.len()]));
        }
        if all_sheets || filter.is_some() || floor < -height * 0.03 || gap > height * 0.02 {
            contact_sheet(
                rig,
                &states,
                &output.join(format!("{}.png", name.to_ascii_lowercase())),
            );
        }
        report.push_str(&format!(
            "{name},{:?},{:.6},{:.6},{},{:.6}\n",
            locomotion(&name),
            -floor / height,
            gap / height,
            pairs.len(),
            root_lift / height
        ));
    }
    std::fs::write(output.join("quality.csv"), report).unwrap();
}

#[test]
#[ignore = "explicit windowless GPU review of the production skin attributes and poses"]
fn pokemon_motion_gpu_review() {
    use bevy::{
        app::PluginsState,
        camera::{RenderTarget, ScalingMode},
        prelude::*,
        render::{
            mesh::{
                VertexAttributeValues,
                skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            },
            view::screenshot::{Screenshot, ScreenshotCaptured},
        },
        window::ExitCondition,
    };
    use std::{
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };
    let output = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_POKEMON_REVIEW_DIR").expect("GPU review output directory"),
    );
    std::fs::create_dir_all(&output).unwrap();
    let names = std::env::var("CRYSTAL_POKEMON_REVIEW_SPECIES")
        .unwrap_or_else(|_| "PIKACHU,FURRET,LUGIA,GLIGAR,ARTICUNO,CHARIZARD".into());
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            }),
    );
    let deadline = Instant::now() + Duration::from_secs(60);
    while app.plugins_state() != PluginsState::Ready {
        assert!(
            Instant::now() < deadline,
            "GPU initialization did not complete"
        );
        bevy::tasks::tick_global_task_pools_on_main_thread();
        std::thread::sleep(Duration::from_millis(5));
    }
    app.finish();
    app.cleanup();
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            1536,
            512,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.world_mut().spawn((
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        AmbientLight {
            color: Color::WHITE,
            brightness: 100.,
            ..default()
        },
        Msaa::Off,
        RenderTarget::Image(image.clone().into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 6.4,
                height: 2.14,
            },
            near: 0.1,
            far: 100.,
            ..OrthographicProjection::default_3d()
        }),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.88, 0.87, 0.83)),
            ..default()
        },
        Transform::from_xyz(0., 1.5, 8.).looking_at(Vec3::new(0., 0.55, 0.), Vec3::Y),
    ));
    app.world_mut().spawn((
        DirectionalLight {
            illuminance: 6000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-3., 5., 4.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let floor = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Plane3d::default().mesh().size(30., 30.));
    let floor_material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.64, 0.66, 0.60),
            ..default()
        });
    app.world_mut()
        .spawn((Mesh3d(floor), MeshMaterial3d(floor_material),
            Transform::from_xyz(0., if std::env::var_os("CRYSTAL_POKEMON_REVIEW_SWIMMING").is_some() { -0.35 } else { 0. }, 0.)));
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.85,
            cull_mode: None,
            ..default()
        });
    for name in names.split(',') {
        let canonical = crate::species_rig::for_species(name).unwrap();
        let rig = world_rig(canonical);
        let fit = 1.1 / (rig.neutral_bounds.1 - rig.neutral_bounds.0).max_element();
        let mut roots = Vec::new();
        for frame in 0..4 {
            let rig = if frame == 0 || frame == 3 {
                canonical
            } else {
                rig
            };
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            match frame {
                0 => {
                    for (p, j) in pose.iter_mut().zip(&rig.joints) {
                        *p = j.bind;
                    }
                }
                1 | 2 if std::env::var_os("CRYSTAL_POKEMON_REVIEW_SWIMMING").is_some() => rig.sample_swimming(
                    if frame == 1 { 0.20 } else { 0.80 },
                    if frame == 1 { TAU * 0.125 } else { TAU * 0.625 },
                    1., &mut pose,
                ),
                1 | 2 if std::env::var_os("CRYSTAL_POKEMON_REVIEW_WING_EXTREMES").is_some()
                    && locomotion(rig.species.name()) == Locomotion::Fly => rig.sample_locomotion(
                    (if frame == 1 { 0.25 } else { 0.75 }) / flight_beat(rig.species.name()).0,
                    if frame == 1 { TAU * 0.20 } else { TAU * 0.80 },
                    1., &mut pose,
                ),
                1 | 2 => rig.sample_locomotion(
                    if frame == 1 { 0.20 } else { 0.80 },
                    if frame == 1 { TAU * 0.20 } else { TAU * 0.80 },
                    1.,
                    &mut pose,
                ),
                _ => {
                    canonical
                        .sample_into(
                            Clip::Attack,
                            canonical.clip(Clip::Attack).duration * 0.4,
                            Playback::Clamp,
                            &mut pose[..canonical.joints.len()],
                        )
                        .unwrap();
                    battle_float(canonical, &mut pose);
                    for i in canonical.joints.len()..pose.len() {
                        pose[i] = rig.joints[i].bind;
                    }
                }
            }
            let mut mesh = rig.neutral.clone().into_mesh();
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(rig.joint_indices.clone()),
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, rig.joint_weights.clone());
            let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
            let binds = app
                .world_mut()
                .resource_mut::<Assets<SkinnedMeshInverseBindposes>>()
                .add(SkinnedMeshInverseBindposes::from(
                    rig.joints
                        .iter()
                        .map(|j| j.inverse_bind)
                        .collect::<Vec<_>>(),
                ));
            let root = app
                .world_mut()
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz((frame as f32 - 1.5) * 1.55, 0.002, 0.)
                        .with_rotation(Quat::from_rotation_y(std::env::var("CRYSTAL_POKEMON_REVIEW_YAW")
                            .ok().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.45)))
                        .with_scale(Vec3::splat(fit)),
                ))
                .id();
            let mut joints = Vec::new();
            for (i, j) in rig.joints.iter().enumerate() {
                let entity = app.world_mut().spawn((pose[i], Visibility::default())).id();
                app.world_mut()
                    .entity_mut(j.parent.map_or(root, |p| joints[p]))
                    .add_child(entity);
                joints.push(entity);
            }
            app.world_mut().entity_mut(root).insert(SkinnedMesh {
                inverse_bindposes: binds,
                joints,
            });
            roots.push(root);
        }
        for _ in 0..40 {
            app.update();
            std::thread::sleep(Duration::from_millis(5));
        }
        let completion: Arc<Mutex<Option<Result<(), String>>>> = Default::default();
        let saved = completion.clone();
        let path = output.join(format!("{}-gpu.png", name.to_ascii_lowercase()));
        app.world_mut()
            .spawn(Screenshot::image(image.clone()))
            .observe(move |event: On<ScreenshotCaptured>| {
                let result = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| e.to_string())
                    .and_then(|im| {
                        let colors: std::collections::HashSet<_> =
                            im.to_rgb8().pixels().map(|p| p.0).collect();
                        if colors.len() < 32 {
                            return Err("GPU review produced an empty or flat frame".into());
                        }
                        im.save(&path).map_err(|e| e.to_string())
                    });
                *saved.lock().unwrap() = Some(result);
            });
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            app.update();
            if let Some(result) = completion.lock().unwrap().take() {
                result.unwrap();
                break;
            }
            assert!(
                Instant::now() < deadline,
                "GPU readback timed out for {name}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        for root in roots {
            app.world_mut().entity_mut(root).despawn();
        }
    }
}
