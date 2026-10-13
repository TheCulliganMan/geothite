//! Bound shared GPU allocations without changing any mesh payload.
use bevy::{prelude::*, render::{RenderApp, mesh::allocator::MeshAllocatorSettings}};

pub(super) fn register(app: &mut App) {
    let Some(render) = app.get_sub_app_mut(RenderApp) else { return; };
    let mut settings = render.world_mut().get_resource_or_init::<MeshAllocatorSettings>();
    // WebGL gives each vertex array a separate slab, but shares index slabs.
    // Keep their growth/copy peaks bounded rather than growing one allocation
    // toward the native allocator's 512 MiB default. Oversized source meshes
    // still receive dedicated buffers, so this never truncates geometry.
    const MAX_SHARED_BUFFER_BYTES: u64 = 16 * 1024 * 1024;
    settings.max_slab_size = settings.max_slab_size.min(MAX_SHARED_BUFFER_BYTES);
    settings.min_slab_size = settings.min_slab_size.min(settings.max_slab_size);
    settings.large_threshold = settings.large_threshold.min(MAX_SHARED_BUFFER_BYTES);
}
