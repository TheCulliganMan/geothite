//! Identical authored geometry is shared across model families and palettes.
//! Palette, UV and semantic ownership remain local to each asset variant.
use crate::mesh::SurfaceMeshData;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, ops::Deref, sync::{Arc, Mutex, OnceLock, Weak}};

#[derive(Debug, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) positions: Vec<[f32; 3]>,
    pub(crate) normals: Vec<[f32; 3]>,
    pub(crate) indices: Vec<u32>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ModelSurface {
    pub(crate) geometry: Arc<Geometry>,
    pub(crate) uvs: Vec<[f32; 2]>,
    pub(crate) colors: Vec<[f32; 4]>,
    pub(crate) cutaway_ranges: Vec<std::ops::Range<usize>>,
}
impl Deref for ModelSurface {
    type Target = Geometry;
    fn deref(&self) -> &Geometry { &self.geometry }
}
impl From<SurfaceMeshData> for ModelSurface {
    fn from(surface: SurfaceMeshData) -> Self {
        let SurfaceMeshData { positions, normals, indices, uvs, colors, cutaway_ranges }=surface;
        let candidate=Geometry { positions,normals,indices };
        let mut digest=Sha256::new();
        for values in [&candidate.positions,&candidate.normals] {
            digest.update((values.len() as u64).to_le_bytes());
            for v in values {for value in v {digest.update(value.to_bits().to_le_bytes());}}
        }
        digest.update((candidate.indices.len() as u64).to_le_bytes());
        for index in &candidate.indices {digest.update(index.to_le_bytes());}
        let key:[u8;32]=digest.finalize().into();
        static CACHE:OnceLock<Mutex<HashMap<[u8;32],Vec<Weak<Geometry>>>>>=OnceLock::new();
        let mut cache=CACHE.get_or_init(||Mutex::new(HashMap::new())).lock().unwrap();
        cache.retain(|_,entries| {entries.retain(|entry|entry.strong_count()>0);!entries.is_empty()});
        let entries=cache.entry(key).or_default();
        // A hash is a lookup hint only; exact geometry equality authorizes reuse.
        let geometry=entries.iter().filter_map(Weak::upgrade).find(|existing|**existing==candidate)
            .unwrap_or_else(|| {let geometry=Arc::new(candidate);entries.push(Arc::downgrade(&geometry));geometry});
        Self {geometry,uvs,colors,cutaway_ranges}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_assets_reuse_geometry_without_sharing_palette_or_uv_ownership() {
        let source=SurfaceMeshData {positions:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]],
            normals:vec![[0.,0.,1.];3],indices:vec![0,1,2],uvs:vec![[0.,0.];3],colors:vec![[1.,0.,0.,1.];3],..Default::default()};
        let first=ModelSurface::from(source.clone());
        let mut variant=source.clone();variant.colors.fill([0.,0.,1.,1.]);variant.uvs.fill([0.2,0.4]);
        variant.cutaway_ranges.push(0..3);
        let second=ModelSurface::from(variant);
        assert!(Arc::ptr_eq(&first.geometry,&second.geometry));
        assert_ne!(first.colors,second.colors);assert_ne!(first.uvs,second.uvs);
        assert!(first.cutaway_ranges.is_empty());assert_eq!(second.cutaway_ranges,vec![0..3]);
        let mut different=source;different.positions[1][0]=1.25;
        let third=ModelSurface::from(different);
        assert!(!Arc::ptr_eq(&first.geometry,&third.geometry));
        assert_eq!(first.positions[1],[1.,0.,0.]);
    }
}
