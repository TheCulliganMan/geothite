//! Exact CPU-only occluder transfer. Asset handles never cross the worker boundary.
use super::*;
use crate::terrain_worker::{
    blobs::{BlobStore, Bulk},
    mesh_buffers::MeshWire,
};
use bevy::platform::time::Instant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

const MAX_OVERLAY_DEPTH: usize = 64;

/// One table must cover all related group subsets so shared prototype geometry
/// and collision surfaces remain shared after the main thread restores them.
#[derive(Serialize, Deserialize)]
pub(crate) struct GroupTable {
    groups: Vec<GroupWire>,
    surfaces: Vec<SurfaceWire>,
    shared_leaves: Vec<SharedWire>,
}

#[derive(Serialize, Deserialize)]
struct GroupWire {
    source_range: [u64; 2],
    surface: u32,
    leaves: Vec<LeafWire>,
    opaque_leaf: Option<LeafWire>,
    overlay: Option<Box<GroupWire>>,
    transform: TransformWire,
    coverage_fade: bool,
    depth_coverage_fade: bool,
    residual_opacity: u32,
    camera_wall: Option<[u32;3]>,
    runtime_lazy_sorted: bool,
}

#[derive(Serialize, Deserialize)]
struct LeafWire {
    mesh: MeshWire,
    center: [u32; 3],
    bounds_center: [u32; 3],
    bounds_half_extents: [u32; 3],
    shared: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct SharedWire {
    mesh: MeshWire,
}

#[derive(Serialize, Deserialize)]
struct TransformWire {
    translation: [u32; 3],
    rotation: [u32; 4],
    scale: [u32; 3],
}

#[derive(Serialize, Deserialize)]
struct BoundsWire {
    min: [u32; 3],
    max: [u32; 3],
}

#[derive(Serialize, Deserialize)]
struct SurfaceWire {
    bounds: Option<BoundsWire>,
    state: SurfaceState,
}

#[derive(Serialize, Deserialize)]
enum SurfaceState {
    Ready(TreeWire),
    Deferred(SourceWire),
}

#[derive(Serialize, Deserialize)]
struct SourceWire {
    positions: Bulk<[u32; 3]>,
    normals: Bulk<[u32; 3]>,
    uvs: Bulk<[u32; 2]>,
    colors: Bulk<[u32; 4]>,
    indices: Bulk<u32>,
    cutaway_ranges: Bulk<[u64; 2]>,
}

#[derive(Serialize, Deserialize)]
struct TreeWire {
    positions: Bulk<[u32; 3]>,
    triangles: TrianglesWire,
    nodes: NodeStorage,
}

#[derive(Serialize, Deserialize)]
enum TrianglesWire {
    Packed {
        triangles: Bulk<[u16; 2]>,
        exceptions: Bulk<[u16; 3]>,
    },
    Narrow(Bulk<[u16; 3]>),
    Wide(Bulk<[u32; 3]>),
}

#[derive(Serialize, Deserialize)]
struct NodeWire {
    bounds: BoundsWire,
    triangles: [u64; 2],
    // None preserves the local usize::MAX sentinel across 32/64-bit hosts.
    children: [Option<u32>; 2],
}

enum NodeStorage {
    Plain(Vec<NodeWire>),
    Paged(Bulk<[u64; 7]>),
}
impl NodeWire {
    fn record(&self) -> [u64; 7] {
        let bits = [
            self.bounds.min[0],
            self.bounds.min[1],
            self.bounds.min[2],
            self.bounds.max[0],
            self.bounds.max[1],
            self.bounds.max[2],
        ];
        [
            u64::from(bits[0]) | (u64::from(bits[1]) << 32),
            u64::from(bits[2]) | (u64::from(bits[3]) << 32),
            u64::from(bits[4]) | (u64::from(bits[5]) << 32),
            self.triangles[0],
            self.triangles[1],
            self.children[0].map_or(u64::MAX, u64::from),
            self.children[1].map_or(u64::MAX, u64::from),
        ]
    }
    fn from_record(v: [u64; 7]) -> Result<Self, String> {
        let child = |v| {
            if v == u64::MAX {
                Ok(None)
            } else {
                u32::try_from(v)
                    .map(Some)
                    .map_err(|_| "collision child ordinal overflow".to_owned())
            }
        };
        Ok(Self {
            bounds: BoundsWire {
                min: [v[0] as u32, (v[0] >> 32) as u32, v[1] as u32],
                max: [(v[1] >> 32) as u32, v[2] as u32, (v[2] >> 32) as u32],
            },
            triangles: [v[3], v[4]],
            children: [child(v[5])?, child(v[6])?],
        })
    }
}
impl Serialize for NodeStorage {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if crate::terrain_worker::blobs::header_encoding_active() {
            match self {
                Self::Plain(nodes) => nodes
                    .iter()
                    .map(NodeWire::record)
                    .collect::<Bulk<_>>()
                    .serialize(serializer),
                Self::Paged(nodes) => nodes.serialize(serializer),
            }
        } else {
            match self {
                Self::Plain(nodes) => nodes.serialize(serializer),
                Self::Paged(nodes) => nodes
                    .as_slice()
                    .map_err(serde::ser::Error::custom)?
                    .iter()
                    .copied()
                    .map(NodeWire::from_record)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::ser::Error::custom)?
                    .serialize(serializer),
            }
        }
    }
}
impl<'de> Deserialize<'de> for NodeStorage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if crate::terrain_worker::blobs::header_decoding_active() {
            Bulk::deserialize(deserializer).map(Self::Paged)
        } else {
            Vec::deserialize(deserializer).map(Self::Plain)
        }
    }
}
impl NodeStorage {
    fn resolve(&mut self, store: &mut BlobStore) -> Result<bool, String> {
        match self {
            Self::Plain(_) => Ok(true),
            Self::Paged(v) => v.resolve(store),
        }
    }
    #[cfg(test)]
    fn into_nodes(self) -> Result<Vec<NodeWire>, String> {
        match self {
            Self::Plain(v) => Ok(v),
            Self::Paged(v) => v
                .into_vec()?
                .into_iter()
                .map(NodeWire::from_record)
                .collect(),
        }
    }
    fn into_iter(self) -> Result<NodeIterator, String> {
        Ok(match self {
            Self::Plain(v) => NodeIterator::Plain(v.into_iter()),
            Self::Paged(v) => NodeIterator::Paged(v.into_vec()?.into_iter()),
        })
    }
}
enum NodeIterator {
    Plain(std::vec::IntoIter<NodeWire>),
    Paged(std::vec::IntoIter<[u64; 7]>),
}
impl NodeIterator {
    fn len(&self) -> usize {
        match self {
            Self::Plain(v) => v.len(),
            Self::Paged(v) => v.len(),
        }
    }
    fn next(&mut self) -> Option<Result<NodeWire, String>> {
        match self {
            Self::Plain(v) => v.next().map(Ok),
            Self::Paged(v) => v.next().map(NodeWire::from_record),
        }
    }
}

