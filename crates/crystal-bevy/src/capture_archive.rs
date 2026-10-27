//! Immutable originals and encoded outputs for explicitly requested captures.
//!
//! Latest filenames remain compatible with existing capture tools. Every old
//! file is archived byte-for-byte before replacement, and every new encoding is
//! archived before it is installed. Archives live beside the output under
//! `.render-archive/<output filename>/` and are never rewritten or deleted.
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use image::{DynamicImage, ImageEncoder};
use sha2::{Digest, Sha256};

/// Retains exact existing output bytes, without decoding or rewriting the file.
///
/// Returns `None` if there is no existing output. Other read errors, archive
/// creation/write failures or an inconsistent existing immutable file are
/// returned; callers must not replace the requested output after an error.
pub fn archive_existing(output: &Path) -> Result<Option<PathBuf>> {
    match fs::read(output) {
        Ok(bytes) => archive_bytes(output, "encoded", extension(output), &bytes).map(Some),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => {
            Err(error).with_context(|| format!("read previous capture {}", output.display()))
        }
    }
}

/// Archives old and new exact encoded bytes before replacing the latest file.
///
/// No image conversion occurs. This also supports a caller's explicit PNG
/// encoder settings. Returns the immutable copy of the new bytes. If archiving
/// fails the old output remains untouched. A later latest-file write failure is
/// returned with both complete versions already retained in the archive.
///
/// # Errors
/// Returns read/write/path errors or an immutable archive content mismatch.
pub fn write_capture_bytes(output: &Path, bytes: &[u8]) -> Result<PathBuf> {
    archive_existing(output)?;
    let archived = archive_bytes(output, "encoded", extension(output), bytes)?;
    fs::write(output, bytes)
        .with_context(|| format!("write latest capture {}", output.display()))?;
    Ok(archived)
}

/// Archives a full-resolution image in its supplied mode without changing latest.
///
/// Call before resizing or converting pixels. `stage` identifies this original
/// or derived artifact (ASCII letters, digits, hyphen or underscore only).
/// This does not replace any existing image, including rejected review passes.
///
/// # Errors
/// Returns PNG encoding, invalid stage, I/O or archive content mismatch errors.
pub fn archive_capture_image(output: &Path, stage: &str, image: &DynamicImage) -> Result<PathBuf> {
    let encoded = encode_png(image)?;
    archive_bytes(output, stage, "png", &encoded)
}

/// Saves a PNG and keeps the exact prior and new encodings immutably.
///
/// The supplied image's dimensions/color mode are preserved by the PNG encoder.
/// Use `archive_capture_image` first when this image is a conversion or resize.
///
/// # Errors
/// Returns PNG encoding, I/O or immutable archive content mismatch errors.
pub fn write_capture_png(output: &Path, image: &DynamicImage) -> Result<PathBuf> {
    // Preserve prior evidence even when encoding this new frame is rejected.
    archive_existing(output)?;
    let encoded = encode_png(image)?;
    write_capture_bytes(output, &encoded)
}

