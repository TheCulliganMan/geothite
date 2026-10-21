//! Native live-palette experiment. Shared bind geometry stays immutable while
//! each actor owns affine garment controls in Bevy's ordinary skinning path.
//! This remains opt-in until shape, contacts and frame cost pass review.
use super::{CENTER_Z, Field, Profile, TOP, body_collision, cloth_vertices};
use crate::{
    new_bark_actors::{
        ModeledActor,
        character_meshes::{CharacterKind, SOURCE_KINDS},
    },
    open_people::PersonRig,
};
use bevy::{
    prelude::*,
    render::mesh::{
        VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
};
use std::sync::{Arc, Mutex};

const SIDES: usize = 24;
const ROWS: usize = 10;
const BASE_CONTROLS: usize = SIDES * ROWS;
const ANGLES: [f32; SIDES] = [
    0., 9., 18., 27., 36., 45., 63., 90., 117., 135., 153., 162., 180., 198., 207., 225., 243.,
    270., 297., 315., 324., 333., 342., 351.,
];

pub(crate) fn enabled() -> bool {
    std::env::var("CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL").as_deref() == Ok("live")
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        PostUpdate,
        settle.after(bevy::transform::TransformSystems::Propagate),
    );
}

#[derive(Default)]
struct Cached {
    palette: Vec<Mat4>,
    matrices: Vec<Mat4>,
    solves: u64,
}

pub(crate) struct Template {
    key: &'static str,
    points: Vec<[f32; 3]>,
    joints: Vec<[u16; 4]>,
    weights: Vec<[f32; 4]>,
    cloth: Vec<bool>,
    originals: Vec<usize>,
    mapped_joints: Vec<[u16; 4]>,
    mapped_weights: Vec<[f32; 4]>,
    centers: Vec<Vec3>,
    fit: super::control_fit::Fit,
    cache: Mutex<Cached>,
}

