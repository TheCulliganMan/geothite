//! Startup QA only. Transform the installed shader in memory; no shader copy is shipped.
use bevy::{
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
    shader::{Shader, Source},
};
use sha2::{Digest, Sha256};
use std::borrow::Cow;

const STOCK_SHA256: &str = "aeeed30d74bbc88f1037113b6385ab8002c15245a9a2a1b2157d370e01dd26ea";
const PREVIOUS: &str =
    "max_depth_view = min(-previous_view_uniforms.clip_from_view[3][2], max_depth_view);";
const CURRENT: &str = "max_depth_view = min(-view.clip_from_view[3][2], max_depth_view);";

#[derive(Resource, Clone, Default, ExtractResource)]
#[extract_app(bevy::render::RenderApp)]
pub(super) struct VerifiedPatch {
    pub requested: bool,
    pub verified: bool,
    pub shader_id: Option<AssetId<Shader>>,
    patched_digest: Option<String>,
}

pub(super) fn register(app: &mut App) {
    let requested = std::env::var("CRYSTAL_SHADOW_ORTHO_CLAMP").as_deref() == Ok("on");
    app.insert_resource(VerifiedPatch {
        requested,
        ..default()
    })
    .add_plugins(ExtractResourcePlugin::<VerifiedPatch>::default());
    if requested {
        app.add_systems(Update, patch_loaded_shader);
    }
}

fn digest(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}
fn replacement(uniform: &str, legacy: &str) -> String {
    // A block keeps the surrounding WESL @if/@else attached to one statement.
    format!(
        "{{\n        // crystal_shadow_orthographic_near_v1\n        if {uniform}.clip_from_view[3][3] == 1.0 && {uniform}.clip_from_view[2][3] == 0.0 {{\n            max_depth_view = min((1.0 - {uniform}.clip_from_view[3][2]) / {uniform}.clip_from_view[2][2], max_depth_view);\n        }} else {{\n            {legacy}\n        }}\n    }}"
    )
}
fn replace_clamps(source: &str) -> Result<String, &'static str> {
    if source.contains("crystal_shadow_orthographic_near_v1")
        || source.matches(PREVIOUS).count() != 1
        || source.matches(CURRENT).count() != 1
    {
        return Err("unexpected near-clamp clauses");
    }
    Ok(source
        .replace(PREVIOUS, &replacement("previous_view_uniforms", PREVIOUS))
        .replace(CURRENT, &replacement("view", CURRENT)))
}
fn patch_loaded_shader(
    mut shaders: ResMut<Assets<Shader>>,
    mut state: ResMut<VerifiedPatch>,
    mut last: Local<Option<String>>,
) {
    let ids: Vec<_> = shaders
        .iter()
        .filter(|(_, shader)| shader.path.ends_with("/mesh_preprocess.wesl"))
        .map(|(id, _)| id)
        .collect();
    let mut status = "waiting for embedded shader".to_owned();
    state.verified = false;
    state.shader_id = None;
    if ids.len() == 1 {
        state.shader_id = Some(ids[0]);
        let shader = shaders.get(ids[0]).unwrap();
        if let Source::Wesl(source) = &shader.source {
            let hash = digest(source);
            if state.patched_digest.as_deref() == Some(&hash) {
                state.verified = true;
                status = "verified patched source".into();
            } else if hash == STOCK_SHA256 {
                match replace_clamps(source) {
                    Ok(patched) => {
                        state.patched_digest = Some(digest(&patched));
                        // Update precedes PostUpdate Assets<Shader>::asset_events.
                        // ExtractSchedule then invalidates dependent PipelineCache
                        // entries before the render-world eligibility guard runs.
                        // Only Source changes; path/imports/defs/dependencies stay exact.
                        shaders.get_mut(ids[0]).unwrap().source = Source::Wesl(Cow::Owned(patched));
                        state.verified = true;
                        status =
                            "patched verified Bevy 0.20 source; awaiting compiled pipelines".into();
                    }
                    Err(reason) => status = reason.into(),
                }
            } else {
                status = format!("rejected unexpected shader sha256={hash}");
            }
        } else {
            status = "rejected non-WESL shader".into();
        }
    } else if ids.len() > 1 {
        status = "rejected ambiguous shader assets".into();
    }
    if std::env::var("CRYSTAL_GPU_CULLING_TRACE").as_deref() == Ok("on")
        && last.as_ref() != Some(&status)
    {
        eprintln!(
            "shadow_hzb_ortho_patch verified={} status={status}",
            state.verified
        );
        *last = Some(status);
    }
}

