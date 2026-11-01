//! Presentation spacing uses the same physical model as the world renderer.
use bevy::prelude::*;
use crystal_render_api::{VisualActorId, VisualPokemonDimensions, VisualWorldFrame};

/// Trail distance in visual pixels. Missing model/dimension data keeps the
/// caller's ordinary sprite-card spacing; it never invents a physical size.
/// Distance is independent of camera controls, so looking around cannot move
/// a companion. A half-metre surface gap and a comfortable standing view leave
/// room for forward horns, snouts, broad shoulders and taller bodies.
pub fn pokemon_following_distance(
    frame: &VisualWorldFrame,
    species: &str,
    dimensions: &VisualPokemonDimensions,
) -> Option<f32> {
    let meters = *dimensions.meters.get(species)?;
    let canonical = crate::battle_species_models::rig(species)?;
    let reposed = canonical.species == crate::species_rig::Species::Furret;
    let rig = if reposed { crate::species_rig::motion::world_rig(canonical) } else { canonical };
    let reference = crate::battle_layout::world_reference_span(species,rig);
    let pixels_per_meter = frame.tile_size.y * 2.;
    if !meters.is_finite() || meters <= 0. || !reference.is_finite() || reference <= 0.
        || !pixels_per_meter.is_finite() || pixels_per_meter <= 0. { return None; }
    let scale = meters / reference;
    let (lo, hi) = if reposed { furret_following_bounds(rig) } else { rig.animated_bounds };
    let eye = frame.actors.iter().find(|a| a.id == VisualActorId::Player)
        .map(crate::new_bark_actors::player_eye_height)?;
    // A neutral, wide first-person view looks down 20 degrees. Reserve five
    // degrees above the silhouette and horizontal room for a square viewport.
    // User zoom/pitch still work normally; this does not auto-frame the camera.
    let above_eye = ((hi.y * scale - eye) / 25_f32.to_radians().tan()).max(0.);
    let half_width = lo.x.abs().max(hi.x.abs()) * scale / 40_f32.to_radians().tan();
    let clearance = 0.5_f32.max(above_eye).max(half_width);
    let distance = (hi.z.max(0.) * scale + clearance).max(1.) * pixels_per_meter;
    distance.is_finite().then_some(distance)
}

