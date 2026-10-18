//! Bicycle geometry contains no baked human. Riders use the shared weighted skin.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::*;
use std::f32::consts::TAU;

// Outer tyre radius, including its tube cross-section, rests on y=0.
const WHEEL_HEIGHT: f32 = 0.205 + 0.025;

fn tube(mesh: &mut SurfaceMeshData, a: Vec3, b: Vec3, radius: f32, color: [f32; 4]) {
    let axis = (b - a).normalize();
    let u = axis
        .cross(if axis.x.abs() < 0.8 { Vec3::X } else { Vec3::Y })
        .normalize();
    let v = axis.cross(u);
    let base = mesh.positions.len() as u32;
    for point in [a, b] {
        for i in 0..16 {
            let angle = i as f32 / 16.0 * TAU;
            let n = u * angle.cos() + v * angle.sin();
            mesh.positions.push((point + n * radius).to_array());
            mesh.normals.push(n.to_array());
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push(color);
        }
    }
    for i in 0..16u32 {
        let j = (i + 1) % 16;
        mesh.indices.extend([
            base + i,
            base + j,
            base + 16 + i,
            base + j,
            base + 16 + j,
            base + 16 + i,
        ]);
    }
    for (point, n, ring, reverse) in [(a, -axis, 0, true), (b, axis, 16, false)] {
        let center = mesh.positions.len() as u32;
        mesh.positions.push(point.to_array());
        mesh.normals.push(n.to_array());
        mesh.uvs.push([0.0, 0.0]);
        mesh.colors.push(color);
        for i in 0..16u32 {
            let j = (i + 1) % 16;
            let mut triangle = [center, base + ring + i, base + ring + j];
            if reverse {
                triangle.swap(1, 2);
            }
            mesh.indices.extend(triangle);
        }
    }
}
fn wheel(mesh: &mut SurfaceMeshData, z: f32, r: f32, thickness: f32, color: [f32; 4]) {
    let base = mesh.positions.len() as u32;
    for i in 0..48 {
        let a = i as f32 / 48.0 * TAU;
        let radial = Vec3::new(0.0, a.cos(), a.sin());
        for j in 0..12 {
            let b = j as f32 / 12.0 * TAU;
            let n = radial * b.cos() + Vec3::X * b.sin();
            mesh.positions
                .push((Vec3::new(0.0, WHEEL_HEIGHT, z) + radial * r + n * thickness).to_array());
            mesh.normals.push(n.to_array());
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push(color);
        }
    }
    for i in 0..48u32 {
        for j in 0..12u32 {
            let a = base + i * 12 + j;
            let b = base + ((i + 1) % 48) * 12 + j;
            let c = base + ((i + 1) % 48) * 12 + (j + 1) % 12;
            let d = base + i * 12 + (j + 1) % 12;
            mesh.indices.extend([a, b, c, a, c, d]);
        }
    }
}
pub(crate) fn mesh(female: bool) -> SurfaceMeshData {
    let mut m = SurfaceMeshData::default();
    let ink = [0.025, 0.035, 0.045, 1.0];
    let metal = [0.58, 0.64, 0.68, 1.0];
    let paint = if female {
        [0.025, 0.36, 0.34, 1.0]
    } else {
        [0.65, 0.04, 0.05, 1.0]
    };
    for z in [-0.37, 0.40] {
        wheel(&mut m, z, 0.205, 0.025, ink);
        wheel(&mut m, z, 0.182, 0.011, metal);
        for i in 0..10 {
            let a = i as f32 / 10.0 * TAU;
            tube(
                &mut m,
                Vec3::new(0.0, WHEEL_HEIGHT, z),
                Vec3::new(0.0, WHEEL_HEIGHT + a.cos() * 0.178, z + a.sin() * 0.178),
                0.005,
                metal,
            );
        }
    }
    let seat = Vec3::new(0.0, 0.445, -0.12);
    let crank = Vec3::new(0.0, 0.265, -0.015);
    let steer = Vec3::new(0.0, 0.56, 0.33);
    let back = Vec3::new(0.0, WHEEL_HEIGHT, -0.37);
    let front = Vec3::new(0.0, WHEEL_HEIGHT, 0.40);
    for (a, b) in [
        (seat, crank),
        (crank, steer),
        (steer, seat),
        (back, seat),
        (back, crank),
        (steer, front),
    ] {
        tube(&mut m, a, b, 0.022, paint);
    }
    tube(&mut m, seat, Vec3::new(0.0, 0.50, -0.12), 0.016, metal);
    tube(
        &mut m,
        Vec3::new(-0.08, 0.50, -0.12),
        Vec3::new(0.08, 0.50, -0.12),
        0.035,
        ink,
    );
    tube(&mut m, steer, Vec3::new(0.0, 0.710, 0.25), 0.018, metal);
    tube(
        &mut m,
        Vec3::new(-0.25, 0.720, 0.25),
        Vec3::new(0.25, 0.720, 0.25),
        0.018,
        metal,
    );
    for s in [-1.0, 1.0] {
        tube(
            &mut m,
            Vec3::new(s * 0.19, 0.720, 0.25),
            Vec3::new(s * 0.27, 0.720, 0.25),
            0.023,
            ink,
        );
    }
    m
}

