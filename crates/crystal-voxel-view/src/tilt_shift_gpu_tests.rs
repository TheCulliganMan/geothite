use super::*;
use bevy::{app::PluginsState, render::renderer::RenderQueue, window::{ExitCondition, WindowPlugin}};

#[test]
#[ignore = "requires a native GPU; compares both production blur shaders offscreen"]
fn incremental_gaussian_preserves_pixels_and_reduces_filter_work() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: None, exit_condition: ExitCondition::DontExit, ..default()
    }).disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>());
    register(&mut app);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.plugins_state() != PluginsState::Ready {
        assert!(std::time::Instant::now() < deadline);
        bevy::tasks::tick_global_task_pools_on_main_thread();
        std::thread::yield_now();
    }
    app.finish(); app.cleanup(); app.update();
    let (layout, vertex) = {
        let render = app.sub_app(RenderApp).world();
        let pipeline = render.resource::<Pipeline>();
        (pipeline.layout.clone(), pipeline.vertex.clone())
    };
    let ids = {
        let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
        [false, true].map(|cached| ["horizontal", "vertical"].map(|entry| {
            cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("blur_pixel_comparison".into()),
                layout: vec![layout.clone()],
                vertex: vertex.clone(), fragment: Some(FragmentState {
                    shader: SHADER, entry_point: Some(entry.into()),
                    shader_defs: if cached { vec![] } else { vec!["DIRECT_GAUSSIAN".into()] },
                    targets: vec![Some(ColorTargetState { format: TextureFormat::Rgba8Unorm,
                        blend: None, write_mask: ColorWrites::ALL })], ..default()
                }), ..default()
            })
        }))
    };
    loop {
        app.update();
        let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
        if ids.iter().flatten().all(|id| cache.get_render_pipeline(*id).is_some()) { break; }
        assert!(std::time::Instant::now() < deadline, "production shader compilation timed out");
        std::thread::yield_now();
    }
    let render = app.sub_app(RenderApp).world();
    let device = render.resource::<RenderDevice>();
    let queue = render.resource::<RenderQueue>();
    let cache = render.resource::<PipelineCache>();
    let sampler = device.create_sampler(&SamplerDescriptor {
        min_filter: FilterMode::Linear, mag_filter: FilterMode::Linear, ..default()
    });
    let uniform = device.create_buffer(&BufferDescriptor { label: Some("blur_test_band"),
        size: TiltShift::min_size().get(), usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST, mapped_at_creation: false });
    for (width, height) in [(127u32, 65u32), (320, 257), (1184, 812)] {
        let extent = Extent3d { width, height, depth_or_array_layers: 1 };
        let image = || device.create_texture(&TextureDescriptor {
            label: Some("blur_comparison_image"), size: extent, mip_level_count: 1,
            sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC, view_formats: &[],
        });
        let source = image();
        // Fine edges, color transitions and non-opaque alpha expose blur and
        // sampling mistakes; odd sizes expose pixel-center/row rounding.
        let pixels: Vec<u8> = (0..height).flat_map(|y| (0..width).flat_map(move |x| {
            [((x * 17 + y * 3) % 256) as u8, ((x / 5 + y / 7) % 2 * 255) as u8,
                ((y * 11 + x / 3) % 256) as u8, ((x + y) % 256) as u8]
        })).collect();
        queue.write_texture(TexelCopyTextureInfo { texture: &source, mip_level: 0,
            origin: Origin3d::ZERO, aspect: TextureAspect::All }, &pixels,
            TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(width * 4), rows_per_image: None }, extent);
        let source_view = source.create_view(&default());
        let textures = [[image(), image()], [image(), image()]];
        let views = textures.each_ref().map(|pair| pair.each_ref().map(|t| t.create_view(&default())));
        let groups = [false, true].map(|cached| {
            [&source_view, &views[usize::from(cached)][0]].map(|view| {
                let entries = DynamicBindGroupEntries::sequential((view, &sampler, uniform.as_entire_binding()));
                device.create_bind_group("blur_comparison", &cache.get_bind_group_layout(&layout), &entries)
            })
        });
        let encode = |encoder: &mut bevy::render::render_resource::CommandEncoder, cached: usize| {
            for pass_index in 0..2 {
                let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("blur_comparison"), color_attachments: &[Some(RenderPassColorAttachment {
                        view: &views[cached][pass_index], depth_slice: None, resolve_target: None,
                        ops: Operations::default(),
                    })], ..default()
                });
                pass.set_pipeline(cache.get_render_pipeline(ids[cached][pass_index]).unwrap());
                pass.set_bind_group(0, &groups[cached][pass_index], &[0]);
                pass.draw(0..3, 0..1);
            }
        };
        let bands = [Vec4::new(0.3, 0.14, 2.5, 3.), Vec4::new(0.7, 0.20, 2.5, 36.),
            Vec4::new(-0.1, 0.14, 2.5, 36.), Vec4::new(1.1, 0.14, 2.5, 64.)];
        for band in bands {
            let bytes: Vec<u8> = band.to_array().into_iter().flat_map(f32::to_le_bytes).collect();
            queue.write_buffer(&uniform, 0, &bytes);
            let stride = (width * 4).div_ceil(256) * 256;
            let readbacks = [0, 1].map(|_| device.create_buffer(&BufferDescriptor {
                label: Some("blur_pixel_readback"), size: u64::from(stride * height),
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ, mapped_at_creation: false,
            }));
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
            for path in 0..2 {
                encode(&mut encoder, path);
                encoder.copy_texture_to_buffer(TexelCopyTextureInfo { texture: &textures[path][1],
                    mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All },
                    TexelCopyBufferInfo { buffer: &readbacks[path], layout: TexelCopyBufferLayout {
                        offset: 0, bytes_per_row: Some(stride), rows_per_image: None } }, extent);
            }
            queue.submit([encoder.finish()]);
            let mut results = Vec::new();
            for buffer in &readbacks {
                let (send, recv) = std::sync::mpsc::channel();
                buffer.slice(..).map_async(MapMode::Read, move |result| { send.send(result).unwrap(); });
                device.poll(PollType::wait_indefinitely()).unwrap();
                recv.recv_timeout(std::time::Duration::from_secs(10)).unwrap().unwrap();
                let mapped = buffer.slice(..).get_mapped_range().unwrap();
                results.push((0..height as usize).flat_map(|y|
                    mapped[y * stride as usize..y * stride as usize + width as usize * 4].iter().copied()).collect::<Vec<_>>());
            }
            let differences: Vec<_> = results[0].iter().zip(&results[1]).map(|(a,b)| a.abs_diff(*b)).collect();
            let max = *differences.iter().max().unwrap();
            let mean = differences.iter().map(|&n| f64::from(n)).sum::<f64>() / differences.len() as f64;
            assert!(max <= 1 && mean < 0.01, "{width}x{height} {band:?}: max={max} mean={mean}");
            for buffer in readbacks { buffer.unmap(); }
        }
        // Exercise the actual production entry shader with no blur: full-view
        // masks/flash, native SCX direction/wrapping and square-pixel trainer art.
        for case in 0..5 {
            let mut effect = crystal_render_api::VisualWorldEntryEffect { active: true, ..default() };
            match case {
                0 => effect.black_cells = [UVec4::MAX; 3],
                1 => effect.light = 1.,
                2 => effect.line_x_offsets = [1; 144],
                3 => effect.black_cells[2][3] = 1 << 7, // final source cell (19,17)
                4 => {
                    effect.trainer_cells = [UVec4::MAX; 3];
                    effect.trainer_pixels = UVec4::splat(0xe4e4e4e4);
                    effect.trainer_palette = [Vec4::new(1.,0.,0.,1.), Vec4::new(0.,1.,0.,1.),
                        Vec4::new(0.,0.,1.,1.), Vec4::ONE];
                }
                _ => unreachable!(),
            }
            if case == 2 && width != 320 { continue; } // integer two-output-pixel source shift
            let settings = TiltShift::entry_only(&effect);
            let mut bytes = bevy::render::render_resource::encase::UniformBuffer::new(Vec::new());
            bytes.write(&settings).unwrap();
            queue.write_buffer(&uniform, 0, bytes.as_ref());
            let stride = (width * 4).div_ceil(256) * 256;
            let output = device.create_buffer(&BufferDescriptor { label: Some("entry_pixel_readback"),
                size: u64::from(stride * height), usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false });
            let mut encoder = device.create_command_encoder(&default());
            encode(&mut encoder, 1);
            encoder.copy_texture_to_buffer(TexelCopyTextureInfo { texture: &textures[1][1],
                mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All },
                TexelCopyBufferInfo { buffer: &output, layout: TexelCopyBufferLayout {
                    offset: 0, bytes_per_row: Some(stride), rows_per_image: None } }, extent);
            queue.submit([encoder.finish()]);
            let (send, recv) = std::sync::mpsc::channel();
            output.slice(..).map_async(MapMode::Read, move |result| { send.send(result).unwrap(); });
            device.poll(PollType::wait_indefinitely()).unwrap();
            recv.recv_timeout(std::time::Duration::from_secs(10)).unwrap().unwrap();
            let mapped = output.slice(..).get_mapped_range().unwrap();
            for y in 0..height { for x in 0..width {
                let offset = (y * stride + x * 4) as usize;
                let uv = Vec2::new((x as f32 + 0.5) / width as f32, (y as f32 + 0.5) / height as f32);
                let mut expected: [u8;4] = pixels[((y * width + x) * 4) as usize..][..4].try_into().unwrap();
                match case {
                    0 => expected = [0,0,0,255],
                    1 => expected[..3].fill(255),
                    2 => { let from = ((y * width + (x + width - 2) % width) * 4) as usize;
                        expected = pixels[from..from+4].try_into().unwrap(); }
                    3 if (uv.x * 20.) as u32 == 19 && (uv.y * 18.) as u32 == 17 => expected = [0,0,0,255],
                    4 => {
                        let source_x = (x as f32 + 0.5 - width as f32 * 0.5) * 144. / height as f32 + 80.;
                        if (0.0..160.0).contains(&source_x) {
                            expected = [[255,0,0,255],[0,255,0,255],[0,0,255,255],[255;4]][source_x.floor() as usize % 4];
                        }
                    }
                    _ => {}
                }
                assert!(mapped[offset..offset+4].iter().zip(expected).all(|(a,b)|a.abs_diff(b)<=1),
                    "entry case {case}, {width}x{height}, pixel {x},{y}: {:?} vs {expected:?}", &mapped[offset..offset+4]);
            }}
            drop(mapped); output.unmap();
        }
        // Restore a neutral effect before measuring the unchanged blur kernel.
        queue.write_buffer(&uniform, 0, &vec![0; TiltShift::min_size().get() as usize]);
        if width == 1184 && std::env::var_os("CRYSTAL_TILT_SHIFT_PIXEL_ONLY").is_none() {
          for diameter in [3.0, (height as f32 * 0.018).clamp(3.0, 36.0) * 0.55, 36.0] {
            let band: Vec<u8> = Vec4::new(0.5, 0.20, 2.5, diameter).to_array()
                .into_iter().flat_map(f32::to_le_bytes).collect();
            queue.write_buffer(&uniform, 0, &band);
            let mut samples = [Vec::new(), Vec::new()];
            for batch in 0..8 {
                for path in [batch % 2, 1 - batch % 2] {
                    let start = std::time::Instant::now();
                    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
                    for _ in 0..24 { encode(&mut encoder, path); }
                    queue.submit([encoder.finish()]);
                    device.poll(PollType::wait_indefinitely()).unwrap();
                    if batch > 0 { samples[path].push(start.elapsed().as_secs_f64() * 1000. / 24.); }
                }
            }
            for values in &mut samples { values.sort_by(f64::total_cmp); }
            println!("production blur at {width}x{height}, diameter={diameter:.2}, median batch ms/frame: direct={:.3}, incremental={:.3}",
                samples[0][samples[0].len()/2], samples[1][samples[1].len()/2]);
          }
        }
    }
}