impl Template {
    pub(crate) fn new(look: crate::npc_appearance::Look, rig: &PersonRig, mesh: &mut Mesh) -> Self {
        let key = look.texture_key;
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("garment positions");
        };
        let points = points.clone();
        let Some(VertexAttributeValues::Uint16x4(joints)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
        else {
            panic!("garment joints");
        };
        let joints = joints.clone();
        let Some(VertexAttributeValues::Float32x4(weights)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
        else {
            panic!("garment weights");
        };
        let weights = weights.clone();
        // Separate trousers already articulate with the two leg chains.
        // Keep that actual production skin in review, and run the same
        // contact/normal/loop/instance checks without fitting a skirt field.
        let tailored = look.costume == "hakama";
        let cloth = if tailored {
            vec![false; points.len()]
        } else {
            cloth_vertices(rig, &points, &joints, &weights)
        };
        // Only the upper seam and attached details still need body joints.
        // Compact those joints; collision continues to use the full body skin.
        let mut originals = Vec::new();
        for i in (0..points.len()).filter(|&i| !cloth[i]) {
            for slot in 0..4 {
                if weights[i][slot] > 0. {
                    originals.push(joints[i][slot] as usize);
                }
            }
        }
        originals.sort_unstable();
        originals.dedup();
        assert!(originals.iter().all(|&joint| joint < rig.skin_nodes.len()));
        assert!(
            !originals.is_empty()
                && originals.len() + if tailored { 0 } else { BASE_CONTROLS } <= 256,
            "garment palette exceeds GPU joint limit"
        );
        if tailored {
            let mapped_joints: Vec<[u16; 4]> = joints
                .iter()
                .zip(&weights)
                .map(|(joints, weights)| {
                    std::array::from_fn(|slot| {
                        if weights[slot] > 0. {
                            originals.binary_search(&(joints[slot] as usize)).unwrap() as u16
                        } else {
                            0
                        }
                    })
                })
                .collect();
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(mapped_joints.clone()),
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights.clone());
            return Self {
                key,
                points,
                joints,
                mapped_joints,
                mapped_weights: weights.clone(),
                weights,
                cloth,
                originals,
                centers: Vec::new(),
                fit: super::control_fit::Fit::empty(),
                cache: default(),
            };
        }
        let profile = Profile::from_shell(&points);
        let bottom = profile.rings.last().unwrap().height;
        // Resolve the fitted seam and the authored rolled hem separately.
        // Uniform rows lose both short changes in the neutral profile.
        let heights = [
            TOP,
            TOP - 0.02,
            TOP - 0.07,
            TOP - 0.30,
            (TOP + bottom) * 0.5,
            bottom + 0.30,
            bottom + 0.10,
            bottom + 0.045,
            bottom + 0.015,
            bottom,
        ];
        let extra_rows = ((256 - originals.len() - BASE_CONTROLS) / 2).min(ROWS - 4);
        let mut row_angles = vec![ANGLES.to_vec(); ROWS];
        for angles in &mut row_angles[ROWS - extra_rows..] {
            angles.extend([54., 306.]);
            angles.sort_by(f32::total_cmp);
        }
        let mut centers = Vec::new();
        let mut offsets = Vec::new();
        for (row, angles) in row_angles.iter().enumerate() {
            offsets.push(centers.len());
            let y = heights[row];
            for &angle in angles {
                let angle = angle.to_radians();
                let radius = profile.radius(y, angle);
                centers.push(Vec3::new(
                    angle.sin() * radius,
                    y,
                    CENTER_Z + angle.cos() * radius,
                ));
            }
        }
        let mut mapped_joints = joints.clone();
        let mut mapped_weights = weights.clone();
        for (index, p) in points.iter().enumerate() {
            if !cloth[index] {
                mapped_joints[index] = joints[index]
                    .map(|joint| originals.binary_search(&(joint as usize)).unwrap_or(0) as u16);
                continue;
            }
            let side = p[0]
                .atan2(p[2] - CENTER_Z)
                .rem_euclid(std::f32::consts::TAU)
                .to_degrees();
            let y = p[1].clamp(bottom, TOP);
            let upper = heights
                .partition_point(|&height| height >= y)
                .saturating_sub(1)
                .min(ROWS - 2);
            let b = (heights[upper] - y) / (heights[upper] - heights[upper + 1]);
            let bracket = |row: usize| {
                let angles = &row_angles[row];
                let left = angles
                    .partition_point(|&angle| angle <= side)
                    .saturating_sub(1)
                    .min(angles.len() - 1);
                let right = (left + 1) % angles.len();
                let end = if right == 0 { 360. } else { angles[right] };
                (
                    offsets[row] + left,
                    offsets[row] + right,
                    (side - angles[left]) / (end - angles[left]),
                )
            };
            let a = bracket(upper);
            let c = bracket(upper + 1);
            mapped_joints[index] = [a.0, a.1, c.0, c.1].map(|i| (originals.len() + i) as u16);
            mapped_weights[index] = [
                (1. - a.2) * (1. - b),
                a.2 * (1. - b),
                (1. - c.2) * b,
                c.2 * b,
            ];
        }
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(mapped_joints.clone()),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, mapped_weights.clone());
        let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
        let fit = super::control_fit::Fit::new(&points, &indices, &cloth,
            &mapped_joints, &mapped_weights, originals.len(), centers.len());
        Self {
            key,
            points,
            joints,
            weights,
            cloth,
            originals,
            mapped_joints,
            mapped_weights,
            centers,
            fit,
            cache: default(),
        }
    }

    pub(crate) fn binds(&self, rig: &PersonRig) -> SkinnedMeshInverseBindposes {
        let mut binds: Vec<_> = self
            .originals
            .iter()
            .map(|&i| rig.inverse_binds[i])
            .collect();
        binds.resize(binds.len() + self.centers.len(), Mat4::IDENTITY);
        SkinnedMeshInverseBindposes::from(binds)
    }

    fn field(&self, rig: &PersonRig, body: &Mesh, palette: &[Mat4]) -> Option<Field> {
        (!self.centers.is_empty()).then(|| {
            Field::new(
                rig,
                &self.points,
                &self.joints,
                &self.weights,
                palette,
                Some(body_collision(body, rig, palette)),
            )
        })
    }

    fn controls(&self, field: &Field) -> Vec<Mat4> {
        let mut controls = self.centers
            .iter()
            .map(|&center| {
                let angle = center
                    .x
                    .atan2(center.z - CENTER_Z)
                    .rem_euclid(std::f32::consts::TAU);
                let skin = field.waist_skin(angle);
                let mapped = field.map(center, skin);
                let derivative = |axis: Vec3| {
                    (field.map(center + axis * 0.001, skin)
                        - field.map(center - axis * 0.001, skin))
                        / 0.002
                };
                let basis = Mat3::from_cols(
                    derivative(Vec3::X),
                    derivative(Vec3::Y),
                    derivative(Vec3::Z),
                );
                // Interpolation of the field samples already carries part of
                // the curvature. Full Taylor frames count it twice between
                // controls. Split the residual derivative with interpolation;
                // retain the full waist affine so rigid motion stays exact.
                let linear = Mat3::from_mat4(skin);
                let basis = linear + (basis - linear) * 0.5;
                assert!(
                    basis.is_finite() && basis.determinant() > 1e-6,
                    "{} live garment control collapsed at {center:?}",
                    self.key
                );
                Mat4::from_cols(
                    basis.x_axis.extend(0.),
                    basis.y_axis.extend(0.),
                    basis.z_axis.extend(0.),
                    (mapped - basis * center).extend(1.),
                )
            })
            .collect::<Vec<_>>();
        if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_UNFITTED_CONTROLS").is_none() {
            self.fit.correct(&self.points, &mut controls, |_, p| field.map(p, Mat4::IDENTITY));
        }
        controls
    }

    fn solve(&self, rig: &PersonRig, body: &Mesh, palette: &[Mat4], cache: &mut Cached) -> bool {
        if cache.palette.len() == palette.len()
            && cache
                .palette
                .iter()
                .zip(palette)
                .all(|(a, b)| a.abs_diff_eq(*b, 0.00001))
        {
            return false;
        }
        let started = std::time::Instant::now();
        let field = self.field(rig, body, palette);
        cache.matrices = field
            .as_ref()
            .map(|field| self.controls(field))
            .unwrap_or_default();
        cache.palette.clear();
        cache.palette.extend_from_slice(palette);
        cache.solves += 1;
        if cache.solves <= 3 && std::env::var_os("CRYSTAL_PEOPLE_REVIEW_DRAPE_TRACE").is_some() {
            eprintln!(
                "{} live garment solve {}: {:?}",
                self.key,
                cache.solves,
                started.elapsed()
            );
        }
        true
    }

    fn vertex_matrix(&self, index: usize, palette: &[Mat4], controls: &[Mat4]) -> Mat4 {
        (0..4)
            .map(|slot| {
                let joint = self.mapped_joints[index][slot] as usize;
                let matrix = if joint < self.originals.len() {
                    palette[self.originals[joint]]
                } else {
                    controls[joint - self.originals.len()]
                };
                matrix * self.mapped_weights[index][slot]
            })
            .sum()
    }

    fn export(&self, palette: &[Mat4], controls: &[Mat4]) {
        let Some(directory) = std::env::var_os("CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES") else {
            return;
        };
        let positions: Vec<_> = self
            .points
            .iter()
            .enumerate()
            .map(|(i, &p)| {
                self.vertex_matrix(i, palette, controls)
                    .transform_point3(Vec3::from_array(p))
                    .to_array()
            })
            .collect();
        std::fs::write(std::path::PathBuf::from(directory).join(format!("{}.live-drape-posed.json", self.key)),
            serde_json::to_vec(&serde_json::json!({"positions":positions,
                "scope":if self.centers.is_empty() {"native production joint palette; tailored trousers"}
                    else {"native live GPU control palette; experimental shape, not accepted production cloth"}})).unwrap()).unwrap();
    }
}

