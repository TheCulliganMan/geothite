//! Retain GPU storage for composed RGBA surfaces and upload their changed region.
use bevy::{
    asset::{AssetEvent, AssetId, Assets, RenderAssetUsages},
    extract::{Extract, ExtractSchedule},
    image::ImageSampler,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_asset::{RenderAssets, prepare_assets},
        render_resource::{
            Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
            TextureDimension, TextureFormat, TextureViewDescriptor,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};
use std::collections::{HashMap, HashSet};

const LABEL: &str = "crystal-streamed-rgba";

/// Opt a CPU-readable, nearest-sampled RGBA surface into retained GPU uploads.
/// Ordinary source artwork continues through Bevy's standard image pipeline.
pub fn stream_composed_image(image: &mut Image) {
    assert_eq!(image.texture_descriptor.dimension, TextureDimension::D2);
    assert!(matches!(image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm));
    assert_eq!(image.texture_descriptor.size.depth_or_array_layers, 1);
    assert_eq!(image.texture_descriptor.mip_level_count, 1);
    assert_eq!(image.texture_descriptor.sample_count, 1);
    image.texture_descriptor.label = Some(LABEL);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD;
    image.sampler = ImageSampler::nearest();
}

/// Fixed-size, fixed-format streamed surfaces retain their GPU view on pixel updates.
pub fn is_streamed_image(image: &Image) -> bool {
    image.texture_descriptor.label == Some(LABEL)
}

pub(super) fn install(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<PendingImages>()
        .init_resource::<ResidentImages>()
        .add_systems(ExtractSchedule, extract_images)
        // Materials depend on standard image preparation. Run before it so
        // they see newly created composed surfaces in the same render frame.
        .add_systems(
            Render,
            upload_images
                .in_set(RenderSystems::PrepareAssets)
                .before(prepare_assets::<GpuImage>),
        );
}

#[derive(Resource, Default)]
struct PendingImages {
    changed: Vec<(AssetId<Image>, Image)>,
    removed: HashSet<AssetId<Image>>,
}
#[derive(Resource, Default)]
struct ResidentImages(HashMap<AssetId<Image>, (UVec2, TextureFormat, Vec<u8>)>);

fn extract_images(
    mut events: Extract<MessageReader<AssetEvent<Image>>>,
    images: Extract<Res<Assets<Image>>>,
    mut pending: ResMut<PendingImages>,
) {
    let mut changed = HashSet::new();
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed.insert(*id);
            }
            AssetEvent::Removed { id } | AssetEvent::Unused { id } => {
                pending.removed.insert(*id);
            }
            AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }
    for id in changed {
        if let Some(image) = images.get(id) {
            if is_streamed_image(image) {
                pending.changed.push((id, image.clone()));
            }
        }
    }
}

fn upload_images(
    mut pending: ResMut<PendingImages>,
    mut resident: ResMut<ResidentImages>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    for id in pending.removed.drain() {
        if resident.0.remove(&id).is_some() {
            gpu_images.remove(id);
        }
    }
    for (id, image) in pending.changed.drain(..) {
        let size = image.size();
        let format = image.texture_descriptor.format;
        let data = image
            .data
            .as_ref()
            .expect("streamed images retain CPU pixels");
        assert_eq!(data.len(), size.x as usize * size.y as usize * 4);
        if let Some((old_size, old_format, old_data)) = resident.0.get(&id)
            && *old_size == size
            && *old_format == image.texture_descriptor.format
            && let Some(gpu) = gpu_images.get(id)
        {
            if let Some([x, y, width, height]) = changed_rect(old_data, data, size.x as usize) {
                let offset = (y * size.x as usize + x) * 4;
                queue.write_texture(
                    TexelCopyTextureInfo {
                        texture: &gpu.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: x as u32,
                            y: y as u32,
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    &data[offset..],
                    TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(size.x * 4),
                        rows_per_image: None,
                    },
                    Extent3d {
                        width: width as u32,
                        height: height as u32,
                        depth_or_array_layers: 1,
                    },
                );
            }
        } else {
            let texture = device.create_texture_with_data(
                &queue,
                &image.texture_descriptor,
                bevy::render::render_resource::TextureDataOrder::default(),
                data,
            );
            let texture_view = texture.create_view(&TextureViewDescriptor::default());
            let ImageSampler::Descriptor(sampler) = &image.sampler else {
                unreachable!("streamed images require an explicit nearest sampler")
            };
            gpu_images.insert(
                id,
                GpuImage {
                    texture,
                    texture_view,
                    texture_descriptor: image.texture_descriptor,
                    texture_view_descriptor: image.texture_view_descriptor,
                    had_data: true,
                    sampler: device.create_sampler(&sampler.as_wgpu()),
                },
            );
        }
        resident.0.insert(
            id,
            (size, format, image.data.expect("streamed images retain CPU pixels")),
        );
    }
}