/// Both rider ankles and the articulated prop use this pedal trajectory.
fn pedal_center(phase: f32, side: usize) -> Vec3 {
    let sign = if side == 0 { 1. } else { -1. };
    let angle = phase + if side == 0 { 0. } else { std::f32::consts::PI };
    Vec3::new(sign * 0.11, 0.265 + 0.078 * angle.sin(),
        -0.015 + 0.078 * angle.cos())
}

/// Independent entities keep each rider's pedal phase independent, while the
/// platforms remain level rather than rotating through the shoes.
pub(crate) struct Pedals { crank: Entity, platforms: [Entity; 2] }
impl Pedals {
    pub(crate) fn set_phase(&self, commands: &mut Commands, phase: f32) {
        commands.entity(self.crank).insert(Transform::from_xyz(0., 0.265, -0.015)
            .with_rotation(Quat::from_rotation_x(-phase)));
        for side in 0..2 {
            commands.entity(self.platforms[side])
                .insert(Transform::from_translation(pedal_center(phase, side)));
        }
    }
}

pub(crate) fn spawn_pedals(commands: &mut Commands, meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>, root: Entity, phase: f32) -> Pedals {
    use bevy::camera::visibility::RenderLayers;
    use crate::new_bark_actors::ModeledActor;
    let mut crank = SurfaceMeshData::default();
    let metal = [0.58, 0.64, 0.68, 1.0];
    for sign in [-1., 1.] {
        tube(&mut crank, Vec3::new(sign * 0.075, 0., 0.),
            Vec3::new(sign * 0.075, 0., sign * 0.078), 0.011, metal);
        tube(&mut crank, Vec3::new(sign * 0.075, 0., sign * 0.078),
            Vec3::new(sign * 0.11, 0., sign * 0.078), 0.011, metal);
    }
    let crank = commands.spawn((Mesh3d(meshes.add(crank.into_mesh())),
        MeshMaterial3d(material.clone()), Transform::default(), ModeledActor,
        RenderLayers::layer(crate::VOXEL_RENDER_LAYER))).id();
    let mut platform = SurfaceMeshData::default();
    // A broad, level rubber platform supports the shoe sole.
    tube(&mut platform, Vec3::new(-0.030, 0., 0.),
        Vec3::new(0.030, 0., 0.), 0.016, [0.025, 0.035, 0.045, 1.]);
    let platform = meshes.add(platform.into_mesh());
    let platforms = std::array::from_fn(|_| commands.spawn((Mesh3d(platform.clone()),
        MeshMaterial3d(material.clone()), Transform::default(), ModeledActor,
        RenderLayers::layer(crate::VOXEL_RENDER_LAYER))).id());
    commands.entity(root).add_children(&[crank, platforms[0], platforms[1]]);
    let pedals = Pedals { crank, platforms };
    pedals.set_phase(commands, phase);
    pedals
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the supplied external character GLBs"]
    fn mounted_world_cache_preserves_recursive_matrix_bits_and_resets_each_pose() {
        let rig = crate::open_people::get("human-male").unwrap();
        let mut transforms = rig.nodes.clone();
        let mut cache = vec![None; transforms.len()];
        for step in 0..32 {
            rig.sample("Idle", step as f32 / 32. * rig.duration("Idle"), &mut transforms);
            pose(rig, &mut transforms, step as f32 / 32. * TAU, Vec3::splat(0.35));
            cache.fill(None);
            for node in (0..transforms.len()).rev() {
                assert_eq!(joint_world(rig, &transforms, node).to_cols_array().map(f32::to_bits),
                    cached_joint_world(rig, &transforms, node, &mut cache).to_cols_array().map(f32::to_bits));
            }
        }
    }
    #[test]
    fn mounted_pedals_follow_crank_without_tilting_or_syncing_other_riders() {
        use bevy::ecs::world::CommandQueue;
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut commands = Commands::new(&mut queue, &world);
        let root_a = commands.spawn(Transform::default()).id();
        let root_b = commands.spawn(Transform::default()).id();
        let a = spawn_pedals(&mut commands, &mut meshes, Handle::default(), root_a, 0.);
        let b = spawn_pedals(&mut commands, &mut meshes, Handle::default(), root_b, 0.73);
        queue.apply(&mut world);
        let other = *world.get::<Transform>(b.crank).unwrap();
        for step in 0..32 {
            let mut commands = Commands::new(&mut queue, &world);
            a.set_phase(&mut commands, step as f32 / 32. * TAU);
            queue.apply(&mut world);
            assert_eq!(*world.get::<Transform>(b.crank).unwrap(), other);
            let crank = world.get::<Transform>(a.crank).unwrap();
            let crank_mesh = meshes.get(&world.get::<Mesh3d>(a.crank).unwrap().0).unwrap();
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(vertices)) =
                crank_mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!("crank positions"); };
            for platform in a.platforms {
                let pad = world.get::<Transform>(platform).unwrap();
                assert_eq!(pad.rotation, Quat::IDENTITY, "shoe platform must stay level");
                assert!(pad.translation.y - 0.016 > 0., "pedal must clear the ground");
                let gap = vertices.iter().map(|p| crank.to_matrix().transform_point3(
                    Vec3::from_array(*p)).distance(pad.translation)).fold(f32::INFINITY, f32::min);
                assert!(gap < 1e-6, "actual crank mesh misses its platform: {gap}");
                assert_eq!(world.get::<ChildOf>(platform).unwrap().parent(), root_a);
            }
        }
    }

    #[test]
    fn bicycle_surfaces_have_finite_unit_normals_and_outward_triangles() {
        for female in [false, true] {
            let m = mesh(female);
            let bottom = m.positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
            assert!(bottom.abs() < 1e-6, "bicycle tyres must rest on the floor: {bottom}");
            assert_eq!(m.positions.len(), m.normals.len());
            assert_eq!(m.positions.len(), m.colors.len());
            for n in &m.normals {
                let n = Vec3::from_array(*n);
                assert!(n.is_finite() && (n.length() - 1.0).abs() < 1e-5);
            }
            for face in m.indices.chunks_exact(3) {
                let a = Vec3::from_array(m.positions[face[0] as usize]);
                let b = Vec3::from_array(m.positions[face[1] as usize]);
                let c = Vec3::from_array(m.positions[face[2] as usize]);
                let normal = face
                    .iter()
                    .map(|&i| Vec3::from_array(m.normals[i as usize]))
                    .sum::<Vec3>();
                assert!(
                    (b - a).cross(c - a).dot(normal) > 0.0,
                    "inward bicycle triangle"
                );
            }
        }
    }
}

