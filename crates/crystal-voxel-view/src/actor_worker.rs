//! Optional CPU-only appearance worker. Controllers, poses and GPU assets stay
//! in the main application; this boundary transfers final immutable meshes.
use crate::new_bark_actors::character_meshes::{CharacterKind, SOURCE_KINDS};
use crate::open_people_skin::{PreparedAppearance, worker_wire::AppearanceWire};
use crate::terrain_worker::blobs::{self, BlobStore, PageSession};
use bevy::platform::time::Instant;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[cfg(all(target_arch = "wasm32", feature = "actor-worker-browser"))]
pub(crate) mod browser;
#[cfg(all(target_arch = "wasm32", feature = "actor-worker-browser"))]
pub fn browser_people_sources(json: Option<&str>) {
    browser::people_sources(json);
}
#[cfg(all(target_arch = "wasm32", feature = "actor-worker-browser"))]
pub fn browser_people_sources_value(json: wasm_bindgen::JsValue, bytes: usize) {
    browser::people_sources_value(json, bytes);
}

const REQUEST_MAGIC: &[u8; 8] = b"GACTR001";
const RESPONSE_VERSION: u16 = 3;
pub(crate) const MAX_APPEARANCES: usize = 8;
const MAX_REQUEST_BYTES: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Key {
    pub source: &'static str,
    pub base: &'static str,
    pub kind: CharacterKind,
}
pub(crate) fn key_for_source(source: &str) -> Result<Key, String> {
    let kind = crate::new_bark_actors::worker_kind_for_source(source)
        .ok_or("unsupported actor worker source")?;
    let canonical = SOURCE_KINDS
        .iter()
        .find_map(|&(name, candidate)| (candidate == kind).then_some(name))
        .or_else(|| (kind == CharacterKind::Outdoorsman).then_some("outdoorsman"))
        .ok_or("actor source has no canonical identity")?;
    let base = crate::npc_appearance::look(kind).base;
    if !matches!(base, "human-male" | "human-female") {
        return Err("actor worker encountered an unbounded production rig base".into());
    }
    Ok(Key {
        source: canonical,
        base,
        kind,
    })
}