fn mesh_wire(mesh: &Mesh) -> Result<MeshWire, String> {
    let _ = mesh
        .try_attributes()
        .map_err(|_| "terrain mesh CPU attributes have moved to GPU")?;
    mesh.try_indices_option()
        .map_err(|_| "terrain mesh CPU indices have moved to GPU")?;
    MeshWire::capture(mesh)
}

fn vec3_bits(value: Vec3) -> [u32; 3] {
    value.to_array().map(f32::to_bits)
}
fn vec3(value: [u32; 3]) -> Vec3 {
    Vec3::from_array(value.map(f32::from_bits))
}
fn float_arrays<const N: usize>(values: Vec<[u32; N]>) -> Result<Vec<[f32; N]>, String>
where
    [u32; N]: bytemuck::Pod,
    [f32; N]: bytemuck::Pod,
{
    // Equal-size, equal-alignment bit arrays transfer their allocation while
    // preserving signed zero and NaN payloads without a full-buffer copy.
    bytemuck::allocation::try_cast_vec(values)
        .map_err(|(error, _)| format!("invalid collision float-buffer layout: {error}"))
}
fn vec3_positions(values: Vec<[u32; 3]>) -> Vec<Vec3> {
    // Current Bevy Vec3 has the same Pod layout as [u32; 3]. Keep a safe
    // fallback if a future engine changes its size or alignment.
    match bytemuck::allocation::try_cast_vec(values) {
        Ok(positions) => positions,
        Err((_, values)) => values.into_iter().map(vec3).collect(),
    }
}
fn child_wire(child: usize) -> Result<Option<u32>, String> {
    if child == UNOWNED {
        Ok(None)
    } else {
        u32::try_from(child)
            .map(Some)
            .map_err(|_| "collision child ordinal overflow".to_owned())
    }
}
fn range_wire(value: &Range<usize>) -> Result<[u64; 2], String> {
    Ok([
        u64::try_from(value.start).map_err(|_| "range start overflow")?,
        u64::try_from(value.end).map_err(|_| "range end overflow")?,
    ])
}
fn range(value: [u64; 2]) -> Result<Range<usize>, String> {
    let start = usize::try_from(value[0]).map_err(|_| "range start exceeds local address space")?;
    let end = usize::try_from(value[1]).map_err(|_| "range end exceeds local address space")?;
    if start > end {
        return Err("inverted occluder range".into());
    }
    Ok(start..end)
}

impl BoundsWire {
    fn capture(value: Bounds) -> Self {
        Self {
            min: vec3_bits(value.min),
            max: vec3_bits(value.max),
        }
    }
    fn restore(self) -> Result<Bounds, String> {
        let bounds = Bounds {
            min: vec3(self.min),
            max: vec3(self.max),
        };
        if !bounds.min.is_finite() || !bounds.max.is_finite() || !bounds.min.cmple(bounds.max).all()
        {
            return Err("invalid collision bounds".into());
        }
        Ok(bounds)
    }
}

impl SourceWire {
    fn resolve(&mut self, store: &mut BlobStore) -> Result<bool, String> {
        let mut ready = true;
        ready &= self.positions.resolve(store)?;
        ready &= self.normals.resolve(store)?;
        ready &= self.uvs.resolve(store)?;
        ready &= self.colors.resolve(store)?;
        ready &= self.indices.resolve(store)?;
        ready &= self.cutaway_ranges.resolve(store)?;
        Ok(ready)
    }
    fn capture(source: &SurfaceMeshData) -> Result<Self, String> {
        Ok(Self {
            positions: source
                .positions
                .iter()
                .map(|v| v.map(f32::to_bits))
                .collect(),
            normals: source.normals.iter().map(|v| v.map(f32::to_bits)).collect(),
            uvs: source.uvs.iter().map(|v| v.map(f32::to_bits)).collect(),
            colors: source.colors.iter().map(|v| v.map(f32::to_bits)).collect(),
            indices: source.indices.clone().into(),
            cutaway_ranges: source
                .cutaway_ranges
                .iter()
                .map(range_wire)
                .collect::<Result<_, _>>()?,
        })
    }
    #[cfg(test)]
    fn restore(self) -> Result<SurfaceMeshData, String> {
        let positions = float_arrays(self.positions.into_vec()?)?;
        let indices = self.indices.into_vec()?;
        if indices.len() % 3 != 0
            || indices.iter().any(|&i| i as usize >= positions.len())
            || positions.iter().any(|&p| !Vec3::from_array(p).is_finite())
        {
            return Err("invalid deferred collision geometry".into());
        }
        let cutaway_ranges = self
            .cutaway_ranges
            .into_vec()?
            .into_iter()
            .map(range)
            .collect::<Result<Vec<_>, _>>()?;
        if cutaway_ranges.iter().any(|r| r.end > positions.len()) {
            return Err("deferred source ownership exceeds vertices".into());
        }
        Ok(SurfaceMeshData {
            positions,
            normals: float_arrays(self.normals.into_vec()?)?,
            uvs: float_arrays(self.uvs.into_vec()?)?,
            colors: float_arrays(self.colors.into_vec()?)?,
            indices,
            cutaway_ranges,
        })
    }
}

