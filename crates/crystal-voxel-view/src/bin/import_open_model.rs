//! Convert external static GLB/glTF scenery, preserving UVs and leaf cutouts.
//! Outputs are external content-pack inputs, never files to commit.
use base64::Engine as _;
use bevy::math::{Mat4, Vec3};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Arrangement {
    parts: Vec<Placement>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    model: PathBuf,
    #[serde(default)]
    translation: [f32; 3],
    #[serde(default = "unit_scale")]
    scale: [f32; 3],
}
fn unit_scale() -> [f32; 3] {
    [1.; 3]
}

#[derive(Serialize)]
struct Texture {
    width: u32,
    height: u32,
    rgba_base64: String,
    alpha_cutoff: Option<f32>,
    double_sided: bool,
    wrap_u: &'static str,
    wrap_v: &'static str,
}

#[derive(Serialize)]
struct Primitive {
    name: String,
    material: String,
    base_color: [f32; 4],
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    uvs: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    texture: Option<Texture>,
}

fn wrap(mode: gltf::texture::WrappingMode) -> &'static str {
    match mode {
        gltf::texture::WrappingMode::ClampToEdge => "clamp",
        gltf::texture::WrappingMode::MirroredRepeat => "mirror",
        gltf::texture::WrappingMode::Repeat => "repeat",
    }
}

fn visit(
    node: gltf::Node<'_>,
    parent: Mat4,
    buffers: &[Vec<u8>],
    folder: &Path,
    parts: &mut Vec<Primitive>,
) -> Result<(), Box<dyn Error>> {
    let matrix = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
    if !matrix.is_finite() || matrix.determinant().abs() < 1e-8 {
        return Err("singular node transform".into());
    }
    if node.skin().is_some() {
        return Err("solid scenery import does not support skins".into());
    }
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            if primitive.morph_targets().next().is_some()
                || primitive.get(&gltf::Semantic::Colors(0)).is_some()
            {
                return Err("morphs and vertex-painted assets need a richer import path".into());
            }
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err("expected triangle primitive".into());
            }
            let pbr = primitive.material().pbr_metallic_roughness();
            let material = primitive.material();
            if material.alpha_mode() != gltf::material::AlphaMode::Opaque
                && pbr.base_color_texture().is_none()
            {
                return Err("untextured translucent scenery is unsupported".into());
            }
            let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(Vec::as_slice));
            let texture = if let Some(info) = pbr.base_color_texture() {
                if info.tex_coord() != 0 {
                    return Err("only TEXCOORD_0 scenery is supported".into());
                }
                let image = info.texture().source();
                let bytes = match image.source() {
                    gltf::image::Source::Uri { uri, .. } => read_local(folder, uri)?,
                    gltf::image::Source::View { view, .. } => buffers
                        .get(view.buffer().index())
                        .and_then(|b| b.get(view.offset()..view.offset() + view.length()))
                        .ok_or("invalid image buffer view")?
                        .to_vec(),
                };
                let mut decoder =
                    image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
                let mut limits = image::Limits::default();
                limits.max_image_width = Some(4096);
                limits.max_image_height = Some(4096);
                decoder.limits(limits);
                let decoded = decoder.decode()?;
                let rgba = if decoded.width() > 512 || decoded.height() > 512 {
                    decoded.resize(512, 512, image::imageops::FilterType::Triangle)
                } else {
                    decoded
                }
                .into_rgba8();
                Some(Texture {
                    width: rgba.width(),
                    height: rgba.height(),
                    rgba_base64: base64::engine::general_purpose::STANDARD.encode(rgba.as_raw()),
                    // Leaf-card BLEND becomes a crisp, depth-writing paper cutout.
                    alpha_cutoff: match material.alpha_mode() {
                        gltf::material::AlphaMode::Opaque => None,
                        _ => Some(material.alpha_cutoff().unwrap_or(0.5)),
                    },
                    double_sided: material.double_sided(),
                    wrap_u: wrap(info.texture().sampler().wrap_s()),
                    wrap_v: wrap(info.texture().sampler().wrap_t()),
                })
            } else {
                None
            };
            let uvs: Vec<f32> = if texture.is_some() {
                reader
                    .read_tex_coords(0)
                    .ok_or("textured scenery is missing UVs")?
                    .into_f32()
                    .flatten()
                    .collect()
            } else {
                Vec::new()
            };
            let positions: Vec<Vec3> = reader
                .read_positions()
                .ok_or("missing positions or unsupported compression")?
                .map(|p| matrix.transform_point3(Vec3::from_array(p)))
                .collect();
            let normal_matrix = matrix.inverse().transpose();
            let normals: Vec<Vec3> = reader
                .read_normals()
                .ok_or("missing normals")?
                .map(|n| {
                    normal_matrix
                        .transform_vector3(Vec3::from_array(n))
                        .normalize()
                })
                .collect();
            let mut indices: Vec<u32> = reader
                .read_indices()
                .ok_or("missing indices")?
                .into_u32()
                .collect();
            if positions.is_empty()
                || normals.len() != positions.len()
                || indices.is_empty()
                || (texture.is_some()
                    && (uvs.len() != positions.len() * 2 || uvs.iter().any(|v| !v.is_finite())))
                || indices.len() % 3 != 0
                || indices.iter().any(|&i| i as usize >= positions.len())
                || positions.iter().chain(&normals).any(|v| !v.is_finite())
            {
                return Err("invalid scenery geometry".into());
            }
            if matrix.determinant() < 0.0 {
                for face in indices.chunks_exact_mut(3) {
                    face.swap(1, 2);
                }
            }
            parts.push(Primitive {
                uvs,
                texture,
                name: node.name().unwrap_or("scenery").into(),
                material: primitive.material().name().unwrap_or("paper").into(),
                base_color: pbr.base_color_factor(),
                positions: positions.iter().flat_map(|p| p.to_array()).collect(),
                normals: normals.iter().flat_map(|n| n.to_array()).collect(),
                indices,
            });
        }
    }
    for child in node.children() {
        visit(child, matrix, buffers, folder, parts)?;
    }
    Ok(())
}

