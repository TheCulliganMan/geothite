//! Owned CPU-only terrain messages. Asset handles never cross Wasm heaps.
//!
//! This boundary is experimental and does not register a game system. The
//! main renderer retains authoritative images and applies completed revisions.
pub(crate) mod blobs;
#[cfg(all(target_arch = "wasm32", feature = "terrain-worker-browser"))]
pub(crate) mod browser;
pub(crate) mod mesh_buffers;
pub(crate) mod packed;
pub(crate) mod records;
#[cfg(all(target_arch = "wasm32", feature = "terrain-worker-browser"))]
pub fn browser_scenery_sources(json: Option<&str>) {
    browser::scenery_sources(json);
}
#[cfg(all(target_arch = "wasm32", feature = "terrain-worker-browser"))]
pub fn browser_scenery_sources_value(json: wasm_bindgen::JsValue, bytes: usize) {
    browser::scenery_sources_value(json, bytes);
}
fn trace_enabled() -> bool {
    crate::loading_trace::enabled()
}
mod profiles;
pub(crate) mod request;
mod response;

include!(concat!(env!("OUT_DIR"), "/terrain_kernel_identity.rs"));
#[cfg(feature = "actor-worker")]
pub(crate) fn kernel_identity() -> [u8; 32] { TERRAIN_KERNEL_ID }

#[derive(serde::Serialize, serde::Deserialize)]
struct Request {
    job: u64,
    kernel: [u8; 32],
    full_detail: bool,
    frame: request::Frame,
    samples: crate::mesh::worker_wire::SamplesWire,
    profiles: serde_json::Value,
}

pub(crate) struct SubmittedRequest {
    pub(crate) bytes: Vec<u8>,
    pub(crate) bindings: request::TextureBindings,
}

pub(crate) fn capture_request(
    job: u64,
    frame: &crystal_render_api::VisualWorldFrame,
    samples: &crate::mesh::TerrainImageSamples,
    document: &crate::live_profiles::Document,
) -> Result<SubmittedRequest, String> {
    let _trace = crate::loading_trace::span("worker.capture_request", || {
        serde_json::json!({
            "job":job,"tiles":frame.tiles.len(),"actors":frame.actors.len(),
        })
    });
    frame
        .validate()
        .map_err(|e| format!("invalid terrain request: {e:?}"))?;
    if !frame.active {
        return Err("inactive terrain does not submit worker jobs".into());
    }
    let (frame_wire, mut bindings) = request::Frame::capture(frame);
    let samples = crate::mesh::worker_wire::SamplesWire::capture(samples, &mut bindings, frame);
    let bytes = encode(&Request {
        job,
        kernel: TERRAIN_KERNEL_ID,
        full_detail: crate::new_bark_models::scenery_full_detail(),
        frame: frame_wire,
        samples,
        profiles: profiles::capture(document),
    })?;
    Ok(SubmittedRequest { bytes, bindings })
}

/// Execute only the immutable production terrain CPU pipeline. This function
/// starts no Bevy application and creates no image/material/mesh asset handles.
fn build_cpu_response(bytes: &[u8]) -> Result<response::Response, String> {
    let request: Request = {
        let _trace = crate::loading_trace::span(
            "worker.decode_request",
            || serde_json::json!({"bytes":bytes.len()}),
        );
        decode(bytes)?
    };
    if request.kernel != TERRAIN_KERNEL_ID {
        return Err("terrain worker canonical model/kernel identity mismatch; rebuild both artifacts from one source snapshot".into());
    }
    if request.full_detail != crate::new_bark_models::scenery_full_detail() {
        return Err("terrain worker scenery detail configuration mismatch".into());
    }
    let mut bindings = request::TextureBindings::worker(request.frame.texture_count())?;
    let frame = request.frame.restore(&bindings)?;
    if !frame.active {
        return Err("inactive terrain does not execute worker jobs".into());
    }
    let samples = request.samples.restore(&bindings)?;
    let profiles = profiles::restore(request.profiles)?;
    let pending =
        crate::mesh::begin_runtime_terrain_mesh_with_profiles(&frame, &samples, &profiles)
            .map_err(|e| format!("terrain build failed: {e:?}"))?;
    let built = crate::prepare_runtime_terrain(pending.into_streamed(), &frame);
    let slots = bindings.len();
    let _capture = crate::loading_trace::span("worker.capture_response", || {
        let meshes = built.textured_meshes.iter().chain(&built.solid_meshes)
            .map(|batch| &batch.mesh)
            .chain(built.scenery.iter().map(|(batch, _)| &batch.mesh))
            .chain(built.background.iter().map(|(mesh, _)| mesh))
            .chain(built.cave_enclosure.iter())
            .chain(built.instances.iter().map(|(mesh, _)| mesh))
            .chain(built.reveal_join_batches.iter().map(|(mesh, _)| mesh))
            .chain([&built.animated_textured_mesh, &built.animated_solid_mesh]);
        let (count, bytes) = meshes.fold((0_u64, 0_u64), |(count, bytes), mesh| {
            let attributes = mesh.try_attributes().map(|attributes|
                attributes.map(|(_, values)| values.get_bytes().len() as u64).sum::<u64>())
                .unwrap_or(0);
            let indices = mesh.try_indices_option().ok().flatten().map_or(0, |indices| match indices {
                bevy::mesh::Indices::U16(values) => values.len() as u64 * 2,
                bevy::mesh::Indices::U32(values) => values.len() as u64 * 4,
            });
            (count + 1, bytes + attributes + indices)
        });
        serde_json::json!({"owned_meshes":count,"transferred_mesh_bytes":bytes})
    });
    let response = response::Response::capture_owned(request.job, built, &frame, &mut bindings)?;
    if bindings.len() != slots {
        return Err("terrain output introduced an unbound image".into());
    }
    // Mesh allocations moved into the wire; remaining builder metadata was
    // dropped before encoding pages. No full mesh copy survives capture.
    Ok(response)
}

