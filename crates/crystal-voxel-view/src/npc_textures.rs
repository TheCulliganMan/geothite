//! External generated NPC UV paintings. Pixels never enter the tracked catalog.
use std::{collections::BTreeMap, sync::OnceLock};

pub(crate) struct Texture {
    pub size: (u32, u32),
    pub rgba: Vec<u8>,
}
static TEXTURES: OnceLock<BTreeMap<String, Texture>> = OnceLock::new();

pub(crate) fn decode(bytes: &[u8]) -> Result<Texture, String> {
    if bytes.len() > 4 * 1024 * 1024 { return Err("NPC texture exceeds 4 MiB".into()); }
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(2048); limits.max_image_height = Some(2048);
    limits.max_alloc = Some(16 * 1024 * 1024); reader.limits(limits);
    let image = reader.decode().map_err(|e| e.to_string())?;
    if image.width() != image.height() || image.width() < 128 {
        return Err("NPC UV painting must be square and at least 128px".into());
    }
    let image = image.resize_exact(512, 512, image::imageops::FilterType::Lanczos3).into_rgba8();
    Ok(Texture { size: (512, 512), rgba: image.into_raw() })
}

pub(crate) fn install(textures: BTreeMap<String, Texture>) -> Result<(), String> {
    TEXTURES.set(textures).map_err(|_| "NPC textures frozen; restart to refresh".into())
}
pub(crate) fn frozen() -> bool { TEXTURES.get().is_some() }
pub(crate) fn get(name: &str) -> Option<&'static Texture> {
    TEXTURES.get_or_init(|| {
        #[allow(unused_mut)]
        let mut out = BTreeMap::new();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(root) = std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT") {
            use std::io::Read;
            for name in crate::open_people::npc_texture_names() {
                let path = std::path::PathBuf::from(&root).join(format!("people/npc-textures/{name}.png"));
                let Ok(file) = std::fs::File::open(path) else { continue; };
                let mut bytes = Vec::new();
                if file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes).is_ok() {
                    if let Ok(texture) = decode(&bytes) { out.insert(name.into(), texture); }
                }
            }
        }
        out
    }).get(name)
}

/// Normalize resolution for upload/bundling without altering UV coordinates.
pub fn normalize_npc_texture(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    let texture = decode(bytes)?;
    let mut output = Vec::new();
    image::codecs::png::PngEncoder::new_with_quality(&mut output,
        image::codecs::png::CompressionType::Best, image::codecs::png::FilterType::Adaptive)
        .write_image(&texture.rgba, texture.size.0, texture.size.1, image::ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn png(width: u32, height: u32) -> Vec<u8> {
        use image::ImageEncoder;
        let pixels = [42, 91, 173, 255].repeat((width * height) as usize);
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(&pixels, width, height, image::ExtendedColorType::Rgba8).unwrap();
        bytes
    }
    #[test]
    fn rejects_invalid_and_incompatible_uv_paintings() {
        assert!(decode(b"not a PNG").is_err());
        assert!(decode(&vec![0; 4 * 1024 * 1024 + 1]).is_err());
        assert!(decode(&png(128, 256)).is_err());
        assert!(decode(&png(64, 64)).is_err());
        assert!(decode(&png(2049, 2049)).is_err());
    }
    #[test]
    fn normalizes_square_paintings_without_changing_surface_color() {
        let bytes = normalize_npc_texture(&png(128, 128)).unwrap();
        let texture = decode(&bytes).unwrap();
        assert_eq!(texture.size, (512, 512));
        assert!(texture.rgba.chunks_exact(4).all(|p| p == [42, 91, 173, 255]));
    }
}
