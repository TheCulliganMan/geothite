//! Pure camera-rig calculation shared by the Bevy system and tests.

use bevy::prelude::{Quat, Transform, Vec2, Vec3};

use crate::{
    EXPECTED_GRID_SIZE,
    profile::{MAX_PROFILE_HEIGHT, MIN_PROFILE_HEIGHT, SOURCE_TILE_HEIGHT},
};

/// Diorama view measured upward from the horizontal ground plane. Forty-five
/// degrees gives terrain depth and authored vertical faces equal visual
/// weight, instead of the previous more top-down 65-degree presentation.
pub const CAMERA_PITCH_DEGREES: f32 = 45.0;
const CAMERA_FOCAL: f32 = 1.0;
const FAR_DEPTH_MARGIN: f32 = 4096.0;

/// Continuous camera controls; one rotation unit is 45 degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxelCameraControls {
    pub zoom_step: f32,
    pub rotation_step: f32,
    pub pitch_degrees: f32,
    pub first_person: bool,
}

impl Default for VoxelCameraControls {
    fn default() -> Self {
        Self {
            zoom_step: 1.0,
            rotation_step: 0.0,
            pitch_degrees: CAMERA_PITCH_DEGREES,
            first_person: false,
        }
    }
}

impl VoxelCameraControls {
    pub fn new(zoom_step: f32, rotation_step: f32) -> Self {
        Self {
            zoom_step: if zoom_step.is_finite() {
                zoom_step.clamp(0.0, 5.0)
            } else {
                1.0
            },
            rotation_step: if rotation_step.is_finite() {
                rotation_step.rem_euclid(8.0)
            } else {
                0.0
            },
            ..Self::default()
        }
    }

    /// Browser presentation controls share the same bounds as native look input.
    pub fn with_pose(zoom_step: f32, rotation_step: f32, pitch: f32, first_person: bool) -> Self {
        let mut camera = Self::new(zoom_step, rotation_step);
        camera.first_person = first_person;
        camera.pitch_degrees = if pitch.is_finite() {
            pitch.clamp(if first_person { -85.0 } else { 5.0 }, 85.0)
        } else if first_person { 0.0 } else { CAMERA_PITCH_DEGREES };
        camera
    }

    /// Look input is measured in radians; yaw wraps and pitch never flips over.
    pub fn steer(&mut self, yaw: f32, pitch: f32) {
        if yaw.is_finite() && pitch.is_finite() {
            self.rotation_step =
                (self.rotation_step + yaw / std::f32::consts::FRAC_PI_4).rem_euclid(8.0);
            self.pitch_degrees = (self.pitch_degrees + pitch.to_degrees())
                .clamp(if self.first_person { -85.0 } else { 5.0 }, 85.0);
        }
    }

    pub fn toggle_first_person(&mut self, facing: Vec2) {
        self.first_person = !self.first_person;
        self.pitch_degrees = if self.first_person {
            0.0
        } else {
            CAMERA_PITCH_DEGREES
        };
        if self.first_person && facing.is_finite() && facing.length_squared() > 0.0 {
            self.rotation_step = (-facing.x)
                .atan2(facing.y)
                .rem_euclid(std::f32::consts::TAU)
                / std::f32::consts::FRAC_PI_4;
        }
    }

    /// Select cycles the two orbit distances and a trainer-aligned eye-level view.
    pub fn cycle_view(&mut self, facing: Vec2) {
        if self.first_person {
            self.first_person = false;
            self.zoom_step = 3.0;
            self.pitch_degrees = CAMERA_PITCH_DEGREES;
        } else if self.zoom_step > 1.0 {
            self.zoom_step = 0.0;
            self.pitch_degrees = CAMERA_PITCH_DEGREES;
        } else {
            self.toggle_first_person(facing);
            self.zoom_step = 1.0;
        }
    }

    pub fn pose(self, viewport_size: Vec2) -> VoxelCameraPose {
        let controls = Self::new(self.zoom_step, self.rotation_step);
        let mut pose = camera_pose(viewport_size);
        let zoom = 0.75 + controls.zoom_step * 0.25;
        let yaw = controls.rotation_step * std::f32::consts::FRAC_PI_4;
        let pitch = self.pitch_degrees.to_radians();
        let offset = Vec3::new(0.0, pitch.sin(), pitch.cos());
        if self.first_person {
            pose.eye = Vec3::ZERO;
            pose.target = -(Quat::from_rotation_y(yaw) * offset);
            pose.vertical_fov_radians = (75.0 / zoom).to_radians();
            pose.near = (viewport_size.y / EXPECTED_GRID_SIZE.y as f32 * 0.025).max(0.05);
        } else if self.pitch_degrees == CAMERA_PITCH_DEGREES {
            pose.eye = pose.target + Quat::from_rotation_y(yaw) * (pose.eye - pose.target) / zoom;
        } else {
            pose.eye = pose.target
                + Quat::from_rotation_y(yaw) * offset * (CAMERA_FOCAL * viewport_size.y / zoom);
        }
        pose
    }