#[derive(Component)]
struct Control;

#[derive(Component)]
pub(crate) struct Instance {
    source: Arc<Template>,
    rig: Arc<PersonRig>,
    body: Handle<Mesh>,
    original: Vec<Entity>,
    controls: Vec<Entity>,
    palette: Vec<Mat4>,
}

pub(crate) fn spawn(
    source: Arc<Template>,
    rig: Arc<PersonRig>,
    body: Handle<Mesh>,
    binds: Handle<SkinnedMeshInverseBindposes>,
    original: Vec<Entity>,
    owner: Entity,
    commands: &mut Commands,
) -> (SkinnedMesh, Instance) {
    let controls: Vec<_> = (0..source.centers.len())
        .map(|_| {
            let entity = commands
                .spawn((Transform::IDENTITY, Control, ModeledActor))
                .id();
            commands.entity(owner).add_child(entity);
            entity
        })
        .collect();
    let mut joints: Vec<_> = source.originals.iter().map(|&i| original[i]).collect();
    joints.extend_from_slice(&controls);
    let palette = vec![Mat4::IDENTITY; original.len()];
    (
        SkinnedMesh {
            inverse_bindposes: binds,
            joints,
        },
        Instance {
            source,
            rig,
            body,
            original,
            controls,
            palette,
        },
    )
}

