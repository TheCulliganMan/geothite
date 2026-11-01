//! Publish speculative static meshes gradually, then verify render residency.
use bevy::asset::{AssetId, Assets, Handle};
use bevy::mesh::{Indices, Mesh};
use bevy::prelude::{App, Component, IntoScheduleConfigs, Query, Res};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::RenderMesh;
use bevy::render::mesh::allocator::{MeshAllocator, MeshAllocatorSettings};
use bevy::render::render_asset::RenderAssets;
use bevy::render::{Render, RenderApp, RenderSystems};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Scheduling allowance per preload lane and frame, not a memory/residency cap.
// A single larger spatial batch still progresses without being split or lost.
const BYTES_PER_FRAME: usize = 4 * 1024 * 1024;
const GPU_BUFFER_PAGE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Component, ExtractComponent)]
#[extract_app(RenderApp)]
pub(super) struct Receipt {
    meshes: Arc<[AssetId<Mesh>]>,
    resident: Arc<AtomicBool>,
}

impl Receipt {
    fn acknowledge(&self, present: impl Fn(AssetId<Mesh>) -> bool) {
        if !self.resident.load(Ordering::Acquire) && self.meshes.iter().copied().all(present) {
            self.resident.store(true, Ordering::Release);
        }
    }
}

#[derive(Default)]
pub(super) struct Pending {
    require_gpu: bool,
    meshes: VecDeque<(Handle<Mesh>, Mesh)>,
    receipt: Option<Receipt>,
}

impl Pending {
    pub(super) fn require_gpu(&mut self, required: bool) {
        self.require_gpu = required;
    }

    pub(super) fn gpu_required(&self) -> bool {
        self.require_gpu
    }

    pub(super) fn enqueue(&mut self, assets: &Assets<Mesh>, mesh: Mesh) -> Handle<Mesh> {
        let handle = assets.reserve_handle();
        self.meshes.push_back((handle.clone(), mesh));
        handle
    }

    /// Defer meshes added by scenery/fade helpers during the same exclusive
    /// Assets borrow. Keep their exact handles; shared existing assets stay live.
    pub(super) fn capture_added(
        &mut self,
        assets: &mut Assets<Mesh>,
        existing: &HashSet<AssetId<Mesh>>,
    ) {
        let added = assets
            .ids()
            .filter(|id| !existing.contains(id))
            .collect::<Vec<_>>();
        for id in added {
            let handle = assets
                .get_strong_handle(id)
                .expect("scene meshes are inserted with strong generated handles");
            let mesh = assets
                .remove_untracked(id)
                .expect("new scene mesh remains present");
            self.meshes.push_back((handle, mesh));
        }
    }

    pub(super) fn seal(&mut self) -> Receipt {
        let receipt = Receipt {
            meshes: self.meshes.iter().map(|(handle, _)| handle.id()).collect(),
            resident: Arc::new(AtomicBool::new(false)),
        };
        self.receipt = Some(receipt.clone());
        receipt
    }

    pub(super) fn advance(&mut self, assets: &mut Assets<Mesh>) {
        let mut bytes = 0;
        let mut largest_batch_bytes = 0;
        while let Some((_, mesh)) = self.meshes.front() {
            let size = mesh.get_vertex_buffer_size()
                + match mesh.indices() {
                    Some(Indices::U16(indices)) => indices.len() * 2,
                    Some(Indices::U32(indices)) => indices.len() * 4,
                    None => 0,
                };
            if bytes > 0 && bytes + size > BYTES_PER_FRAME {
                break;
            }
            let (handle, mesh) = self.meshes.pop_front().expect("front mesh is present");
            assets
                .insert(handle.id(), mesh)
                .expect("reserved upload handle remains live");
            bytes += size;
            largest_batch_bytes = largest_batch_bytes.max(size);
        }
        if bytes > 0 {
            let _trace = crate::loading_trace::span("region.stage_gpu_meshes", || {
                serde_json::json!({"bytes": bytes, "largest_batch_bytes": largest_batch_bytes,
                    "remaining_batches": self.meshes.len()})
            });
        }
    }

    pub(super) fn ready(&self) -> bool {
        self.meshes.is_empty()
            && (!self.require_gpu
                || self
                    .receipt
                    .as_ref()
                    .is_none_or(|receipt| receipt.resident.load(Ordering::Acquire)))
    }
}