impl TreeWire {
    fn resolve(&mut self, store: &mut BlobStore) -> Result<bool, String> {
        let mut ready = self.positions.resolve(store)?;
        ready &= self.nodes.resolve(store)?;
        ready &= match &mut self.triangles {
            TrianglesWire::Packed {
                triangles,
                exceptions,
            } => {
                let a = triangles.resolve(store)?;
                let b = exceptions.resolve(store)?;
                a && b
            }
            TrianglesWire::Narrow(v) => v.resolve(store)?,
            TrianglesWire::Wide(v) => v.resolve(store)?,
        };
        Ok(ready)
    }
    fn capture(tree: &TriangleTree) -> Result<Self, String> {
        let triangles = match &tree.triangles {
            TriangleIndices::Packed {
                triangles,
                exceptions,
            } => TrianglesWire::Packed {
                triangles: triangles.clone().into(),
                exceptions: exceptions.clone().into(),
            },
            TriangleIndices::Narrow(values) => TrianglesWire::Narrow(values.clone().into()),
            TriangleIndices::Wide(values) => TrianglesWire::Wide(values.clone().into()),
        };
        let nodes = tree
            .nodes
            .iter()
            .map(|node| {
                Ok(NodeWire {
                    bounds: BoundsWire::capture(node.bounds),
                    triangles: range_wire(&node.triangles)?,
                    children: [child_wire(node.children[0])?, child_wire(node.children[1])?],
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            positions: tree.positions.iter().map(|&p| vec3_bits(p)).collect(),
            triangles,
            nodes: NodeStorage::Plain(nodes),
        })
    }
    #[cfg(test)]
    fn restore(self) -> Result<TriangleTree, String> {
        let positions = vec3_positions(self.positions.into_vec()?);
        if positions.iter().any(|p| !p.is_finite()) {
            return Err("invalid BVH positions".into());
        }
        let node_wires = self.nodes.into_nodes()?;
        let triangles = match self.triangles {
            TrianglesWire::Packed {
                triangles,
                exceptions,
            } => {
                let triangles = triangles.into_vec()?;
                let exceptions = exceptions.into_vec()?;
                if triangles.iter().any(|&[first, entry]| {
                    first == u16::MAX && usize::from(entry) >= exceptions.len()
                }) {
                    return Err("invalid packed collision exception".into());
                }
                TriangleIndices::Packed {
                    triangles,
                    exceptions,
                }
            }
            TrianglesWire::Narrow(values) => TriangleIndices::Narrow(values.into_vec()?),
            TrianglesWire::Wide(values) => TriangleIndices::Wide(values.into_vec()?),
        };
        if (0..triangles.len()).any(|i| {
            triangles
                .triangle(i)
                .iter()
                .any(|&corner| corner as usize >= positions.len())
        }) {
            return Err("collision triangle exceeds position buffer".into());
        }
        let node_count = node_wires.len();
        if node_wires.iter().any(|node| {
            node.children
                .iter()
                .flatten()
                .any(|&child| child as usize >= node_count)
        }) {
            return Err("collision child ordinal exceeds node table".into());
        }
        let nodes: Vec<_> = node_wires
            .into_iter()
            .map(|node| {
                Ok(Node {
                    bounds: node.bounds.restore()?,
                    triangles: range(node.triangles)?,
                    children: node
                        .children
                        .map(|child| child.map_or(UNOWNED, |i| i as usize)),
                })
            })
            .collect::<Result<_, String>>()?;
        if triangles.is_empty() != nodes.is_empty() {
            return Err("collision tree root does not match triangle storage".into());
        }
        if !nodes.is_empty() {
            if nodes[0].triangles != (0..triangles.len()) {
                return Err("collision root does not cover every triangle".into());
            }
            let mut seen = vec![false; nodes.len()];
            let mut pending = vec![0usize];
            while let Some(index) = pending.pop() {
                if seen[index] {
                    return Err("collision tree shares or cycles nodes".into());
                }
                seen[index] = true;
                let node = &nodes[index];
                if node.triangles.end > triangles.len() {
                    return Err("collision node range exceeds triangles".into());
                }
                match node.children {
                    [UNOWNED, UNOWNED] => {}
                    [a, b]
                        if a != UNOWNED
                            && b != UNOWNED
                            && a > index
                            && b > index
                            && a < nodes.len()
                            && b < nodes.len() =>
                    {
                        let left = &nodes[a].triangles;
                        let right = &nodes[b].triangles;
                        if left.start != node.triangles.start
                            || left.end != right.start
                            || right.end != node.triangles.end
                        {
                            return Err("collision children do not partition their parent".into());
                        }
                        pending.extend([a, b]);
                    }
                    _ => return Err("invalid collision children".into()),
                }
            }
            if seen.iter().any(|seen| !seen) {
                return Err("unreachable collision node".into());
            }
        }
        Ok(TriangleTree {
            positions,
            triangles,
            nodes,
        })
    }
}

enum SurfaceCursor {
    Deferred {
        source: SurfaceMeshData,
        ranges: std::vec::IntoIter<[u64; 2]>,
        phase: u8,
        cursor: usize,
    },
    Ready {
        tree: TriangleTree,
        input_nodes: NodeIterator,
        node_count: usize,
        seen: Vec<bool>,
        pending: Vec<usize>,
        visited: usize,
        phase: u8,
        cursor: usize,
    },
}
impl SurfaceCursor {
    fn new(state: SurfaceState) -> Result<Self, String> {
        Ok(match state {
            SurfaceState::Deferred(source) => {
                let indices = source.indices.into_vec()?;
                if indices.len() % 3 != 0 {
                    return Err("invalid deferred collision geometry".into());
                }
                Self::Deferred {
                    source: SurfaceMeshData {
                        positions: float_arrays(source.positions.into_vec()?)?,
                        normals: float_arrays(source.normals.into_vec()?)?,
                        uvs: float_arrays(source.uvs.into_vec()?)?,
                        colors: float_arrays(source.colors.into_vec()?)?,
                        indices,
                        cutaway_ranges: Vec::new(),
                    },
                    ranges: source.cutaway_ranges.into_vec()?.into_iter(),
                    phase: 0,
                    cursor: 0,
                }
            }
            SurfaceState::Ready(tree) => {
                let triangles = match tree.triangles {
                    TrianglesWire::Packed {
                        triangles,
                        exceptions,
                    } => TriangleIndices::Packed {
                        triangles: triangles.into_vec()?,
                        exceptions: exceptions.into_vec()?,
                    },
                    TrianglesWire::Narrow(v) => TriangleIndices::Narrow(v.into_vec()?),
                    TrianglesWire::Wide(v) => TriangleIndices::Wide(v.into_vec()?),
                };
                let input_nodes = tree.nodes.into_iter()?;
                let node_count = input_nodes.len();
                if triangles.is_empty() != (node_count == 0) {
                    return Err("collision tree root does not match triangle storage".into());
                }
                Self::Ready {
                    tree: TriangleTree {
                        positions: vec3_positions(tree.positions.into_vec()?),
                        triangles,
                        nodes: Vec::with_capacity(node_count),
                    },
                    input_nodes,
                    node_count,
                    seen: Vec::with_capacity(node_count),
                    pending: Vec::new(),
                    visited: 0,
                    phase: 0,
                    cursor: 0,
                }
            }
        })
    }
    // Exactly one bounded validation/conversion operation. Transitions are also
    // charged, so empty/malformed metadata cannot monopolize a frame.
    fn step(&mut self) -> Result<bool, String> {
        match self {
            Self::Deferred {
                source,
                ranges,
                phase,
                cursor,
            } => match *phase {
                0 => {
                    if let Some(p) = source.positions.get(*cursor) {
                        if !Vec3::from_array(*p).is_finite() {
                            return Err("invalid deferred collision geometry".into());
                        }
                        *cursor += 1;
                    } else {
                        *phase = 1;
                        *cursor = 0;
                    }
                }
                1 => {
                    if let Some(&i) = source.indices.get(*cursor) {
                        if i as usize >= source.positions.len() {
                            return Err("invalid deferred collision geometry".into());
                        }
                        *cursor += 1;
                    } else {
                        *phase = 2;
                    }
                }
                _ => {
                    if let Some(r) = ranges.next() {
                        let r = range(r)?;
                        if r.end > source.positions.len() {
                            return Err("deferred source ownership exceeds vertices".into());
                        }
                        source.cutaway_ranges.push(r);
                    } else {
                        return Ok(true);
                    }
                }
            },
            Self::Ready {
                tree,
                input_nodes,
                node_count,
                seen,
                pending,
                visited,
                phase,
                cursor,
            } => match *phase {
                0 => {
                    if let Some(p) = tree.positions.get(*cursor) {
                        if !p.is_finite() {
                            return Err("invalid BVH positions".into());
                        }
                        *cursor += 1;
                    } else {
                        *phase = 1;
                        *cursor = 0;
                    }
                }
                1 => {
                    if *cursor < tree.triangles.len() {
                        if let TriangleIndices::Packed {
                            triangles,
                            exceptions,
                        } = &tree.triangles
                        {
                            let [first, entry] = triangles[*cursor];
                            if first == u16::MAX && usize::from(entry) >= exceptions.len() {
                                return Err("invalid packed collision exception".into());
                            }
                        }
                        if tree
                            .triangles
                            .triangle(*cursor)
                            .iter()
                            .any(|&i| i as usize >= tree.positions.len())
                        {
                            return Err("collision triangle exceeds position buffer".into());
                        }
                        *cursor += 1;
                    } else {
                        *phase = 2;
                    }
                }
                2 => {
                    if let Some(node) = input_nodes.next() {
                        let node = node?;
                        if node
                            .children
                            .iter()
                            .flatten()
                            .any(|&i| i as usize >= *node_count)
                        {
                            return Err("collision child ordinal exceeds node table".into());
                        }
                        tree.nodes.push(Node {
                            bounds: node.bounds.restore()?,
                            triangles: range(node.triangles)?,
                            children: node.children.map(|i| i.map_or(UNOWNED, |i| i as usize)),
                        });
                        seen.push(false);
                    } else {
                        if let Some(root) = tree.nodes.first() {
                            if root.triangles != (0..tree.triangles.len()) {
                                return Err("collision root does not cover every triangle".into());
                            }
                            pending.push(0);
                        }
                        *phase = 3;
                    }
                }
                _ => {
                    if let Some(i) = pending.pop() {
                        if seen[i] {
                            return Err("collision tree shares or cycles nodes".into());
                        }
                        seen[i] = true;
                        *visited += 1;
                        let node = &tree.nodes[i];
                        if node.triangles.end > tree.triangles.len() {
                            return Err("collision node range exceeds triangles".into());
                        }
                        match node.children {
                            [UNOWNED, UNOWNED] => {}
                            [a, b]
                                if a != UNOWNED
                                    && b != UNOWNED
                                    && a > i
                                    && b > i
                                    && a < tree.nodes.len()
                                    && b < tree.nodes.len() =>
                            {
                                let left = &tree.nodes[a].triangles;
                                let right = &tree.nodes[b].triangles;
                                if left.start != node.triangles.start
                                    || left.end != right.start
                                    || right.end != node.triangles.end
                                {
                                    return Err(
                                        "collision children do not partition their parent".into()
                                    );
                                }
                                pending.extend([a, b]);
                            }
                            _ => return Err("invalid collision children".into()),
                        }
                    } else {
                        if *visited != tree.nodes.len() {
                            return Err("unreachable collision node".into());
                        }
                        return Ok(true);
                    }
                }
            },
        }
        Ok(false)
    }
    fn finish(self, bounds: Option<Bounds>) -> CollisionSurface {
        let (tree, source) = match self {
            Self::Ready { tree, .. } => (OnceLock::from(tree), None),
            Self::Deferred { source, .. } => (OnceLock::new(), Some(source)),
        };
        CollisionSurface {
            bounds,
            tree,
            source: Mutex::new(source),
        }
    }
}

#[derive(Default)]
struct Capture {
    surfaces: Vec<SurfaceWire>,
    surface_slots: HashMap<usize, u32>,
    shared_leaves: Vec<SharedWire>,
    shared_slots: HashMap<usize, u32>,
}

impl Capture {
    fn surface(&mut self, surface: &Arc<CollisionSurface>) -> Result<u32, String> {
        let key = Arc::as_ptr(surface) as usize;
        if let Some(&slot) = self.surface_slots.get(&key) {
            return Ok(slot);
        }
        let source = surface
            .source
            .lock()
            .map_err(|_| "poisoned collision source")?;
        let state = match (surface.tree.get(), source.as_ref()) {
            (Some(tree), None) => SurfaceState::Ready(TreeWire::capture(tree)?),
            (None, Some(source)) => SurfaceState::Deferred(SourceWire::capture(source)?),
            _ => return Err("collision state is incomplete or being constructed".into()),
        };
        let slot = u32::try_from(self.surfaces.len()).map_err(|_| "too many collision surfaces")?;
        self.surfaces.push(SurfaceWire {
            bounds: surface.bounds.map(BoundsWire::capture),
            state,
        });
        self.surface_slots.insert(key, slot);
        Ok(slot)
    }
    fn shared(&mut self, shared: &Arc<Mutex<SharedLeaf>>) -> Result<u32, String> {
        let key = Arc::as_ptr(shared) as usize;
        if let Some(&slot) = self.shared_slots.get(&key) {
            return Ok(slot);
        }
        let shared = shared.lock().map_err(|_| "poisoned shared terrain mesh")?;
        if shared.uploaded.is_some() {
            return Err("uploaded mesh handle cannot cross terrain worker".into());
        }
        let mesh = shared
            .mesh
            .as_ref()
            .ok_or("shared terrain mesh has no CPU buffers")?;
        let slot = u32::try_from(self.shared_leaves.len())
            .map_err(|_| "too many shared terrain meshes")?;
        self.shared_leaves.push(SharedWire {
            mesh: mesh_wire(mesh)?,
        });
        self.shared_slots.insert(key, slot);
        Ok(slot)
    }
    fn leaf(&mut self, leaf: &PreparedLeaf) -> Result<LeafWire, String> {
        if leaf.uploaded.is_some() {
            return Err("uploaded leaf handle cannot cross terrain worker".into());
        }
        Ok(LeafWire {
            mesh: mesh_wire(&leaf.mesh)?,
            center: vec3_bits(leaf.center),
            bounds_center: Vec3::from(leaf.bounds.center).to_array().map(f32::to_bits),
            bounds_half_extents: Vec3::from(leaf.bounds.half_extents)
                .to_array()
                .map(f32::to_bits),
            shared: leaf
                .shared
                .as_ref()
                .map(|shared| self.shared(shared))
                .transpose()?,
        })
    }
    fn group(&mut self, group: &PreparedGroup, depth: usize) -> Result<GroupWire, String> {
        if depth > MAX_OVERLAY_DEPTH {
            return Err("terrain overlay nesting exceeds limit".into());
        }
        Ok(GroupWire {
            source_range: range_wire(&group.source_range)?,
            surface: self.surface(&group.surface)?,
            leaves: group
                .leaves
                .iter()
                .map(|leaf| self.leaf(leaf))
                .collect::<Result<_, _>>()?,
            opaque_leaf: group
                .opaque_leaf
                .as_ref()
                .map(|leaf| self.leaf(leaf))
                .transpose()?,
            overlay: group
                .overlay
                .as_ref()
                .map(|overlay| self.group(overlay, depth + 1).map(Box::new))
                .transpose()?,
            transform: TransformWire {
                translation: vec3_bits(group.transform.translation),
                rotation: group.transform.rotation.to_array().map(f32::to_bits),
                scale: vec3_bits(group.transform.scale),
            },
            coverage_fade: group.coverage_fade,
            depth_coverage_fade: group.depth_coverage_fade,
            residual_opacity: group.residual_opacity.to_bits(),
            camera_wall: group.camera_wall.map(|p|p.map(f32::to_bits)),
            runtime_lazy_sorted: group.runtime_lazy_sorted,
        })
    }
}

impl GroupTable {
    pub(crate) fn capture(groups: &[PreparedGroup]) -> Result<Self, String> {
        let mut capture = Capture::default();
        let groups = groups
            .iter()
            .map(|group| capture.group(group, 0))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            groups,
            surfaces: capture.surfaces,
            shared_leaves: capture.shared_leaves,
        })
    }
    pub(crate) fn begin_restore(self) -> GroupRestore {
        GroupRestore {
            surfaces_in: self.surfaces.into(),
            shared_in: self.shared_leaves.into(),
            groups_in: self.groups.into(),
            surfaces: Vec::new(),
            shared: Vec::new(),
            active_surface: None,
            frames: Vec::new(),
            output: Vec::new(),
            done: false,
        }
    }
    pub(crate) fn restore(self) -> Result<Vec<PreparedGroup>, String> {
        let mut restore = self.begin_restore();
        let mut store = BlobStore::default();
        if !restore.advance(
            &mut store,
            Instant::now() + std::time::Duration::from_secs(3600),
            usize::MAX,
        )? {
            return Err("terrain buffer pages are incomplete".into());
        }
        restore.finish()
    }
}

