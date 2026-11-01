//! Incremental CPU object assembly. A completed region is published atomically.
use super::*;
use crate::occluder_fade::worker_wire::GroupRestore;
use bevy::platform::time::Instant;
use std::collections::VecDeque;

pub(crate) struct Restore {
    groups: Option<GroupRestore>,
    group_lengths: Vec<u64>,
    textures: VecDeque<TextureWire>,
    ready_textures: Vec<Arc<crate::scenery_surfaces::SceneryTexture>>,
    scenery_fades: Vec<(u32, u8)>,
    ready_groups: VecDeque<(Vec<crate::occluder_fade::PreparedGroup>, u32, u8)>,
    scenery: VecDeque<(Batch, u32)>,
    textured: VecDeque<Batch>,
    solid: VecDeque<Batch>,
    background: Option<(MeshWire, u32)>,
    enclosure: Option<MeshWire>,
    instances: VecDeque<(MeshWire, Vec<[u32; 3]>)>,
    joins: VecDeque<(MeshWire, [[u32; 3]; 2])>,
    animated_textured: Option<MeshWire>,
    animated_solid: Option<MeshWire>,
    footing: Option<Bulk<u32>>,
    pillars: Option<Vec<[u64; 2]>>,
    streamed: Option<crate::mesh::worker_wire::StreamWire>,
    built: BuiltTerrain,
}

impl Restore {
    pub(crate) fn new(response: Response) -> Result<Self, String> {
        if response.group_lengths.len() != response.scenery_fades.len() + 2 {
            return Err("terrain group subset count mismatch".into());
        }
        fn empty() -> Mesh {
            crate::mesh::SurfaceMeshData::default().into_mesh()
        }
        Ok(Self {
            groups: Some(response.groups.begin_restore()),
            group_lengths: response.group_lengths,
            textures: response.textures.into(),
            ready_textures: Vec::new(),
            scenery_fades: response.scenery_fades,
            ready_groups: VecDeque::new(),
            scenery: response.scenery.into(),
            textured: response.textured.into(),
            solid: response.solid.into(),
            background: response.background,
            enclosure: response.enclosure,
            instances: response.instances.into(),
            joins: response.joins.into(),
            animated_textured: Some(response.animated_textured),
            animated_solid: Some(response.animated_solid),
            footing: Some(response.footing),
            pillars: Some(response.pillars),
            streamed: response.streamed,
            built: BuiltTerrain {
                city_emitters: response.city_emitters.into_iter().map(|e|crate::city_lights::Emitter {
                    position:Vec4::from_array(e[0].map(f32::from_bits)),
                    direction:Vec4::from_array(e[1].map(f32::from_bits)),
                }).collect(),
                streamed_groves: None,
                scenery_fades: Vec::new(),
                scenery: Vec::new(),
                background: None,
                cave_enclosure: None,
                instances: Vec::new(),
                reveal_join_batches: Vec::new(),
                fade_textured_groups: Vec::new(),
                fade_solid_groups: Vec::new(),
                footing_heights: Vec::new(),
                textured_meshes: Vec::new(),
                solid_meshes: Vec::new(),
                animated_textured_mesh: empty(),
                animated_solid_mesh: empty(),
                animated_pillars: Vec::new(),
                train_doors: response.train_doors,
            },
        })
    }

    fn texture(&self, slot: u32) -> Result<Arc<crate::scenery_surfaces::SceneryTexture>, String> {
        self.ready_textures
            .get(slot as usize)
            .cloned()
            .ok_or_else(|| "invalid scenery texture slot".into())
    }

