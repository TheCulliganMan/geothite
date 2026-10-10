//! Convert externally supplied static, solid-material GLBs to renderer JSON.
//! Outputs are external content-pack inputs, never files to commit.
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
struct Primitive {
    name: String,
    material: String,
    base_color: [f32; 4],
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
}

fn visit(
    node: gltf::Node<'_>,
    parent: Mat4,
    blob: &[u8],
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
            if pbr.base_color_texture().is_some()
                || primitive.material().alpha_mode() != gltf::material::AlphaMode::Opaque
            {
                return Err("textured/translucent assets need the textured scenery path".into());
            }
            let reader = primitive.reader(|buffer| match buffer.source() {
                gltf::buffer::Source::Bin => Some(blob),
                _ => None,
            });
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
        visit(child, matrix, blob, parts)?;
    }
    Ok(())
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
    let blob = gltf.blob.as_deref().ok_or("expected self-contained GLB")?;
    let scene = gltf
        .default_scene()
        .or_else(|| gltf.scenes().next())
        .ok_or("no scene")?;
    for node in scene.nodes() {
        visit(node, transform, blob, parts)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: import_open_model SOURCE.glb|ARRANGEMENT.json EXTERNAL_OUTPUT.mesh.json".into(),
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
            model.blob.as_deref().unwrap(),
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