pub(super) fn register(app: &mut App) {
    #[cfg(not(target_arch = "wasm32"))]
    crate::gpu_retirement::register(app);
    if app.get_sub_app(RenderApp).is_none() {
        return;
    }
    app.world_mut()
        .resource_mut::<crate::TerrainRevisionCache>()
        .uploads
        .require_gpu(true);
    app.add_plugins(ExtractComponentPlugin::<Receipt>::default());
    let render = app.sub_app_mut(RenderApp);
    // Growing Bevy's default slabs can copy hundreds of resident megabytes
    // during one small terrain upload. Fixed-size pages avoid that unbounded
    // relocation; the allocator creates more pages as content arrives. This
    // changes allocation granularity, not total terrain or memory residency.
    let mut allocation = render.world_mut().resource_mut::<MeshAllocatorSettings>();
    allocation.min_slab_size = GPU_BUFFER_PAGE_BYTES;
    allocation.max_slab_size = GPU_BUFFER_PAGE_BYTES;
    allocation.large_threshold = GPU_BUFFER_PAGE_BYTES;
    render.add_systems(Render, acknowledge.after(RenderSystems::PrepareAssets));
}

fn acknowledge(
    receipts: Query<&Receipt>,
    meshes: Res<RenderAssets<RenderMesh>>,
    allocator: Res<MeshAllocator>,
) {
    let _trace = crate::loading_trace::span("gpu.mesh_buffers", || {
        serde_json::json!({
            "allocator_owned_bytes": allocator.slabs_size(),
            "pages": allocator.slab_count(),
            "relocated_meshes": allocator.meshes_displaced_by_slab_growth().count(),
            "meaning": "mesh allocator-owned GPU buffers; excludes in-flight retirements, textures, and CPU memory",
        })
    });
    for receipt in &receipts {
        receipt.acknowledge(|id| meshes.get(id).is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::{BYTES_PER_FRAME, Pending};
    use bevy::asset::{Assets, RenderAssetUsages};
    use bevy::mesh::Mesh;
    use bevy::render::render_resource::PrimitiveTopology;

    fn mesh(bytes: usize) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.; 3]; bytes / 12 + 1]);
        mesh
    }

    #[test]
    fn staged_scene_waits_for_every_asset_and_render_receipt() {
        let mut assets = Assets::<Mesh>::default();
        let mut pending = Pending::default();
        pending.require_gpu(true);
        let first = pending.enqueue(&assets, mesh(BYTES_PER_FRAME));
        let second = pending.enqueue(&assets, mesh(48));
        let receipt = pending.seal();
        pending.advance(&mut assets);
        assert!(assets.contains(first.id()));
        assert!(
            !assets.contains(second.id()),
            "oversized first batch must not starve or upload its neighbor"
        );
        receipt.acknowledge(|id| assets.contains(id));
        assert!(
            !pending.ready(),
            "a partially resident scene cannot replace live terrain"
        );
        pending.advance(&mut assets);
        assert!(assets.contains(second.id()));
        assert!(
            !pending.ready(),
            "CPU insertion alone cannot prove GPU residency"
        );
        receipt.acknowledge(|id| assets.contains(id));
        assert!(pending.ready());
    }

    #[test]
    fn scenery_meshes_keep_their_handles_and_wait_in_the_same_upload_queue() {
        let mut assets = Assets::<Mesh>::default();
        let shared = assets.add(mesh(48));
        let existing = assets.ids().collect();
        let mut pending = Pending::default();
        let first = pending.enqueue(&assets, mesh(BYTES_PER_FRAME));
        let scenery = assets.add(mesh(48));
        pending.capture_added(&mut assets, &existing);
        assert!(assets.contains(shared.id()));
        assert!(!assets.contains(scenery.id()));
        pending.seal();
        pending.advance(&mut assets);
        assert!(assets.contains(first.id()));
        assert!(!assets.contains(scenery.id()));
        assert!(!pending.ready());
        pending.advance(&mut assets);
        assert!(assets.contains(scenery.id()));
        assert!(pending.ready());
    }

    #[test]
    fn canceled_scene_does_not_publish_its_reserved_assets() {
        let assets = Assets::<Mesh>::default();
        let mut pending = Pending::default();
        let handle = pending.enqueue(&assets, mesh(48));
        drop(pending);
        assert!(!assets.contains(handle.id()));
    }
}
