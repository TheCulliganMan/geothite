use crate::live_profiles::{Document, Mask};

// Profile fields are intentionally converted here instead of broadening the
// editor's private document into a public serialized application contract.
pub(super) fn capture(document: &Document) -> serde_json::Value {
    fn mask(mask: Mask) -> &'static str {
        match mask {
            Mask::None => "none",
            Mask::Ground => "ground",
        }
    }
    serde_json::json!({
        "objects": document.objects.iter().map(|object| serde_json::json!({
            "name": object.name, "tileset": object.tileset, "map": object.map, "maps": object.maps,
            "metatile": object.metatile, "metatiles": object.metatiles, "origin": object.origin,
            "tiles": object.tiles, "ground": object.ground, "top_pixels": object.top_pixels,
            "depth_pixels": object.depth_pixels, "footing_pixels": object.footing_pixels,
            "mask": mask(object.mask),
            "parts": object.parts.iter().map(|part| serde_json::json!({
                "rect": part.rect, "top_pixels": part.top_pixels, "depth_pixels": part.depth_pixels,
                "base_pixels": part.base_pixels, "height_pixels": part.height_pixels,
                "bevel_pixels": part.bevel_pixels, "offset_pixels": part.offset_pixels, "mask": mask(part.mask)
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "atmosphere": document.atmosphere.as_ref().map(|a| serde_json::json!({
            "maps": a.maps, "start_tiles": a.start_tiles, "end_tiles": a.end_tiles, "opacity": a.opacity
        }))
    })
}

pub(super) fn restore(value: serde_json::Value) -> Result<Document, String> {
    // Apply the authoritative editor validator again after message decoding.
    // This is small profile metadata, never another game/model content load.
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    Document::parse(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shipped_profiles_roundtrip_without_default_or_edit_loss() {
        let mut document: Document = serde_json::from_slice(include_bytes!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .unwrap();
        assert!(!document.objects.is_empty());
        document.objects[0].depth_pixels = f32::from_bits(0x3f123456);
        document.objects[0].footing_pixels = Some(-0.);
        let captured = capture(&document);
        let decoded = restore(captured.clone()).unwrap();
        assert_eq!(capture(&decoded), captured);
        assert_eq!(
            decoded.objects[0].depth_pixels.to_bits(),
            document.objects[0].depth_pixels.to_bits()
        );
        assert_eq!(
            decoded.objects[0].footing_pixels.unwrap().to_bits(),
            (-0.0f32).to_bits()
        );
    }
}