fn joint_world(rig: &crate::open_people::PersonRig, pose: &[Transform], node: usize) -> Mat4 {
    rig.parents[node].map(|parent| joint_world(rig, pose, parent))
        .unwrap_or(Mat4::IDENTITY) * pose[node].to_matrix()
}

fn cached_joint_world(rig: &crate::open_people::PersonRig, pose: &[Transform],
    node: usize, cache: &mut [Option<Mat4>]) -> Mat4 {
    if let Some(matrix) = cache[node] { return matrix; }
    let parent = rig.parents[node].map(|parent| cached_joint_world(rig, pose, parent, cache))
        .unwrap_or(Mat4::IDENTITY);
    let matrix = parent * pose[node].to_matrix();
    cache[node] = Some(matrix);
    matrix
}

/// Include the final painted body in the mounted culling envelope. A weighted
/// vertex lies inside the union of its contributing joint boxes, so this avoids
/// reskinning every vertex for every sample during asset construction.
pub(crate) fn body_bounds(rig: &crate::open_people::PersonRig,
    positions: &[[f32; 3]], ids: &[[u16; 4]], weights: &[[f32; 4]],
    scale: Vec3, head_scale: f32, hand_scale: f32) -> (Vec3, Vec3) {
    let _trace = crate::loading_trace::span("actor.mounted_envelope", || serde_json::json!({"vertices":positions.len()}));
    let mut boxes = vec![(Vec3::splat(f32::INFINITY),
        Vec3::splat(f32::NEG_INFINITY)); rig.skin_nodes.len()];
    for ((p, ids), weights) in positions.iter().zip(ids).zip(weights) {
        let p = Vec3::from_array(*p);
        for i in 0..4 {
            if weights[i] > 0. {
                let bounds = &mut boxes[ids[i] as usize];
                bounds.0 = bounds.0.min(p);
                bounds.1 = bounds.1.max(p);
            }
        }
    }
    let mut bounds = rig.bounds;
    let mut transforms = rig.nodes.clone();
    let mut world = vec![None; transforms.len()];
    for idle in 0..8 {
        for step in 0..73 {
            rig.sample("Idle", idle as f32 / 8. * rig.duration("Idle"), &mut transforms);
            if step < 64 {
                pose(rig, &mut transforms, step as f32 / 64. * TAU, scale);
            } else if step < 68 {
                // Seated soles extend below the cushion-centered actor root.
                // Include them in the same production skin culling envelope.
                seat_pose(rig, &mut transforms, scale, 0.15 * (step - 63) as f32);
            } else {
                surf_pose(rig, &mut transforms, scale, 0.15 + (step - 68) as f32 * 0.08);
            }
            transforms[rig.skin_nodes[rig.head_joint]].scale *= head_scale;
            for hand in &rig.hand_joints {
                for &joint in hand {
                    let node = rig.skin_nodes[joint];
                    let mut parent = rig.parents[node];
                    let mut descendant = false;
                    while let Some(ancestor) = parent {
                        if hand.iter().any(|&j| rig.skin_nodes[j] == ancestor) {
                            descendant = true;
                            break;
                        }
                        parent = rig.parents[ancestor];
                    }
                    if !descendant { transforms[node].scale *= hand_scale; }
                }
            }
            // The pose is final here. All joint boxes share ancestor matrices;
            // retain the recursive multiplication order and reset each pose.
            world.fill(None);
            let inverse = cached_joint_world(rig, &transforms, rig.mesh_node, &mut world).inverse();
            for (joint, &(min, max)) in boxes.iter().enumerate() {
                if !min.is_finite() { continue; }
                let matrix = inverse * cached_joint_world(rig, &transforms, rig.skin_nodes[joint], &mut world)
                    * rig.inverse_binds[joint];
                for x in [min.x, max.x] {
                    for y in [min.y, max.y] {
                        for z in [min.z, max.z] {
                            let p = matrix.transform_point3(Vec3::new(x, y, z));
                            assert!(p.is_finite(), "invalid mounted body envelope");
                            bounds.0 = bounds.0.min(p);
                            bounds.1 = bounds.1.max(p);
                        }
                    }
                }
            }
        }
    }
    bounds
}