fn settle(
    meshes: Res<Assets<Mesh>>,
    globals: Query<&GlobalTransform, Without<Control>>,
    mut instances: Query<(&GlobalTransform, &mut Instance), Without<Control>>,
    mut controls: Query<&mut GlobalTransform, With<Control>>,
) {
    for (owner, mut instance) in &mut instances {
        let owner = owner.to_matrix();
        let inverse = owner.inverse();
        let Instance {
            source,
            rig,
            body,
            original,
            controls: entities,
            palette,
        } = &mut *instance;
        for (i, &entity) in original.iter().enumerate() {
            palette[i] = inverse
                * globals
                    .get(entity)
                    .expect("live garment source joint")
                    .to_matrix()
                * rig.inverse_binds[i];
        }
        let body = meshes
            .get(body.id())
            .expect("live garment retains its collision body");
        let mut cache = source.cache.lock().unwrap();
        if source.solve(rig, body, palette, &mut cache) {
            source.export(palette, &cache.matrices);
        }
        for (&entity, matrix) in entities.iter().zip(&cache.matrices) {
            // A Transform cannot represent the shear in this derivative.
            // Publish the complete affine after hierarchy propagation. These
            // leaf joints retain their parent solely for actor lifecycle.
            controls
                .get_mut(entity)
                .expect("live garment control joint")
                .set_if_neq(GlobalTransform::from(owner * *matrix));
        }
    }
}

pub(crate) fn validate() {
    assert!(
        enabled(),
        "live garment validation requires CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL=live"
    );
    for kind in [
        CharacterKind::Nurse,
        CharacterKind::Karen,
        CharacterKind::Kurt,
    ] {
        validate_kind(kind);
    }
    validate_instances(CharacterKind::Nurse);
}

pub(crate) fn validate_roster() {
    assert!(
        enabled(),
        "live garment validation requires CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL=live"
    );
    let kinds: Vec<_> = SOURCE_KINDS
        .iter()
        .map(|&(_, kind)| kind)
        .filter(|&kind| crate::garment_body::hem(crate::npc_appearance::look(kind)).is_some())
        .collect();
    assert_eq!(
        kinds.len(),
        25,
        "all closed-garment identities must be reviewed"
    );
    let mut failed = Vec::new();
    for kind in kinds {
        if std::panic::catch_unwind(|| validate_kind(kind)).is_err() {
            failed.push(crate::npc_appearance::look(kind).texture_key);
        }
    }
    assert!(
        failed.is_empty(),
        "live garment roster failures: {failed:?}"
    );
    for (_, kind) in SOURCE_KINDS
        .iter()
        .filter(|(_, kind)| crate::garment_body::hem(crate::npc_appearance::look(*kind)).is_some())
    {
        validate_instances(*kind);
    }
    eprintln!(
        "All 25 garment identities passed native pose, shading, lower-leg contact and loop checks; tailored-trouser trial={}",
        crate::tailored_trousers::trial_enabled()
    );
}

