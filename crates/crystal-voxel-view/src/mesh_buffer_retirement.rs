//! Release abandoned WebGPU mesh slabs after submitted frames, without waiting
//! for browser GC. Live and allocator-reserved slabs remain untouched.
use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        mesh::allocator::MeshAllocator,
        render_resource::{Buffer, BufferId},
        renderer::RenderAdapterInfo,
    },
};

#[derive(Resource, Default)]
struct Buffers(HashMap<BufferId, (Buffer, u8)>);

pub(super) fn register(app: &mut App) {
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render.init_resource::<Buffers>().add_systems(
        Render,
        retire
            .after(RenderSystems::Render)
            .before(RenderSystems::Cleanup),
    );
}

fn retire(
    info: Res<RenderAdapterInfo>,
    allocator: Res<MeshAllocator>,
    mut buffers: ResMut<Buffers>,
) {
    if info.backend.to_str() != "webgpu" {
        return;
    }
    let current: Vec<_> = allocator
        .slabs
        .keys()
        .filter_map(|&id| allocator.buffer_for_slab(id).cloned())
        .collect();
    let live: HashSet<_> = current.iter().map(Buffer::id).collect();
    let mut retired = 0;
    buffers.0.retain(|id, (buffer, missing)| {
        if live.contains(id) {
            *missing = 0;
            return true;
        }
        *missing += 1;
        if *missing < 3 {
            return true;
        }
        // All commands from this and previous frames have been submitted. The
        // WebGPU destroy contract releases storage after those commands finish.
        // No allocator slab can reuse this buffer: it is absent from all slabs.
        buffer.destroy();
        retired += 1;
        false
    });
    for buffer in current {
        buffers.0.entry(buffer.id()).or_insert((buffer, 0));
    }
    if retired > 0 {
        let _trace = crate::loading_trace::span(
            "gpu.retire_abandoned_mesh_buffers",
            || serde_json::json!({"buffers": retired, "allocator_buffers": live.len()}),
        );
    }
}