fn rotate_world(rig: &crate::open_people::PersonRig, pose: &mut [Transform], node: usize, delta: Quat) {
    let parent = rig.parents[node].map(|p| joint_world(rig, pose, p).to_scale_rotation_translation().1)
        .unwrap_or(Quat::IDENTITY);
    pose[node].rotation = (parent.inverse() * delta * parent * pose[node].rotation).normalize();
}

fn wrist_node(rig: &crate::open_people::PersonRig, side: usize) -> usize {
    let hand = &rig.hand_joints[side];
    let joint = hand.iter().find(|&&joint| {
        rig.parents[rig.skin_nodes[joint]].is_none_or(|parent|
            !hand.iter().any(|&other| rig.skin_nodes[other] == parent))
    }).expect("mounted supplied rig requires a wrist");
    rig.skin_nodes[*joint]
}

fn foot_node(rig: &crate::open_people::PersonRig, side: usize) -> usize {
    let shin = rig.skin_nodes[rig.leg_joints[side][1]];
    *rig.skin_nodes.iter().find(|&&node| rig.parents[node] == Some(shin))
        .expect("mounted supplied rig requires a foot")
}

// Two-joint CCD retains the seeded bend direction and bone lengths. Targets
// are in the supplied hierarchy's world coordinates, before the actor root.
fn solve_limb(rig: &crate::open_people::PersonRig, pose: &mut [Transform],
    upper: usize, lower: usize, end: usize, target: Vec3) {
    for _ in 0..48 {
        let endpoint = joint_world(rig, pose, end).transform_point3(Vec3::ZERO);
        if endpoint.distance_squared(target) < 1e-10 { break; }
        for node in [lower, upper] {
            let origin = joint_world(rig, pose, node).transform_point3(Vec3::ZERO);
            let from = joint_world(rig, pose, end).transform_point3(Vec3::ZERO) - origin;
            let to = target - origin;
            if from.length_squared() > 1e-10 && to.length_squared() > 1e-10 {
                rotate_world(rig, pose, node,
                    Quat::from_rotation_arc(from.normalize(), to.normalize()));
            }
        }
    }
}

