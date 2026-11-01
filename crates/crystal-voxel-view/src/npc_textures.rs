//! External generated NPC UV paintings. Pixels never enter the tracked catalog.
use std::{collections::BTreeMap, sync::{Mutex, OnceLock}};

pub(crate) struct Texture {
    pub size: (u32, u32),
    pub rgba: Vec<u8>,
}
pub(crate) struct CachedTexture {
    png: Mutex<Option<Vec<u8>>>,
    decoded: OnceLock<Texture>,
}
impl CachedTexture {
    pub(crate) fn validate(bytes: Vec<u8>) -> Result<Self, String> {
        // Validate every painting before publishing the catalog, but retain
        // its compact source until an actor actually needs the pixels.
        decode(&bytes)?;
        Ok(Self { png: Mutex::new(Some(bytes)), decoded: OnceLock::new() })
    }
    fn get(&self) -> &Texture {
        self.decoded.get_or_init(|| {
            let bytes = self.png.lock().unwrap().take().expect("validated painting source");
            // These bytes are immutable and were already fully validated.
            decode(&bytes).expect("validated painting decodes identically")
        })
    }
    #[cfg(test)]
    fn loaded(texture: Texture) -> Self {
        Self { png: Mutex::new(None), decoded: OnceLock::from(texture) }
    }
}
static TEXTURES: OnceLock<BTreeMap<String, CachedTexture>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static NATIVE_TEXTURES: OnceLock<BTreeMap<String, (std::path::PathBuf, OnceLock<Texture>)>> = OnceLock::new();

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

