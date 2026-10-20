//! App-owned appearance staging. Worker messages cannot mutate poses or assets.
use super::*;
use crate::open_people_skin::PeopleGpu;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalActorWorkerReady)]
    fn js_ready(activate: bool) -> Result<u8, JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalActorWorkerSubmit)]
    fn js_submit(job: u32, request: &[u8], trace: bool) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalActorWorkerCancel)]
    fn js_cancel(job: u32) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalActorWorkerPeople)]
    fn js_people(json: JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __crystalActorWorkerFallback)]
    fn js_fallback(error: &str) -> Result<(), JsValue>;
}
struct Pending {
    id: u32,
    keys: Vec<Key>,
    restore: Option<Restore>,
    result: Option<Result<Vec<(Key, PreparedAppearance)>, String>>,
    complete: bool,
}
thread_local! {
    static NEXT: std::cell::Cell<u32> = const {std::cell::Cell::new(0)};
    static PENDING: RefCell<Option<Pending>> = const {RefCell::new(None)};
}
struct Ticket(u32);
impl Drop for Ticket {
    fn drop(&mut self) {
        let removed = PENDING.with(|cell| {
            let mut state = cell.borrow_mut();
            if state.as_ref().is_some_and(|p| p.id == self.0) {
                state.take();
                true
            } else {
                false
            }
        });
        if removed {
            let _ = js_cancel(self.0);
        }
    }
}
fn submit(keys: &[Key]) -> Result<Ticket, String> {
    let id = NEXT.with(|next| {
        let id = next.get().wrapping_add(1).max(1);
        next.set(id);
        id
    });
    let bytes = capture_request(u64::from(id), keys)?;
    PENDING.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.is_some() {
            return Err("actor worker already has pending work".to_owned());
        }
        *state = Some(Pending {
            id,
            keys: keys.to_vec(),
            restore: None,
            result: None,
            complete: false,
        });
        Ok(())
    })?;
    let ticket = Ticket(id);
    js_submit(id, &bytes, crate::loading_trace::enabled()).map_err(|e| format!("{e:?}"))?;
    Ok(ticket)
}
fn poll(ticket: &Ticket) -> Option<Result<Vec<(Key, PreparedAppearance)>, String>> {
    PENDING.with(|cell| {
        let mut state = cell.borrow_mut();
        let Some(p) = state.as_mut().filter(|p| p.id == ticket.0) else {
            return Some(Err("actor job cancelled".into()));
        };
        if p.complete || p.result.as_ref().is_some_and(Result::is_err) {
            if let Some(result) = p.result.take() {
                state.take();
                return Some(result);
            }
        }
        None
    })
}

#[derive(Default)]
pub(crate) struct Warmup {
    interest: Vec<Key>,
    pending: Option<Ticket>,
    failed: bool,
}
impl Warmup {
    pub(crate) fn prepared_uploads_enabled(&self) -> bool {
        !self.failed && js_ready(false).unwrap_or(0) == 2
    }
    pub(crate) fn pending(
        &mut self,
        keys: Vec<Key>,
        enabled: bool,
        gpu: &mut PeopleGpu,
    ) -> bool {
        if !enabled {
            self.pending = None;
            self.interest.clear();
            gpu.2.lock().expect("prepared actor lock").clear();
            return false;
        }
        if self.failed {
            return false;
        }
        if self.interest != keys {
            self.pending = None;
            self.interest = keys;
            gpu.2
                .lock()
                .expect("prepared actor lock")
                .retain(|(base, kind), _| {
                    self.interest
                        .iter()
                        .any(|k| k.base == *base && k.kind == *kind)
                });
        }
        if let Some(ticket) = &self.pending {
            match poll(ticket) {
                None => return true,
                Some(Ok(appearances)) => {
                    self.pending = None;
                    let mut staged = gpu.2.lock().expect("prepared actor lock");
                    for (key, appearance) in appearances {
                        if self.interest.contains(&key)
                            && !gpu.0.contains_key(&(key.base, key.kind))
                        {
                            staged.insert((key.base, key.kind), appearance);
                        }
                    }
                }
                Some(Err(error)) => {
                    self.pending = None;
                    self.failed = true;
                    let _ = js_fallback(&error);
                    return false;
                }
            }
        }
        let missing = {
            let staged = gpu.2.lock().expect("prepared actor lock");
            self.interest
                .iter()
                .copied()
                .filter(|key| {
                    !gpu.0.contains_key(&(key.base, key.kind))
                        && !staged.contains_key(&(key.base, key.kind))
                })
                .take(MAX_APPEARANCES)
                .collect::<Vec<_>>()
        };
        if missing.is_empty() {
            return false;
        }
        match js_ready(true).unwrap_or(0) {
            1 => true,
            2 => match submit(&missing) {
                Ok(ticket) => {
                    self.pending = Some(ticket);
                    true
                }
                Err(error) => {
                    self.failed = true;
                    let _ = js_fallback(&error);
                    false
                }
            },
            _ => {
                self.pending = None;
                false
            }
        }
    }
}
pub(crate) fn people_sources(json: Option<&str>) {
    if js_ready(false).unwrap_or(0) == 0 {
        return;
    }
    people_sources_value(
        json.map_or(JsValue::NULL, JsValue::from_str),
        json.map_or(0, str::len),
    );
}
pub(crate) fn people_sources_value(json: JsValue, bytes: usize) {
    if js_ready(false).unwrap_or(0) == 0 {
        return;
    }
    let _trace = crate::loading_trace::span(
        "actor_worker.forward_people",
        || serde_json::json!({"bytes":bytes}),
    );
    let _ = js_people(json);
}

