//! QA-only browser bridge. Rust owns all message validation and CPU restoration;
//! JavaScript owns Worker messages, transfers, and the short scheduling window.
use super::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalTerrainWorkerReady)]
    fn js_ready(activate: bool) -> Result<u8, JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalTerrainWorkerCapacity)]
    fn js_capacity() -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalTerrainWorkerSubmit)]
    fn js_submit(job: u32, request: &[u8], trace: bool) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalTerrainWorkerCancel)]
    fn js_cancel(job: u32) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalTerrainWorkerScenery)]
    fn js_scenery(json: JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalTerrainWorkerFallback)]
    fn js_fallback(error: &str) -> Result<(), JsValue>;
}

struct Pending {
    submitted: SubmittedRequest,
    restore: Option<PagedTerrainRestore>,
    result: Option<Result<crate::BuiltTerrain, String>>,
    transport_complete: bool,
}
thread_local! {
    static NEXT: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static PENDING: RefCell<BTreeMap<u32, Pending>> = const { RefCell::new(BTreeMap::new()) };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Availability {
    Disabled,
    Initializing,
    Ready,
}
pub(crate) fn availability() -> Availability {
    match js_ready(true).unwrap_or(0) {
        1 => Availability::Initializing,
        2 => Availability::Ready,
        _ => Availability::Disabled,
    }
}
pub(crate) fn fallback(error: &str) {
    let _ = js_fallback(error);
}
pub(crate) fn scenery_sources(json: Option<&str>) {
    scenery_sources_value(
        json.map_or(JsValue::NULL, JsValue::from_str),
        json.map_or(0, str::len),
    );
}
pub(crate) fn scenery_sources_value(json: JsValue, bytes: usize) {
    let _trace = crate::loading_trace::span(
        "worker.forward_scenery",
        || serde_json::json!({"bytes":bytes}),
    );
    // Retain the original validated response string until first 3D interest;
    // do not clone a large Rust String merely to turn it back into JavaScript.
    let _ = js_scenery(json);
}

pub(crate) struct Ticket(u32);

/// A busy worker is scheduling pressure, not a failed terrain implementation.
pub(crate) fn busy() -> bool {
    let capacity = js_capacity().unwrap_or(1).max(1) as usize;
    PENDING.with(|cell| cell.borrow().len() >= capacity)
}

/// Foreground demand supersedes one oldest ticket if every worker is occupied.
pub(crate) fn cancel_pending() {
    let id = PENDING.with(|cell| cell.borrow().first_key_value().map(|(&id, _)| id));
    if let Some(id) = id {
        cancel(id, "foreground_priority");
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        cancel(self.0, "owner_dropped");
    }
}
fn cancel(id: u32, reason: &'static str) {
    let removed = PENDING.with(|cell| {
        let mut state = cell.borrow_mut();
        state.remove(&id).is_some()
    });
    if removed {
        let _trace = crate::loading_trace::span("worker.cancel_request", || {
            serde_json::json!({"job": id, "reason": reason})
        });
        let _ = js_cancel(id);
    }
}
fn message(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:?}"))
}
pub(crate) fn submit(
    frame: &crystal_render_api::VisualWorldFrame,
    samples: &crate::mesh::TerrainImageSamples,
    document: &crate::live_profiles::Document,
) -> Result<Ticket, String> {
    if busy() {
        return Err("terrain worker already has a pending job".into());
    }
    let id = NEXT.with(|next| {
        let id = next.get().wrapping_add(1).max(1);
        next.set(id);
        id
    });
    let submitted = capture_request(u64::from(id), frame, samples, document)?;
    PENDING.with(|cell| {
        let capacity = js_capacity().unwrap_or(1).max(1) as usize;
        if cell.borrow().len() >= capacity {
            return Err("terrain worker already has a pending job".into());
        }
        cell.borrow_mut().insert(
            id,
            Pending {
                submitted,
                restore: None,
                result: None,
                transport_complete: false,
            },
        );
        Ok::<_, String>(())
    })?;
    let sent = PENDING.with(|cell| {
        let state = cell.borrow();
        let pending = state.get(&id).expect("submitted terrain ticket is present");
        js_submit(id, &pending.submitted.bytes, super::trace_enabled())
    });
    if let Err(error) = sent {
        cancel(id, "submit_failed");
        return Err(message(error));
    }
    Ok(Ticket(id))
}
pub(crate) enum Poll {
    Pending,
    Ready(crate::BuiltTerrain),
    Superseded,
    Failed(String),
}
pub(crate) fn poll(ticket: &Ticket) -> Poll {
    PENDING.with(|cell| {
        let mut state = cell.borrow_mut();
        let Some(pending) = state.get_mut(&ticket.0) else {
            return Poll::Superseded;
        };
        if pending.result.as_ref().is_some_and(Result::is_err) || pending.transport_complete {
            if let Some(result) = pending.result.take() {
                state.remove(&ticket.0);
                return match result {
                    Ok(built) => Poll::Ready(built),
                    Err(error) => Poll::Failed(error),
                };
            }
        }
        Poll::Pending
    })
}
fn with_pending<T>(
    id: u32,
    operation: impl FnOnce(&mut Pending) -> Result<T, String>,
) -> Result<T, JsValue> {
    PENDING.with(|cell| {
        let mut state = cell.borrow_mut();
        let pending = state
            .get_mut(&id)
            .ok_or_else(|| JsValue::from_str("stale terrain worker job"))?;
        operation(pending).map_err(|error| JsValue::from_str(&error))
    })
}
#[wasm_bindgen]
pub fn crystal_terrain_worker_header(id: u32, bytes: &[u8]) -> Result<(), JsValue> {
    with_pending(id, |p| {
        if p.restore.is_some() || p.result.is_some() {
            return Err("duplicate terrain header".into());
        }
        p.restore = Some(PagedTerrainRestore::from_header(
            u64::from(id),
            bytes,
            &p.submitted,
        )?);
        Ok(())
    })
}
#[wasm_bindgen]
pub fn crystal_terrain_worker_page(id: u32, bytes: &[u8]) -> Result<(), JsValue> {
    with_pending(id, |p| {
        p.restore
            .as_mut()
            .ok_or("terrain page precedes header")?
            .append_page(bytes)?;
        Ok(())
    })
}
#[wasm_bindgen]
pub fn crystal_terrain_worker_advance(id: u32, budget_ms: f64) -> Result<bool, JsValue> {
    with_pending(id, |p| {
        if p.result.is_some() {
            return Ok(true);
        }
        if !budget_ms.is_finite() || budget_ms < 0. {
            return Err("invalid terrain restore budget".into());
        }
        let deadline = bevy::platform::time::Instant::now()
            + std::time::Duration::from_secs_f64(budget_ms.min(4.) / 1000.);
        let ready = p
            .restore
            .as_mut()
            .ok_or("terrain advance precedes header")?
            .advance(deadline, 8192)?;
        if ready {
            p.result = Some(p.restore.take().unwrap().finish());
        }
        Ok(ready)
    })
}
#[wasm_bindgen]
pub fn crystal_terrain_worker_complete(id: u32) -> Result<(), JsValue> {
    with_pending(id, |p| {
        if p.result.is_none() {
            return Err("terrain transport completed before restoration".into());
        }
        p.transport_complete = true;
        Ok(())
    })
}
#[wasm_bindgen]
pub fn crystal_terrain_worker_error(id: u32, error: &str) {
    let _ = with_pending(id, |p| {
        p.restore = None;
        p.result = Some(Err(error.to_owned()));
        Ok(())
    });
}

#[wasm_bindgen]
pub fn crystal_terrain_worker_callback_timing(id: u32, stage: u8, milliseconds: f64, bytes: u32) {
    let name = match stage {
        0 => "worker.callback_header",
        1 => "worker.callback_page",
        2 => "worker.callback_advance",
        _ => return,
    };
    crate::loading_trace::external_elapsed(
        name,
        milliseconds,
        serde_json::json!({"job":id,"bytes":bytes}),
    );
}