pub fn execute_cpu_job(bytes: &[u8]) -> Result<Vec<u8>, String> {
    encode(&build_cpu_response(bytes)?)
}

/// A separate worker owns these buffers until bounded pages are acknowledged.
pub struct CpuTerrainJob {
    job: u64,
    header: Vec<u8>,
    pages: blobs::PageSession,
    trace: String,
}
impl CpuTerrainJob {
    pub fn header(&self) -> &[u8] {
        &self.header
    }
    pub fn next_page(&mut self) -> Result<Option<Vec<u8>>, String> {
        self.pages.next_page()
    }
    pub fn acknowledge(&mut self, buffer: u32, offset: u64) -> Result<(), String> {
        self.pages.acknowledge(self.job, buffer, offset)
    }
    pub fn finished(&self) -> bool {
        self.pages.finished()
    }
    pub fn raw_bytes(&self) -> usize {
        self.pages.raw_bytes()
    }
    pub fn trace_json(&self) -> &str {
        &self.trace
    }
}

/// Run only on the terrain worker. Trace time zero belongs to that worker's
/// separate capture; the JavaScript bridge reports its epoch start/end clocks.
pub fn begin_paged_cpu_job(bytes: &[u8], trace: bool) -> Result<CpuTerrainJob, String> {
    struct TraceScope;
    impl Drop for TraceScope {
        fn drop(&mut self) {
            crate::loading_trace::stop();
        }
    }
    let _trace_scope = trace.then(|| {
        crate::loading_trace::start();
        TraceScope
    });
    let response = build_cpu_response(bytes)?;
    let job = response.job;
    let (header, pages) = blobs::encode_header(job, &response)?;
    drop(response);
    let trace = if trace {
        crate::loading_trace::snapshot()
    } else {
        String::new()
    };
    Ok(CpuTerrainJob {
        job,
        header,
        pages,
        trace,
    })
}

pub(crate) struct PagedTerrainRestore {
    job: u64,
    blobs: blobs::BlobStore,
    bindings: request::TextureBindings,
    restore: response::Restore,
    complete: bool,
}
impl PagedTerrainRestore {
    pub(crate) fn from_header(
        job: u64,
        bytes: &[u8],
        submitted: &SubmittedRequest,
    ) -> Result<Self, String> {
        let (response, blobs) = blobs::decode_header::<response::Response>(job, bytes)?;
        if response.job != job {
            return Err("terrain response job mismatch".into());
        }
        Ok(Self {
            job,
            blobs,
            bindings: submitted.bindings.clone(),
            restore: response::Restore::new(response)?,
            complete: false,
        })
    }
    pub(crate) fn append_page(&mut self, bytes: &[u8]) -> Result<(u32, u64), String> {
        blobs::append_packet(&mut self.blobs, self.job, bytes)
    }
    pub(crate) fn advance(
        &mut self,
        deadline: bevy::platform::time::Instant,
        max_elements: usize,
    ) -> Result<bool, String> {
        self.complete =
            self.restore
                .advance(&mut self.blobs, &self.bindings, deadline, max_elements)?;
        Ok(self.complete)
    }
    pub(crate) fn finish(self) -> Result<crate::BuiltTerrain, String> {
        if !self.complete || !self.blobs.all_consumed() {
            return Err("terrain pages or CPU restoration incomplete".into());
        }
        Ok(self.restore.finish())
    }
}