fn bounded_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Visitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most eight appearances")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            use serde::de::Error;
            if seq.size_hint().is_some_and(|count| count > MAX_APPEARANCES) {
                return Err(A::Error::custom("too many actor appearances"));
            }
            let mut values = Vec::new();
            while let Some(value) = seq.next_element()? {
                if values.len() == MAX_APPEARANCES {
                    return Err(A::Error::custom("too many actor appearances"));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(Visitor(std::marker::PhantomData))
}
#[derive(Serialize, Deserialize)]
struct Request {
    job: u64,
    kernel: [u8; 32],
    #[serde(deserialize_with = "bounded_vec")]
    sources: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct Response {
    version: u16,
    job: u64,
    #[serde(deserialize_with = "bounded_vec")]
    appearances: Vec<(String, AppearanceWire)>,
}
pub(crate) fn capture_request(job: u64, keys: &[Key]) -> Result<Vec<u8>, String> {
    if keys.is_empty() || keys.len() > MAX_APPEARANCES {
        return Err("invalid actor batch length".into());
    }
    let mut seen = std::collections::HashSet::new();
    for key in keys {
        if key_for_source(key.source)? != *key || !seen.insert(key.source) {
            return Err("duplicate or invalid actor key".into());
        }
    }
    let request = Request {
        job,
        kernel: crate::terrain_worker::kernel_identity(),
        sources: keys.iter().map(|key| key.source.to_owned()).collect(),
    };
    let mut bytes = REQUEST_MAGIC.to_vec();
    ciborium::into_writer(&request, &mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}
fn decode_request(bytes: &[u8]) -> Result<Request, String> {
    if bytes.len() < 8 || bytes.len() > MAX_REQUEST_BYTES || &bytes[..8] != REQUEST_MAGIC {
        return Err("invalid actor request header/size".into());
    }
    let mut reader = std::io::Cursor::new(&bytes[8..]);
    let request: Request = ciborium::from_reader(&mut reader).map_err(|e| e.to_string())?;
    if reader.position() as usize != bytes.len() - 8 {
        return Err("trailing actor request bytes".into());
    }
    if request.kernel != crate::terrain_worker::kernel_identity() {
        return Err("actor worker kernel/model identity mismatch".into());
    }
    if request.sources.is_empty() {
        return Err("empty actor batch".into());
    }
    let mut seen = std::collections::HashSet::new();
    for source in &request.sources {
        let key = key_for_source(source)?;
        if key.source != source || !seen.insert(key.source) {
            return Err("duplicate or noncanonical actor source".into());
        }
    }
    Ok(request)
}

pub fn install_people_sources(json: &str) -> Result<(), String> {
    // The exact payload already installed by the main client is forwarded once.
    // The existing validator owns rig/painting semantics; no second catalog is authored.
    crate::open_people::install_open_people_bundle(json)
}
pub struct CpuActorJob {
    job: u64,
    header: Vec<u8>,
    pages: PageSession,
    trace: String,
    cache: String,
}
impl CpuActorJob {
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
    pub fn cache_json(&self) -> &str {
        &self.cache
    }
}
pub fn begin_paged_cpu_job(bytes: &[u8], trace: bool) -> Result<CpuActorJob, String> {
    struct TraceScope;
    impl Drop for TraceScope {
        fn drop(&mut self) {
            crate::loading_trace::stop();
        }
    }
    let _capture = trace.then(|| {
        crate::loading_trace::start();
        TraceScope
    });
    let request = {
        let _trace = crate::loading_trace::span(
            "actor_worker.decode_request",
            || serde_json::json!({"bytes":bytes.len()}),
        );
        decode_request(bytes)?
    };
    let mut appearances = Vec::new();
    for source in request.sources {
        let key = key_for_source(&source)?;
        let _trace = crate::loading_trace::span(
            "actor_worker.prepare",
            || serde_json::json!({"source":key.source,"base":key.base}),
        );
        let rig =
            crate::open_people::get(key.base).ok_or("required actor worker rig is unavailable")?;
        let appearance = crate::open_people_skin::prepare_appearance(rig, key.kind);
        let wire = AppearanceWire::capture(&appearance)?;
        drop(appearance);
        appearances.push((source, wire));
    }
    let response = Response {
        version: RESPONSE_VERSION,
        job: request.job,
        appearances,
    };
    let (header, pages) = blobs::encode_header(request.job, &response)?;
    let prepared_sources = response
        .appearances
        .iter()
        .map(|(source, _)| source.clone())
        .collect::<Vec<_>>();
    drop(response);
    let cache = serde_json::to_string(&serde_json::json!({
        "rigs":crate::open_people::worker_cache_keys(),"appearance_cache_keys":[],
        "last_appearance_keys":prepared_sources,
        "meaning":"fixed installed rig catalog; at most two canonical refined bases; final appearances transferred and dropped",
    })).map_err(|e| e.to_string())?;
    let trace = if trace {
        crate::loading_trace::snapshot()
    } else {
        String::new()
    };
    Ok(CpuActorJob {
        job: request.job,
        header,
        pages,
        trace,
        cache,
    })
}

pub(crate) struct Restore {
    job: u64,
    blobs: BlobStore,
    pending: VecDeque<(Key, AppearanceWire)>,
    ready: Vec<(Key, PreparedAppearance)>,
}
impl Restore {
    pub(crate) fn from_header(job: u64, header: &[u8], expected: &[Key]) -> Result<Self, String> {
        let (response, blobs) = blobs::decode_header::<Response>(job, header)?;
        if response.version != RESPONSE_VERSION
            || response.job != job
            || response.appearances.len() != expected.len()
        {
            return Err("actor response version/job/interest mismatch".into());
        }
        let mut pending = VecDeque::new();
        for ((source, appearance), &key) in response.appearances.into_iter().zip(expected) {
            if source != key.source {
                return Err("actor response canonical key mismatch".into());
            }
            pending.push_back((key, appearance));
        }
        Ok(Self {
            job,
            blobs,
            pending,
            ready: Vec::new(),
        })
    }
    pub(crate) fn append(&mut self, page: &[u8]) -> Result<(u32, u64), String> {
        blobs::append_packet(&mut self.blobs, self.job, page)
    }
    pub(crate) fn advance(&mut self, deadline: Instant) -> Result<bool, String> {
        while Instant::now() < deadline {
            let Some((_, wire)) = self.pending.front_mut() else {
                return Ok(true);
            };
            if !wire.resolve(&mut self.blobs)? {
                return Ok(false);
            }
            let (key, wire) = self.pending.pop_front().unwrap();
            self.ready.push((key, wire.restore()?));
        }
        Ok(self.pending.is_empty())
    }
    pub(crate) fn finish(self) -> Result<Vec<(Key, PreparedAppearance)>, String> {
        if !self.pending.is_empty() || !self.blobs.all_consumed() {
            return Err("actor response remains incomplete".into());
        }
        Ok(self.ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_actor_keys_preserve_alias_identity_and_bound_rig_catalog() {
        assert_eq!(
            key_for_source("chris_bike").unwrap(),
            key_for_source("chris").unwrap()
        );
        assert_eq!(
            key_for_source("kris_bike").unwrap(),
            key_for_source("kris").unwrap()
        );
        for (source, _) in SOURCE_KINDS {
            assert!(matches!(
                key_for_source(source).unwrap().base,
                "human-male" | "human-female"
            ));
        }
        assert!(key_for_source("invented_actor").is_err());
        assert_eq!(key_for_source("outdoorsman").unwrap().source, "outdoorsman");
    }
    #[test]
    #[ignore = "requires supplied character rigs"]
    fn complete_actor_job_pages_restore_exact_production_appearance() {
        let key = key_for_source("chris").unwrap();
        let request = capture_request(91, &[key]).unwrap();
        let mut job = begin_paged_cpu_job(&request, false).unwrap();
        assert!(job.header().len() < 8192);
        let mut restore = Restore::from_header(91, job.header(), &[key]).unwrap();
        assert!(
            Restore::from_header(91, job.header(), &[key_for_source("kris").unwrap()]).is_err()
        );
        assert!(!restore.advance(Instant::now()).unwrap());
        let mut page_count = 0;
        while let Some(page) = job.next_page().unwrap() {
            assert!(page.len() <= blobs::PAGE_BYTES + 32);
            let ack = restore.append(&page).unwrap();
            job.acknowledge(ack.0, ack.1).unwrap();
            let _ = restore
                .advance(Instant::now() + std::time::Duration::from_micros(50))
                .unwrap();
            page_count += 1;
        }
        assert!(job.finished());
        assert!(
            restore
                .advance(Instant::now() + std::time::Duration::from_secs(1))
                .unwrap()
        );
        let mut appearances = restore.finish().unwrap();
        assert_eq!(appearances.len(), 1);
        let (restored_key, appearance) = appearances.pop().unwrap();
        assert_eq!(restored_key, key);
        let rig = crate::open_people::get(key.base).unwrap();
        let direct = crate::open_people_skin::prepare_appearance(rig, key.kind);
        let encode = |appearance: &PreparedAppearance| {
            let mut bytes = Vec::new();
            ciborium::into_writer(&AppearanceWire::capture(appearance).unwrap(), &mut bytes)
                .unwrap();
            bytes
        };
        assert_eq!(encode(&appearance), encode(&direct));
        let cache: serde_json::Value = serde_json::from_str(job.cache_json()).unwrap();
        assert_eq!(cache["last_appearance_keys"], serde_json::json!(["chris"]));
        assert_eq!(cache["appearance_cache_keys"], serde_json::json!([]));
        eprintln!(
            "actor_worker_real header_bytes={} raw_bytes={} pages={} request_bytes={}",
            job.header().len(),
            job.raw_bytes(),
            page_count,
            request.len()
        );
    }

    #[test]
    fn actor_requests_reject_wrong_kernel_before_rig_lookup_and_shutdown_trace() {
        let key = key_for_source("chris").unwrap();
        let bytes = capture_request(7, &[key]).unwrap();
        assert_eq!(decode_request(&bytes).unwrap().sources, ["chris"]);
        let mut request = decode_request(&bytes).unwrap();
        request.kernel[0] ^= 1;
        let mut wrong = REQUEST_MAGIC.to_vec();
        ciborium::into_writer(&request, &mut wrong).unwrap();
        assert!(
            begin_paged_cpu_job(&wrong, true)
                .err()
                .unwrap()
                .contains("identity mismatch")
        );
        assert!(!crate::loading_trace::enabled());
        assert!(capture_request(1, &[]).is_err());
        assert!(capture_request(1, &[key; MAX_APPEARANCES + 1]).is_err());
        let mut trailing = bytes;
        trailing.push(0);
        assert!(decode_request(&trailing).is_err());
    }
}