    /// Use the same interpolated player foot and terrain support as the actor rig.
    pub fn world_pose(
        self,
        frame: &crystal_render_api::VisualWorldFrame,
        heights: &[f32],
    ) -> VoxelCameraPose {
        let mut pose = self.pose(frame.viewport_size);
        if !self.first_person {
            if let Some(player) = frame.actors.iter().find(|a|
                a.id == crystal_render_api::VisualActorId::Player) {
                let height=crate::footing::resolved_footing_height(frame,
                    crate::footing::actor_foot(player), heights).unwrap_or(0.);
                pose.eye.y += height;
                pose.target.y += height;
            }
        }
        if self.first_person {
            if let Some(player) = frame
                .actors
                .iter()
                .find(|a| a.id == crystal_render_api::VisualActorId::Player)
            {
                let foot = crate::footing::actor_foot(player);
                let height = crate::footing::resolved_footing_height(frame, foot, heights)
                    .or_else(|| crate::footing::footing_height(frame, foot))
                    .unwrap_or(0.0);
                let eye = crate::footing::visual_point_to_voxel(
                    foot,
                    height
                        + frame.tile_size.y
                            * 2.0
                            * crate::new_bark_actors::player_eye_height(player),
                );
                pose.target += eye;
                pose.eye = eye;
            }
        }
        pose
    }
    /// Orbit cameras follow the gradual terrain rise; first person follows the
    /// rider's airborne height. Neither reads a screen-space sprite lift.
    pub(crate) fn world_pose_with_traversal(
        self,
        frame: &crystal_render_api::VisualWorldFrame,
        heights: &[f32],
        traversal: Option<&crystal_render_api::VisualPlayerTraversal>,
    ) -> VoxelCameraPose {
        let mut pose = self.world_pose(frame, heights);
        if let Some(player) = frame.actors.iter().find(|a| a.id == crystal_render_api::VisualActorId::Player)
            && let Some((height, arc)) = crate::footing::traversal_height(frame, player, heights, traversal)
            && arc > 0.0
        {
            let ground = crate::footing::resolved_footing_height(frame, crate::footing::actor_foot(player), heights).unwrap_or(height);
            let step = traversal.and_then(|t| t.step).unwrap();
            let from = crate::footing::resolved_footing_height(frame, step.from, heights).unwrap_or(ground);
            let to = crate::footing::resolved_footing_height(frame, step.to, heights).unwrap_or(ground);
            let camera_height = if self.first_person { height } else { from + (to - from) * step.progress.clamp(0.0, 1.0) };
            pose.eye.y += camera_height - ground;
            pose.target.y += camera_height - ground;
        }
        pose
    }

}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxelCameraPose {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub vertical_fov_radians: f32,
    pub near: f32,
    pub far: f32,
}

impl VoxelCameraPose {
    pub fn transform(self) -> Transform {
        Transform::from_translation(self.eye).looking_at(self.target, self.up)
    }
}

pub fn camera_pose(viewport_size: Vec2) -> VoxelCameraPose {
    let pitch = CAMERA_PITCH_DEGREES.to_radians();
    let tile_height = viewport_size.y / EXPECTED_GRID_SIZE.y as f32;
    let profile_scale = tile_height / SOURCE_TILE_HEIGHT;
    let target_height = (MAX_PROFILE_HEIGHT + MIN_PROFILE_HEIGHT) * 0.5 * profile_scale;
    let target = Vec3::new(0.0, target_height, 0.0);
    let camera_distance = CAMERA_FOCAL * viewport_size.y;
    let eye = target
        + Vec3::new(
            0.0,
            camera_distance * pitch.sin(),
            camera_distance * pitch.cos(),
        );
    VoxelCameraPose {
        eye,
        target,
        up: Vec3::Y,
        // Tie distance and field of view to the source viewport. Straight
        // down, this frames exactly one viewport-height of world pixels;
        // tilting introduces honest perspective instead of orthographically
        // crushing the ground into a short rectangle.
        vertical_fov_radians: 2.0 * (1.0 / (2.0 * CAMERA_FOCAL)).atan(),
        near: (camera_distance * 0.05).max(1.0),
        far: camera_distance * 4.0 + FAR_DEPTH_MARGIN,
    }
}