pub(crate) fn restore_response(
    bytes: &[u8],
    expected_job: u64,
    submitted: &SubmittedRequest,
) -> Result<crate::BuiltTerrain, String> {
    let response: response::Response = decode(bytes)?;
    if response.job != expected_job {
        return Err("terrain worker response job mismatch".into());
    }
    response.restore(&submitted.bindings)
}

const MAGIC: &[u8; 8] = b"GTRNW003";
const MAX_MESSAGE_BYTES: usize = 512 * 1024 * 1024;

fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes = MAGIC.to_vec();
    ciborium::into_writer(value, &mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err("terrain message exceeds size limit".into());
    }
    Ok(bytes)
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    if bytes.len() > MAX_MESSAGE_BYTES || !bytes.starts_with(MAGIC) {
        return Err("unsupported or oversized terrain message".into());
    }
    let mut reader = std::io::Cursor::new(&bytes[MAGIC.len()..]);
    let value = ciborium::from_reader(&mut reader).map_err(|e| e.to_string())?;
    if reader.position() as usize != bytes.len() - MAGIC.len() {
        return Err("trailing terrain message bytes".into());
    }
    Ok(value)
}

/// Install the same optional scenery sources before the first worker job.
/// No game content pack or actor catalog is loaded by this boundary.
pub fn install_scenery_sources(json: &str) -> Result<(), String> {
    crate::open_models::install_open_model_bundle(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crystal_render_api::*;
    use std::sync::Arc;

    #[test]
    fn worker_rejects_stale_kernel_and_legacy_request_before_any_geometry_build() {
        let frame = VisualWorldFrame::default();
        let (frame_wire, mut bindings) = request::Frame::capture(&frame);
        let samples = crate::mesh::worker_wire::SamplesWire::capture(
            &crate::mesh::TerrainImageSamples::default(),
            &mut bindings,
            &frame,
        );
        let mut kernel = TERRAIN_KERNEL_ID;
        kernel[0] ^= 1;
        let mut bytes = encode(&Request {
            job: 1,
            kernel,
            full_detail: crate::new_bark_models::scenery_full_detail(),
            frame: frame_wire,
            samples,
            profiles: serde_json::Value::Null,
        })
        .unwrap();
        assert!(
            execute_cpu_job(&bytes)
                .unwrap_err()
                .contains("kernel identity mismatch")
        );
        bytes[..8].copy_from_slice(b"GTRNW002");
        assert!(execute_cpu_job(&bytes).unwrap_err().contains("unsupported"));
    }

    #[test]
    fn worker_trace_scope_stops_after_rejected_request() {
        assert!(begin_paged_cpu_job(b"invalid", true).is_err());
        assert!(!crate::loading_trace::enabled());
        assert!(
            crate::loading_trace::span("worker.trace_shutdown_probe", || serde_json::json!({}))
                .is_none()
        );
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK; builds full source regions"]
    fn external_pack_regions_worker_roundtrip_preserves_exact_prepared_output() {
        let path = std::env::var("CRYSTAL_RENDER_TEST_PACK")
            .expect("set CRYSTAL_RENDER_TEST_PACK to a completed local pack");
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack = loaded.pack();
        let profiles = crate::live_profiles::LiveProfiles::default();
        for (job, map_name) in [
            "NewBarkTown",
            "ViridianCity",
            "Route2",
            "PlayersHouse1F",
            "GoldenrodMagnetTrainStation",
            "SaffronMagnetTrainStation",
            "DarkCaveVioletEntrance",
            "Route29",
        ]
        .into_iter()
        .enumerate()
        {
            let module = &pack.data().maps[map_name];
            let tileset = &module.attributes.tileset_name;
            let layout = &pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let sheet = image::load_from_memory(
                &pack.runtime_files()[&format!("gfx/tilesets/{tileset}.png")],
            )
            .unwrap()
            .to_rgba8();
            let width = module.attributes.width as u32 * 4;
            let height = module.attributes.height as u32 * 4;
            let bindings = request::TextureBindings::worker(1).unwrap();
            let mut images = Assets::<Image>::default();
            let mut art = std::collections::HashMap::new();
            let mut frame = VisualWorldFrame {
                active: true,
                map_id: Arc::from(map_name),
                source_map_size_core_tiles: Some(UVec2::new(width / 2, height / 2)),
                terrain_revision: 7,
                map_texture: bindings.handle(0).unwrap(),
                center: Vec2::ZERO,
                viewport_size: Vec2::new(160., 144.)
                    .min(Vec2::new(width as f32 * 8., height as f32 * 8.)),
                tile_size: Vec2::splat(8.),
                grid_size: UVec2::new(width, height),
                ..Default::default()
            };
            // These fixtures use actual authenticated source layouts and PNG
            // pixels; they do not claim compositor palette/animation acceptance.
            for row in 0..height {
                for column in 0..width {
                    let block = module.blocks[(row / 4 * (width / 4) + column / 4) as usize];
                    let tile_index =
                        layout[block as usize * 16 + (row % 4 * 4 + column % 4) as usize] as u16;
                    let texture = art
                        .entry(tile_index)
                        .or_insert_with(|| {
                            let columns = sheet.width() / 8;
                            let x = u32::from(tile_index) % columns * 8;
                            let y = u32::from(tile_index) / columns * 8;
                            assert!(
                                y + 8 <= sheet.height(),
                                "{map_name} tile {tile_index} outside supplied art"
                            );
                            let pixels = image::imageops::crop_imm(&sheet, x, y, 8, 8)
                                .to_image()
                                .into_raw();
                            images.add(Image::new(
                                bevy::render::render_resource::Extent3d {
                                    width: 8,
                                    height: 8,
                                    depth_or_array_layers: 1,
                                },
                                bevy::render::render_resource::TextureDimension::D2,
                                pixels,
                                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                bevy::asset::RenderAssetUsages::MAIN_WORLD,
                            ))
                        })
                        .clone();
                    frame.tiles.push(VisualTile {
                        column,
                        row,
                        texture,
                        animation_frames: None,
                        priority: false,
                        source: VisualTileSource {
                            tileset_id: Arc::from(tileset.as_str()),
                            metatile_id: block,
                            subtile_column: (column % 4) as u8,
                            subtile_row: (row % 4) as u8,
                            tile_index,
                        },
                    });
                }
            }
            // Actor occupancy is a real input to cave enclosure construction.
            frame.actors.push(VisualActor {
                id: VisualActorId::Player,
                source_id: Arc::from("player"),
                model_palette: None,
                texture: frame.map_texture.clone(),
                center: Vec2::new(-16., 16.),
                size: Vec2::splat(16.),
                flip_x: false,
                above_priority: false,
                facing: Some(Vec2::NEG_Y),
            });
            if map_name == "Route29" {
                frame.jump_ledges = vec![VisualJumpLedge {
                    start: IVec2::new(8, 8),
                    end: IVec2::new(12, 8),
                    descent: IVec2::Y,
                }]
                .into();
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Exercise native ownership preparation on actual authored map
                // input, alongside the complete terrain geometry roundtrip.
                let prepared = crate::water_surface::PreparedImages::for_frame(&frame);
                let mask = crate::cave_water::mask(&frame);
                let shore = mask.as_ref().and_then(|_| crate::water_surface::shoreline(&frame));
                for (actual, expected) in [(prepared.mask, mask), (prepared.shore, shore)] {
                    assert_eq!(actual.is_some(), expected.is_some(), "{map_name} water availability");
                    if let (Some(actual), Some(expected)) = (actual, expected) {
                        assert_eq!(actual.data, expected.data, "{map_name} water pixels");
                        assert_eq!(actual.texture_descriptor.size, expected.texture_descriptor.size);
                        assert_eq!(actual.texture_descriptor.format, expected.texture_descriptor.format);
                        assert_eq!(actual.texture_descriptor.dimension, expected.texture_descriptor.dimension);
                        assert_eq!(format!("{:?}",actual.sampler),format!("{:?}",expected.sampler));
                    }
                }
            }
            let samples = crate::mesh::TerrainImageSamples::capture(&frame, &images);
            let submitted =
                capture_request(job as u64, &frame, &samples, &profiles.document).unwrap();
            let started = std::time::Instant::now();
            let direct = crate::prepare_runtime_terrain(
                crate::mesh::begin_runtime_terrain_mesh_with_profiles(
                    &frame,
                    &samples,
                    &profiles.document,
                )
                .unwrap()
                .into_streamed(),
                &frame,
            );
            assert_eq!(direct.train_doors,matches!(map_name,"GoldenrodMagnetTrainStation"|"SaffronMagnetTrainStation"),
                "{map_name}: selected train metadata");
            let direct_ms = started.elapsed().as_secs_f64() * 1000.;
            let (_, mut bindings) = request::Frame::capture(&frame);
            let expected = encode(
                &response::Response::capture(job as u64, &direct, &frame, &mut bindings).unwrap(),
            )
            .unwrap();
            let started = std::time::Instant::now();
            let bytes = execute_cpu_job(&submitted.bytes).unwrap();
            let worker_ms = started.elapsed().as_secs_f64() * 1000.;
            assert!(
                bytes == expected,
                "{map_name} worker changed prepared output: lengths {} / {}, first differing byte {:?}",
                bytes.len(),
                expected.len(),
                bytes.iter().zip(&expected).position(|(a, b)| a != b)
            );
            let started = std::time::Instant::now();
            let restored = restore_response(&bytes, job as u64, &submitted).unwrap();
            let decode_ms = started.elapsed().as_secs_f64() * 1000.;
            let recaptured = encode(
                &response::Response::capture(job as u64, &restored, &frame, &mut bindings).unwrap(),
            )
            .unwrap();
            assert!(
                recaptured == expected,
                "{map_name} restore changed output: lengths {} / {}, first differing byte {:?}",
                recaptured.len(),
                expected.len(),
                recaptured.iter().zip(&expected).position(|(a, b)| a != b)
            );
            assert!(restore_response(&bytes, job as u64 + 1, &submitted).is_err());
            if std::env::var_os("CRYSTAL_TERRAIN_WORKER_PAGED_PROOF").is_some() {
                let mut producer = begin_paged_cpu_job(&submitted.bytes, false).unwrap();
                let header_bytes = producer.header().len();
                if std::env::var_os("CRYSTAL_TERRAIN_WORKER_HEADER_PROFILE").is_some() {
                    let header: ciborium::Value =
                        ciborium::from_reader(&producer.header()[16..]).unwrap();
                    if let ciborium::Value::Map(fields) = header {
                        for (name, value) in fields {
                            let mut encoded = Vec::new();
                            ciborium::into_writer(&value, &mut encoded).unwrap();
                            eprintln!(
                                "terrain_worker_header map={map_name} field={name:?} bytes={}",
                                encoded.len()
                            );
                        }
                    }
                }
                let header_started = std::time::Instant::now();
                let mut receiver =
                    PagedTerrainRestore::from_header(job as u64, producer.header(), &submitted)
                        .unwrap();
                let header_decode_ms = header_started.elapsed().as_secs_f64() * 1000.;
                let mut page_count = 0usize;
                let mut max_append_ms = 0f64;
                let mut max_restore_ms = 0f64;
                let mut turns = 0usize;
                loop {
                    // Production produces these on another thread. Keep its
                    // page-copy cost outside the measured main-thread slice.
                    let mut pages = Vec::new();
                    for _ in 0..2 {
                        if let Some(page) = producer.next_page().unwrap() {
                            pages.push(page)
                        }
                    }
                    for page in pages {
                        let started = std::time::Instant::now();
                        let ack = receiver.append_page(&page).unwrap();
                        max_append_ms = max_append_ms.max(started.elapsed().as_secs_f64() * 1000.);
                        producer.acknowledge(ack.0, ack.1).unwrap();
                        page_count += 1;
                    }
                    let started = bevy::platform::time::Instant::now();
                    let complete = receiver
                        .advance(started + std::time::Duration::from_millis(4), 8192)
                        .unwrap();
                    max_restore_ms = max_restore_ms.max(started.elapsed().as_secs_f64() * 1000.);
                    turns += 1;
                    if complete && producer.finished() {
                        break;
                    }
                    assert!(
                        turns < 1_000_000,
                        "paged restoration failed to make bounded progress"
                    );
                }
                let restored = receiver.finish().unwrap();
                let recaptured = encode(
                    &response::Response::capture(job as u64, &restored, &frame, &mut bindings)
                        .unwrap(),
                )
                .unwrap();
                assert!(
                    recaptured == expected,
                    "{map_name} paged restoration changed exact production output"
                );
                eprintln!(
                    "terrain_worker_paged map={map_name} header_bytes={header_bytes} header_decode_ms={header_decode_ms:.3} raw_bytes={} pages={page_count} turns={turns} max_append_ms={max_append_ms:.3} max_restore_ms={max_restore_ms:.3}",
                    producer.raw_bytes()
                );
            }
            eprintln!(
                "terrain_worker map={map_name} tiles={} request_bytes={} response_bytes={} direct_ms={direct_ms:.3} worker_encode_ms={worker_ms:.3} decode_ms={decode_ms:.3}",
                frame.tiles.len(),
                submitted.bytes.len(),
                bytes.len()
            );
        }
    }
}