fn validate_kind(kind: CharacterKind) {
    let look = crate::npc_appearance::look(kind);
    let rig = crate::open_people::get(look.base).unwrap();
    let mut appearance = crate::open_people_skin::prepare_appearance(rig, kind);
    let (body, costume) = appearance.review_drape_meshes();
    let source = Template::new(look, rig, costume);
    if kind == CharacterKind::Falkner {
        crate::npc_appearance::validate_falkner_fringe(rig);
    }
    if look.costume == "hakama" {
        crate::tailored_trousers::validate(rig, look);
        assert!(
            source.centers.is_empty(),
            "tailored legs must keep their production skin"
        );
    }
    let torso_attachment = crate::kimono_obi::build(rig, look).map(|obi| {
        let count = obi.surface.positions.len();
        let start = source
            .points
            .windows(count)
            .position(|points| points == obi.surface.positions)
            .expect("authored obi must remain in the final costume");
        start..start + count
    });
    assert!(
        look.costume == "hakama" || source.cloth.iter().any(|&cloth| cloth),
        "a garment must still drive hanging fabric"
    );
    let Some(VertexAttributeValues::Float32x3(normals)) = costume.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        panic!("live garment validation normals");
    };
    let faces: Vec<_> = costume.indices().unwrap().iter().collect();
    let leg_mask = super::leg_joint_mask(rig);
    let contact_body = if look.costume == "hakama" {
        super::contact_validation::LowerLegs::with_waist(body, &leg_mask, rig.hips_joint)
    } else {
        super::contact_validation::LowerLegs::new(body, &leg_mask)
    };
    let locomotion = crate::human_locomotion::HumanLocomotion::for_character(rig, kind);
    let idle = crate::human_idle::HumanIdle::new(rig, kind);
    let mut pose = rig.nodes.clone();
    let mut maximum_error = 0_f32;
    let mut worst_vertex = None;
    let mut previous = None;
    let mut maximum_step = 0_f32;
    let mut loop_start: Option<Vec<Vec3>> = None;
    let mut minimum_normal_dot = 1_f32;
    let mut cloth_correction = 0_f32;
    let started = std::time::Instant::now();
    for frame in 0..=32 {
        if frame == 0 {
            idle.sample(rig, &locomotion, 0., &mut pose);
        } else {
            locomotion.sample(
                rig,
                (frame - 1) as f32 / 31. * std::f32::consts::TAU,
                1.,
                &mut pose,
            );
        }
        let world = |node| crate::human_locomotion::world(rig, &pose, node);
        let inverse = world(rig.mesh_node).inverse();
        let palette: Vec<_> = rig
            .skin_nodes
            .iter()
            .zip(&rig.inverse_binds)
            .map(|(&node, bind)| inverse * world(node) * *bind)
            .collect();
        let field = source.field(rig, body, &palette);
        let controls = field
            .as_ref()
            .map(|field| source.controls(field))
            .unwrap_or_default();
        let mut current = Vec::new();
        let mut shaded = Vec::new();
        for (index, &p) in source.points.iter().enumerate() {
            let p = Vec3::from_array(p);
            let matrix = source.vertex_matrix(index, &palette, &controls);
            let original: Mat4 = (0..4)
                .map(|slot| {
                    palette[source.joints[index][slot] as usize] * source.weights[index][slot]
                })
                .sum();
            let actual = matrix.transform_point3(p);
            let expected = field
                .as_ref()
                .map(|field| field.map_vertex(index, p, original))
                .unwrap_or_else(|| original.transform_point3(p));
            let error = actual.distance(expected);
            if !source.cloth[index] {
                assert!(
                    actual.distance(original.transform_point3(p)) < 0.00001,
                    "authored torso skin changed on {} at vertex {index}",
                    look.texture_key
                );
            } else {
                cloth_correction =
                    cloth_correction.max(actual.distance(original.transform_point3(p)));
            }
            if torso_attachment
                .as_ref()
                .is_some_and(|range| range.contains(&index))
            {
                assert!(
                    !source.cloth[index] && actual.distance(original.transform_point3(p)) < 0.00001,
                    "{} obi must follow its authored torso skin through movement",
                    look.texture_key
                );
            }
            if p.y < TOP {
                assert!(
                    matrix.is_finite() && Mat3::from_mat4(matrix).determinant() > 1e-6,
                    "{} live palette collapsed at frame {frame}, {p:?}",
                    look.texture_key
                );
            }
            if error > maximum_error {
                maximum_error = error;
                worst_vertex = Some((frame, index, p, actual, expected));
            }
            current.push(actual);
            shaded.push(
                (Mat3::from_mat4(matrix).inverse().transpose() * Vec3::from_array(normals[index]))
                    .normalize(),
            );
        }
        for face in faces.chunks_exact(3) {
            let y = [face[0], face[1], face[2]].map(|i| source.points[i][1]);
            // Closing rims now have their own normal islands. Check them as
            // well as vertical panels through the actual control palette.
            if y.iter().any(|&y| y >= TOP) {
                continue;
            }
            let geometric =
                (current[face[1]] - current[face[0]]).cross(current[face[2]] - current[face[0]]);
            assert!(
                geometric.length_squared() > 1e-12,
                "live garment collapsed a triangle"
            );
            let normal = (shaded[face[0]] + shaded[face[1]] + shaded[face[2]]).normalize();
            let facing = geometric.normalize().dot(normal);
            minimum_normal_dot = minimum_normal_dot.min(facing);
            assert!(
                facing > 0.,
                "{} inward live fabric shading at frame {frame}, triangle {face:?}: {facing}",
                look.texture_key
            );
        }
        let fabric: Vec<_> = faces
            .chunks_exact(3)
            .map(|face| [current[face[0]], current[face[1]], current[face[2]]])
            .collect();
        if look.costume == "hakama" {
            // The separately authored obi and bow have occupied overlapping
            // joins. Their authored skin/attachment checks above remain;
            // measure cloth self-contact and the leg/hip body against the
            // complete trouser component.
            let trousers: Vec<_> = faces
                .chunks_exact(3)
                .filter(|face| {
                    !face.iter().any(|i| {
                        torso_attachment
                            .as_ref()
                            .is_some_and(|range| range.contains(i))
                    })
                })
                .map(|face| [current[face[0]], current[face[1]], current[face[2]]])
                .collect();
            assert!(!trousers.is_empty(), "missing posed trouser surface");
            let self_crossings = super::contact_validation::crossings(&trousers, &trousers);
            let body_crossings =
                super::contact_validation::crossings(&contact_body.pose(&palette), &trousers);
            assert!(
                self_crossings.is_empty() && body_crossings.is_empty(),
                "{} trouser crossings at frame {frame}: {} strict fabric self pairs; {} leg/hip-body pairs, examples {:?}",
                look.texture_key,
                self_crossings.len() / 2,
                body_crossings.len(),
                &body_crossings[..body_crossings.len().min(8)]
            );
        } else {
            let crossings =
                super::contact_validation::crossings(&contact_body.pose(&palette), &fabric);
            assert!(
                crossings.is_empty(),
                "{} lower-leg/fabric crossings at frame {frame}: {} pairs, examples {:?}",
                look.texture_key,
                crossings.len(),
                &crossings[..crossings.len().min(8)]
            );
        }
        if frame == 1 {
            loop_start = Some(current.clone());
        }
        if frame == 32 {
            assert!(
                current
                    .iter()
                    .zip(loop_start.as_ref().unwrap())
                    .all(|(a, b)| a.distance(*b) < 0.001),
                "live garment has a loop seam"
            );
        }
        if frame > 1 {
            let prior: &Vec<Vec3> = previous.as_ref().unwrap();
            for (a, b) in current.iter().zip(prior) {
                maximum_step = maximum_step.max(a.distance(*b));
            }
        }
        previous = Some(current);
    }
    eprintln!(
        "{}: live control max position error={maximum_error:.6}, minimum normal alignment={minimum_normal_dot:.6}, max 1/31-cycle step={maximum_step:.6}, elapsed={:?}",
        look.texture_key,
        started.elapsed()
    );
    assert!(
        maximum_error < 0.035,
        "{} live palette exceeds frozen-field tolerance: {maximum_error}, worst (frame, vertex, bind, actual, target)={worst_vertex:?}",
        look.texture_key
    );
    assert!(
        look.costume == "hakama" || cloth_correction > 0.01,
        "{} still uses its old lower-fabric skin",
        look.texture_key
    );
}

