//! Native, explicitly enabled shadow-only two-phase culling. Reuses Bevy's
//! stock shaders, depth pyramids, work lists, and indirect draw buffers.
use crate::{VoxelMaterial, VoxelSun, VoxelWorldCamera};
use bevy::{
    camera::{primitives::Aabb, visibility::CascadesVisibleEntities},
    core_pipeline::{
        mip_generation::experimental::depth::{
            DownsampleDepthPipelines, ViewDepthPyramid, ViewDownsampleDepthBindGroup,
        },
        prepass::PreviousViewData,
        schedule::{Core3d, Core3dSystems},
    },
    light::Cascades,
    pbr::{
        EARLY_SHADOW_PASS, LATE_SHADOW_PASS, LightEntity, MeshInputUniform, MeshUniform,
        PhasePreprocessBindGroups, PreprocessBindGroups, PreprocessPipelines, ShadowView,
        per_view_shadow_pass, prepare_lights, prepare_previous_view_uniforms,
    },
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        batching::gpu_preprocessing::{
            BatchedInstanceBuffers, LatePreprocessWorkItemIndirectParameters,
            PreprocessWorkItemBuffers,
        },
        diagnostic::RecordDiagnostics,
        occlusion_culling::{
            OcclusionCulling, OcclusionCullingSubview, OcclusionCullingSubviewEntities,
        },
        render_resource::{ComputePassDescriptor, PipelineCache},
        renderer::{RenderContext, ViewQuery},
        view::{ExtractedView, RetainedViewEntity, ViewUniformOffset},
    },
};
use std::collections::HashMap;
mod ortho_patch;
mod skinned_bounds;

#[derive(Resource, Default)]
struct Proofs(HashMap<RetainedViewEntity, Option<Mat4>>, [usize; 7], bool);
#[derive(Component)]
struct Eligible;
#[derive(Resource, Default)]
struct History(HashMap<RetainedViewEntity, PreviousViewData>);

pub(super) fn register(app: &mut App) {
    ortho_patch::register(app);
    skinned_bounds::register::<VoxelMaterial>(app);
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render
        .init_resource::<Proofs>()
        .init_resource::<History>()
        .add_systems(ExtractSchedule, extract_proofs)
        .add_systems(
            Render,
            guard_views
                .in_set(RenderSystems::CreateViews)
                .after(prepare_lights)
                .before(prepare_previous_view_uniforms),
        )
        .add_systems(
            Core3d,
            (
                early_pyramids
                    .after(per_view_shadow_pass::<EARLY_SHADOW_PASS>)
                    .before(late_retest)
                    .before(per_view_shadow_pass::<LATE_SHADOW_PASS>),
                late_retest.before(per_view_shadow_pass::<LATE_SHADOW_PASS>),
                late_pyramids.after(Core3dSystems::MainPass),
            ),
        );
}