/// Pose the actual weighted rider against bike-local contact targets. Scale
/// is the production person root scale, including the player's width.
pub(crate) fn pose(rig: &crate::open_people::PersonRig, transforms: &mut [Transform],
    phase: f32, scale: Vec3) {
    let target = |p: Vec3| p / scale + Vec3::Y * rig.floor;
    let hips = rig.skin_nodes[rig.hips_joint];
    let desired_hips = target(Vec3::new(0., 0.540, -0.12));
    let parent = rig.parents[hips].map(|p| joint_world(rig, transforms, p))
        .unwrap_or(Mat4::IDENTITY);
    transforms[hips].translation = parent.inverse().transform_point3(desired_hips);
    // A small forward torso lean brings the shoulder within arm reach of
    // the grips. Keep the head upright instead of pitching the face down.
    let chest = rig.skin_nodes[rig.chest_joint];
    rotate_world(rig, transforms, chest, Quat::from_rotation_x(0.50));
    let head = rig.skin_nodes[rig.head_joint];
    rotate_world(rig, transforms, head, Quat::from_rotation_x(-0.50));
    for side in 0..2 {
        let sign = if side == 0 { 1. } else { -1. };
        let pedal = pedal_center(phase, side);
        let thigh = rig.skin_nodes[rig.leg_joints[side][0]];
        let shin = rig.skin_nodes[rig.leg_joints[side][1]];
        let foot = foot_node(rig, side);
        // Keep the shoe's authored level orientation. Idle contains ankle
        // rocking, which would otherwise move the sole through the pedal.
        let foot_rest = joint_world(rig, &rig.nodes, foot);
        let foot_rotation = foot_rest.to_scale_rotation_translation().1;
        rotate_world(rig, transforms, thigh, Quat::from_rotation_x(-1.1));
        rotate_world(rig, transforms, shin, Quat::from_rotation_x(1.7));
        // Preserve the fitted player sole clearance, then account for the
        // taller NPC's floor-to-ankle distance without stretching its bones.
        let ankle_clearance = 0.0775 + (foot_rest.transform_point3(Vec3::ZERO).y - rig.floor)
            * (scale.y - 1.20 / rig.height);
        let ankle = target(pedal + Vec3::new(0., ankle_clearance, -0.035));
        solve_limb(rig, transforms, thigh, shin, foot, ankle);
        let current = joint_world(rig, transforms, foot).to_scale_rotation_translation().1;
        rotate_world(rig, transforms, foot, foot_rotation * current.inverse());
        let wrist = wrist_node(rig, side);
        let lower = rig.parents[wrist].expect("mounted wrist requires a forearm");
        let upper = rig.parents[lower].expect("mounted forearm requires an upper arm");
        rotate_world(rig, transforms, upper, Quat::from_rotation_x(-1.0));
        rotate_world(rig, transforms, lower, Quat::from_rotation_x(-0.2));
        // Fit the rounded production finger pads to the rubber surface.
        // Measured contact requires a small outward radial adjustment, rather
        // than placing the reshaped fingers inside the previous grip datum.
        // The supplied right hand is asymmetric. Its larger NPC palm needs
        // a little more radial clearance; retain both fitted player grips.
        let clearance = if side == 1 { (scale.y * rig.height - 1.20).max(0.) * 0.01 } else { 0. };
        solve_limb(rig, transforms, upper, lower, wrist,
            target(Vec3::new(sign * 0.215, 0.7365 + clearance, 0.1965 - clearance)));
        // Align the finger hinge with the horizontal grip, rather than
        // inheriting the sideways fist orientation from a walking arm.
        let desired = Quat::from_rotation_x(1.05) * if side == 0 {
            Quat::IDENTITY
        } else { Quat::from_rotation_y(std::f32::consts::PI) };
        let current = joint_world(rig, transforms, wrist).to_scale_rotation_translation().1;
        rotate_world(rig, transforms, wrist, desired * current.inverse());
    }
    // A partly closed grip leaves room for the bar between palm and fingers.
    // The fully closed Idle fist has no such space and intersects the grip.
    for &finger in &rig.finger_nodes {
        transforms[finger].rotation = rig.nodes[finger].rotation
            .slerp(transforms[finger].rotation, 0.55).normalize();
    }
    // The source right thumb is not an exact mirror of the left. Fold its
    // base around the bar axis so its tip clears the inner metal section.
    let thumb = rig.thumb_nodes[1].expect("mounted supplied rig requires a right thumb base");
    rotate_world(rig, transforms, thumb, Quat::from_rotation_x(-0.27));
}