/// Retains exact GPU readback bytes and their format/extent before conversion.
///
/// The JSON descriptor names its immutable binary sibling, allowing the native
/// color format, alpha, dimensions and mip/layer order to be reused later.
/// This writes no game catalog or source asset; it only retains the explicitly
/// requested rendered frame. Both binary and descriptor must persist before the
/// caller proceeds to decode, resize or write the latest screenshot.
///
/// # Errors
/// Returns an error for absent CPU readback data, encoding, I/O or an immutable
/// content mismatch. A partially completed archive never authorizes overwrite.
#[cfg(feature = "bevy-shell")]
pub fn archive_gpu_readback(output: &Path, image: &bevy::prelude::Image) -> Result<PathBuf> {
    let bytes = image
        .data
        .as_deref()
        .context("GPU capture has no CPU readback bytes to retain")?;
    let raw = archive_bytes(output, "gpu-readback", "bin", bytes)?;
    let descriptor = serde_json::to_vec_pretty(&serde_json::json!({
        "raw_file":raw.file_name().and_then(|name| name.to_str()).context("raw readback archive filename")?,
        "format":format!("{:?}",image.texture_descriptor.format),
        "dimension":format!("{:?}",image.texture_descriptor.dimension),
        "width":image.texture_descriptor.size.width,
        "height":image.texture_descriptor.size.height,
        "depth_or_array_layers":image.texture_descriptor.size.depth_or_array_layers,
        "mip_level_count":image.texture_descriptor.mip_level_count,
        "sample_count":image.texture_descriptor.sample_count,
        "data_order":format!("{:?}",image.data_order),
        "byte_length":bytes.len(),
        "semantics":"Exact native screenshot readback before color conversion or resizing"
    }))?;
    archive_bytes(output, "gpu-readback", "json", &descriptor)
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    // DynamicImage::write_to may silently convert unsupported color types.
    // The direct encoder either preserves this actual mode or returns an error.
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(
            image.as_bytes(),
            image.width(),
            image.height(),
            image.color().into(),
        )
        .context("encode capture PNG without color conversion")?;
    Ok(bytes)
}