// The stock occlusion shader clamps view Z to -projection[3][2]. Directional
// cascades have projection[3][2]=1 and unclipped-depth casters. Prove this clamp
// is inert for every caster before using that unmodified shader.
fn behind_stock_near_clamp(
    bounds: &Aabb,
    transform: &GlobalTransform,
    view_from_world: Mat4,
) -> bool {
    if !view_from_world.is_finite() {
        return false;
    }
    crate::shadow_bounds::corners(bounds, transform)
        .iter()
        .all(|&point| point.is_finite() && view_from_world.transform_point3(point).z <= -2.0)
}
fn extract_proofs(
    mut proofs: ResMut<Proofs>,
    world: Extract<Res<crystal_render_api::VisualWorldFrame>>,
    cameras: Extract<Query<Entity, With<VoxelWorldCamera>>>,
    lights: Extract<Query<(Entity, &Cascades, &CascadesVisibleEntities), With<VoxelSun>>>,
    casters: Extract<
        Query<(
            &Mesh3d,
            Option<&Aabb>,
            &GlobalTransform,
            Has<bevy::mesh::morph::MeshMorphWeights>,
            Option<&bevy::mesh::skinning::SkinnedMesh>,
            Option<&skinned_bounds::FreshBounds>,
            Option<&MeshMaterial3d<VoxelMaterial>>,
            Has<MeshMaterial3d<StandardMaterial>>,
        )>,
    >,
    meshes: Extract<Res<Assets<Mesh>>>,
    binds: Extract<Res<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>>,
    joints: Extract<Query<&GlobalTransform>>,
    frame: Extract<Res<bevy::diagnostic::FrameCount>>,
    materials: Extract<Res<Assets<VoxelMaterial>>>,
    patch: Extract<Res<ortho_patch::VerifiedPatch>>,
) {
    proofs.0.clear();
    let world_supported = world.active && super::supported(&world.map_id);
    proofs.1 = [0; 7];
    proofs.2 = patch.requested && patch.verified;
    let corrected = proofs.2;
    for camera in &cameras {
        for (light, cascades, visible) in &lights {
            let (Some(cascades), Some(visible)) = (
                cascades.cascades.get(&camera),
                visible.entities.get(&camera),
            ) else {
                continue;
            };
            for (index, (cascade, entities)) in cascades.iter().zip(visible).enumerate() {
                let key = RetainedViewEntity::new(light.into(), Some(camera.into()), index as u32);
                proofs.0.insert(key, None);
                if !world_supported {
                    proofs.1[0] += 1;
                    continue;
                }
                let view_from_world = Mat4::from(
                    GlobalTransform::from(cascade.world_from_cascade)
                        .affine()
                        .inverse(),
                );
                let mut rejection = None;
                if !cascade.clip_from_world.is_finite()
                    || !view_from_world.is_finite()
                    || !ortho_patch::standard_reverse_ortho(cascade.clip_from_cascade)
                {
                    rejection = Some(1);
                } else {
                    for &entity in &entities.entities {
                        let Ok((
                            mesh,
                            bounds,
                            transform,
                            morph,
                            skin,
                            fresh_skin,
                            material,
                            standard,
                        )) = casters.get(entity)
                        else {
                            rejection = Some(2);
                            break;
                        };
                        if material.is_none() && !standard {
                            rejection = Some(2);
                            break;
                        }
                        let skin_safe = skin.is_none_or(|skin| {
                            corrected
                                && meshes.get(&mesh.0).is_some_and(|asset| {
                                    binds.get(&skin.inverse_bindposes).is_some_and(|bind| {
                                        skinned_bounds::verified(
                                            fresh_skin, frame.0, mesh, asset, skin, bind,
                                            transform, bounds, &joints,
                                        )
                                    })
                                })
                        });
                        if morph || !skin_safe {
                            rejection = Some(3);
                            break;
                        }
                        if material.is_some_and(|handle| {
                            materials.get(&handle.0).map_or(true, |m| {
                                let grass = m.extension.grass_rustle;
                                let active = !grass.contact.is_finite()
                                    || !grass.height_scale.is_finite()
                                    || (grass.contact.z > 0.0 && grass.contact.w != 0.0)
                                    || (grass.height_scale.x > 0.0 && grass.height_scale.x != 1.0);
                                active
                                    && meshes.get(&mesh.0).map_or(true, |mesh| {
                                        match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                                            None => false,
                                            Some(bevy::mesh::VertexAttributeValues::Float32x2(
                                                uvs,
                                            )) => uvs.iter().any(|uv| {
                                                !uv[0].is_finite() || (uv[0] - 84000.0).abs() < 0.5
                                            }),
                                            Some(_) => true,
                                        }
                                    })
                            })
                        }) {
                            rejection = Some(4);
                            break;
                        }
                        match bounds {
                            Some(bounds)
                                if !(if corrected {
                                    crate::shadow_bounds::corners(bounds, transform).iter().all(
                                        |&point| {
                                            point.is_finite()
                                                && view_from_world
                                                    .transform_point3(point)
                                                    .is_finite()
                                        },
                                    )
                                } else {
                                    behind_stock_near_clamp(bounds, transform, view_from_world)
                                }) =>
                            {
                                rejection = Some(5);
                                break;
                            }
                            None if !crate::shadow_bounds::proven_empty(meshes.get(&mesh.0)) => {
                                rejection = Some(6);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                if let Some(reason) = rejection {
                    proofs.1[reason] += 1;
                    continue;
                }
                proofs.0.insert(key, Some(cascade.clip_from_world));
            }
        }
    }
}

fn pipelines_ready(
    depth: Option<&DownsampleDepthPipelines>,
    preprocess: Option<&PreprocessPipelines>,
    cache: &PipelineCache,
) -> bool {
    let (Some(depth), Some(preprocess)) = (depth, preprocess) else {
        return false;
    };
    [
        depth.first.pipeline_id,
        depth.second.pipeline_id,
        preprocess
            .early_gpu_occlusion_culling_preprocess
            .pipeline_id,
        preprocess.late_gpu_occlusion_culling_preprocess.pipeline_id,
    ]
    .iter()
    .all(|id| id.is_some_and(|id| cache.get_compute_pipeline(id).is_some()))
}
fn current_matrices(view: &ExtractedView) -> PreviousViewData {
    let view_from_world = Mat4::from(view.world_from_view.affine().inverse());
    let clip_from_world = view
        .clip_from_world
        .unwrap_or(view.clip_from_view * view_from_world);
    PreviousViewData {
        view_from_world,
        clip_from_world,
        unjittered_clip_from_world: clip_from_world,
        clip_from_view: view.clip_from_view,
        world_from_clip: clip_from_world.inverse(),
        view_from_clip: view.clip_from_view.inverse(),
    }
}
fn guard_views(
    mut commands: Commands,
    proofs: Res<Proofs>,
    mut history: ResMut<History>,
    views: Query<(Entity, &ExtractedView, &LightEntity, Has<OcclusionCulling>), With<ShadowView>>,
    depth: Option<Res<DownsampleDepthPipelines>>,
    preprocess: Option<Res<PreprocessPipelines>>,
    cache: Res<PipelineCache>,
    trace: Res<super::SupportTrace>,
    patch: Res<ortho_patch::VerifiedPatch>,
    mut last_report: Local<Option<(bool, usize, usize, [usize; 7], bool)>>,
) {
    // Shader Modified events flush in main PostUpdate and invalidate every
    // dependent cache entry in ExtractSchedule, before this render system.
    // Ready therefore means the verified replacement has actually compiled;
    // also prove these are pipelines for the exact patched shader asset.
    let patched_pipelines = patch.verified
        && patch.shader_id.is_some_and(|shader| {
            preprocess.as_deref().is_some_and(|p| {
                [
                    p.early_gpu_occlusion_culling_preprocess.pipeline_id,
                    p.late_gpu_occlusion_culling_preprocess.pipeline_id,
                ]
                .iter()
                .all(|id| {
                    id.is_some_and(|id| {
                        cache.get_compute_pipeline_descriptor(id).shader.id() == shader
                    })
                })
            })
        });
    let ready = pipelines_ready(depth.as_deref(), preprocess.as_deref(), &cache)
        && (!proofs.2 || patched_pipelines);
    history
        .0
        .retain(|key, _| proofs.0.get(key).is_some_and(Option::is_some));
    let mut activated = 0;
    for (entity, view, light, marked) in &views {
        if !matches!(light, LightEntity::Directional { .. }) {
            continue;
        }
        let Some(proof) = proofs.0.get(&view.retained_view_entity) else {
            continue;
        };
        let eligible = (!proofs.2 || patch.verified)
            && marked
            && ready
            && proof.as_ref().is_some_and(|matrix| {
                view.clip_from_world == Some(*matrix)
                    && ortho_patch::standard_reverse_ortho(view.clip_from_view)
            });
        if eligible {
            activated += 1;
            let previous = history
                .0
                .get(&view.retained_view_entity)
                .cloned()
                .unwrap_or_else(|| current_matrices(view));
            commands.entity(entity).insert((Eligible, previous));
        } else {
            // Stock frustum rendering remains authoritative on unsupported
            // cascades, animation, missing matrices, or incomplete pipelines.
            commands
                .entity(entity)
                .remove::<(Eligible, OcclusionCulling)>();
        }
    }
    let proof_count = proofs.0.values().filter(|proof| proof.is_some()).count();
    let stats = (ready, proof_count, activated, proofs.1, proofs.2);
    if trace.0 && *last_report != Some(stats) {
        eprintln!(
            "shadow_hzb_guard pipelines_ready={ready} owned_cascades={} bounds_safe={proof_count} activated={activated} orthographic_patch={} rejected_world={} matrix={} unknown_caster={} animated={} grass={} near_clamp={} missing_bounds={}",
            proofs.0.len(),
            proofs.2,
            proofs.1[0],
            proofs.1[1],
            proofs.1[2],
            proofs.1[3],
            proofs.1[4],
            proofs.1[5],
            proofs.1[6]
        );
        *last_report = Some(stats);
    }
}

fn early_pyramids(
    root: ViewQuery<Option<&OcclusionCullingSubviewEntities>>,
    views: Query<
        (
            &ViewDepthPyramid,
            &ViewDownsampleDepthBindGroup,
            &OcclusionCullingSubview,
        ),
        With<Eligible>,
    >,
    pipelines: Option<Res<DownsampleDepthPipelines>>,
    cache: Res<PipelineCache>,
    mut ctx: RenderContext,
) {
    let (Some(links), Some(pipelines)) = (root.into_inner(), pipelines.as_deref()) else {
        return;
    };
    let (Some(first), Some(second)) = (
        pipelines
            .first
            .pipeline_id
            .and_then(|p| cache.get_compute_pipeline(p)),
        pipelines
            .second
            .pipeline_id
            .and_then(|p| cache.get_compute_pipeline(p)),
    ) else {
        return;
    };
    for &entity in &links.0 {
        let Ok((pyramid, binding, subview)) = views.get(entity) else {
            continue;
        };
        pyramid.downsample_depth_with_ctx(
            "crystal_shadow_early_downsample_depth",
            &mut ctx,
            UVec2::splat(subview.depth_texture_size),
            binding,
            first,
            second,
        );
    }
}

fn late_retest(
    root: ViewQuery<Option<&OcclusionCullingSubviewEntities>>,
    views: Query<(&ExtractedView, &PreprocessBindGroups, &ViewUniformOffset), With<Eligible>>,
    buffers: Res<BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>,
    pipelines: Res<PreprocessPipelines>,
    cache: Res<PipelineCache>,
    mut ctx: RenderContext,
) {
    let Some(links) = root.into_inner() else {
        return;
    };
    // Do not emit an empty named pass and mistake it for active shadow HZB.
    if !links.0.iter().any(|&entity| views.get(entity).is_ok()) {
        return;
    }
    let Some(pipeline) = pipelines
        .late_gpu_occlusion_culling_preprocess
        .pipeline_id
        .and_then(|p| cache.get_compute_pipeline(p))
    else {
        return;
    };
    let recorder = ctx.diagnostic_recorder();
    let recorder = recorder.as_deref();
    let mut pass = ctx
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("crystal_shadow_late_mesh_preprocessing"),
            timestamp_writes: None,
        });
    let span = recorder.pass_span(&mut pass, "crystal_shadow_late_mesh_preprocessing");
    pass.set_pipeline(pipeline);
    for &entity in &links.0 {
        let Ok((view, bindings, uniform)) = views.get(entity) else {
            continue;
        };
        for (phase, phase_buffers) in &buffers.phase_instance_buffers {
            let Some(PreprocessWorkItemBuffers::Indirect {
                gpu_occlusion_culling: Some(work),
                ..
            }) = phase_buffers
                .work_item_buffers
                .get(&view.retained_view_entity)
            else {
                continue;
            };
            let Some(PhasePreprocessBindGroups::IndirectOcclusionCulling {
                late_indexed,
                late_non_indexed,
                ..
            }) = bindings.get(phase)
            else {
                continue;
            };
            for (binding, indirect, index) in [
                (
                    late_indexed,
                    phase_buffers
                        .late_indexed_indirect_parameters_buffer
                        .buffer(),
                    work.late_indirect_parameters_indexed_offset,
                ),
                (
                    late_non_indexed,
                    phase_buffers
                        .late_non_indexed_indirect_parameters_buffer
                        .buffer(),
                    work.late_indirect_parameters_non_indexed_offset,
                ),
            ] {
                let (Some(binding), Some(indirect)) = (binding, indirect) else {
                    continue;
                };
                pass.set_immediates(0, &index.to_le_bytes());
                pass.set_bind_group(0, binding, &[uniform.offset]);
                pass.dispatch_workgroups_indirect(
                    indirect,
                    u64::from(index) * size_of::<LatePreprocessWorkItemIndirectParameters>() as u64,
                );
            }
        }
    }
    span.end(&mut pass);
}
fn late_pyramids(
    root: ViewQuery<Option<&OcclusionCullingSubviewEntities>>,
    views: Query<
        (
            &ExtractedView,
            &ViewDepthPyramid,
            &ViewDownsampleDepthBindGroup,
            &OcclusionCullingSubview,
        ),
        With<Eligible>,
    >,
    pipelines: Option<Res<DownsampleDepthPipelines>>,
    cache: Res<PipelineCache>,
    mut history: ResMut<History>,
    mut ctx: RenderContext,
) {
    let (Some(links), Some(pipelines)) = (root.into_inner(), pipelines.as_deref()) else {
        return;
    };
    let (Some(first), Some(second)) = (
        pipelines
            .first
            .pipeline_id
            .and_then(|p| cache.get_compute_pipeline(p)),
        pipelines
            .second
            .pipeline_id
            .and_then(|p| cache.get_compute_pipeline(p)),
    ) else {
        return;
    };
    for &entity in &links.0 {
        let Ok((view, pyramid, binding, subview)) = views.get(entity) else {
            continue;
        };
        pyramid.downsample_depth_with_ctx(
            "crystal_shadow_late_downsample_depth",
            &mut ctx,
            UVec2::splat(subview.depth_texture_size),
            binding,
            first,
            second,
        );
        // These matrices now describe the exact depth pyramid sampled by the
        // next early phase, including light/camera movement between frames.
        history
            .0
            .insert(view.retained_view_entity, current_matrices(view));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shadow_hzb_stock_near_clamp_guard_retains_unknown_or_crossing_casters() {
        let bounds = Aabb::from_min_max(Vec3::splat(-0.5), Vec3::splat(0.5));
        assert!(behind_stock_near_clamp(
            &bounds,
            &GlobalTransform::from_translation(Vec3::new(0., 0., -3.)),
            Mat4::IDENTITY
        ));
        for z in [-2., -1., 0., 100.] {
            assert!(!behind_stock_near_clamp(
                &bounds,
                &GlobalTransform::from_translation(Vec3::new(0., 0., z)),
                Mat4::IDENTITY
            ));
        }
        assert!(!behind_stock_near_clamp(
            &bounds,
            &GlobalTransform::IDENTITY,
            Mat4::from_cols_array(&[f32::NAN; 16])
        ));
    }
    #[test]
    fn cascade_history_keeps_last_built_projection_instead_of_next_camera_matrix() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let key = RetainedViewEntity::new(entity.into(), Some(entity.into()), 2);
        let view = ExtractedView {
            retained_view_entity: key,
            clip_from_world: Some(Mat4::from_translation(Vec3::X)),
            clip_from_view: Mat4::IDENTITY,
            world_from_view: GlobalTransform::IDENTITY,
            target_format: bevy::render::render_resource::TextureFormat::Depth32Float,
            viewport: UVec4::new(0, 0, 1024, 1024),
            color_grading: default(),
            invert_culling: false,
        };
        let built = current_matrices(&view);
        let mut history = History::default();
        history.0.insert(key, built.clone());
        let moved = ExtractedView {
            clip_from_world: Some(Mat4::from_translation(Vec3::Y)),
            ..view
        };
        assert_eq!(history.0[&key].clip_from_world, built.clip_from_world);
        assert_ne!(
            history.0[&key].clip_from_world,
            current_matrices(&moved).clip_from_world
        );
        assert_eq!(
            current_matrices(&moved).world_from_clip,
            moved.clip_from_world.unwrap().inverse()
        );
    }
}
