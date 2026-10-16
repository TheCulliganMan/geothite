//! Lossless CPU mesh buffers; GPU handles and allocator state stay local.
use super::blobs::{BlobStore, Bulk};
use bevy::{
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
    render::render_resource::{PrimitiveTopology, VertexFormat},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct MeshWire {
    usage: u8,
    attributes: Vec<(u8, Values)>,
    indices: Option<IndexWire>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Values {
    Float2(Bulk<[u32; 2]>),
    Float3(Bulk<[u32; 3]>),
    Float4(Bulk<[u32; 4]>),
    Uint16x4(Bulk<[u16; 4]>),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum IndexWire {
    Narrow(Bulk<u16>),
    Wide(Bulk<u32>),
}

fn float_buffer<const N: usize>(bits: Vec<[u32; N]>) -> Vec<[f32; N]>
where
    [u32; N]: bytemuck::Pod,
    [f32; N]: bytemuck::Pod,
{
    bytemuck::allocation::try_cast_vec(bits).unwrap_or_else(|(_, bits)| {
        bits.into_iter()
            .map(|value| value.map(f32::from_bits))
            .collect()
    })
}

pub(super) fn float_bits<const N: usize>(values: Vec<[f32; N]>) -> Vec<[u32; N]>
where
    [f32; N]: bytemuck::Pod,
    [u32; N]: bytemuck::Pod,
{
    bytemuck::allocation::try_cast_vec(values)
        .unwrap_or_else(|(_, values)| values.into_iter().map(|v| v.map(f32::to_bits)).collect())
}

impl MeshWire {
    /// Transfer a worker-owned mesh instead of retaining an original and a
    /// complete wire copy at the same time. GPU-extracted meshes still fail.
    pub(crate) fn capture_owned(mut mesh: Mesh) -> Result<Self, String> {
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            return Err("terrain worker requires triangle-list meshes".into());
        }
        let attributes = mesh
            .try_attributes()
            .map_err(|e| e.to_string())?
            .map(|(attribute, _)| *attribute)
            .collect::<Vec<_>>();
        let mut captured = Vec::with_capacity(attributes.len());
        for attribute in attributes {
            let id = [
                Mesh::ATTRIBUTE_POSITION,
                Mesh::ATTRIBUTE_NORMAL,
                Mesh::ATTRIBUTE_UV_0,
                Mesh::ATTRIBUTE_UV_1,
                Mesh::ATTRIBUTE_COLOR,
                Mesh::ATTRIBUTE_TANGENT,
                Mesh::ATTRIBUTE_JOINT_INDEX,
                Mesh::ATTRIBUTE_JOINT_WEIGHT,
            ]
            .iter()
            .position(|a| a.id == attribute.id)
            .ok_or("unsupported terrain vertex attribute")? as u8;
            let values = match mesh
                .try_remove_attribute(attribute.id)
                .map_err(|e| e.to_string())?
            {
                VertexAttributeValues::Float32x2(v) => Values::Float2(float_bits(v).into()),
                VertexAttributeValues::Float32x3(v) => Values::Float3(float_bits(v).into()),
                VertexAttributeValues::Float32x4(v) => Values::Float4(float_bits(v).into()),
                VertexAttributeValues::Uint16x4(v) => Values::Uint16x4(v.into()),
                _ => return Err("unsupported terrain vertex storage".into()),
            };
            captured.push((id, values));
        }
        let indices = mesh
            .try_remove_indices()
            .map_err(|e| e.to_string())?
            .map(|i| match i {
                Indices::U16(v) => IndexWire::Narrow(v.into()),
                Indices::U32(v) => IndexWire::Wide(v.into()),
            });
        Ok(Self {
            usage: mesh.asset_usage.bits(),
            attributes: captured,
            indices,
        })
    }

    pub(crate) fn capture(mesh: &Mesh) -> Result<Self, String> {
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            return Err("terrain worker requires triangle-list meshes".into());
        }
        let mut attributes = Vec::new();
        for (attribute, values) in mesh.try_attributes().map_err(|e| e.to_string())? {
            let id = [
                Mesh::ATTRIBUTE_POSITION,
                Mesh::ATTRIBUTE_NORMAL,
                Mesh::ATTRIBUTE_UV_0,
                Mesh::ATTRIBUTE_UV_1,
                Mesh::ATTRIBUTE_COLOR,
                Mesh::ATTRIBUTE_TANGENT,
                Mesh::ATTRIBUTE_JOINT_INDEX,
                Mesh::ATTRIBUTE_JOINT_WEIGHT,
            ]
            .iter()
            .position(|a| a.id == attribute.id)
            .ok_or("unsupported terrain vertex attribute")? as u8;
            let values = match values {
                VertexAttributeValues::Float32x2(v) => {
                    Values::Float2(v.iter().map(|v| v.map(f32::to_bits)).collect())
                }
                VertexAttributeValues::Float32x3(v) => {
                    Values::Float3(v.iter().map(|v| v.map(f32::to_bits)).collect())
                }
                VertexAttributeValues::Float32x4(v) => {
                    Values::Float4(v.iter().map(|v| v.map(f32::to_bits)).collect())
                }
                VertexAttributeValues::Uint16x4(v) => Values::Uint16x4(v.clone().into()),
                _ => return Err("unsupported terrain vertex storage".into()),
            };
            attributes.push((id, values));
        }
        Ok(Self {
            usage: mesh.asset_usage.bits(),
            attributes,
            indices: mesh
                .try_indices_option()
                .map_err(|e| e.to_string())?
                .map(|i| match i {
                    Indices::U16(v) => IndexWire::Narrow(v.clone().into()),
                    Indices::U32(v) => IndexWire::Wide(v.clone().into()),
                }),
        })
    }

    pub(crate) fn resolve(&mut self, blobs: &mut BlobStore) -> Result<bool, String> {
        let mut ready = true;
        for (_, value) in &mut self.attributes {
            ready &= match value {
                Values::Float2(v) => v.resolve(blobs)?,
                Values::Float3(v) => v.resolve(blobs)?,
                Values::Float4(v) => v.resolve(blobs)?,
                Values::Uint16x4(v) => v.resolve(blobs)?,
            };
        }
        if let Some(indices) = &mut self.indices {
            ready &= match indices {
                IndexWire::Narrow(v) => v.resolve(blobs)?,
                IndexWire::Wide(v) => v.resolve(blobs)?,
            };
        }
        Ok(ready)
    }

    pub(crate) fn into_mesh(self) -> Result<Mesh, String> {
        let usage = bevy::asset::RenderAssetUsages::from_bits(self.usage)
            .ok_or("invalid terrain mesh usage")?;
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, usage);
        let mut seen = std::collections::HashSet::new();
        for (id, values) in self.attributes {
            if !seen.insert(id) {
                return Err("duplicate terrain mesh attribute".into());
            }
            let mut attribute = *[
                Mesh::ATTRIBUTE_POSITION,
                Mesh::ATTRIBUTE_NORMAL,
                Mesh::ATTRIBUTE_UV_0,
                Mesh::ATTRIBUTE_UV_1,
                Mesh::ATTRIBUTE_COLOR,
                Mesh::ATTRIBUTE_TANGENT,
                Mesh::ATTRIBUTE_JOINT_INDEX,
                Mesh::ATTRIBUTE_JOINT_WEIGHT,
            ]
            .get(id as usize)
            .ok_or("unknown terrain mesh attribute")?;
            let values = match values {
                Values::Float2(v) => {
                    attribute.format = VertexFormat::Float32x2;
                    VertexAttributeValues::Float32x2(float_buffer(v.into_vec()?))
                }
                Values::Float3(v) => {
                    attribute.format = VertexFormat::Float32x3;
                    VertexAttributeValues::Float32x3(float_buffer(v.into_vec()?))
                }
                Values::Float4(v) => {
                    attribute.format = VertexFormat::Float32x4;
                    VertexAttributeValues::Float32x4(float_buffer(v.into_vec()?))
                }
                Values::Uint16x4(v) => {
                    attribute.format = VertexFormat::Uint16x4;
                    VertexAttributeValues::Uint16x4(v.into_vec()?)
                }
            };
            mesh.insert_attribute(attribute, values);
        }
        if let Some(indices) = self.indices {
            mesh.insert_indices(match indices {
                IndexWire::Narrow(v) => Indices::U16(v.into_vec()?),
                IndexWire::Wide(v) => Indices::U32(v.into_vec()?),
            });
        }
        Ok(mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_capture_preserves_full_mesh_and_transfers_original_allocations() {
        for width in [0, 16, 32] {
            let count = 16_386;
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                bevy::asset::RenderAssetUsages::all(),
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.25, -0.0, 2.0]; count]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[-0.0, 1.0, 0.0]; count]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[-0.0, 0.125]; count]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.25, -0.0]; count]);
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                vec![[f32::from_bits(0x7fa12345), -0.0, 0.5, 1.0]; count],
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[1.0, -0.0, 0.0, -1.0]; count]);
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(vec![[1_u16, 2, 3, 4]; count]),
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_WEIGHT,
                vec![[0.25, 0.5, 0.25, -0.0]; count],
            );
            match width {
                16 => mesh.insert_indices(Indices::U16((0..count as u16).collect())),
                32 => mesh.insert_indices(Indices::U32((0..count as u32).collect())),
                _ => (),
            }
            let original_attributes = mesh
                .attributes()
                .map(|(a, v)| {
                    (
                        a.id,
                        a.format,
                        v.get_bytes().as_ptr() as usize,
                        v.get_bytes().to_vec(),
                    )
                })
                .collect::<Vec<_>>();
            let original_indices = mesh.indices().cloned();
            let index_pointer = mesh.indices().map(|v| match v {
                Indices::U16(v) => v.as_ptr() as usize,
                Indices::U32(v) => v.as_ptr() as usize,
            });
            let control = super::super::encode(&MeshWire::capture(&mesh).unwrap()).unwrap();
            let wire = MeshWire::capture_owned(mesh).unwrap();
            assert_eq!(control, super::super::encode(&wire).unwrap());
            let restored = wire.into_mesh().unwrap();
            assert_eq!(restored.asset_usage, bevy::asset::RenderAssetUsages::all());
            for ((id, format, pointer, bytes), (attribute, values)) in
                original_attributes.into_iter().zip(restored.attributes())
            {
                assert_eq!((id, format), (attribute.id, attribute.format));
                assert_eq!(bytes, values.get_bytes());
                assert_eq!(
                    pointer,
                    values.get_bytes().as_ptr() as usize,
                    "owned attribute must retain its original allocation"
                );
            }
            assert_eq!(original_indices.as_ref(), restored.indices());
            assert_eq!(
                index_pointer,
                restored.indices().map(|v| match v {
                    Indices::U16(v) => v.as_ptr() as usize,
                    Indices::U32(v) => v.as_ptr() as usize,
                })
            );
        }
    }

    #[test]
    fn owned_capture_rejects_gpu_extracted_and_nontriangle_meshes() {
        let mut extracted = Mesh::from(Cuboid::default());
        let _gpu_data = extracted.take_gpu_data().unwrap();
        assert!(MeshWire::capture_owned(extracted).is_err());
        assert!(
            MeshWire::capture_owned(Mesh::new(
                PrimitiveTopology::LineList,
                bevy::asset::RenderAssetUsages::all()
            ))
            .is_err()
        );
    }

    #[test]
    fn float_storage_cast_reuses_allocation_and_preserves_every_bit() {
        let bits = vec![[0x80000000, 0x7fc12345, 0x3f123456]; 16384];
        let pointer = bits.as_ptr() as usize;
        let capacity = bits.capacity();
        let values = float_buffer(bits);
        assert_eq!(values.as_ptr() as usize, pointer);
        assert_eq!(values.capacity(), capacity);
        assert!(
            values
                .iter()
                .all(|v| v.map(f32::to_bits) == [0x80000000, 0x7fc12345, 0x3f123456])
        );
    }

    #[test]
    fn compact_and_full_mesh_roundtrip_preserves_attribute_bytes_and_index_width() {
        for compact in [false, true] {
            let mut mesh = Mesh::from(Cuboid::default());
            let count = mesh.count_vertices();
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_1,
                vec![[-0.0, f32::from_bits(0x3f123456)]; count],
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.3, -0., 0.7, 1.]; count]);
            if compact {
                crate::compact_vertices::remove_constant_color_alpha(&mut mesh);
                mesh.insert_indices(Indices::U16(
                    mesh.indices().unwrap().iter().map(|i| i as u16).collect(),
                ));
            }
            let wire = MeshWire::capture(&mesh).unwrap();
            let bytes = super::super::encode(&wire).unwrap();
            let decoded: MeshWire = super::super::decode(&bytes).unwrap();
            let restored = decoded.into_mesh().unwrap();
            assert_eq!(wire, MeshWire::capture(&restored).unwrap());
            assert_eq!(
                mesh.attributes()
                    .map(|(a, v)| (a.id, a.format, v.get_bytes().to_vec()))
                    .collect::<Vec<_>>(),
                restored
                    .attributes()
                    .map(|(a, v)| (a.id, a.format, v.get_bytes().to_vec()))
                    .collect::<Vec<_>>()
            );
            assert_eq!(mesh.indices(), restored.indices());
        }
    }
}
