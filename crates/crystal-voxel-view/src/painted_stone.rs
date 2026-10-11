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
    let pixels = image::load_from_memory(bytes)
        .expect("embedded original sandstone must decode")
        .into_rgba8();
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
    image
}

#[cfg(test)]
mod tests {
    use super::*;
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