#[cfg(test)]
fn camera_forward(pose: VoxelCameraPose) -> Vec3 {
    (pose.target - pose.eye).normalize()
}

pub fn card_rotation_toward_camera(pose: VoxelCameraPose) -> Quat {
    let toward_eye = (pose.eye - pose.target).normalize_or_zero();
    let yaw = toward_eye.x.atan2(toward_eye.z);
    let pitch = toward_eye.y.asin();
    // The source sprites are front-on drawings. Rotate the card into the
    // camera plane around its bottom pivot so its pixels remain square on
    // screen instead of being foreshortened by the terrain pitch.
    Quat::from_rotation_y(yaw) * Quat::from_rotation_x(-pitch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_cycles_close_wide_and_trainer_aligned_first_person() {
        let mut camera = VoxelCameraControls::new(3., 2.);
        camera.cycle_view(Vec2::NEG_Y);
        assert_eq!(camera.zoom_step, 0.);
        assert!(!camera.first_person);
        camera.cycle_view(Vec2::NEG_Y);
        assert!(camera.first_person);
        assert!((camera.rotation_step - 4.).abs() < 0.00001);
        assert_eq!(camera.pitch_degrees, 0.);
        camera.cycle_view(Vec2::X);
        assert!(!camera.first_person);
        assert_eq!(camera.zoom_step, 3.);
    }

    #[test]
    fn presentation_pose_bounds_pitch_and_first_person_zoom_changes_fov() {
        let viewport = Vec2::new(640.0, 576.0);
        let wide = VoxelCameraControls::with_pose(0.0, -1.0, -100.0, true);
        let close = VoxelCameraControls::with_pose(5.0, 9.0, 100.0, true);
        assert_eq!(wide.pitch_degrees, -85.0);
        assert_eq!(wide.rotation_step, 7.0);
        assert_eq!(close.pitch_degrees, 85.0);
        assert!(close.pose(viewport).vertical_fov_radians < wide.pose(viewport).vertical_fov_radians);
        assert_eq!(VoxelCameraControls::with_pose(1.0, 0.0, -100.0, false).pitch_degrees, 5.0);
        assert_eq!(VoxelCameraControls::with_pose(1.0, 0.0, f32::NAN, true).pitch_degrees, 0.0);
    }

    #[test]
    fn first_person_faces_the_trainer_direction_and_steers_without_flipping() {
        for facing in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            let mut camera = VoxelCameraControls::default();
            camera.toggle_first_person(facing);
            let pose = camera.pose(Vec2::new(640.0, 576.0));
            let expected = Vec3::new(facing.x, 0.0, -facing.y);
            assert!(pose.transform().forward().dot(expected) > 0.999);
            assert!(pose.near < 1.0);
            assert_eq!(pose.vertical_fov_radians, 75.0_f32.to_radians());
            camera.steer(100.0, -100.0);
            assert_eq!(camera.pitch_degrees, -85.0);
            assert!(camera.rotation_step >= 0.0 && camera.rotation_step < 8.0);
            let before = camera;
            camera.steer(f32::NAN, 1.0);
            assert_eq!(camera, before);
            camera.toggle_first_person(facing);
            assert!(!camera.first_person);
            assert_eq!(camera.pitch_degrees, CAMERA_PITCH_DEGREES);
        }
    }

    #[test]
    fn first_person_eye_tracks_interpolated_player_and_terrain_support() {
        use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame};
        let mut frame = VisualWorldFrame {
            center: Vec2::new(20.0, 30.0),
            viewport_size: Vec2::splat(32.0),
            tile_size: Vec2::splat(8.0),
            grid_size: bevy::prelude::UVec2::splat(4),
            actors: vec![VisualActor {
                id: VisualActorId::Player,
                source_id: "player".into(),
                model_palette: None,
                texture: Default::default(),
                center: Vec2::new(20.0, 34.0),
                size: Vec2::splat(8.0),
                flip_x: false,
                above_priority: false,
                facing: Some(Vec2::Y),
            }],
            ..Default::default()
        };
        let mut camera = VoxelCameraControls::default();
        let orbit_base=camera.pose(frame.viewport_size);
        let orbit=camera.world_pose(&frame,&vec![7.;16]);
        assert_eq!(orbit.eye.y,orbit_base.eye.y+7.);
        assert_eq!(orbit.target.y,orbit_base.target.y+7.);
        camera.toggle_first_person(Vec2::Y);
        let heights = vec![7.0; 16];
        let pose = camera.world_pose(&frame, &heights);
        assert_eq!(pose.eye.x, 20.0);
        assert_eq!(pose.eye.z, -30.0);
        assert!(pose.eye.y > 7.0);
        frame.actors[0].center.x += 0.25;
        assert_eq!(camera.world_pose(&frame, &heights).eye.x, 20.25);
        assert!((camera.world_pose(&frame, &[0.0; 16]).eye.y - (pose.eye.y - 7.0)).abs() < 0.0001);
    }

    #[test]
    fn camera_controls_preserve_default_pose_and_bound_zoom() {
        let viewport = Vec2::new(640.0, 576.0);
        assert_eq!(
            VoxelCameraControls::default().pose(viewport),
            camera_pose(viewport)
        );
        let base = camera_pose(viewport);
        let zoomed = VoxelCameraControls::new(5.0, 0.0).pose(viewport);
        assert!(
            ((zoomed.eye - zoomed.target).length() * 2.0 - (base.eye - base.target).length()).abs()
                < 0.001
        );
        assert_eq!(
            VoxelCameraControls::new(255.0, 255.0),
            VoxelCameraControls::new(5.0, 7.0)
        );
    }

    #[test]
    fn orbit_keeps_target_pitch_and_billboards_aligned_at_every_angle() {
        let viewport = Vec2::new(640.0, 576.0);
        let base = camera_pose(viewport);
        for step in 0..8 {
            let pose = VoxelCameraControls::new(1.0, step as f32).pose(viewport);
            assert_eq!(pose.target, base.target);
            assert!((pose.eye.y - base.eye.y).abs() < 0.001);
            assert!(
                ((pose.eye - pose.target).length() - (base.eye - base.target).length()).abs()
                    < 0.001
            );
            let rotation = card_rotation_toward_camera(pose);
            assert!(
                rotation
                    .mul_vec3(Vec3::Z)
                    .dot((pose.eye - pose.target).normalize())
                    > 0.999
            );
            assert!(
                rotation
                    .mul_vec3(Vec3::Y)
                    .dot(pose.transform().up().as_vec3())
                    > 0.999
            );
        }
        assert_eq!(VoxelCameraControls::new(1.0, 8.0).pose(viewport), base);
    }

    #[test]
    fn analog_orbit_preserves_fractional_angles_and_rejects_nonfinite_input() {
        let viewport = Vec2::new(640.0, 576.0);
        let pose = VoxelCameraControls::new(1.5, 0.25).pose(viewport);
        let offset = pose.eye - pose.target;
        assert!((offset.x.atan2(offset.z) - std::f32::consts::FRAC_PI_4 * 0.25).abs() < 0.00001);
        assert_eq!(
            VoxelCameraControls::new(f32::NAN, f32::INFINITY),
            VoxelCameraControls::default()
        );
    }

    #[test]
    fn camera_is_tilted_at_the_requested_pitch() {
        let pose = camera_pose(Vec2::new(160.0, 144.0));
        let toward_eye = (pose.eye - pose.target).normalize();
        let actual_pitch = toward_eye.y.asin().to_degrees();

        assert!((actual_pitch - CAMERA_PITCH_DEGREES).abs() < 0.001);
        assert!(pose.eye.y > pose.target.y);
        assert!(pose.eye.z > pose.target.z);
    }

    #[test]
    fn perspective_focal_frames_one_viewport_height_when_top_down() {
        let viewport = Vec2::new(160.0, 144.0);
        let pose = camera_pose(viewport);
        let distance = CAMERA_FOCAL * viewport.y;
        let framed_height = 2.0 * distance * (pose.vertical_fov_radians * 0.5).tan();
        assert!((framed_height - viewport.y).abs() < 1.0e-5);
        assert!(pose.near > 0.0);
        assert!(pose.far > distance);
    }

    #[test]
    fn runtime_projection_depth_range_contains_authored_world() {
        let pose = camera_pose(Vec2::new(640.0, 576.0));
        assert!(pose.near < (pose.eye - pose.target).length());
        assert!(pose.far > (pose.eye - pose.target).length() + 640.0);
    }

    #[test]
    fn transform_faces_the_camera_target() {
        let pose = camera_pose(Vec2::new(160.0, 144.0));
        let transform = pose.transform();
        assert!(transform.forward().dot(camera_forward(pose)) > 0.999);
    }

    #[test]
    fn actor_card_faces_the_camera_without_foreshortening() {
        let pose = camera_pose(Vec2::new(640.0, 576.0));
        let rotation = card_rotation_toward_camera(pose);
        let toward_eye = (pose.eye - pose.target).normalize();
        let camera_up = pose.transform().up().as_vec3();

        assert!(rotation.mul_vec3(Vec3::Z).dot(toward_eye) > 0.999);
        assert!(rotation.mul_vec3(Vec3::Y).dot(camera_up) > 0.999);
    }
}