struct GroupFrame {
    wire: GroupWire,
    input_leaves: std::vec::IntoIter<LeafWire>,
    pending_leaf: Option<LeafWire>,
    leaves: Vec<PreparedLeaf>,
    opaque_leaf: Option<PreparedLeaf>,
    overlay: Option<Box<PreparedGroup>>,
    phase: u8,
}
impl GroupFrame {
    fn new(mut wire: GroupWire) -> Self {
        let input_leaves = std::mem::take(&mut wire.leaves).into_iter();
        let leaves = Vec::with_capacity(input_leaves.len());
        Self {
            wire,
            input_leaves,
            pending_leaf: None,
            leaves,
            opaque_leaf: None,
            overlay: None,
            phase: 0,
        }
    }
    fn finish(self, surfaces: &[Arc<CollisionSurface>]) -> Result<PreparedGroup, String> {
        Ok(PreparedGroup {
            source_range: range(self.wire.source_range)?,
            surface: surfaces
                .get(self.wire.surface as usize)
                .cloned()
                .ok_or("invalid collision surface ordinal")?,
            leaves: self.leaves,
            opaque_leaf: self.opaque_leaf,
            overlay: self.overlay,
            transform: Transform {
                translation: vec3(self.wire.transform.translation),
                rotation: Quat::from_array(self.wire.transform.rotation.map(f32::from_bits)),
                scale: vec3(self.wire.transform.scale),
            },
            coverage_fade: self.wire.coverage_fade,
            depth_coverage_fade: self.wire.depth_coverage_fade,
            residual_opacity: f32::from_bits(self.wire.residual_opacity),
            camera_wall: self.wire.camera_wall.map(|p|p.map(f32::from_bits)),
            runtime_lazy_sorted: self.wire.runtime_lazy_sorted,
        })
    }
}

