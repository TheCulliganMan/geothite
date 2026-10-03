//! GPU skin instances and render-only body poses. Source effects and combat
//! retain their own clocks; this module only samples authored joint channels.
use super::*;
use crate::species_rig::{Clip, Playback, SpeciesRig};
use bevy::render::{
    mesh::{
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
        VertexAttributeValues,
    },
    primitives::Aabb,
};

const BLEND_SECONDS: f32 = 0.10;

#[derive(Component)]
pub(super) struct BattleSkinJoint;

pub(super) struct SkinnedActor {
    pub rig: &'static SpeciesRig,
    joints: Vec<Entity>,
    pose: BodyPose,
}

/// Only allocated at actor creation. A frame updates small local transforms,
/// never vertex buffers, hierarchy, materials, or glTF data.
struct BodyPose {
    current: Vec<Transform>,
    target: Vec<Transform>,
    from: Vec<Transform>,
    clip: Clip,
    attack_id: Arc<str>,
    attack_progress: f32,
    idle_seconds: f32,
    hit_seconds: f32,
    had_hit: bool,
    hit_progress: f32,
    blend_seconds: f32,
    battle_active: bool,
}

impl BodyPose {
    fn new(rig: &SpeciesRig) -> Self {
        let bind: Vec<_> = rig.joints.iter().map(|j| j.bind).collect();
        Self {
            current: bind.clone(),
            target: bind.clone(),
            from: bind,
            clip: Clip::Idle,
            attack_id: Arc::from(""),
            attack_progress: 0.0,
            idle_seconds: 0.0,
            hit_seconds: rig.clip(Clip::Hit).duration,
            had_hit: false,
            hit_progress: 0.0,
            blend_seconds: BLEND_SECONDS,
            battle_active: false,
        }
    }

    fn advance(
        &mut self,
        rig: &SpeciesRig,
        frame: &VisualBattleFrame,
        side: VisualBattleSide,
        shown: bool,
        delta: f32,
    ) {
        if !frame.active {
            if self.battle_active {
                self.clip = Clip::Idle;
                self.had_hit = false;
                self.hit_seconds = rig.clip(Clip::Hit).duration;
                self.blend_seconds = BLEND_SECONDS;
                self.battle_active = false;
            }
            return;
        }
        self.battle_active = true;
        let delta = if delta.is_finite() {
            delta.max(0.0)
        } else {
            0.0
        };
        if shown {
            self.idle_seconds =
                (self.idle_seconds + delta).rem_euclid(rig.clip(Clip::Idle).duration);
        }
        let attack = frame
            .cues
            .iter()
            .find(|cue| cue.side == side && cue.kind == VisualBattleCueKind::Move);
        let impact = frame.cues.iter().find(|cue| {
            cue.side == side && cue.kind == VisualBattleCueKind::Impact && cue.damaging
        });
        let hit_started =
            impact.is_some_and(|cue| !self.had_hit || cue.progress + 0.0001 < self.hit_progress);
        if hit_started {
            // The real visible HP-loss cue starts a short local recoil. Its
            // progress is not a zero-based time (one HP can start near 1.0).
            self.hit_seconds = 0.0;
        } else {
            self.hit_seconds = (self.hit_seconds + delta).min(rig.clip(Clip::Hit).duration);
        }
        self.had_hit = impact.is_some();
        if let Some(cue) = impact {
            self.hit_progress = cue.progress;
        }
        let clip = if attack.is_some() {
            Clip::Attack
        } else if impact.is_some() || self.hit_seconds < rig.clip(Clip::Hit).duration {
            Clip::Hit
        } else {
            Clip::Idle
        };
        let changed_attack = attack.is_some_and(|cue| cue.move_id != self.attack_id);
        let rewound_attack = attack.is_some_and(|cue| {
            self.clip == Clip::Attack
                && cue.move_id == self.attack_id
                && cue.progress + 0.0001 < self.attack_progress
        });
        if clip != self.clip || changed_attack || hit_started || rewound_attack {
            self.from.copy_from_slice(&self.current);
            self.blend_seconds = if rewound_attack { BLEND_SECONDS } else { 0.0 };
            self.clip = clip;
        }
        let seconds = match clip {
            Clip::Idle => self.idle_seconds,
            Clip::Attack => {
                let cue = attack.unwrap();
                if cue.move_id != self.attack_id {
                    self.attack_id = cue.move_id.clone();
                }
                self.attack_progress = cue.progress;
                cue.progress.clamp(0.0, 1.0) * rig.clip(Clip::Attack).duration
            }
            Clip::Hit => self.hit_seconds,
        };
        self.blend_seconds = (self.blend_seconds + delta).min(BLEND_SECONDS);
        rig.sample_into(
            clip,
            seconds,
            if clip == Clip::Idle {
                Playback::Loop
            } else {
                Playback::Clamp
            },
            &mut self.target,
        )
        .expect("validated finite body-pose time");
        let t = (self.blend_seconds / BLEND_SECONDS).clamp(0.0, 1.0);
        let weight = t * t * (3.0 - 2.0 * t);
        for ((pose, target), from) in self.current.iter_mut().zip(&self.target).zip(&self.from) {
            // These authored skins only rotate joints. Retain exact bind
            // translations/scale, including identity root and planted feet.
            *pose = *target;
            if weight < 1.0 {
                pose.rotation = from.rotation.slerp(target.rotation, weight).normalize();
            }
        }
    }
}

