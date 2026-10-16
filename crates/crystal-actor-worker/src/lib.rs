//! Separate Wasm heap for actor CPU jobs. No game, DOM, or renderer startup.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = __crystalActorEpoch)]
    fn epoch_milliseconds() -> f64;
}

#[wasm_bindgen]
pub fn install_people_sources(json: &str) -> Result<(), JsValue> {
    crystal_voxel_view::actor_worker::install_people_sources(json)
        .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub struct ActorJob {
    inner: crystal_voxel_view::actor_worker::CpuActorJob,
    trace_origin_epoch: f64,
}
#[wasm_bindgen]
impl ActorJob {
    pub fn trace_origin_epoch(&self) -> f64 {
        self.trace_origin_epoch
    }
    pub fn header(&self) -> Vec<u8> {
        self.inner.header().to_vec()
    }
    pub fn next_page(&mut self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .next_page()
            .map(|page| page.unwrap_or_default())
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn acknowledge(&mut self, buffer: u32, offset: u32) -> Result<(), JsValue> {
        self.inner
            .acknowledge(buffer, u64::from(offset))
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn finished(&self) -> bool {
        self.inner.finished()
    }
    pub fn raw_bytes(&self) -> u32 {
        self.inner.raw_bytes() as u32
    }
    pub fn cache_json(&self) -> String {
        self.inner.cache_json().to_owned()
    }
    pub fn trace_json(&self) -> String {
        self.inner.trace_json().to_owned()
    }
}
#[wasm_bindgen]
pub fn begin_actor_job(request: &[u8], trace: bool) -> Result<ActorJob, JsValue> {
    // Read the worker's clock after wasm-bindgen has copied the request and
    // immediately before Rust starts its capture. The outer JS CPU span also
    // accounts for that input copy rather than losing it from the waterfall.
    let trace_origin_epoch = epoch_milliseconds();
    crystal_voxel_view::actor_worker::begin_paged_cpu_job(request, trace)
        .map(|inner| ActorJob {
            inner,
            trace_origin_epoch,
        })
        .map_err(|e| JsValue::from_str(&e))
}