fn read_local(folder: &Path, uri: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let relative = Path::new(uri);
    if uri.contains(':')
        || relative
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("expected relative local scenery buffer/image path".into());
    }
    let path = folder.join(relative);
    if std::fs::metadata(&path)?.len() > 16 * 1024 * 1024 {
        return Err("scenery buffer/image exceeds 16 MiB".into());
    }
    let bytes = std::fs::read(path)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("scenery buffer/image exceeds 16 MiB".into());
    }
    Ok(bytes)
}

fn import(
    source: &Path,
    transform: Mat4,
    parts: &mut Vec<Primitive>,
) -> Result<(), Box<dyn Error>> {
    if std::fs::metadata(source)?.len() > 16 * 1024 * 1024 {
        return Err("input exceeds 16 MiB".into());
    }
    let bytes = std::fs::read(source)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("input exceeds 16 MiB".into());
    }
    let gltf = gltf::Gltf::from_slice(&bytes)?;
    if gltf.animations().next().is_some() {
        return Err("static scenery import does not retain animation".into());
    }
    let folder = source.parent().unwrap_or(Path::new("."));
    let mut buffers = Vec::new();
    for buffer in gltf.buffers() {
        buffers.push(match buffer.source() {
            gltf::buffer::Source::Bin => gltf.blob.clone().ok_or("missing GLB buffer")?,
            gltf::buffer::Source::Uri(uri) => read_local(folder, uri)?,
        });
    }
    let scene = gltf
        .default_scene()
        .or_else(|| gltf.scenes().next())
        .ok_or("no scene")?;
    for node in scene.nodes() {
        visit(node, transform, &buffers, folder, parts)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 && !(args.len() == 4 && args[2] == "--cutout-color") {
        return Err(
            "usage: import_open_model SOURCE.glb|SOURCE.gltf|ARRANGEMENT.json EXTERNAL_OUTPUT.mesh.json [--cutout-color RRGGBB]".into(),
        );
    }
    let source = Path::new(&args[0]);
    let mut parts = Vec::new();
    if source.extension().is_some_and(|ext| ext == "json") {
        if std::fs::metadata(source)?.len() > 65536 {
            return Err("arrangement exceeds 64 KiB".into());
        }
        let arrangement: Arrangement = serde_json::from_slice(&std::fs::read(source)?)?;
        if arrangement.parts.is_empty() || arrangement.parts.len() > 64 {
            return Err("arrangement needs 1–64 parts".into());
        }
        for part in arrangement.parts {
            let transform = Mat4::from_scale_rotation_translation(
                Vec3::from_array(part.scale),
                bevy::math::Quat::IDENTITY,
                Vec3::from_array(part.translation),
            );
            import(
                &source
                    .parent()
                    .ok_or("arrangement has no parent")?
                    .join(part.model),
                transform,
                &mut parts,
            )?;
        }
    } else {
        import(source, Mat4::IDENTITY, &mut parts)?;
    }
    if args.len() == 4 {
        let hex = args[3]
            .to_str()
            .ok_or("cutout color must be UTF-8")?
            .trim_start_matches('#');
        if hex.len() != 6 {
            return Err("cutout color needs six hexadecimal digits".into());
        }
        let rgb = u32::from_str_radix(hex, 16)?;
        for part in &mut parts {
            if let Some(texture) = part.texture.as_mut().filter(|t| t.alpha_cutoff.is_some()) {
                let mut pixels =
                    base64::engine::general_purpose::STANDARD.decode(&texture.rgba_base64)?;
                // Matte colored paper: preserve every alpha/UV contour while
                // replacing the flat leaf albedo with an explicit art palette.
                for pixel in pixels.chunks_exact_mut(4) {
                    pixel[..3].fill(255);
                }
                texture.rgba_base64 = base64::engine::general_purpose::STANDARD.encode(pixels);
                for channel in 0..3 {
                    let value = ((rgb >> (16 - channel * 8)) & 255) as f32 / 255.;
                    part.base_color[channel] = if value <= 0.04045 {
                        value / 12.92
                    } else {
                        ((value + 0.055) / 1.055).powf(2.4)
                    };
                }
            }
        }
    }
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for p in &parts {
        for v in p.positions.chunks_exact(3) {
            let v = Vec3::new(v[0], v[1], v[2]);
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    if !lo.is_finite() || (hi - lo).min_element() <= 0.0 {
        return Err("empty or flat scenery".into());
    }
    // Ground the imported asset, keeping its shape and relative part placement.
    for p in &mut parts {
        for v in p.positions.chunks_exact_mut(3) {
            v[1] -= lo.y;
        }
    }
    hi.y -= lo.y;
    lo.y = 0.0;
    let output = serde_json::json!({
        "name": source.file_stem().ok_or("source has no filename")?.to_string_lossy(),
        "coordinate_system": "+Y up, +Z front", "origin": [0,0,0],
        "preserve_aspect": true,
        "bounds": {"min":lo.to_array(), "max":hi.to_array()},
        "dimensions": (hi-lo).to_array(), "primitive_count":parts.len(),
        "triangle_count":parts.iter().map(|p|p.indices.len()/3).sum::<usize>(),
        "primitives":parts,
    });
    let dest = Path::new(&args[1]);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, serde_json::to_vec(&output)?)?;
    println!("converted {} to {}", source.display(), dest.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_reflection_preserves_surface_winding_and_inverse_transpose_normals() {
        let mut binary = Vec::new();
        for value in [0_f32, 0., 0., 1., 0., 0., 0., 1., 1.] {
            binary.extend(value.to_le_bytes());
        }
        for _ in 0..3 {
            for value in [0_f32, -0.70710677, 0.70710677] {
                binary.extend(value.to_le_bytes());
            }
        }
        for value in [0_u16, 1, 2] {
            binary.extend(value.to_le_bytes());
        }
        let document = serde_json::json!({
            "asset":{"version":"2.0"}, "buffers":[{"byteLength":78}],
            "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36},
                {"buffer":0,"byteOffset":36,"byteLength":36},
                {"buffer":0,"byteOffset":72,"byteLength":6}],
            "accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[1,1,1]},
                {"bufferView":1,"componentType":5126,"count":3,"type":"VEC3"},
                {"bufferView":2,"componentType":5123,"count":3,"type":"SCALAR"}],
            "materials":[{"pbrMetallicRoughness":{"baseColorFactor":[0.5,0.3,0.2,1]}}],
            "meshes":[{"primitives":[{"attributes":{"POSITION":0,"NORMAL":1},"indices":2,"material":0}]}],
            "nodes":[{"translation":[5,-1,2],"children":[1]},{"mesh":0,"scale":[-2,3,4]}],
            "scenes":[{"nodes":[0]}],"scene":0
        });
        let mut json = serde_json::to_vec(&document).unwrap();
        while json.len() % 4 != 0 {
            json.push(b' ');
        }
        while binary.len() % 4 != 0 {
            binary.push(0);
        }
        let total = 28 + json.len() + binary.len();
        let mut bytes = b"glTF".to_vec();
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend((total as u32).to_le_bytes());
        bytes.extend((json.len() as u32).to_le_bytes());
        bytes.extend(b"JSON");
        bytes.extend(json);
        bytes.extend((binary.len() as u32).to_le_bytes());
        bytes.extend(b"BIN\0");
        bytes.extend(binary);
        let model = gltf::Gltf::from_slice(&bytes).unwrap();
        let mut parts = Vec::new();
        visit(
            model.nodes().next().unwrap(),
            Mat4::IDENTITY,
            &[model.blob.clone().unwrap()],
            Path::new("."),
            &mut parts,
        )
        .unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].positions, [5., -1., 2., 3., -1., 2., 5., 2., 6.]);
        assert_eq!(parts[0].indices, [0, 2, 1]);
        for (actual, expected) in parts[0]
            .normals
            .chunks_exact(3)
            .flat_map(|n| n.iter())
            .zip([0., -0.8, 0.6].into_iter().cycle())
        {
            assert!((actual - expected).abs() < 1e-6);
        }
        assert_eq!(parts[0].base_color, [0.5, 0.3, 0.2, 1.]);
    }
}