fn extension(output: &Path) -> &str {
    output
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("bin")
}
fn archive_bytes(output: &Path, stage: &str, extension: &str, bytes: &[u8]) -> Result<PathBuf> {
    ensure!(
        !stage.is_empty()
            && stage
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
        "capture archive stage must contain only ASCII letters, digits, hyphen or underscore"
    );
    ensure!(
        !extension.is_empty() && extension.bytes().all(|byte| byte.is_ascii_alphanumeric()),
        "capture archive extension must be ASCII alphanumeric"
    );
    let filename = output
        .file_name()
        .context("capture output needs a filename")?;
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let directory = parent.join(".render-archive").join(filename);
    fs::create_dir_all(&directory)
        .with_context(|| format!("create capture archive {}", directory.display()))?;
    let digest = format!("{:x}", Sha256::digest(bytes));
    let path = directory.join(format!("{stage}-{digest}.{extension}"));
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(bytes)
                .with_context(|| format!("retain capture {}", path.display()))?;
            file.flush()
                .with_context(|| format!("flush capture archive {}", path.display()))?;
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            ensure!(
                fs::read(&path)
                    .with_context(|| format!("verify capture archive {}", path.display()))?
                    == bytes,
                "immutable capture archive differs from its hash: {}",
                path.display()
            );
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("create immutable capture {}", path.display()));
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use image::{Rgba, RgbaImage};

    use super::*;

    fn fixture_output(name: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "geothite-capture-archive-{}-{time}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).expect("test output directory");
        // Tiny synthetic I/O fixtures remain available for inspection. No real
        // render, archive or image is deleted by this test helper.
        directory.join(name)
    }
    fn image(alpha: u8) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(3, 2, Rgba([11, 37, 93, alpha])))
    }

    #[test]
    fn retains_exact_old_bytes_and_every_new_full_resolution_png() {
        let output = fixture_output("latest.png");
        let original = b"previous encoding preserved without decoding";
        fs::write(&output, original).expect("existing output fixture");
        let first = write_capture_png(&output, &image(27)).expect("first archived capture");
        let first_bytes = fs::read(&output).expect("first latest bytes");
        let previous = archive_bytes(&output, "encoded", "png", original)
            .expect("exact previous bytes archive");
        assert_eq!(fs::read(previous).expect("old bytes"), original);
        assert_eq!(fs::read(&first).expect("first archive"), first_bytes);
        let second = write_capture_png(&output, &image(199)).expect("second archived capture");
        assert_ne!(first, second);
        assert_eq!(fs::read(&first).expect("retained first"), first_bytes);
        assert_eq!(
            fs::read(&second).expect("retained second"),
            fs::read(&output).expect("latest")
        );
        let current = image::open(&output).expect("latest PNG");
        assert_eq!((current.width(), current.height()), (3, 2));
        assert_eq!(current.to_rgba8().get_pixel(0, 0).0, [11, 37, 93, 199]);
    }

    #[test]
    fn retains_original_alpha_before_rgb_conversion_and_is_idempotent() {
        let output = fixture_output("alpha.png");
        let native = image(17);
        let original = archive_capture_image(&output, "original", &native).expect("original RGBA");
        let before = fs::read(&original).expect("original bytes");
        let converted = DynamicImage::ImageRgb8(native.to_rgb8());
        write_capture_png(&output, &converted).expect("compatible RGB latest");
        assert_eq!(
            archive_capture_image(&output, "original", &native).expect("same immutable version"),
            original
        );
        assert_eq!(fs::read(&original).expect("unchanged original"), before);
        assert_eq!(
            image::open(&original).expect("RGBA original").color(),
            image::ColorType::Rgba8
        );
        assert_eq!(
            image::open(&output).expect("RGB latest").color(),
            image::ColorType::Rgb8
        );
        assert_eq!(
            image::open(original)
                .expect("retained alpha")
                .to_rgba8()
                .get_pixel(0, 0)
                .0[3],
            17
        );
    }

    #[test]
    fn archive_failure_or_corruption_never_overwrites_latest() {
        let output = fixture_output("protected.png");
        fs::write(&output, b"must survive").expect("existing fixture");
        let directory = output.parent().expect("parent").join(".render-archive");
        fs::write(&directory, b"blocked archive directory").expect("I/O failure fixture");
        assert!(write_capture_bytes(&output, b"new capture").is_err());
        assert_eq!(
            fs::read(&output).expect("protected output"),
            b"must survive"
        );
        let rejected = fixture_output("rejected.png");
        let prior = b"exact prior encoding before rejected new color mode";
        fs::write(&rejected, prior).expect("prior rejected-frame fixture");
        let unsupported = DynamicImage::ImageRgb32F(image::Rgb32FImage::from_pixel(
            1,
            1,
            image::Rgb([0.1, 0.2, 0.3]),
        ));
        assert!(write_capture_png(&rejected, &unsupported).is_err());
        assert_eq!(
            fs::read(&rejected).expect("prior latest survives conversion failure"),
            prior
        );
        let prior_path = rejected
            .parent()
            .expect("parent")
            .join(".render-archive")
            .join("rejected.png")
            .join(format!("encoded-{:x}.png", Sha256::digest(prior)));
        assert_eq!(
            fs::read(prior_path).expect("prior bytes archived before rejected conversion"),
            prior
        );
        let other = fixture_output("corrupt.png");
        let immutable = write_capture_bytes(&other, b"first").expect("initial version");
        // Deliberate test-only corruption, never a production archive write.
        fs::write(&immutable, b"damaged archive fixture").expect("corruption fixture");
        assert!(write_capture_bytes(&other, b"second").is_err());
        assert_eq!(
            fs::read(&other).expect("latest survives corruption"),
            b"first"
        );
    }

    #[cfg(feature = "bevy-shell")]
    #[test]
    fn raw_gpu_readback_and_descriptor_preserve_native_bytes_before_conversion() {
        use bevy::asset::RenderAssetUsages;
        use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
        let output = fixture_output("gpu.png");
        let bytes = vec![1, 2, 3, 4, 9, 8, 7, 6];
        let native = bevy::prelude::Image::new(
            Extent3d {
                width: 2,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bytes.clone(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let descriptor = archive_gpu_readback(&output, &native).expect("native readback archive");
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(&descriptor).expect("descriptor bytes"))
                .expect("descriptor");
        assert_eq!(metadata["width"], 2);
        assert_eq!(metadata["height"], 1);
        assert_eq!(metadata["format"], "Rgba8UnormSrgb");
        assert_eq!(
            fs::read(
                descriptor
                    .parent()
                    .expect("archive directory")
                    .join(metadata["raw_file"].as_str().expect("raw filename"))
            )
            .expect("exact GPU bytes"),
            bytes
        );
        assert!(!output.exists(), "raw preservation must not replace latest");
    }
}