/// Fit the rider's legs around a narrow back or fold them onto a broad back.
/// The Pokémon keeps its physical size and the supplied bones keep their lengths.
pub(crate) fn surf_pose(rig: &crate::open_people::PersonRig, transforms: &mut [Transform], scale: Vec3, half_width: f32) {
    let target = |point: Vec3| point / scale + Vec3::Y * rig.floor;
    let hips = rig.skin_nodes[rig.hips_joint];
    let parent = rig.parents[hips].map(|p| joint_world(rig, transforms, p))
        .unwrap_or(Mat4::IDENTITY);
    transforms[hips].translation = parent.inverse()
        .transform_point3(target(Vec3::new(0., 0.09, 0.)));
    for side in 0..2 {
        let sign = if side == 0 { 1. } else { -1. };
        let thigh = rig.skin_nodes[rig.leg_joints[side][0]];
        let shin = rig.skin_nodes[rig.leg_joints[side][1]];
        let foot = foot_node(rig, side);
        rotate_world(rig, transforms, thigh, Quat::from_rotation_x(-1.1));
        rotate_world(rig, transforms, shin, Quat::from_rotation_x(1.7));
        // A shell wider than the rider's legs must support their feet on top;
        // forcing the small-mount straddle would put both shins inside it.
        let folded = ((half_width - 0.24) / 0.18).clamp(0., 1.);
        solve_limb(rig, transforms, thigh, shin, foot,
            target(Vec3::new(sign * (half_width + 0.07).clamp(0.22, 0.38),
                -0.26 + 0.42 * folded, 0.06 + 0.20 * folded)));
        let level = joint_world(rig, &rig.nodes, foot).to_scale_rotation_translation().1;
        let current = joint_world(rig, transforms, foot).to_scale_rotation_translation().1;
        rotate_world(rig, transforms, foot, level * current.inverse());
        let wrist = wrist_node(rig, side);
        let lower = rig.parents[wrist].unwrap();
        let upper = rig.parents[lower].unwrap();
        solve_limb(rig, transforms, upper, lower, wrist,
            target(Vec3::new(sign * 0.18, 0.16, 0.26)));
    }
    crate::hand_pose::relax(rig, transforms);
}