fn with_pending<T>(
    id: u32,
    operation: impl FnOnce(&mut Pending) -> Result<T, String>,
) -> Result<T, JsValue> {
    PENDING.with(|cell| {
        let mut state = cell.borrow_mut();
        let p = state
            .as_mut()
            .filter(|p| p.id == id)
            .ok_or_else(|| JsValue::from_str("stale actor job"))?;
        operation(p).map_err(|e| JsValue::from_str(&e))
    })
}
#[wasm_bindgen]
pub fn crystal_actor_worker_header(id: u32, bytes: &[u8]) -> Result<(), JsValue> {
    with_pending(id, |p| {
        if p.restore.is_some() || p.result.is_some() {
            return Err("duplicate actor header".into());
        }
        p.restore = Some(Restore::from_header(u64::from(id), bytes, &p.keys)?);
        Ok(())
    })
}
#[wasm_bindgen]
pub fn crystal_actor_worker_page(id: u32, bytes: &[u8]) -> Result<(), JsValue> {
    with_pending(id, |p| {
        p.restore
            .as_mut()
            .ok_or("actor page before header")?
            .append(bytes)?;
        Ok(())
    })
}
#[wasm_bindgen]
pub fn crystal_actor_worker_advance(id: u32, budget_ms: f64) -> Result<bool, JsValue> {
    with_pending(id, |p| {
        if p.result.is_some() {
            return Ok(true);
        }
        if !budget_ms.is_finite() {
            return Err("invalid actor restore budget".into());
        }
        let deadline =
            Instant::now() + std::time::Duration::from_secs_f64(budget_ms.clamp(0., 4.) / 1000.);
        if p.restore
            .as_mut()
            .ok_or("actor advance before header")?
            .advance(deadline)?
        {
            p.result = Some(Ok(p.restore.take().unwrap().finish()?));
            return Ok(true);
        }
        Ok(false)
    })
}
#[wasm_bindgen]
pub fn crystal_actor_worker_complete(id: u32) -> Result<(), JsValue> {
    with_pending(id, |p| {
        p.complete = true;
        Ok(())
    })
}
#[wasm_bindgen]
pub fn crystal_actor_worker_error(id: u32, error: &str) {
    let _ = with_pending(id, |p| {
        p.result = Some(Err(error.to_owned()));
        Ok(())
    });
}
#[wasm_bindgen]
pub fn crystal_actor_worker_callback_timing(id: u32, stage: u8, milliseconds: f64, bytes: u32) {
    let name = match stage {
        0 => "actor_worker.callback_header",
        1 => "actor_worker.callback_page",
        2 => "actor_worker.callback_advance",
        _ => return,
    };
    crate::loading_trace::external_elapsed(
        name,
        milliseconds,
        serde_json::json!({"job":id,"bytes":bytes}),
    );
}
