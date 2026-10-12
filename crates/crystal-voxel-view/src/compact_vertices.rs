//! Lossless removal of a constant, unused attribute. The caller must establish
//! that neither its material textures nor procedural effects consume UV0.
use bevy::{mesh::VertexAttributeValues, prelude::*};

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