    pub(crate) fn advance(
        &mut self,
        blobs: &mut BlobStore,
        bindings: &TextureBindings,
        deadline: Instant,
        max_elements: usize,
    ) -> Result<bool, String> {
        if let Some(groups) = &mut self.groups {
            if groups.advance(blobs, deadline, max_elements)? {
                let mut groups = self.groups.take().unwrap().finish()?.into_iter();
                let mut subsets = Vec::new();
                for &count in &self.group_lengths {
                    let count =
                        usize::try_from(count).map_err(|_| "terrain group count overflow")?;
                    if count > groups.len() {
                        return Err("terrain group range exceeds table".into());
                    }
                    subsets.push(groups.by_ref().take(count).collect::<Vec<_>>());
                }
                if groups.next().is_some() {
                    return Err("unreferenced terrain groups".into());
                }
                let mut subsets = subsets.into_iter();
                self.built.fade_textured_groups =
                    subsets.next().ok_or("missing textured groups")?;
                self.built.fade_solid_groups = subsets.next().ok_or("missing solid groups")?;
                for ((texture, domain), groups) in self.scenery_fades.drain(..).zip(subsets) {
                    self.ready_groups.push_back((groups, texture, domain));
                }
            }
        }
        while Instant::now() < deadline {
            if let Some(texture) = self.textures.front_mut() {
                if !texture.resolve(blobs)? {
                    break;
                }
                self.ready_textures
                    .push(self.textures.pop_front().unwrap().restore()?);
                continue;
            }
            if let Some((groups, texture, domain)) = self.ready_groups.pop_front() {
                self.built.scenery_fades.push((
                    groups,
                    self.texture(texture)?,
                    restore_domain(domain)?,
                ));
                continue;
            }
            if let Some(batch) = self.textured.front_mut() {
                if !batch.resolve(blobs)? {
                    break;
                }
                self.built
                    .textured_meshes
                    .push(self.textured.pop_front().unwrap().restore()?);
                continue;
            }
            if let Some(batch) = self.solid.front_mut() {
                if !batch.resolve(blobs)? {
                    break;
                }
                self.built
                    .solid_meshes
                    .push(self.solid.pop_front().unwrap().restore()?);
                continue;
            }
            if let Some((batch, texture)) = self.scenery.front_mut() {
                if !batch.resolve(blobs)? {
                    break;
                }
                let slot = *texture;
                let texture = self.texture(slot)?;
                let (batch, _) = self.scenery.pop_front().unwrap();
                self.built.scenery.push((batch.restore()?, texture));
                continue;
            }
            if let Some((mesh, _)) = &mut self.background {
                if !mesh.resolve(blobs)? {
                    break;
                }
                let (mesh, slot) = self.background.take().unwrap();
                self.built.background = Some((mesh.into_mesh()?, bindings.handle(slot)?));
                continue;
            }
            if let Some(mesh) = &mut self.enclosure {
                if !mesh.resolve(blobs)? {
                    break;
                }
                self.built.cave_enclosure = Some(self.enclosure.take().unwrap().into_mesh()?);
                continue;
            }
            if let Some((mesh, _)) = self.instances.front_mut() {
                if !mesh.resolve(blobs)? {
                    break;
                }
                let (mesh, origins) = self.instances.pop_front().unwrap();
                self.built.instances.push((
                    mesh.into_mesh()?,
                    origins.into_iter().map(|v| v.map(f32::from_bits)).collect(),
                ));
                continue;
            }
            if let Some((mesh, _)) = self.joins.front_mut() {
                if !mesh.resolve(blobs)? {
                    break;
                }
                let (mesh, bounds) = self.joins.pop_front().unwrap();
                self.built.reveal_join_batches.push((
                    mesh.into_mesh()?,
                    crate::maze_reveal_batches::MazeRevealBatch {
                        min: Vec3::from_array(bounds[0].map(f32::from_bits)),
                        max: Vec3::from_array(bounds[1].map(f32::from_bits)),
                    },
                ));
                continue;
            }
            if let Some(mesh) = &mut self.animated_textured {
                if !mesh.resolve(blobs)? {
                    break;
                }
                self.built.animated_textured_mesh =
                    self.animated_textured.take().unwrap().into_mesh()?;
                continue;
            }
            if let Some(mesh) = &mut self.animated_solid {
                if !mesh.resolve(blobs)? {
                    break;
                }
                self.built.animated_solid_mesh = self.animated_solid.take().unwrap().into_mesh()?;
                continue;
            }
            if let Some(footing) = &mut self.footing {
                if !footing.resolve(blobs)? {
                    break;
                }
                self.built.footing_heights =
                    bytemuck::allocation::try_cast_vec(self.footing.take().unwrap().into_vec()?)
                        .map_err(|(e, _)| e.to_string())?;
                continue;
            }
            if let Some(pillars) = self.pillars.take() {
                self.built.animated_pillars = pillars
                    .into_iter()
                    .map(|p| {
                        Ok([
                            usize::try_from(p[0]).map_err(|_| "pillar offset overflow")?,
                            usize::try_from(p[1]).map_err(|_| "pillar offset overflow")?,
                        ])
                    })
                    .collect::<Result<_, String>>()?;
                continue;
            }
            if let Some(stream) = self.streamed.as_mut() {
                if !stream.resolve(blobs, deadline, max_elements)? {
                    break;
                }
            }
            if let Some(stream) = self.streamed.take() {
                self.built.streamed_groves = Some(stream.restore(bindings)?);
                continue;
            }
            break;
        }
        Ok(self.groups.is_none()
            && self.textures.is_empty()
            && self.scenery.is_empty()
            && self.textured.is_empty()
            && self.solid.is_empty()
            && self.background.is_none()
            && self.enclosure.is_none()
            && self.instances.is_empty()
            && self.joins.is_empty()
            && self.animated_textured.is_none()
            && self.animated_solid.is_none()
            && self.ready_groups.is_empty()
            && self.footing.is_none()
            && self.pillars.is_none()
            && self.streamed.is_none())
    }
    pub(crate) fn finish(self) -> BuiltTerrain {
        self.built
    }
}
