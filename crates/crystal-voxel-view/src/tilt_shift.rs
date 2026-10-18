//! A trainer-centered focus band, applied to the 3D world before tonemapping.
use bevy::{
    asset::load_internal_asset,
    core_pipeline::{FullscreenShader, schedule::Core3d, tonemapping::tonemapping},
    post_process::bloom::bloom,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        view::ViewTarget,
    },
};
use std::collections::HashMap;

const SHADER: Handle<Shader> = Handle::Uuid(
    bevy::asset::uuid::Uuid::from_u128(0x9f26_15d2_114c_4b83_a25d_8514_1f0c_780a),
    std::marker::PhantomData,
);

#[derive(Component, Clone, Copy, PartialEq, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub(super) struct TiltShift {
    // Focus Y, sharp half-width, falloff exponent, maximum blur diameter.
    // A vec4 keeps the exact same uniform layout on WebGL and WebGPU.
    pub band: Vec4,
}

impl TiltShift {
    /// Exploration needs nearby architecture and people to stay readable.
    /// Retain a gentle miniature falloff at the edges without erasing the
    /// scenery detail that occupies the upper and lower thirds of a town view.
    pub fn readable_exterior(mut self) -> Self {
        self.band.y = self.band.y.max(0.20);
        self.band.w *= 0.55;
        self
    }

    pub fn for_trainer(
        projection: &Projection,
        camera: &Transform,
        torso: Vec3,
        half_height: f32,
        framebuffer_height: f32,
    ) -> Option<Self> {
        let clip_from_world = projection.get_clip_from_view() * camera.to_matrix().inverse();
        let project_y = |point: Vec3| {
            let clip = clip_from_world * point.extend(1.0);
            (clip.is_finite() && clip.w > 0.0).then(|| 0.5 - clip.y / clip.w * 0.5)
        };
        let center = project_y(torso)?;
        let head = project_y(torso + Vec3::Y * half_height)?;
        let feet = project_y(torso - Vec3::Y * half_height)?;
        // Protect nearby scenery as well as the complete trainer. The ramp
        // starts outside this band and only reaches full strength at the edge.
        let half_width = (head - center).abs().max((feet - center).abs()) + 0.055;
        Some(Self {
            band: Vec4::new(
                center,
                half_width.max(0.14),
                2.5,
                (framebuffer_height * 0.018).clamp(3.0, 36.0),
            ),
        })
    }

    #[cfg(test)]
    pub fn blur_diameter(self, y: f32) -> f32 {
        let edge_distance = if y < self.band.x {
            self.band.x
        } else {
            1.0 - self.band.x
        };
        let transition = (edge_distance - self.band.y).max(0.0001);
        let t = (((y - self.band.x).abs() - self.band.y) / transition).clamp(0.0, 1.0);
        // Zero slope and curvature at the sharp band, then a continuous
        // increase right to the viewport edge, without an outer plateau.
        t.powf(self.band.z) * self.band.w
    }
}

pub(super) fn register(app: &mut App) {
    if app.get_sub_app(RenderApp).is_none() {
        return;
    }
    load_internal_asset!(app, SHADER, "tilt_shift.wesl", Shader::from_wesl);
    app.add_plugins((
        ExtractComponentPlugin::<TiltShift>::default(),
        UniformComponentPlugin::<TiltShift>::default(),
    ));
    app.get_sub_app_mut(RenderApp)
        .unwrap()
        .add_systems(RenderStartup, init_pipeline)
        .add_systems(Render, prepare.in_set(RenderSystems::PrepareBindGroups))
        .add_systems(Core3d, draw.after(bloom).before(tonemapping));
}

#[derive(Resource)]
struct Pipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    vertex: VertexState,
    formats: HashMap<TextureFormat, [CachedRenderPipelineId; 2]>,
}

fn init_pipeline(
    mut commands: Commands,
    device: Res<RenderDevice>,
    fullscreen: Res<FullscreenShader>,
) {
    commands.insert_resource(Pipeline {
        layout: BindGroupLayoutDescriptor::new(
            "tilt_shift_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<TiltShift>(true),
                ),
            ),
        ),
        sampler: device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        }),
        vertex: fullscreen.to_vertex_state(),
        formats: HashMap::new(),
    });
}

#[derive(Component)]
struct Prepared {
    pipelines: [CachedRenderPipelineId; 2],
    uniform: BufferId,
    a: (TextureViewId, BindGroup),
    b: (TextureViewId, BindGroup),
}

fn prepare(
    mut commands: Commands,
    mut pipeline: ResMut<Pipeline>,
    cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<TiltShift>>,
    device: Res<RenderDevice>,
    mut views: Query<(Entity, &ViewTarget, Option<&mut Prepared>), With<TiltShift>>,
) {
    let Some(binding) = uniforms.uniforms().binding() else {
        return;
    };
    let Some(buffer) = uniforms.uniforms().buffer() else {
        return;
    };
    for (entity, target, current) in &mut views {
        let format = target.main_texture_format();
        let ids = if let Some(ids) = pipeline.formats.get(&format) {
            *ids
        } else {
            let ids = ["horizontal", "vertical"].map(|entry| {
                cache.queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some(format!("tilt_shift_{entry}").into()),
                    layout: vec![pipeline.layout.clone()],
                    vertex: pipeline.vertex.clone(),
                    fragment: Some(FragmentState {
                        shader: SHADER,
                        entry_point: Some(entry.into()),
                        targets: vec![Some(ColorTargetState {
                            format,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                        ..default()
                    }),
                    ..default()
                })
            });
            pipeline.formats.insert(format, ids);
            ids
        };
        let a = target.main_texture_view();
        let b = target.main_texture_other_view();
        if current.as_ref().is_some_and(|p| {
            p.a.0 == a.id() && p.b.0 == b.id() && p.uniform == buffer.id() && p.pipelines == ids
        }) {
            continue;
        }
        let bind = |texture: &TextureView| {
            (
                texture.id(),
                device.create_bind_group(
                    "tilt_shift_bind_group",
                    &cache.get_bind_group_layout(&pipeline.layout),
                    &BindGroupEntries::sequential((texture, &pipeline.sampler, binding.clone())),
                ),
            )
        };
        commands.entity(entity).insert(Prepared {
            pipelines: ids,
            uniform: buffer.id(),
            a: bind(a),
            b: bind(b),
        });
    }
}

fn draw(
    view: ViewQuery<(
        &TiltShift,
        &ViewTarget,
        &DynamicUniformIndex<TiltShift>,
        &Prepared,
    )>,
    cache: Res<PipelineCache>,
    mut ctx: RenderContext,
) {
    let (_, target, index, prepared) = view.into_inner();
    // Do not flip the target unless BOTH passes are ready to execute.
    let (Some(horizontal), Some(vertical)) = (
        cache.get_render_pipeline(prepared.pipelines[0]),
        cache.get_render_pipeline(prepared.pipelines[1]),
    ) else {
        return;
    };
    for pipeline in [horizontal, vertical] {
        let output = target.post_process_write();
        let group = if output.source.id() == prepared.a.0 {
            &prepared.a.1
        } else {
            &prepared.b.1
        };
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("tilt_shift_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: output.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                ..default()
            });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[index.index()]);
        pass.draw(0..3, 0..1);
    }
}