/// Owns unpublished terrain CPU data. Dropping it cancels restoration without
/// uploading assets or altering the currently visible scene.
pub(crate) struct GroupRestore {
    surfaces_in: VecDeque<SurfaceWire>,
    shared_in: VecDeque<SharedWire>,
    groups_in: VecDeque<GroupWire>,
    surfaces: Vec<Arc<CollisionSurface>>,
    shared: Vec<Arc<Mutex<SharedLeaf>>>,
    active_surface: Option<(Option<Bounds>, SurfaceCursor)>,
    frames: Vec<GroupFrame>,
    output: Vec<PreparedGroup>,
    done: bool,
}
impl GroupRestore {
    /// Validation/conversion is charged per element, including traversal within
    /// a single large BVH or group. Completed blobs transfer ownership in O(1).
    /// Returns true only when finish() can publish the complete ordered table.
    pub(crate) fn advance(
        &mut self,
        blobs: &mut BlobStore,
        deadline: Instant,
        max_elements: usize,
    ) -> Result<bool, String> {
        for _ in 0..max_elements {
            if self.done {
                return Ok(true);
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            if let Some((_, active)) = &mut self.active_surface {
                if active.step()? {
                    let (bounds, active) = self.active_surface.take().unwrap();
                    self.surfaces.push(Arc::new(active.finish(bounds)));
                }
                continue;
            }
            if let Some(surface) = self.surfaces_in.front_mut() {
                let ready = match &mut surface.state {
                    SurfaceState::Ready(t) => t.resolve(blobs)?,
                    SurfaceState::Deferred(s) => s.resolve(blobs)?,
                };
                if !ready {
                    return Ok(false);
                }
                let surface = self.surfaces_in.pop_front().unwrap();
                self.active_surface = Some((
                    surface.bounds.map(BoundsWire::restore).transpose()?,
                    SurfaceCursor::new(surface.state)?,
                ));
                continue;
            }
            if let Some(shared) = self.shared_in.front_mut() {
                if !shared.mesh.resolve(blobs)? {
                    return Ok(false);
                }
                let shared = self.shared_in.pop_front().unwrap();
                self.shared.push(Arc::new(Mutex::new(SharedLeaf {
                    mesh: Some(shared.mesh.into_mesh()?),
                    uploaded: None,
                })));
                continue;
            }
            if self.frames.is_empty() {
                if let Some(group) = self.groups_in.pop_front() {
                    self.frames.push(GroupFrame::new(group));
                } else {
                    self.done = true;
                    return Ok(true);
                }
                continue;
            }
            let frame = self.frames.last_mut().unwrap();
            if let Some(leaf) = &mut frame.pending_leaf {
                if !leaf.mesh.resolve(blobs)? {
                    return Ok(false);
                }
                let leaf = frame.pending_leaf.take().unwrap();
                let restored = PreparedLeaf {
                    mesh: leaf.mesh.into_mesh()?,
                    uploaded: None,
                    center: vec3(leaf.center),
                    bounds: Aabb {
                        center: vec3(leaf.bounds_center).into(),
                        half_extents: vec3(leaf.bounds_half_extents).into(),
                    },
                    shared: leaf
                        .shared
                        .map(|i| {
                            self.shared
                                .get(i as usize)
                                .cloned()
                                .ok_or("invalid shared mesh ordinal".to_owned())
                        })
                        .transpose()?,
                };
                if frame.phase == 0 {
                    frame.leaves.push(restored);
                } else {
                    frame.opaque_leaf = Some(restored);
                }
                continue;
            }
            match frame.phase {
                0 => {
                    if let Some(leaf) = frame.input_leaves.next() {
                        frame.pending_leaf = Some(leaf);
                    } else {
                        frame.phase = 1;
                        frame.pending_leaf = frame.wire.opaque_leaf.take();
                    }
                }
                1 => {
                    frame.phase = 2;
                    if let Some(overlay) = frame.wire.overlay.take() {
                        if self.frames.len() > MAX_OVERLAY_DEPTH {
                            return Err("terrain overlay nesting exceeds limit".into());
                        }
                        self.frames.push(GroupFrame::new(*overlay));
                    }
                }
                _ => {
                    let group = self.frames.pop().unwrap().finish(&self.surfaces)?;
                    if let Some(parent) = self.frames.last_mut() {
                        parent.overlay = Some(Box::new(group));
                    } else {
                        self.output.push(group);
                    }
                }
            }
        }
        Ok(self.done)
    }
    pub(crate) fn finish(self) -> Result<Vec<PreparedGroup>, String> {
        if !self.done {
            return Err("terrain group restoration is incomplete".into());
        }
        Ok(self.output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_park_tree_roundtrip_preserves_wide_indices_and_shared_placements() {
        use crate::foliage::{GroveFamily, instance_variation};
        use crate::terrain_worker::blobs::{append_packet, decode_header, encode_header};

        // Exercise real loaded geometry as well as the deferred map recipes.
        // The seven-bough architecture crosses the u16 vertex limit once its
        // leaves have undersides; neither transfer nor merging may narrow it.
        let coordinates = (0..256)
            .map(|x| [x * 4, 12])
            .find(|&p| instance_variation(p).0 == 1)
            .unwrap();
        let library = crate::forest_instances::Library::default();
        let mut original = library.prepare(
            [-32., 32., -32., 32.],
            0.,
            16.,
            coordinates,
            [false; 4],
            GroveFamily::ParkBroadleaf,
        );
        assert_eq!(original.len(), 1);
        original.extend(library.prepare(
            [80., 112., -48., -16.],
            3.,
            8.,
            coordinates,
            [false; 4],
            GroveFamily::ParkBroadleaf,
        ));
        assert_eq!(original.len(), 2);
        assert_ne!(original[0].transform, original[1].transform);
        let assert_shared_geometry = |groups: &[PreparedGroup]| {
            assert!(Arc::ptr_eq(&groups[0].surface, &groups[1].surface));
            assert!(groups[0].surface.tree.get().is_none());
            let a = groups[0]
                .opaque_leaf
                .as_ref()
                .unwrap()
                .shared
                .as_ref()
                .unwrap();
            let b = groups[1]
                .opaque_leaf
                .as_ref()
                .unwrap()
                .shared
                .as_ref()
                .unwrap();
            assert!(Arc::ptr_eq(a, b));
            let shared = a.lock().unwrap();
            let mesh = shared.mesh.as_ref().unwrap();
            assert!(mesh.count_vertices() > u16::MAX as usize);
            assert!(matches!(mesh.indices(), Some(bevy::mesh::Indices::U32(v))
                if v.iter().any(|&i| i > u16::MAX as u32)));
            for (a, b) in groups[0].leaves.iter().zip(&groups[1].leaves) {
                assert!(Arc::ptr_eq(
                    a.shared.as_ref().unwrap(),
                    b.shared.as_ref().unwrap()
                ));
            }
        };
        assert_shared_geometry(&original);
        let table = GroupTable::capture(&original).unwrap();
        assert_eq!(table.surfaces.len(), 1);
        assert_eq!(table.shared_leaves.len(), original[0].leaves.len() + 1);
        let expected = bytes(&table);
        let (header, mut session) = encode_header(37, &table).unwrap();
        let (decoded, mut store) = decode_header::<GroupTable>(37, &header).unwrap();
        let mut restore = decoded.begin_restore();
        while let Some(page) = session.next_page().unwrap() {
            let (id, offset) = append_packet(&mut store, 37, &page).unwrap();
            session.acknowledge(37, id, offset).unwrap();
            restore
                .advance(
                    &mut store,
                    Instant::now() + std::time::Duration::from_millis(4),
                    8192,
                )
                .unwrap();
        }
        assert!(session.finished());
        let mut advances = 0;
        while !restore
            .advance(
                &mut store,
                Instant::now() + std::time::Duration::from_millis(4),
                8192,
            )
            .unwrap()
        {
            advances += 1;
            assert!(advances < 10_000, "park geometry restoration stalled");
        }
        assert!(store.all_consumed());
        let restored = restore.finish().unwrap();
        assert_eq!(bytes(&GroupTable::capture(&restored).unwrap()), expected);
        assert_shared_geometry(&restored);
        for (a, b, expected) in [
            (Vec3::new(-64., 32., 0.), Vec3::new(64., 32., 0.), true),
            (
                Vec3::new(-64., 32., 1000.),
                Vec3::new(64., 32., 1000.),
                false,
            ),
        ] {
            assert_eq!(original[0].surface.blocks(a, b), expected);
            assert_eq!(restored[0].surface.blocks(a, b), expected);
        }
        assert!(restored[1].surface.tree.get().is_some());
        assert!(restored[1].surface.source.lock().unwrap().is_none());
    }

    #[test]
    fn paged_group_restore_yields_inside_large_surface_and_leaf_list() {
        use crate::terrain_worker::blobs::{append_packet, decode_header, encode_header};
        let mut original = groups();
        let mut tree = TriangleTree::new(&source());
        tree.positions
            .extend((0..100_000).map(|i| Vec3::new(i as f32, -0.0, 0.0)));
        let surface = Arc::new(CollisionSurface {
            bounds: original[0].surface.bounds,
            tree: OnceLock::from(tree),
            source: Mutex::new(None),
        });
        original[0].surface = surface.clone();
        original[1].surface = surface;
        let leaf = original[0].leaves[0].clone();
        original[0].leaves.extend(std::iter::repeat_n(leaf, 128));
        let table = GroupTable::capture(&original).unwrap();
        let expected = bytes(&table);
        let (header, mut session) = encode_header(31, &table).unwrap();
        assert!(header.len() < expected.len() / 4);
        let (decoded, mut store) = decode_header::<GroupTable>(31, &header).unwrap();
        let mut restore = decoded.begin_restore();
        assert!(
            !restore
                .advance(&mut store, Instant::now(), usize::MAX)
                .unwrap()
        );
        assert!(
            !restore
                .advance(
                    &mut store,
                    Instant::now() + std::time::Duration::from_secs(1),
                    3
                )
                .unwrap()
        );
        while let Some(page) = session.next_page().unwrap() {
            let (id, offset) = append_packet(&mut store, 31, &page).unwrap();
            session.acknowledge(31, id, offset).unwrap();
            restore
                .advance(
                    &mut store,
                    Instant::now() + std::time::Duration::from_secs(1),
                    3,
                )
                .unwrap();
        }
        assert!(session.finished());
        // All pages are present, but a single budget cannot validate the large
        // surface or assemble a giant leaf list in one operation.
        assert!(
            !restore
                .advance(
                    &mut store,
                    Instant::now() + std::time::Duration::from_secs(1),
                    3
                )
                .unwrap()
        );
        let mut advances = 0;
        while !restore
            .advance(
                &mut store,
                Instant::now() + std::time::Duration::from_secs(1),
                257,
            )
            .unwrap()
        {
            advances += 1;
            assert!(advances < 1000);
        }
        assert!(advances > 300);
        assert!(store.all_consumed());
        let restored = restore.finish().unwrap();
        assert_eq!(bytes(&GroupTable::capture(&restored).unwrap()), expected);
        assert!(Arc::ptr_eq(&restored[0].surface, &restored[1].surface));
        assert!(Arc::ptr_eq(
            restored[0].leaves[0].shared.as_ref().unwrap(),
            restored[0].leaves[128].shared.as_ref().unwrap()
        ));
        assert!(restored[2].surface.tree.get().is_none());
        assert_eq!(
            restored[0]
                .surface
                .blocks(Vec3::new(0.3, 0.4, -1.), Vec3::new(0.3, 0.4, 1.)),
            original[0]
                .surface
                .blocks(Vec3::new(0.3, 0.4, -1.), Vec3::new(0.3, 0.4, 1.))
        );
    }

    #[test]
    fn paged_nodes_preserve_v2_bytes_and_reject_invalid_children_incrementally() {
        let node = NodeWire {
            bounds: BoundsWire {
                min: [0x80000000, 1, 2],
                max: [3, 4, 5],
            },
            triangles: [1u64 << 32, (1u64 << 32) + 7],
            children: [None, Some(u32::MAX)],
        };
        let before = NodeStorage::Plain(vec![node]);
        let mut a = Vec::new();
        ciborium::into_writer(&before, &mut a).unwrap();
        let NodeStorage::Plain(nodes) = &before else {
            unreachable!()
        };
        let packed = NodeStorage::Paged(nodes.iter().map(NodeWire::record).collect());
        let mut b = Vec::new();
        ciborium::into_writer(&packed, &mut b).unwrap();
        assert_eq!(a, b);

        let mut table = GroupTable::capture(&groups()).unwrap();
        let SurfaceState::Ready(tree) = &mut table.surfaces[0].state else {
            unreachable!()
        };
        let NodeStorage::Plain(nodes) = &tree.nodes else {
            unreachable!()
        };
        let mut records: Vec<_> = nodes.iter().map(NodeWire::record).collect();
        records[0][5] = u64::from(u32::MAX);
        tree.nodes = NodeStorage::Paged(records.into());
        let mut restore = table.begin_restore();
        let mut store = BlobStore::default();
        let mut error = None;
        for _ in 0..1000 {
            match restore.advance(
                &mut store,
                Instant::now() + std::time::Duration::from_secs(1),
                1,
            ) {
                Err(e) => {
                    error = Some(e);
                    break;
                }
                Ok(true) => panic!("invalid BVH accepted"),
                Ok(false) => {}
            }
        }
        assert!(error.unwrap().contains("child ordinal"));
    }

    #[test]
    fn collision_float_restore_transfers_allocations_and_preserves_bits() {
        fn arrays<const N: usize>(values: Vec<[u32; N]>)
        where
            [u32; N]: bytemuck::Pod,
            [f32; N]: bytemuck::Pod,
        {
            let expected = values.clone();
            let pointer = values.as_ptr() as usize;
            let capacity = values.capacity();
            let restored = float_arrays(values).unwrap();
            assert_eq!(restored.as_ptr() as usize, pointer);
            assert_eq!(restored.capacity(), capacity);
            assert_eq!(
                bytemuck::cast_slice::<_, u32>(&restored),
                bytemuck::cast_slice::<_, u32>(&expected)
            );
        }
        arrays(vec![[0x80000000, 0x7fc12345], [0x7fa12345, 0x3f123456]]);
        arrays(vec![[0x80000000, 0x7fc12345, 0x7fa12345]]);
        arrays(vec![[0x80000000, 0x7fc12345, 0x7fa12345, 0x3f123456]]);

        let bits = vec![[0x80000000, 0x3f123456, 0x7fc12345]];
        let pointer = bits.as_ptr() as usize;
        let restored = vec3_positions(bits.clone());
        assert_eq!(restored[0].to_array().map(f32::to_bits), bits[0]);
        let moved = vec3_positions(bits);
        assert_eq!(
            moved.as_ptr() as usize,
            pointer,
            "current Bevy Vec3 layout should transfer the allocation"
        );

        let mut original = source();
        original.positions[0][0] = -0.0;
        original.normals[0] = [-0.0, f32::from_bits(0x7fc12345), 1.0];
        original.uvs[0] = [f32::from_bits(0x7fa12345), -0.0];
        original.colors[0][2] = f32::from_bits(0x7fc54321);
        let before = SourceWire::capture(&original).unwrap();
        let mut encoded = Vec::new();
        ciborium::into_writer(&before, &mut encoded).unwrap();
        let restored = before.restore().unwrap();
        let mut after = Vec::new();
        ciborium::into_writer(&SourceWire::capture(&restored).unwrap(), &mut after).unwrap();
        assert_eq!(encoded, after);
    }

    fn source() -> SurfaceMeshData {
        SurfaceMeshData {
            positions: vec![[0., 0., 0.], [0., 1., 0.], [1., 1., 0.], [1., 0., 0.]],
            normals: vec![[0., 0., 1.]; 4],
            uvs: vec![[0.; 2]; 4],
            colors: vec![[0.2, 0.3, 0.4, 1.]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            cutaway_ranges: vec![0..4],
        }
    }
    fn bytes(table: &GroupTable) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(table, &mut bytes).unwrap();
        bytes
    }
    fn groups() -> Vec<PreparedGroup> {
        let (_, mut ready) = prepare(source());
        let (_, mut deferred) = prepare_grove(source());
        share_groups(&mut ready);
        share_groups(&mut deferred);
        ready[0].opaque_leaf = Some(ready[0].leaves[0].clone());
        ready[0].overlay = Some(Box::new(deferred[0].clone()));
        ready[0].transform = Transform {
            translation: Vec3::new(-0.0, 2.5, -7.),
            rotation: Quat::from_xyzw(0., 0.38268343, -0.0, 0.9238795),
            scale: Vec3::new(1., 0.5, -1.5),
        };
        ready[0].coverage_fade = true;
        ready[0].depth_coverage_fade = true;
        ready[0].runtime_lazy_sorted = true;
        ready[0].residual_opacity = f32::from_bits(0x7fc01234);
        ready[0].camera_wall = Some([0., -1., -96.]);
        vec![ready[0].clone(), ready.remove(0), deferred.remove(0)]
    }

    #[test]
    fn binary_occluder_roundtrip_preserves_bits_sharing_overlays_and_collision_laziness() {
        let original = groups();
        let table = GroupTable::capture(&original).unwrap();
        let encoded = bytes(&table);
        let decoded: GroupTable = ciborium::de::from_reader(encoded.as_slice()).unwrap();
        let restored = decoded.restore().unwrap();
        assert_eq!(bytes(&GroupTable::capture(&restored).unwrap()), encoded);
        assert!(Arc::ptr_eq(&restored[0].surface, &restored[1].surface));
        assert!(Arc::ptr_eq(
            restored[0].leaves[0].shared.as_ref().unwrap(),
            restored[1].leaves[0].shared.as_ref().unwrap()
        ));
        assert!(Arc::ptr_eq(
            restored[0].leaves[0].shared.as_ref().unwrap(),
            restored[0]
                .opaque_leaf
                .as_ref()
                .unwrap()
                .shared
                .as_ref()
                .unwrap()
        ));
        assert!(Arc::ptr_eq(
            &restored[0].overlay.as_ref().unwrap().surface,
            &restored[2].surface
        ));
        assert!(restored[0].surface.tree.get().is_some());
        assert!(restored[2].surface.tree.get().is_none());
        for x in [-0.5, 0.25, 0.5, 1.5] {
            let a = Vec3::new(x, 0.4, -1.);
            let b = Vec3::new(x, 0.4, 1.);
            let expected = original[0].surface.blocks(a, b);
            assert_eq!(restored[0].surface.blocks(a, b), expected);
            assert_eq!(restored[2].surface.blocks(a, b), expected);
        }
        assert!(restored[2].surface.tree.get().is_some());
        assert!(restored[2].surface.source.lock().unwrap().is_none());
    }

    #[test]
    fn collision_wire_preserves_each_compressed_index_representation() {
        for representation in 0..3 {
            let mut tree = TriangleTree::new(&source());
            let values: Vec<_> = (0..tree.triangles.len())
                .map(|i| tree.triangles.triangle(i))
                .collect();
            tree.triangles = match representation {
                0 => TriangleIndices::Packed {
                    triangles: vec![[0, 0x0201], [u16::MAX, 0]],
                    exceptions: vec![[0, 2, 3]],
                },
                1 => TriangleIndices::Narrow(values.iter().map(|v| v.map(|i| i as u16)).collect()),
                _ => TriangleIndices::Wide(values),
            };
            let wire = TreeWire::capture(&tree).unwrap();
            let mut encoded = Vec::new();
            ciborium::ser::into_writer(&wire, &mut encoded).unwrap();
            let decoded: TreeWire = ciborium::de::from_reader(encoded.as_slice()).unwrap();
            let restored = decoded.restore().unwrap();
            let mut after = Vec::new();
            ciborium::ser::into_writer(&TreeWire::capture(&restored).unwrap(), &mut after).unwrap();
            assert_eq!(after, encoded);
            assert_eq!(
                restored.blocks(Vec3::new(0.3, 0.4, -1.), Vec3::new(0.3, 0.4, 1.)),
                tree.blocks(Vec3::new(0.3, 0.4, -1.), Vec3::new(0.3, 0.4, 1.))
            );
        }
    }

    #[test]
    fn occluder_wire_rejects_asset_handles_bad_ordinals_and_invalid_bvh_storage() {
        let mut values = groups();
        values[0].leaves[0].uploaded = Some(Handle::default());
        assert!(GroupTable::capture(&values).is_err());
        values[0].leaves[0].uploaded = None;
        values[0].leaves[0]
            .shared
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .uploaded = Some(Handle::default());
        assert!(GroupTable::capture(&values).is_err());
        let mut table = GroupTable::capture(&groups()).unwrap();
        table.groups[0].surface = u32::MAX;
        assert!(table.restore().is_err());
        let mut table = GroupTable::capture(&groups()).unwrap();
        table.groups[0].leaves[0].shared = Some(u32::MAX);
        assert!(table.restore().is_err());
        let mut tree = TreeWire::capture(&TriangleTree::new(&source())).unwrap();
        tree.triangles = TrianglesWire::Packed {
            triangles: vec![[u16::MAX, 1]].into(),
            exceptions: vec![].into(),
        };
        assert!(tree.restore().is_err());
        let mut tree = TreeWire::capture(&TriangleTree::new(&source())).unwrap();
        let NodeStorage::Plain(nodes) = &mut tree.nodes else {
            unreachable!()
        };
        nodes[0].children = [Some(0), Some(0)];
        assert!(tree.restore().is_err());
    }
}