fn furret_following_bounds(rig: &'static crate::species_rig::SpeciesRig) -> (Vec3,Vec3) {
    static BOUNDS:std::sync::OnceLock<(Vec3,Vec3)>=std::sync::OnceLock::new();
    *BOUNDS.get_or_init(|| {
        let mut bounds=rig.neutral_bounds;
        let mut pose=rig.joints.iter().map(|joint|joint.bind).collect::<Vec<_>>();
        let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
        // Cache a tight visible envelope, rather than the deliberately large
        // culling sphere or the unrelated upright battle pose.
        for clock in 0..16 { for phase in 0..32 {
            rig.sample_locomotion(rig.clip(crate::species_rig::Clip::Idle).duration*clock as f32/16.,
                std::f32::consts::TAU*phase as f32/32.,1.,&mut pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let p=rig.skin_point(v,&matrices);
                bounds.0=bounds.0.min(p);bounds.1=bounds.1.max(p);
            }
        }}
        let margin=Vec3::splat((rig.neutral_bounds.1.y-rig.neutral_bounds.0.y)*0.01);
        (bounds.0-margin,bounds.1+margin)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::VisualActor;

    fn frame() -> VisualWorldFrame {
        VisualWorldFrame {
            tile_size: Vec2::splat(32.), viewport_size: Vec2::new(1000., 700.),
            actors: vec![VisualActor { id: VisualActorId::Player, source_id: "chris".into(),
                model_palette: None, texture: Default::default(), center: Vec2::new(0.,32.),
                size: Vec2::splat(64.), facing: Some(Vec2::NEG_Y), flip_x: false, above_priority: false }],
            ..Default::default()
        }
    }

    #[test]
    fn pokemon_follow_first_person_eye_matches_rendered_trainer_scale() {
        let frame = frame();let player=&frame.actors[0];
        let pose=crate::camera::VoxelCameraControls::with_pose(0.,0.,20.,true).world_pose(&frame,&[]);
        // Chris's authored body height is 1.20; the displayed player uses the
        // production character scale, including its 1.25 presentation factor.
        let scale=crate::new_bark_actors::character_scale(player,frame.tile_size.y);
        assert!((pose.eye.y-1.20*0.85*scale).abs()<0.00001);
        assert_eq!(pose.eye.x,0.);assert_eq!(pose.eye.z,-32.);
    }

    #[test]
    fn pokemon_follow_keeps_full_size_horns_snouts_and_bodies_in_the_standing_view() {
        let frame=frame();let pixels=frame.tile_size.y*2.;
        let pose=crate::camera::VoxelCameraControls::with_pose(0.,0.,20.,true).world_pose(&frame,&[]);
        let projection=Mat4::perspective_rh(pose.vertical_fov_radians,1000./700.,pose.near,pose.far)
            * Mat4::look_at_rh(pose.eye,pose.target,pose.up);
        for (name,meters) in [("NIDOKING",1.397),("NIDOQUEEN",1.2954),("ARCANINE",1.905),("FERALIGATR",2.3114),("SLOWKING",1.4986),("PIKACHU",0.4064),("FURRET",1.8)] {
            let mut dimensions=VisualPokemonDimensions::default();dimensions.meters.insert(name.into(),meters);
            let gap=pokemon_following_distance(&frame,name,&dimensions).unwrap();
            let canonical=crate::battle_species_models::rig(name).unwrap();
            let rig=crate::species_rig::motion::world_rig(canonical);
            let reference=crate::battle_layout::world_reference_span(name,rig);
            let scale=pixels*meters/reference;
            let mut transforms=rig.joints.iter().map(|j|j.bind).collect::<Vec<_>>();
            let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
            for tick in 0..32 {
                rig.sample_locomotion(tick as f32/32.,tick as f32*std::f32::consts::TAU/32.,1.,&mut transforms);
                rig.skin_matrices_into(&transforms,&mut matrices).unwrap();
                for v in 0..rig.neutral.positions.len() {
                    let p=rig.skin_point(v,&matrices)*scale + Vec3::new(0.,0.,-gap-32.);
                    let clip=projection*p.extend(1.);
                    assert!(clip.w>0. && clip.x.abs()<clip.w && clip.y.abs()<clip.w && clip.z>0. && clip.z<clip.w,
                        "{name} vertex {v} leaves the standing frustum at gait {tick}: {clip:?}, gap {} m",gap/pixels);
                    assert!(p.z<pose.eye.z-pixels*0.45,"{name} intersects the trainer clearance");
                }
            }
            if name=="PIKACHU" {assert_eq!(gap,pixels);}
        }
    }

    #[test]
    fn pokemon_follow_requires_real_dimensions_and_scales_with_map_pixels() {
        let mut frame=frame();let mut dimensions=VisualPokemonDimensions::default();
        assert_eq!(pokemon_following_distance(&frame,"NIDOKING",&dimensions),None);
        for meters in [0.,-1.,f32::NAN,f32::INFINITY] {
            dimensions.meters.insert("NIDOKING".into(),meters);
            assert_eq!(pokemon_following_distance(&frame,"NIDOKING",&dimensions),None);
        }
        dimensions.meters.insert("NIDOKING".into(),1.397);
        let gap=pokemon_following_distance(&frame,"NIDOKING",&dimensions).unwrap();
        frame.tile_size*=2.;assert_eq!(pokemon_following_distance(&frame,"NIDOKING",&dimensions),Some(gap*2.));
        dimensions.meters.insert("UNKNOWN".into(),1.);
        assert_eq!(pokemon_following_distance(&frame,"UNKNOWN",&dimensions),None);
    }
}
