//! Bound shared GPU allocations without changing any mesh payload.
use bevy::{prelude::*, render::{RenderApp, mesh::allocator::MeshAllocatorSettings}};

pub(super) fn register(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    app.add_systems(PostUpdate, align_new_indices.after(crate::occluder_fade::FadeMeshes)
        .before(bevy::asset::AssetEventSystems));
    let Some(render) = app.get_sub_app_mut(RenderApp) else { return; };
    let mut settings = render.world_mut().get_resource_or_init::<MeshAllocatorSettings>();
    // WebGL gives each vertex array a separate slab, but shares index slabs.
    // Native clients also need bounded allocation growth on shared-memory GPUs.
    // Keep their growth/copy peaks bounded rather than growing one allocation
    // toward the native allocator's 512 MiB default. Oversized source meshes
    // still receive dedicated buffers, so this never truncates geometry.
    const MAX_SHARED_BUFFER_BYTES: u64 = 16 * 1024 * 1024;
    settings.max_slab_size = settings.max_slab_size.min(MAX_SHARED_BUFFER_BYTES);
    settings.min_slab_size = settings.min_slab_size.min(settings.max_slab_size);
    settings.large_threshold = settings.large_threshold.min(MAX_SHARED_BUFFER_BYTES);
    // WebGL already requires individual vertex buffers. Keep index buffers
    // individual too: ANGLE's Metal backend can retain translated copies of a
    // shared index slab for each vertex-array binding. Thousands of cutaway
    // leaves otherwise multiply one large slab into gigabytes of driver memory.
    #[cfg(target_arch = "wasm32")]
    { settings.large_threshold = 0; }
    #[cfg(target_arch = "wasm32")]
    render.add_systems(bevy::render::RenderStartup, |info: Res<bevy::render::renderer::RenderAdapterInfo>,
        mut settings: ResMut<MeshAllocatorSettings>| {
        // WebGPU supports shared vertex/index slabs and native batching.
        // Only the GL backend needs dedicated index allocations.
        if info.backend.to_str() != "gl" {
            settings.large_threshold = MAX_SHARED_BUFFER_BYTES;
        }
    });
}

// The dedicated-buffer upload path requires a four-byte mapped size. Widen
// odd-length u16 index lists losslessly instead of adding a drawable triangle.
#[cfg(any(target_arch = "wasm32", test))]
pub(super) fn align_indices(mesh: &mut Mesh) {
    use bevy::mesh::Indices;
    let Ok(Indices::U16(indices)) = mesh.try_indices() else { return; };
    if indices.len() % 2 == 1 {
        let indices = indices.iter().copied().map(u32::from).collect();
        mesh.insert_indices(Indices::U32(indices));
    }
}
#[cfg(any(target_arch = "wasm32", test))]
fn align_new_indices(mut meshes: ResMut<Assets<Mesh>>) {
    let pending: Vec<_> = meshes.iter().filter_map(|(id, mesh)|
        matches!(mesh.try_indices(), Ok(bevy::mesh::Indices::U16(indices)) if indices.len()%2 == 1)
            .then_some(id)).collect();
    for id in pending {
        if let Some(mut mesh) = meshes.get_mut(id) { align_indices(&mut mesh); }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::Indices;
    #[test]
    fn dedicated_index_alignment_preserves_every_index_and_triangle() {
        let mut mesh = Mesh::from(Cuboid::default());
        mesh.insert_indices(Indices::U16(vec![0, 1, 2]));
        align_indices(&mut mesh);
        assert_eq!(mesh.indices(), Some(&Indices::U32(vec![0, 1, 2])));
        mesh.insert_indices(Indices::U16(vec![0, 1, 2, 2, 1, 0]));
        align_indices(&mut mesh);
        assert_eq!(mesh.indices(), Some(&Indices::U16(vec![0, 1, 2, 2, 1, 0])));
    }
    #[test]
    fn alignment_publishes_before_extraction_and_does_not_reupload_later() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_asset::<Mesh>()
            .add_systems(PostUpdate, align_new_indices.before(bevy::asset::AssetEventSystems));
        let mut mesh = Mesh::from(Cuboid::default());
        mesh.insert_indices(Indices::U16(vec![0,1,2]));
        let handle = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
        app.update();
        assert!(matches!(app.world().resource::<Assets<Mesh>>().get(&handle).unwrap().indices(), Some(Indices::U32(_))));
        app.world_mut().resource_mut::<Messages<AssetEvent<Mesh>>>().clear();
        app.update();
        assert_eq!(app.world().resource::<Messages<AssetEvent<Mesh>>>().len(), 0);
    }

}
