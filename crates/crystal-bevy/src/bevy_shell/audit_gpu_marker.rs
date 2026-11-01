//! A real one-thread dispatch keeps Metal from discarding timestamp markers.
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoder, ComputePassDescriptor, ComputePassTimestampWrites, ComputePipeline,
    ComputePipelineDescriptor, Device, QuerySet, ShaderModuleDescriptor, ShaderSource,
};

pub(super) struct GpuMarker {
    pipeline: ComputePipeline,
    bind_group: BindGroup,
    _buffer: Buffer,
}

impl GpuMarker {
    pub(super) fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("audit timestamp marker"),
            source: ShaderSource::Wgsl(
                "@group(0) @binding(0) var<storage, read_write> value: atomic<u32>;
                 @compute @workgroup_size(1) fn main() { atomicAdd(&value, 1u); }"
                    .into(),
            ),
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("audit timestamp marker"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("audit marker side effect"),
            size: 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("audit timestamp marker"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            pipeline,
            bind_group,
            _buffer: buffer,
        }
    }

    pub(super) fn record(
        &self,
        encoder: &mut CommandEncoder,
        query: Option<&QuerySet>,
        begin: Option<u32>,
        end: Option<u32>,
        label: &str,
    ) {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some(label),
            timestamp_writes: query.map(|query| ComputePassTimestampWrites {
                query_set: query,
                beginning_of_pass_write_index: begin,
                end_of_pass_write_index: end,
            }),
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
}