#[cfg(test)]
mod mounted_tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied rider rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn surf_pose_folds_feet_above_broad_backs_without_stretching_bones() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap()).unwrap();
            let scale = Vec3::splat(1.20 / rig.height);
            for half_width in [0.12, 0.30, 0.50, 0.80] {
                let mut transforms = rig.nodes.clone();
                rig.sample("Idle", 0., &mut transforms);
                let before = transforms.clone();
                surf_pose(&rig, &mut transforms, scale, half_width);
                for (node, transform) in transforms.iter().enumerate() {
                    assert_eq!(transform.scale, before[node].scale, "the rider never stretches to fit");
                    if node != rig.skin_nodes[rig.hips_joint] {
                        assert_eq!(transform.translation, before[node].translation);
                    }
                }
                for side in 0..2 {
                    let ankle = (joint_world(&rig, &transforms, foot_node(&rig, side))
                        .transform_point3(Vec3::ZERO) - Vec3::Y * rig.floor) * scale;
                    if half_width >= 0.50 {
                        assert!(ankle.y > 0.12, "broad backs support both feet above the seat: {ankle:?}");
                    } else if half_width == 0.12 {
                        assert!(ankle.y < -0.20, "narrow mounts retain a hanging straddle");
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "requires supplied rider rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_mounted_riders_reach_saddle_grips_and_pedal_circle() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for (name, width) in [("human-male", 1.0), ("human-female", 0.94)] {
            let rig = crate::open_people::PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap()).unwrap();
            let unit = 1.20 / rig.height;
            let scale = Vec3::new(unit * width, unit, unit);
            for step in 0..32 {
                let phase = step as f32 / 32. * std::f32::consts::TAU;
                let mut transforms = rig.nodes.clone();
                rig.sample("Idle", 0., &mut transforms);
                let before = transforms.clone();
                pose(&rig, &mut transforms, phase, scale);
                let position = |node| (joint_world(&rig, &transforms, node)
                    .transform_point3(Vec3::ZERO) - Vec3::Y * rig.floor) * scale;
                assert!(position(rig.skin_nodes[rig.hips_joint])
                    .distance(Vec3::new(0., 0.540, -0.12)) < 0.0001);
                for side in 0..2 {
                    let sign = if side == 0 { 1. } else { -1. };
                    let angle = phase + if side == 0 { 0. } else { std::f32::consts::PI };
                    let grip_error = position(wrist_node(&rig, side))
                        .distance(Vec3::new(sign * 0.215, 0.735, 0.195));
                    let ankle_error = position(foot_node(&rig, side)).distance(Vec3::new(
                        sign * 0.11, 0.3425 + 0.078 * angle.sin(),
                        -0.050 + 0.078 * angle.cos()));
                    assert!(grip_error < 0.006, "{name} step{step} side{side} grip error {grip_error}");
                    assert!(ankle_error < 0.006, "{name} step{step} side{side} ankle error {ankle_error}");
                }
                for (i, t) in transforms.iter().enumerate() {
                    assert!(t.rotation.is_finite() && (t.rotation.length()-1.).abs() < 1e-5);
                    assert_eq!(t.scale, before[i].scale);
                    if i != rig.skin_nodes[rig.hips_joint] { assert_eq!(t.translation, before[i].translation); }
                }
            }
        }
    }
}