pub(super) fn standard_reverse_ortho(p: Mat4) -> bool {
    p.is_finite()
        && p.x_axis.z == 0.0
        && p.y_axis.z == 0.0
        && p.x_axis.w == 0.0
        && p.y_axis.w == 0.0
        && p.z_axis.w == 0.0
        && p.w_axis.w == 1.0
        && p.w_axis.z == 1.0
        && p.z_axis.z > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orthographic_patch_changes_only_two_verified_statements_and_keeps_perspective() {
        let source = format!("prefix\n@if(EARLY_PHASE)\n{PREVIOUS}\n@else\n{CURRENT}\nsuffix");
        let patched = replace_clamps(&source).unwrap();
        assert_eq!(
            patched
                .matches("crystal_shadow_orthographic_near_v1")
                .count(),
            2
        );
        assert_eq!(patched.matches(PREVIOUS).count(), 1);
        assert_eq!(patched.matches(CURRENT).count(), 1);
        assert_eq!(
            patched
                .replace(&replacement("previous_view_uniforms", PREVIOUS), PREVIOUS)
                .replace(&replacement("view", CURRENT), CURRENT),
            source
        );
        assert!(replace_clamps(&patched).is_err());
        assert!(replace_clamps(&source.replace(CURRENT, "changed")).is_err());
        assert!(replace_clamps(&(source.clone() + CURRENT)).is_err());
        assert_ne!(digest(&source), STOCK_SHA256);
    }
    #[test]
    fn reverse_orthographic_near_cap_retains_front_crossing_and_unclipped_casters() {
        for far in [2.0f32, 64.0, 2048.0, 8192.0] {
            let a = 1.0 / far;
            let p = Mat4::from_cols(
                Vec4::X,
                Vec4::Y,
                Vec4::new(0., 0., a, 0.),
                Vec4::new(0., 0., 1., 1.),
            );
            assert!(standard_reverse_ortho(p));
            for nearest_z in [-far * 2., -far, -2., -1., -0.001, 0., 0.001, 1., far] {
                let near_z = (1.0 - p.w_axis.z) / p.z_axis.z;
                let patched_ndc = p.w_axis.z + nearest_z.min(near_z) * p.z_axis.z;
                let true_nearest_ndc = (p.w_axis.z + nearest_z * p.z_axis.z).min(1.0);
                assert!((patched_ndc - true_nearest_ndc).abs() <= 2. * f32::EPSILON);
                if nearest_z >= 0. {
                    assert_eq!(patched_ndc, 1.0);
                }
            }
            // The stock perspective clamp can incorrectly move an orthographic
            // front-crossing bound behind a valid occluder. The correction cannot.
            let occluder = 1.0 - 0.25 * a;
            assert!(1.0 + (-1.0f32).min(0.1) * a < occluder);
            assert!(1.0 + 0.0f32.min(0.1) * a >= occluder);
        }
        for a in [0., -1., f32::NAN, f32::INFINITY] {
            let mut p = Mat4::IDENTITY;
            p.z_axis.z = a;
            p.w_axis.z = 1.;
            assert!(!standard_reverse_ortho(p));
        }
        assert!(!standard_reverse_ortho(
            Mat4::perspective_infinite_reverse_rh(1., 1., 0.1)
        ));
    }
}