pub(super) fn spawn(
    commands: &mut Commands,
    entity: Entity,
    mesh: &mut Mesh,
    inverse_binds: &mut Assets<SkinnedMeshInverseBindposes>,
    cache: &mut HashMap<&'static str, Handle<SkinnedMeshInverseBindposes>>,
    rig: &'static SpeciesRig,
) -> SkinnedActor {
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(rig.joint_indices.clone()),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, rig.joint_weights.clone());
    let bindposes = cache
        .entry(rig.species.name())
        .or_insert_with(|| {
            inverse_binds.add(SkinnedMeshInverseBindposes::from(
                rig.joints
                    .iter()
                    .map(|joint| joint.inverse_bind)
                    .collect::<Vec<_>>(),
            ))
        })
        .clone();
    let mut joints = Vec::with_capacity(rig.joints.len());
    for joint in &rig.joints {
        let child = commands
            .spawn((
                SpatialBundle {
                    transform: joint.bind,
                    ..default()
                },
                BattleSkinJoint,
            ))
            .id();
        commands
            .entity(joint.parent.map_or(entity, |parent| joints[parent]))
            .add_child(child);
        joints.push(child);
    }
    commands.entity(entity).insert((
        SkinnedMesh {
            inverse_bindposes: bindposes,
            joints: joints.clone(),
        },
        Aabb::from_min_max(rig.animated_bounds.0, rig.animated_bounds.1),
    ));
    SkinnedActor {
        rig,
        joints,
        pose: BodyPose::new(rig),
    }
}

pub(super) fn sync(
    frame: Res<VisualBattleFrame>,
    status: Res<BattleViewStatus>,
    time: Res<Time>,
    mut scene: ResMut<BattleScene>,
    mut joints: Query<&mut Transform, With<BattleSkinJoint>>,
) {
    if status.last_error.is_some() {
        return;
    }
    for (index, actor) in scene.actors.iter_mut().enumerate() {
        let Some(actor) = actor else { continue };
        let Some(skin) = &mut actor.skinned else {
            continue;
        };
        let side = if index == 0 {
            VisualBattleSide::Player
        } else {
            VisualBattleSide::Enemy
        };
        let shown = status.active
            && frame.battlers[index].as_ref().is_some_and(|b| {
                b.visible && b.allow_species_model && b.species_id == actor.key.species
            });
        // Observe semantic transitions even in classic view, so returning with
        // F3 cannot replay a stale hit. Idle time pauses while the body is hidden.
        skin.pose
            .advance(skin.rig, &frame, side, shown, time.delta_seconds());
        if shown {
            for (&entity, pose) in skin.joints.iter().zip(&skin.pose.current) {
                if let Ok(mut transform) = joints.get_mut(entity) {
                    transform.set_if_neq(*pose);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "battle_skinning_tests.rs"]
mod tests;