mod changed_region;
use changed_region::changed_rect;

#[cfg(test)]
mod gpu_tests {
    use super::*;

    #[test]
    #[ignore = "requires a native GPU; renders offscreen without a window"]
    fn linear_phase_mask_keeps_gpu_storage_and_exact_updated_bytes() {
        use bevy::{app::PluginsState, window::{WindowPlugin, ExitCondition}};
        use bevy::render::render_resource::{BufferDescriptor, BufferUsages,
            CommandEncoderDescriptor, MapMode, PollType, TexelCopyBufferInfo};
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None, exit_condition: ExitCondition::DontExit, ..default()
        })).add_plugins(crate::VisualWorldRenderPlugin);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.plugins_state() != PluginsState::Ready {
            assert!(std::time::Instant::now() < deadline, "GPU initialization timed out");
            bevy::tasks::tick_global_task_pools_on_main_thread();
            std::thread::yield_now();
        }
        app.finish(); app.cleanup();
        let mut image = Image::new(Extent3d { width: 8, height: 8, depth_or_array_layers: 1 },
            TextureDimension::D2, [128, 64, 32, 255].repeat(64),
            TextureFormat::Rgba8Unorm, RenderAssetUsages::default());
        stream_composed_image(&mut image);
        let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        for _ in 0..4 { app.update(); }
        let (texture_id, view_id) = {
            let gpu = app.sub_app(RenderApp).world().resource::<RenderAssets<GpuImage>>().get(&handle).unwrap();
            assert_eq!(gpu.texture_descriptor.format, TextureFormat::Rgba8Unorm);
            (gpu.texture.id(), gpu.texture_view.id())
        };
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&handle).unwrap()
            .data.as_mut().unwrap()[2] = 96;
        for _ in 0..2 { app.update(); }
        let render = app.sub_app(RenderApp).world();
        let gpu = render.resource::<RenderAssets<GpuImage>>().get(&handle).unwrap();
        assert_eq!(gpu.texture.id(), texture_id, "phase changes must not allocate another texture");
        assert_eq!(gpu.texture_view.id(), view_id, "bound materials must retain their existing view");
        let device = render.resource::<RenderDevice>();
        let queue = render.resource::<RenderQueue>();
        let readback = device.create_buffer(&BufferDescriptor { label: Some("phase-mask-test"),
            size: 256 * 8, usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(TexelCopyTextureInfo { texture: &gpu.texture,
            mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All },
            TexelCopyBufferInfo { buffer: &readback, layout: TexelCopyBufferLayout {
                offset: 0, bytes_per_row: Some(256), rows_per_image: None } },
            Extent3d { width: 8, height: 8, depth_or_array_layers: 1 });
        queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::channel();
        readback.slice(..).map_async(MapMode::Read, move |result| { sender.send(result).unwrap(); });
        device.poll(PollType::wait_indefinitely()).unwrap();
        receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap().unwrap();
        {
            let bytes = readback.slice(..).get_mapped_range().unwrap();
            let images = app.world().resource::<Assets<Image>>();
            let expected = images.get(&handle).unwrap().data.as_ref().unwrap();
            for row in 0..8 { assert_eq!(&bytes[row * 256..row * 256 + 32], &expected[row * 32..row * 32 + 32]); }
        }
        readback.unmap();
        // A true format replacement must still create a matching GPU image,
        // even when dimensions and every source byte are identical.
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&handle).unwrap()
            .texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
        for _ in 0..2 { app.update(); }
        let gpu = app.sub_app(RenderApp).world().resource::<RenderAssets<GpuImage>>().get(&handle).unwrap();
        assert_ne!(gpu.texture.id(), texture_id);
        assert_eq!(gpu.texture_descriptor.format, TextureFormat::Rgba8UnormSrgb);
    }
}