fn validate_instances(kind: CharacterKind) {
    use crate::{new_bark_actors::character_meshes::CharacterKind, open_people_skin::PeopleGpu};
    use bevy::ecs::{system::SystemState, world::CommandQueue};
    let look = crate::npc_appearance::look(kind);
    let rig = crate::open_people::get(look.base).unwrap().clone();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin));
    register(&mut app);
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<StandardMaterial>::default();
    let mut images = Assets::<Image>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut gpu = PeopleGpu::default();
    let mut people = Vec::new();
    let mut roots = Vec::new();
    let mut queue = CommandQueue::default();
    for (index, remote) in [false, true].into_iter().enumerate() {
        let mut commands = Commands::new(&mut queue, app.world());
        let root = commands
            .spawn((
                Transform::from_xyz(index as f32 * 3., 0.2, -2.)
                    .with_rotation(Quat::from_rotation_y(index as f32 * 0.8))
                    .with_scale(Vec3::new(0.7, 0.8, 0.6)),
                Visibility::default(),
            ))
            .id();
        people.push(gpu.spawn(
            look.base,
            kind,
            rig.clone(),
            root,
            remote,
            false,
            None,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            &mut binds,
        ));
        roots.push(root);
        queue.apply(app.world_mut());
    }
    app.insert_resource(meshes);
    app.insert_resource(binds);
    app.update();
    let entities: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Instance>>()
        .iter(app.world())
        .collect();
    assert_eq!(entities.len(), 2);
    let shared = app.world().get::<Mesh3d>(entities[0]).unwrap().0.clone();
    assert_eq!(app.world().get::<Mesh3d>(entities[1]).unwrap().0, shared);
    let first = app.world().get::<Instance>(entities[0]).unwrap();
    let second = app.world().get::<Instance>(entities[1]).unwrap();
    let control_count = first.source.centers.len();
    assert!(Arc::ptr_eq(&first.source, &second.source));
    assert!(
        first
            .controls
            .iter()
            .all(|node| !second.controls.contains(node))
    );
    let stored = app
        .world()
        .resource::<Assets<Mesh>>()
        .get(&shared)
        .unwrap()
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .clone();
    fn snapshot(world: &World, entity: Entity) -> Vec<Vec3> {
        let instance = world.get::<Instance>(entity).unwrap();
        let owner_inverse = world
            .get::<GlobalTransform>(entity)
            .unwrap()
            .to_matrix()
            .inverse();
        let skin = world.get::<SkinnedMesh>(entity).unwrap();
        let binds = world
            .resource::<Assets<SkinnedMeshInverseBindposes>>()
            .get(&skin.inverse_bindposes)
            .unwrap();
        assert_eq!(skin.joints.len(), binds.len());
        assert!(skin.joints.len() <= 256);
        let matrices: Vec<_> = skin
            .joints
            .iter()
            .zip(binds.iter())
            .map(|(&joint, bind)| {
                owner_inverse * world.get::<GlobalTransform>(joint).unwrap().to_matrix() * *bind
            })
            .collect();
        let source = &instance.source;
        source
            .points
            .iter()
            .enumerate()
            .map(|(i, &p)| {
                let skin: Mat4 = (0..4)
                    .map(|slot| {
                        matrices[source.mapped_joints[i][slot] as usize]
                            * source.mapped_weights[i][slot]
                    })
                    .sum();
                skin.transform_point3(Vec3::from_array(p))
            })
            .collect()
    }
    let before: Vec<_> = entities
        .iter()
        .map(|&entity| snapshot(app.world(), entity))
        .collect();
    fn validate_bounds(world: &World, entities: &[Entity], positions: &[Vec<Vec3>]) {
        for (&entity, positions) in entities.iter().zip(positions) {
            let bounds = world
                .get::<bevy::camera::primitives::Aabb>(entity)
                .expect("live costume requires production culling bounds");
            let low: Vec3 = bounds.min().into();
            let high: Vec3 = bounds.max().into();
            for &p in positions {
                assert!(
                    p.is_finite() && p.cmpge(low).all() && p.cmple(high).all(),
                    "live garment vertex {p:?} escapes production bounds {bounds:?}"
                );
            }
        }
    }
    validate_bounds(app.world(), &entities, &before);
    let mut state = SystemState::<Query<&mut Transform, With<ModeledActor>>>::new(app.world_mut());
    people[0].run_blend = 1.;
    people[0].update(
        std::f32::consts::TAU * 0.25,
        1.,
        0.,
        &mut state.get_mut(app.world_mut()).unwrap(),
    );
    app.update();
    let after: Vec<_> = entities
        .iter()
        .map(|&entity| snapshot(app.world(), entity))
        .collect();
    validate_bounds(app.world(), &entities, &after);
    for (&entity, actual) in entities.iter().zip(&after) {
        let world = app.world();
        let instance = world.get::<Instance>(entity).unwrap();
        let inverse = world
            .get::<GlobalTransform>(entity)
            .unwrap()
            .to_matrix()
            .inverse();
        let palette: Vec<_> = instance
            .original
            .iter()
            .zip(&instance.rig.inverse_binds)
            .map(|(&joint, bind)| {
                inverse * world.get::<GlobalTransform>(joint).unwrap().to_matrix() * *bind
            })
            .collect();
        let body = world
            .resource::<Assets<Mesh>>()
            .get(&instance.body)
            .unwrap();
        let source = &instance.source;
        let field = source.field(&instance.rig, body, &palette);
        let mut error = 0_f32;
        for (i, &p) in source.points.iter().enumerate() {
            let skin: Mat4 = (0..4)
                .map(|slot| palette[source.joints[i][slot] as usize] * source.weights[i][slot])
                .sum();
            let p = Vec3::from_array(p);
            let expected = field
                .as_ref()
                .map(|field| field.map_vertex(i, p, skin))
                .unwrap_or_else(|| skin.transform_point3(p));
            error = error.max(actual[i].distance(expected));
        }
        assert!(
            error < 0.035,
            "actual ECS joint/bind palette differs from the cloth field: {error}"
        );
    }
    let distances: Vec<_> = before
        .iter()
        .zip(&after)
        .map(|(a, b)| {
            a.iter()
                .zip(b)
                .map(|(a, b)| a.distance(*b))
                .fold(0_f32, f32::max)
        })
        .collect();
    assert_eq!(
        distances
            .iter()
            .filter(|&&distance| distance > 0.02)
            .count(),
        1,
        "moving one garment must not change the other actor: {distances:?}"
    );
    assert!(distances.iter().any(|&distance| distance < 0.0001));
    assert_eq!(
        app.world()
            .resource::<Assets<Mesh>>()
            .get(&shared)
            .unwrap()
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap(),
        &stored,
        "live animation mutated shared bind geometry"
    );
    app.world_mut().entity_mut(roots[0]).despawn();
    app.update();
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<Control>>()
            .iter(app.world())
            .count(),
        control_count,
        "despawning a person leaked garment palette joints"
    );
    eprintln!(
        "{} live instances pass: shared immutable mesh, distinct native palettes, independent movement, affine world transforms, production culling bounds and owned joint cleanup.",
        look.texture_key
    );
}
