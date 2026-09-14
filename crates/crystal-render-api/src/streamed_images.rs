//! Retain GPU storage for composed RGBA surfaces and upload their changed region.
use bevy::{
    asset::{AssetEvent, AssetId, Assets},
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
        render_asset::{RenderAssetUsages, RenderAssets, prepare_assets},
        render_resource::{
            Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect, TextureDimension,
            TextureFormat, TextureViewDescriptor,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{GpuImage, ImageSampler},
    },
};
use std::collections::{HashMap, HashSet};

const LABEL: &str = "crystal-streamed-rgba";

/// Opt a CPU-readable, nearest-sampled RGBA surface into retained GPU uploads.
/// Ordinary source artwork continues through Bevy's standard image pipeline.
pub fn stream_composed_image(image: &mut Image) {
    assert_eq!(image.texture_descriptor.dimension, TextureDimension::D2);
    assert_eq!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(image.texture_descriptor.size.depth_or_array_layers, 1);
    assert_eq!(image.texture_descriptor.mip_level_count, 1);
    assert_eq!(image.texture_descriptor.sample_count, 1);
    image.texture_descriptor.label = Some(LABEL);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD;
    image.sampler = ImageSampler::nearest();
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
                .in_set(RenderSet::PrepareAssets)
                .before(prepare_assets::<GpuImage>),
        );
}

#[derive(Resource, Default)]
struct PendingImages {
    changed: Vec<(AssetId<Image>, Image)>,
    removed: HashSet<AssetId<Image>>,
}
#[derive(Resource, Default)]
struct ResidentImages(HashMap<AssetId<Image>, (UVec2, Vec<u8>)>);

fn extract_images(
    mut events: Extract<EventReader<AssetEvent<Image>>>,
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
            if image.texture_descriptor.label == Some(LABEL) {
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
        assert_eq!(image.data.len(), size.x as usize * size.y as usize * 4);
        if let Some((old_size, old_data)) = resident.0.get(&id)
            && *old_size == size
            && let Some(gpu) = gpu_images.get(id)
        {
            if let Some([x, y, width, height]) =
                changed_rect(old_data, &image.data, size.x as usize)
            {
                let offset = (y * size.x as usize + x) * 4;
                queue.write_texture(
                    ImageCopyTexture {
                        texture: &gpu.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: x as u32,
                            y: y as u32,
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    &image.data[offset..],
                    ImageDataLayout {
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
                &image.data,
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
                    texture_format: image.texture_descriptor.format,
                    sampler: device.create_sampler(&sampler.as_wgpu()),
                    size,
                    mip_level_count: 1,
                },
            );
        }
        resident.0.insert(id, (size, image.data));
    }
}

mod changed_region;
use changed_region::changed_rect;