pub(crate) fn install(textures: BTreeMap<String, CachedTexture>) -> Result<(), String> {
    require_complete(&textures)?;
    TEXTURES.set(textures).map_err(|_| "NPC textures frozen; restart to refresh".into())
}
pub(crate) fn require_complete<T>(textures: &BTreeMap<String, T>) -> Result<(), String> {
    let missing: Vec<_> = crate::open_people::npc_texture_names().into_iter()
        .filter(|name| !textures.contains_key(*name)).collect();
    if missing.is_empty() { Ok(()) } else {
        Err(format!("required character paintings missing: {}", missing.join(", ")))
    }
}
pub(crate) fn frozen() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    if NATIVE_TEXTURES.get().is_some() { return true; }
    TEXTURES.get().is_some()
}
pub(crate) fn get(name: &str) -> Option<&'static Texture> {
    if let Some(textures) = TEXTURES.get() { return textures.get(name).map(CachedTexture::get); }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let catalog = NATIVE_TEXTURES.get_or_init(|| {
            let root = crate::open_people::model_root();
            crate::open_people::npc_texture_names().into_iter().map(|name| {
                let path = std::path::PathBuf::from(&root).join(format!("people/npc-textures/{name}.png"));
                // Verify the complete roster without decoding/resampling every
                // unused portrait on the first visible character's frame.
                std::fs::File::open(&path)
                    .unwrap_or_else(|error| panic!("read character painting {}: {error}", path.display()));
                (name.to_owned(), (path, OnceLock::new()))
            }).collect()
        });
        return catalog.get(name).map(|(path, texture)| texture.get_or_init(|| load_native(path)));
    }
    #[cfg(target_arch = "wasm32")]
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native(path: &std::path::Path) -> Texture {
    use std::io::Read;
    let _trace = crate::loading_trace::span("actor.load_painting", || serde_json::json!({"path":path}));
    let file = std::fs::File::open(path)
        .unwrap_or_else(|error| panic!("read character painting {}: {error}", path.display()));
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)
        .unwrap_or_else(|error| panic!("read character painting {}: {error}", path.display()));
    // The original decoder, resampling filter and validation remain exact.
    // Corrupt/incompatible used artwork fails explicitly; never substitute it.
    decode(&bytes).unwrap_or_else(|error| panic!("invalid character painting {}: {error}", path.display()))
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
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_catalog_only_decodes_requested_paintings() {
        const CHILD: &str = "CRYSTAL_NATIVE_PAINTING_TEST_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let names = crate::open_people::npc_texture_names();
            let requested = names[0];
            let expected = decode(&std::fs::read(crate::open_people::model_root()
                .join(format!("people/npc-textures/{requested}.png"))).unwrap()).unwrap();
            let actual = get(requested).unwrap();
            assert_eq!(expected.size, actual.size);
            assert_eq!(expected.rgba, actual.rgba);
            assert!(std::ptr::eq(actual, get(requested).unwrap()));
            let catalog = NATIVE_TEXTURES.get().unwrap();
            assert_eq!(catalog.values().filter(|(_, cached)| cached.get().is_some()).count(), 1);
            assert!(frozen());
            if std::env::var(CHILD).unwrap() == "corrupt" {
                let _ = get(names[1]);
                panic!("corrupt requested painting was accepted");
            }
            return;
        }
        let root = std::env::temp_dir().join(format!("geothite-painting-test-{}", std::process::id()));
        assert!(!root.exists());
        let directory = root.join("people/npc-textures");
        std::fs::create_dir_all(&directory).unwrap();
        let names = crate::open_people::npc_texture_names();
        let valid = png(128, 128);
        for (i, name) in names.iter().enumerate() {
            std::fs::write(directory.join(format!("{name}.png")),
                if i == 0 {valid.as_slice()} else {b"unused invalid painting"}).unwrap();
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "npc_textures::tests::native_catalog_only_decodes_requested_paintings", "--nocapture"])
            .env(CHILD, "1").env("CRYSTAL_OPEN_MODEL_ROOT", &root).output().unwrap();
        let corrupt = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "npc_textures::tests::native_catalog_only_decodes_requested_paintings", "--nocapture"])
            .env(CHILD, "corrupt").env("CRYSTAL_OPEN_MODEL_ROOT", &root).output().unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(!corrupt.status.success());
        assert!(String::from_utf8_lossy(&corrupt.stderr).contains("invalid character painting"));
    }
    #[test]
    fn incomplete_roster_cannot_install_and_full_roster_is_required_by_identity() {
        let mut textures: BTreeMap<_, _> = crate::open_people::npc_texture_names().into_iter()
            .map(|name| (name.to_owned(), Texture { size: (1,1), rgba: vec![0,0,0,255] })).collect();
        assert!(require_complete(&textures).is_ok());
        let missing = textures.remove("will").unwrap();
        textures.insert("unrecognized_role".into(), missing);
        assert_eq!(require_complete(&textures).unwrap_err(), "required character paintings missing: will");
        // Invalid input must fail before publishing the one-time cache.
        assert!(install(textures.into_iter().map(|(name, texture)|
            (name, CachedTexture::loaded(texture))).collect()).unwrap_err().contains("will"));
    }
    fn png(width: u32, height: u32) -> Vec<u8> {
        use image::ImageEncoder;
        let pixels = [42, 91, 173, 255].repeat((width * height) as usize);
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(&pixels, width, height, image::ExtendedColorType::Rgba8).unwrap();
        bytes
    }
    #[test]
    fn validated_painting_decodes_only_on_use_and_keeps_every_pixel() {
        let bytes = png(128, 128);
        let expected = decode(&bytes).unwrap();
        let cached = CachedTexture::validate(bytes.clone()).unwrap();
        assert!(cached.decoded.get().is_none());
        assert_eq!(cached.png.lock().unwrap().as_ref().unwrap(), &bytes);
        let actual = cached.get();
        assert_eq!(actual.size, expected.size);
        assert_eq!(actual.rgba, expected.rgba);
        assert!(cached.png.lock().unwrap().is_none(), "release redundant encoded bytes");
        assert!(std::ptr::eq(actual, cached.get()), "reuse the decoded painting");
        assert!(CachedTexture::validate(b"invalid PNG".to_vec()).is_err());
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