/// Upright seated rest with knees forward, soles below the cushion and hands
/// resting on the lap. Bone lengths remain those of the supplied character rig.
pub(crate) fn seat_pose(rig: &crate::open_people::PersonRig, transforms: &mut [Transform], scale: Vec3, height: f32) {
    let target = |point: Vec3| point / scale + Vec3::Y * rig.floor;
    let hips = rig.skin_nodes[rig.hips_joint];
    let parent = rig.parents[hips].map(|p| joint_world(rig, transforms, p)).unwrap_or(Mat4::IDENTITY);
    transforms[hips].translation = parent.inverse().transform_point3(target(Vec3::new(0., 0.08, 0.)));
    for side in 0..2 {
        let thigh = rig.skin_nodes[rig.leg_joints[side][0]];
        let shin = rig.skin_nodes[rig.leg_joints[side][1]];
        let foot = foot_node(rig, side);
        let point = |node| (joint_world(rig, transforms, node).transform_point3(Vec3::ZERO)
            - Vec3::Y * rig.floor) * scale;
        let thigh_position = point(thigh);
        let knee_position = point(shin);
        let ankle_position = point(foot);
        let upper_length = thigh_position.distance(knee_position);
        let lower_length = knee_position.distance(ankle_position);
        // Real bone lengths keep thighs horizontal and shins down. High seats
        // leave short characters' feet hanging rather than stretching legs.
        let ankle_height = (-height + 0.0775).max(thigh_position.y - lower_length);
        rotate_world(rig, transforms, thigh, Quat::from_rotation_x(-1.4));
        rotate_world(rig, transforms, shin, Quat::from_rotation_x(1.5));
        solve_limb(rig, transforms, thigh, shin, foot,
            target(Vec3::new(thigh_position.x, ankle_height, thigh_position.z + upper_length)));
        let level = joint_world(rig, &rig.nodes, foot).to_scale_rotation_translation().1;
        let current = joint_world(rig, transforms, foot).to_scale_rotation_translation().1;
        rotate_world(rig, transforms, foot, level * current.inverse());
        let wrist = wrist_node(rig, side);
        let lower = rig.parents[wrist].unwrap();
        let upper = rig.parents[lower].unwrap();
        solve_limb(rig, transforms, upper, lower, wrist,
            target(Vec3::new(thigh_position.x, thigh_position.y + 0.10, thigh_position.z + upper_length * 0.7)));
    }
    crate::hand_pose::relax(rig, transforms);
}

#[cfg(test)]
mod seating_tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied character rigs"]
    fn seated_actor_feet_land_on_the_same_world_floor_for_different_body_heights() {
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).expect("supplied character rig");
            for body_height in [1.10, 1.20, 1.35] {
                let pixel_scale = 16.;
                let model_scale = Vec3::splat(body_height / rig.height);
                let mut sitting = crate::seating::Sitting::default();
                for _ in 0..16 {
                    // A low seat lets every tested rig plant both feet.
                    sitting.advance(Some((Vec3::new(0., 1., -8.), 1., None)), true,
                        1. / 60., pixel_scale);
                }
                let mut transforms = rig.nodes.clone();
                rig.sample("Idle", 0., &mut transforms);
                seat_pose(rig, &mut transforms, model_scale, sitting.height);
                for side in 0..2 {
                    let ankle = joint_world(rig, &transforms, foot_node(rig, side))
                        .transform_point3(Vec3::ZERO);
                    let world_y = sitting.offset.y + (ankle.y - rig.floor)
                        * model_scale.y * pixel_scale;
                    assert!((world_y - 0.0775 * pixel_scale).abs() < 0.05,
                        "{name} height {body_height}: ankle {world_y} misses floor-level shoe padding");
                }
            }
        }
    }

    #[test]
    #[ignore = "requires supplied character rigs"]
    fn supplied_seated_people_keep_hips_at_cushion_and_feet_above_ground() {
        for name in ["human-male", "human-female", "skater-male", "skater-female"] {
            let rig = crate::open_people::get(name).expect("supplied character rig");
            let scale = Vec3::splat(1.20 / rig.height);
            for height in [0.30, 0.40, 0.50] {
                let mut transforms = rig.nodes.clone();
                rig.sample("Idle", 0., &mut transforms);
                seat_pose(rig, &mut transforms, scale, height);
                let position = |node| (joint_world(rig, &transforms, node)
                    .transform_point3(Vec3::ZERO) - Vec3::Y * rig.floor) * scale;
                assert!(position(rig.skin_nodes[rig.hips_joint]).distance(Vec3::new(0., 0.08, 0.)) < 1e-5);
                for side in 0..2 {
                    let foot = position(foot_node(rig, side));
                    assert!(foot.y + height > 0., "{name} foot below floor: {foot:?}");
                    assert!(foot.z > 0.05, "{name} knee must bend forward");
                }
                assert!(transforms.iter().all(|t|t.translation.is_finite() && t.rotation.is_finite()));
            }
        }
    }
}
