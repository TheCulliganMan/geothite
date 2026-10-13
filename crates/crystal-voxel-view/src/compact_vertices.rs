//! Lossless omission of redundant vertex components. UV0 removal requires the
//! caller to establish that no textures or procedural effects consume it.
use bevy::{mesh::VertexAttributeValues, prelude::*};

/// Vertex fetch supplies 1.0 for the absent fourth float. Preserve the original
/// RGB bits and retain RGBA whenever any authored alpha is not exactly opaque.
pub(super) fn remove_constant_color_alpha(mesh: &mut Mesh) -> usize {
    let Ok(VertexAttributeValues::Float32x4(colors)) = mesh.try_attribute(Mesh::ATTRIBUTE_COLOR)
    else { return 0; };
    let Ok(VertexAttributeValues::Float32x3(positions)) = mesh.try_attribute(Mesh::ATTRIBUTE_POSITION)
    else { return 0; };
    if colors.len() != positions.len()
        || colors.iter().any(|color| color[3].to_bits() != 1.0_f32.to_bits()) {
        return 0;
    }
    let bytes = colors.len() * std::mem::size_of::<f32>();
    let colors = colors.iter().map(|color| [color[0], color[1], color[2]]).collect::<Vec<_>>();
    mesh.insert_attribute(bevy::mesh::MeshVertexAttribute {
        format: bevy::render::render_resource::VertexFormat::Float32x3,
        ..Mesh::ATTRIBUTE_COLOR
    }, colors);
    bytes
}

pub(super) fn solid_uv0_is_unused(material: &StandardMaterial, water_enabled: Vec4) -> bool {
    material.unlit
        && material.base_color_texture.is_none()
        && material.normal_map_texture.is_none()
        && water_enabled.x == 0.0
}

pub(super) fn remove_unused_zero_uv0(mesh: &mut Mesh) -> usize {
    let Ok(VertexAttributeValues::Float32x2(values)) = mesh.try_attribute(Mesh::ATTRIBUTE_UV_0)
    else {
        return 0;
    };
    let Ok(VertexAttributeValues::Float32x3(positions)) =
        mesh.try_attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return 0;
    };
    if values.len() != positions.len() || values.iter().any(|uv| *uv != [0.0, 0.0]) {
        return 0;
    }
    let bytes = values.len() * std::mem::size_of::<[f32; 2]>();
    mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> Mesh {
        let mut mesh = Mesh::from(Cuboid::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]; mesh.count_vertices()]);
        mesh
    }
    #[test]
    fn opaque_color_compaction_keeps_rgb_geometry_and_indices_exact() {
        let mut mesh = source();
        let color = [0.137_f32, -0.0, 0.863, 1.0];
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![color; mesh.count_vertices()]);
        let unchanged: Vec<_> = mesh.attributes()
            .filter(|(a, _)| a.id != Mesh::ATTRIBUTE_COLOR.id)
            .map(|(a, v)| (a.id, v.get_bytes().to_vec())).collect();
        let indices = mesh.indices().cloned();
        let count = mesh.count_vertices();
        assert_eq!(remove_constant_color_alpha(&mut mesh), count * 4);
        let Some(VertexAttributeValues::Float32x3(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else { panic!(); };
        assert!(colors.iter().all(|rgb| rgb.map(f32::to_bits) ==
            [color[0], color[1], color[2]].map(f32::to_bits)));
        let after: Vec<_> = mesh.attributes()
            .filter(|(a, _)| a.id != Mesh::ATTRIBUTE_COLOR.id)
            .map(|(a, v)| (a.id, v.get_bytes().to_vec())).collect();
        assert_eq!(unchanged, after);
        assert_eq!(indices, mesh.indices().cloned());
        assert_eq!(remove_constant_color_alpha(&mut mesh), 0);
    }
    #[test]
    fn translucent_invalid_and_extracted_colors_keep_their_original_storage() {
        for alpha in [0.0, -0.0, 0.5, f32::NAN, f32::INFINITY] {
            let mut mesh = source();
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.3, 0.2, 0.1, alpha]; mesh.count_vertices()]);
            assert_eq!(remove_constant_color_alpha(&mut mesh), 0);
            assert!(matches!(mesh.attribute(Mesh::ATTRIBUTE_COLOR), Some(VertexAttributeValues::Float32x4(_))));
        }
        let mut invalid = source();
        invalid.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.3, 0.2, 0.1, 1.0]]);
        assert_eq!(remove_constant_color_alpha(&mut invalid), 0);
        let mut extracted = source();
        extracted.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.3, 0.2, 0.1, 1.0]; extracted.count_vertices()]);
        let _gpu = extracted.take_gpu_data().unwrap();
        assert_eq!(remove_constant_color_alpha(&mut extracted), 0);
    }
    #[test]
    fn removing_unused_uvs_preserves_every_geometry_and_color_bit() {
        let mut mesh = source();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![[0.137, 0.287, 0.863, 1.0]; mesh.count_vertices()],
        );
        let before: Vec<_> = mesh
            .attributes()
            .filter(|(attribute, _)| attribute.id != Mesh::ATTRIBUTE_UV_0.id)
            .map(|(attribute, values)| (attribute.id, values.get_bytes().to_vec()))
            .collect();
        let indices = mesh.indices().cloned();
        let vertices = mesh.count_vertices();
        assert_eq!(remove_unused_zero_uv0(&mut mesh), vertices * 8);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_none());
        let after: Vec<_> = mesh
            .attributes()
            .map(|(attribute, values)| (attribute.id, values.get_bytes().to_vec()))
            .collect();
        assert_eq!(before, after);
        assert_eq!(indices, mesh.indices().cloned());
        assert_eq!(vertices, mesh.count_vertices());
    }
    #[test]
    fn tagged_textured_invalid_and_gpu_extracted_coordinates_are_retained() {
        for uv in [
            [0.00001, 0.0],
            [0.0, 0.4],
            [100000.0, 300000.0],
            [f32::NAN, 0.0],
        ] {
            let mut mesh = source();
            let Some(VertexAttributeValues::Float32x2(values)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
            else {
                panic!();
            };
            values[0] = uv;
            assert_eq!(remove_unused_zero_uv0(&mut mesh), 0);
            assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        }
        let mut invalid = source();
        invalid.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]]);
        assert_eq!(remove_unused_zero_uv0(&mut invalid), 0);
        let mut extracted = source();
        let _gpu = extracted.take_gpu_data().unwrap();
        assert_eq!(remove_unused_zero_uv0(&mut extracted), 0);
    }
    #[test]
    fn textures_lit_materials_and_water_effects_keep_their_coordinates() {
        let mut material = StandardMaterial {
            unlit: true,
            ..default()
        };
        assert!(solid_uv0_is_unused(&material, Vec4::ZERO));
        assert!(!solid_uv0_is_unused(&material, Vec4::X));
        material.base_color_texture = Some(Handle::default());
        assert!(!solid_uv0_is_unused(&material, Vec4::ZERO));
        material.base_color_texture = None;
        material.normal_map_texture = Some(Handle::default());
        assert!(!solid_uv0_is_unused(&material, Vec4::ZERO));
        material.normal_map_texture = None;
        material.unlit = false;
        assert!(!solid_uv0_is_unused(&material, Vec4::ZERO));
    }
}
