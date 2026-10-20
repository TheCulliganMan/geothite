//! Original painted outdoor material, embedded so native and browser builds
//! use the same asset without requiring changes to an external content pack.
use bevy::{
    asset::RenderAssetUsages,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
};

pub(super) fn image() -> Image {
    decode(include_bytes!(
        "../../../assets/terrain/painted-sandstone.png"
    ))
}

pub(super) fn bedrock_image() -> Image {
    decode(include_bytes!(
        "../../../assets/terrain/painted-bedrock.png"
    ))
}

/// Owned CPU results, moved into Assets exactly once. No global decoded copy.
#[cfg(not(target_arch = "wasm32"))]
pub(super) struct PreparedImages {
    pub stone: Image,
    pub bedrock: Image,
}
#[cfg(not(target_arch = "wasm32"))]
impl PreparedImages {
    pub(super) fn decode() -> Self {
        let _trace = crate::loading_trace::span("region.prepare_painted_assets", || serde_json::json!({}));
        Self { stone: image(), bedrock: bedrock_image() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Default)]
pub(super) struct NativePreparation {
    shared: std::sync::Arc<std::sync::Mutex<Preparation>>,
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Preparation {
    task: Option<bevy::tasks::Task<PreparedImages>>,
    images: Option<(bevy::prelude::Handle<Image>, bevy::prelude::Handle<Image>)>,
}
#[cfg(not(target_arch = "wasm32"))]
impl NativePreparation {
    pub fn start(&mut self) {
        if std::env::var("CRYSTAL_PAINTED_IMAGE_WARMUP")
            .is_ok_and(|value| matches!(value.as_str(), "0" | "off" | "false")) { return; }
        let mut shared = self.shared.lock().expect("painted image preparation");
        if shared.task.is_some() || shared.images.is_some() { return; }
        let Some(pool) = bevy::tasks::AsyncComputeTaskPool::try_get() else { return; };
        shared.task = Some(pool.spawn(async { PreparedImages::decode() }));
    }
    pub fn started(&self) -> bool {
        let shared = self.shared.lock().expect("painted image preparation");
        shared.task.is_some() || shared.images.is_some()
    }
    #[cfg(test)]
    pub fn pending(&self) -> bool {
        self.shared.lock().expect("painted image preparation").task.is_some()
    }
    /// All scenes receive the same handles. The decoded bytes enter Assets
    /// once, even if the hidden destination reaches this point first.
    pub fn poll(&mut self, images: &mut bevy::prelude::Assets<Image>)
        -> Option<(bevy::prelude::Handle<Image>, bevy::prelude::Handle<Image>)> {
        use bevy::tasks::futures_lite::future;
        let mut shared = self.shared.lock().expect("painted image preparation");
        if let Some(handles) = &shared.images { return Some(handles.clone()); }
        let result = shared.task.as_mut().and_then(|task|
            future::block_on(future::poll_once(task)));
        let prepared = result?;
        shared.task = None;
        let handles = (
            crate::add_terrain_image(images, prepared.stone, "painted_stone"),
            crate::add_terrain_image(images, prepared.bedrock, "painted_bedrock"),
        );
        shared.images = Some(handles.clone());
        Some(handles)
    }
    pub fn cancel(&mut self) {
        // Detach this scene, preserving work still owned by another scene.
        if self.started() { *self = Self::default(); }
    }
    #[cfg(test)]
    pub(super) fn from_task(task: bevy::tasks::Task<PreparedImages>) -> Self {
        Self { shared: std::sync::Arc::new(std::sync::Mutex::new(Preparation {
            task: Some(task), images: None,
        })) }
    }
}

/// Relative mineral detail for vertex-painted moving cave stones. Ice keeps
/// the same fractures with neutral pigment beneath its blue frost colors.
pub(super) fn movable_bedrock_image(frozen: bool) -> Image {
    let mut image = bedrock_image();
    let linear = |v: u8| {
        let v=v as f32/255.;
        if v<=0.04045 {v/12.92} else {((v+0.055)/1.055).powf(2.4)}
    };
    let srgb = |v: f32| {
        let v=v.clamp(0.,1.);
        let v=if v<=0.0031308 {v*12.92} else {1.055*v.powf(1./2.4)-0.055};
        (v*255.).round() as u8
    };
    for p in image.data.as_mut().expect("CPU image pixels").chunks_exact_mut(4) {
        let ratio=[linear(p[0])/0.25, linear(p[1])/0.21, linear(p[2])/0.16];
        let grey=ratio[0]*0.2126+ratio[1]*0.7152+ratio[2]*0.0722;
        for i in 0..3 { p[i]=srgb(0.55+0.35*if frozen {grey} else {ratio[i]}); }
    }
    image
}

fn decode(bytes: &[u8]) -> Image {
    let pixels = {
        let _trace = crate::loading_trace::span("region.decode_painted_png", || serde_json::json!({"encoded_bytes":bytes.len()}));
        image::load_from_memory(bytes)
            .expect("embedded original sandstone must decode")
            .into_rgba8()
    };
    let _trace = crate::loading_trace::span("region.prepare_painted_image", || serde_json::json!({"width":pixels.width(),"height":pixels.height()}));
    let mut image = Image::new(
        Extent3d {
            width: pixels.width(),
            height: pixels.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    if crate::render_quality::requested().materials() {
        crate::texture_filtering::mipmap(&mut image, None);
    }
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn background_painted_images_match_exact_decode_and_transfer_once() {
        let pool = bevy::tasks::TaskPoolBuilder::new().num_threads(1).build();
        let mut preparation = NativePreparation::from_task(pool.spawn(async { PreparedImages::decode() }));
        let mut destination = preparation.clone();
        let mut assets = bevy::prelude::Assets::<Image>::default();
        let deadline = std::time::Instant::now()+std::time::Duration::from_secs(10);
        let handles = loop {
            if let Some(images) = destination.poll(&mut assets) { break images; }
            assert!(std::time::Instant::now()<deadline);
            std::thread::yield_now();
        };
        for (handle,direct) in [(&handles.0,image()), (&handles.1,bedrock_image())] {
            let background = assets.get(handle).unwrap();
            assert_eq!(background.data,direct.data);
            assert_eq!(format!("{:?}",background.texture_descriptor),format!("{:?}",direct.texture_descriptor));
            assert_eq!(format!("{:?}",background.sampler),format!("{:?}",direct.sampler));
            assert_eq!(background.asset_usage,direct.asset_usage);
        }
        assert!(!preparation.pending());
        assert_eq!(preparation.poll(&mut assets),Some(handles.clone()));
        assert_eq!(destination.poll(&mut assets),Some(handles));
        assert_eq!(assets.len(),2,"both scenes must reuse the same GPU image assets");
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn canceling_painted_preparation_drops_pending_app_owned_work() {
        use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
        struct Dropped(Arc<AtomicBool>);
        impl Drop for Dropped { fn drop(&mut self) { self.0.store(true,Ordering::SeqCst); } }
        let pool = bevy::tasks::TaskPoolBuilder::new().num_threads(1).build();
        let dropped = Arc::new(AtomicBool::new(false));
        let witness = Dropped(dropped.clone());
        let mut preparation = NativePreparation::from_task(pool.spawn(async move {
            let _witness = witness;
            bevy::tasks::futures_lite::future::pending::<PreparedImages>().await
        }));
        preparation.cancel();
        let deadline = std::time::Instant::now()+std::time::Duration::from_secs(5);
        while !dropped.load(Ordering::SeqCst) {
            assert!(std::time::Instant::now()<deadline);
            std::thread::yield_now();
        }
        assert!(preparation.poll(&mut bevy::prelude::Assets::default()).is_none());
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn canceling_one_scene_preserves_the_other_scenes_pending_images() {
        use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
        let runs=Arc::new(AtomicUsize::new(0));
        let worker_runs=runs.clone();
        let pool=bevy::tasks::TaskPoolBuilder::new().num_threads(1).build();
        let mut foreground=NativePreparation::from_task(pool.spawn(async move {
            worker_runs.fetch_add(1,Ordering::SeqCst);
            PreparedImages {stone:Image::default(),bedrock:Image::default()}
        }));
        let mut destination=foreground.clone();
        foreground.cancel();
        assert!(!foreground.started());
        destination.start();
        let mut assets=bevy::prelude::Assets::default();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
        let handles=loop {
            if let Some(handles)=destination.poll(&mut assets) {break handles;}
            assert!(std::time::Instant::now()<deadline);
            std::thread::yield_now();
        };
        assert_eq!(runs.load(Ordering::SeqCst),1);
        assert_eq!(destination.poll(&mut assets),Some(handles));
        assert_eq!(assets.len(),2);
    }
    #[test]
    fn frozen_moving_stone_detail_is_neutral_opaque_and_varied() {
        let image=movable_bedrock_image(true);
        assert!(image.data.as_ref().expect("CPU image pixels").chunks_exact(4).all(|p| p[0]==p[1] && p[1]==p[2] && p[3]==255));
        let min=image.data.as_ref().expect("CPU image pixels").chunks_exact(4).map(|p|p[0]).min().unwrap();
        let max=image.data.as_ref().expect("CPU image pixels").chunks_exact(4).map(|p|p[0]).max().unwrap();
        assert!(max-min>30);
    }
    #[test]
    fn embedded_painted_material_is_opaque_srgb_with_repeat_sampling() {
        for image in [image(), bedrock_image()] {
            assert_eq!(
                image.texture_descriptor.format,
                TextureFormat::Rgba8UnormSrgb
            );
            assert_eq!(image.width(), image.height());
            assert!(image.width() >= 1024);
            assert!(image.data.as_ref().expect("CPU image pixels").chunks_exact(4).all(|p| p[3] == 255));
            let ImageSampler::Descriptor(sampler) = image.sampler else {
                panic!("explicit repeat sampler");
            };
            assert!(matches!(sampler.address_mode_u, ImageAddressMode::Repeat));
            assert!(matches!(sampler.address_mode_v, ImageAddressMode::Repeat));
        }
    }
}
