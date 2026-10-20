#[path = "response_restore.rs"]
mod incremental;
use super::blobs::{BlobStore, Bulk};
use super::{mesh_buffers::MeshWire, request::TextureBindings};
use crate::scenery_surfaces::worker_wire::TextureWire;
use crate::{
    BuiltTerrain,
    terrain_batches::{AuthenticDomain, PreparedBatch},
};
use bevy::{camera::primitives::Aabb, prelude::*};
pub(super) use incremental::Restore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Bounds {
    center: [u32; 3],
    half: [u32; 3],
}
impl Bounds {
    fn capture(b: Aabb) -> Self {
        Self {
            center: b.center.to_array().map(f32::to_bits),
            half: b.half_extents.to_array().map(f32::to_bits),
        }
    }
    fn restore(self) -> Aabb {
        Aabb {
            center: Vec3::from_array(self.center.map(f32::from_bits)).into(),
            half_extents: Vec3::from_array(self.half.map(f32::from_bits)).into(),
        }
    }
}
// Keep V2's original maps, but page fixed-width coverage arrays in V3.
#[derive(Clone, Debug, PartialEq)]
enum BoundsList {
    Plain(Vec<Bounds>),
    Paged(Bulk<[u32; 6]>),
}
impl Serialize for BoundsList {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Plain(bounds) if super::blobs::header_encoding_active() => {
                let values: Bulk<[u32; 6]> = bounds
                    .iter()
                    .map(|b| {
                        [
                            b.center[0],
                            b.center[1],
                            b.center[2],
                            b.half[0],
                            b.half[1],
                            b.half[2],
                        ]
                    })
                    .collect();
                values.serialize(serializer)
            }
            Self::Plain(bounds) => bounds.serialize(serializer),
            Self::Paged(values) => values.serialize(serializer),
        }
    }
}
impl<'de> Deserialize<'de> for BoundsList {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if super::blobs::header_decoding_active() {
            Bulk::deserialize(deserializer).map(Self::Paged)
        } else {
            Vec::deserialize(deserializer).map(Self::Plain)
        }
    }
}
impl BoundsList {
    fn resolve(&mut self, blobs: &mut BlobStore) -> Result<bool, String> {
        match self {
            Self::Plain(_) => Ok(true),
            Self::Paged(values) => values.resolve(blobs),
        }
    }
    fn restore(self) -> Result<Arc<[Aabb]>, String> {
        Ok(match self {
            Self::Plain(bounds) => bounds.into_iter().map(Bounds::restore).collect(),
            Self::Paged(values) => values
                .into_vec()?
                .into_iter()
                .map(|b| {
                    Bounds {
                        center: [b[0], b[1], b[2]],
                        half: [b[3], b[4], b[5]],
                    }
                    .restore()
                })
                .collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Batch {
    mesh: MeshWire,
    #[serde(default)]
    rooted_grass: Option<bool>,
    bounds: Option<Bounds>,
    domain: u8,
    coverage: Option<BoundsList>,
    sight: Option<BoundsList>,
    grass: Option<BoundsList>,
}
fn domain(v: AuthenticDomain) -> u8 {
    match v {
        AuthenticDomain::Unspecified => 0,
        AuthenticDomain::Inside => 1,
        AuthenticDomain::Outside => 2,
        AuthenticDomain::Unresolved => 3,
    }
}
fn restore_domain(v: u8) -> Result<AuthenticDomain, String> {
    match v {
        0 => Ok(AuthenticDomain::Unspecified),
        1 => Ok(AuthenticDomain::Inside),
        2 => Ok(AuthenticDomain::Outside),
        3 => Ok(AuthenticDomain::Unresolved),
        _ => Err("invalid authentic terrain domain".into()),
    }
}
impl Batch {
    #[cfg(test)]
    fn capture(b: &PreparedBatch) -> Result<Self, String> {
        fn coverage(v: &Option<Arc<[Aabb]>>) -> Option<BoundsList> {
            v.as_ref()
                .map(|v| BoundsList::Plain(v.iter().copied().map(Bounds::capture).collect()))
        }
        Ok(Self {
            mesh: MeshWire::capture(&b.mesh)?,
            rooted_grass: Some(b.rooted_grass),
            bounds: b.bounds.map(Bounds::capture),
            domain: domain(b.authentic_domain),
            coverage: coverage(&b.coverage),
            sight: coverage(&b.sight_coverage),
            grass: coverage(&b.grass_coverage),
        })
    }
    fn capture_owned(b: PreparedBatch) -> Result<Self, String> {
        fn coverage(v: &Option<Arc<[Aabb]>>) -> Option<BoundsList> {
            v.as_ref()
                .map(|v| BoundsList::Plain(v.iter().copied().map(Bounds::capture).collect()))
        }
        Ok(Self {
            mesh: MeshWire::capture_owned(b.mesh)?,
            rooted_grass: Some(b.rooted_grass),
            bounds: b.bounds.map(Bounds::capture),
            domain: domain(b.authentic_domain),
            coverage: coverage(&b.coverage),
            sight: coverage(&b.sight_coverage),
            grass: coverage(&b.grass_coverage),
        })
    }
    fn resolve(&mut self, blobs: &mut BlobStore) -> Result<bool, String> {
        let mut ready = self.mesh.resolve(blobs)?;
        for bounds in [&mut self.coverage, &mut self.sight, &mut self.grass] {
            if let Some(bounds) = bounds {
                ready &= bounds.resolve(blobs)?;
            }
        }
        Ok(ready)
    }
    fn restore(self) -> Result<PreparedBatch, String> {
        fn coverage(v: Option<BoundsList>) -> Result<Option<Arc<[Aabb]>>, String> {
            v.map(BoundsList::restore).transpose()
        }
        let mesh = self.mesh.into_mesh()?;
        let rooted_grass = self.rooted_grass.unwrap_or_else(|| crate::terrain_batches::has_rooted_grass(&mesh));
        Ok(PreparedBatch {
            mesh,
            rooted_grass,
            bounds: self.bounds.map(Bounds::restore),
            authentic_domain: restore_domain(self.domain)?,
            coverage: coverage(self.coverage)?,
            sight_coverage: coverage(self.sight)?,
            grass_coverage: coverage(self.grass)?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Response {
    #[serde(default)]
    city_emitters: Vec<[[u32;4];2]>,
    pub(super) job: u64,
    streamed: Option<crate::mesh::worker_wire::StreamWire>,
    groups: crate::occluder_fade::worker_wire::GroupTable,
    group_lengths: Vec<u64>,
    textures: Vec<TextureWire>,
    scenery: Vec<(Batch, u32)>,
    scenery_fades: Vec<(u32, u8)>,
    background: Option<(MeshWire, u32)>,
    enclosure: Option<MeshWire>,
    instances: Vec<(MeshWire, Vec<[u32; 3]>)>,
    joins: Vec<(MeshWire, [[u32; 3]; 2])>,
    footing: Bulk<u32>,
    textured: Vec<Batch>,
    solid: Vec<Batch>,
    animated_textured: MeshWire,
    animated_solid: MeshWire,
    pillars: Vec<[u64; 2]>,
    train_doors: bool,
}

impl Response {
    // Keep the prior borrowed encoder as an independent exact-output oracle
    // for full production-region regression fixtures.
    #[cfg(test)]
    pub(super) fn capture(
        job: u64,
        built: &BuiltTerrain,
        frame: &crystal_render_api::VisualWorldFrame,
        bindings: &mut TextureBindings,
    ) -> Result<Self, String> {
        let mut groups = Vec::new();
        let mut lengths = Vec::new();
        for subset in [&built.fade_textured_groups, &built.fade_solid_groups]
            .into_iter()
            .chain(built.scenery_fades.iter().map(|(groups, _, _)| groups))
        {
            lengths.push(subset.len() as u64);
            groups.extend(subset.iter().cloned());
        }
        let groups = crate::occluder_fade::worker_wire::GroupTable::capture(&groups)?;
        let mut textures = Vec::new();
        let mut texture_slots = std::collections::HashMap::new();
        let mut texture_slot = |texture: &Arc<crate::scenery_surfaces::SceneryTexture>| {
            let pointer = Arc::as_ptr(texture) as usize;
            *texture_slots.entry(pointer).or_insert_with(|| {
                let slot = textures.len() as u32;
                textures.push(TextureWire::capture(texture));
                slot
            })
        };
        let scenery = built
            .scenery
            .iter()
            .map(|(batch, texture)| Ok((Batch::capture(batch)?, texture_slot(texture))))
            .collect::<Result<_, String>>()?;
        let scenery_fades = built
            .scenery_fades
            .iter()
            .map(|(_, texture, d)| (texture_slot(texture), domain(*d)))
            .collect();
        Ok(Self {
            job,
            streamed: built
                .streamed_groves
                .as_ref()
                .map(|s| crate::mesh::worker_wire::StreamWire::capture(s, frame, bindings)),
            groups,
            group_lengths: lengths,
            textures,
            scenery,
            scenery_fades,
            background: built
                .background
                .as_ref()
                .map(|(mesh, texture)| {
                    Ok::<_, String>((MeshWire::capture(mesh)?, bindings.slot(texture)))
                })
                .transpose()?,
            enclosure: built
                .cave_enclosure
                .as_ref()
                .map(MeshWire::capture)
                .transpose()?,
            instances: built
                .instances
                .iter()
                .map(|(m, origins)| {
                    Ok((
                        MeshWire::capture(m)?,
                        origins.iter().map(|o| o.map(f32::to_bits)).collect(),
                    ))
                })
                .collect::<Result<_, String>>()?,
            joins: built
                .reveal_join_batches
                .iter()
                .map(|(m, b)| {
                    Ok((
                        MeshWire::capture(m)?,
                        [
                            b.min.to_array().map(f32::to_bits),
                            b.max.to_array().map(f32::to_bits),
                        ],
                    ))
                })
                .collect::<Result<_, String>>()?,
            city_emitters: built.city_emitters.iter().map(|e|[
                e.position.to_array().map(f32::to_bits),e.direction.to_array().map(f32::to_bits)]).collect(),
            footing: built
                .footing_heights
                .iter()
                .copied()
                .map(f32::to_bits)
                .collect(),
            textured: built
                .textured_meshes
                .iter()
                .map(Batch::capture)
                .collect::<Result<_, _>>()?,
            solid: built
                .solid_meshes
                .iter()
                .map(Batch::capture)
                .collect::<Result<_, _>>()?,
            animated_textured: MeshWire::capture(&built.animated_textured_mesh)?,
            animated_solid: MeshWire::capture(&built.animated_solid_mesh)?,
            train_doors: built.train_doors,
            pillars: built
                .animated_pillars
                .iter()
                .map(|p| p.map(|v| v as u64))
                .collect(),
        })
    }

    pub(super) fn capture_owned(
        job: u64,
        mut built: BuiltTerrain,
        frame: &crystal_render_api::VisualWorldFrame,
        bindings: &mut TextureBindings,
    ) -> Result<Self, String> {
        let mut source_groups = Vec::new();
        let mut lengths = Vec::new();
        for subset in [
            &mut built.fade_textured_groups,
            &mut built.fade_solid_groups,
        ]
        .into_iter()
        .chain(built.scenery_fades.iter_mut().map(|(groups, _, _)| groups))
        {
            lengths.push(subset.len() as u64);
            // PreparedGroup::clone also clones its unshared Mesh buffers.
            // Moving preserves the original Arc identities and subset order.
            source_groups.append(subset);
        }
        let groups = crate::occluder_fade::worker_wire::GroupTable::capture(&source_groups)?;
        drop(source_groups);
        let mut textures = Vec::new();
        let mut texture_slots = std::collections::HashMap::new();
        let mut texture_slot = |texture: &Arc<crate::scenery_surfaces::SceneryTexture>| {
            let pointer = Arc::as_ptr(texture) as usize;
            *texture_slots.entry(pointer).or_insert_with(|| {
                let slot = textures.len() as u32;
                textures.push(TextureWire::capture(texture));
                slot
            })
        };
        let scenery = built
            .scenery
            .into_iter()
            .map(|(batch, texture)| Ok((Batch::capture_owned(batch)?, texture_slot(&texture))))
            .collect::<Result<_, String>>()?;
        let scenery_fades = built
            .scenery_fades
            .iter()
            .map(|(_, texture, d)| (texture_slot(texture), domain(*d)))
            .collect();
        Ok(Self {
            job,
            streamed: built
                .streamed_groves
                .as_ref()
                .map(|s| crate::mesh::worker_wire::StreamWire::capture(s, frame, bindings)),
            groups,
            group_lengths: lengths,
            textures,
            scenery,
            scenery_fades,
            background: built
                .background
                .map(|(mesh, texture)| {
                    Ok::<_, String>((MeshWire::capture_owned(mesh)?, bindings.slot(&texture)))
                })
                .transpose()?,
            enclosure: built
                .cave_enclosure
                .map(MeshWire::capture_owned)
                .transpose()?,
            instances: built
                .instances
                .into_iter()
                .map(|(m, origins)| {
                    Ok((
                        MeshWire::capture_owned(m)?,
                        super::mesh_buffers::float_bits(origins),
                    ))
                })
                .collect::<Result<_, String>>()?,
            joins: built
                .reveal_join_batches
                .into_iter()
                .map(|(m, b)| {
                    Ok((
                        MeshWire::capture_owned(m)?,
                        [
                            b.min.to_array().map(f32::to_bits),
                            b.max.to_array().map(f32::to_bits),
                        ],
                    ))
                })
                .collect::<Result<_, String>>()?,
            city_emitters: built.city_emitters.iter().map(|e|[
                e.position.to_array().map(f32::to_bits),e.direction.to_array().map(f32::to_bits)]).collect(),
            footing: built
                .footing_heights
                .iter()
                .copied()
                .map(f32::to_bits)
                .collect(),
            textured: built
                .textured_meshes
                .into_iter()
                .map(Batch::capture_owned)
                .collect::<Result<_, _>>()?,
            solid: built
                .solid_meshes
                .into_iter()
                .map(Batch::capture_owned)
                .collect::<Result<_, _>>()?,
            animated_textured: MeshWire::capture_owned(built.animated_textured_mesh)?,
            animated_solid: MeshWire::capture_owned(built.animated_solid_mesh)?,
            train_doors: built.train_doors,
            pillars: built
                .animated_pillars
                .iter()
                .map(|p| p.map(|v| v as u64))
                .collect(),
        })
    }

    pub(super) fn restore(self, bindings: &TextureBindings) -> Result<BuiltTerrain, String> {
        let mut groups = self.groups.restore()?.into_iter();
        let mut subsets = Vec::new();
        if self.group_lengths.len() != self.scenery_fades.len() + 2 {
            return Err("terrain group subset count mismatch".into());
        }
        for count in self.group_lengths {
            let count = usize::try_from(count).map_err(|_| "terrain group count overflow")?;
            if count > groups.len() {
                return Err("terrain group range exceeds table".into());
            }
            subsets.push(groups.by_ref().take(count).collect::<Vec<_>>());
        }
        if groups.next().is_some() {
            return Err("unreferenced terrain groups".into());
        }
        let mut subsets = subsets.into_iter();
        let fade_textured_groups = subsets.next().ok_or("missing textured groups")?;
        let fade_solid_groups = subsets.next().ok_or("missing solid groups")?;
        let textures = self
            .textures
            .into_iter()
            .map(TextureWire::restore)
            .collect::<Result<Vec<_>, _>>()?;
        let texture = |slot: u32| {
            textures
                .get(slot as usize)
                .cloned()
                .ok_or_else(|| "invalid scenery texture slot".to_string())
        };
        Ok(BuiltTerrain {
            city_emitters: self.city_emitters.into_iter().map(|e|crate::city_lights::Emitter {
                position:Vec4::from_array(e[0].map(f32::from_bits)),
                direction:Vec4::from_array(e[1].map(f32::from_bits)),
            }).collect(),
            streamed_groves: self.streamed.map(|s| s.restore(bindings)).transpose()?,
            scenery: self
                .scenery
                .into_iter()
                .map(|(b, t)| Ok((b.restore()?, texture(t)?)))
                .collect::<Result<_, String>>()?,
            scenery_fades: self
                .scenery_fades
                .into_iter()
                .zip(subsets)
                .map(|((t, d), g)| Ok((g, texture(t)?, restore_domain(d)?)))
                .collect::<Result<_, String>>()?,
            background: self
                .background
                .map(|(m, t)| Ok::<_, String>((m.into_mesh()?, bindings.handle(t)?)))
                .transpose()?,
            cave_enclosure: self.enclosure.map(MeshWire::into_mesh).transpose()?,
            instances: self
                .instances
                .into_iter()
                .map(|(m, o)| {
                    Ok((
                        m.into_mesh()?,
                        o.into_iter().map(|v| v.map(f32::from_bits)).collect(),
                    ))
                })
                .collect::<Result<_, String>>()?,
            reveal_join_batches: self
                .joins
                .into_iter()
                .map(|(m, b)| {
                    Ok((
                        m.into_mesh()?,
                        crate::maze_reveal_batches::MazeRevealBatch {
                            min: Vec3::from_array(b[0].map(f32::from_bits)),
                            max: Vec3::from_array(b[1].map(f32::from_bits)),
                        },
                    ))
                })
                .collect::<Result<_, String>>()?,
            fade_textured_groups,
            fade_solid_groups,
            footing_heights: bytemuck::allocation::try_cast_vec(self.footing.into_vec()?)
                .map_err(|(error, _)| error.to_string())?,
            textured_meshes: self
                .textured
                .into_iter()
                .map(Batch::restore)
                .collect::<Result<_, _>>()?,
            solid_meshes: self
                .solid
                .into_iter()
                .map(Batch::restore)
                .collect::<Result<_, _>>()?,
            animated_textured_mesh: self.animated_textured.into_mesh()?,
            animated_solid_mesh: self.animated_solid.into_mesh()?,
            train_doors: self.train_doors,
            animated_pillars: self
                .pillars
                .into_iter()
                .map(|p| {
                    Ok([
                        usize::try_from(p[0]).map_err(|_| "pillar offset overflow")?,
                        usize::try_from(p[1]).map_err(|_| "pillar offset overflow")?,
                    ])
                })
                .collect::<Result<_, String>>()?,
        })
    }
}
