//! Opt-in, bounded loading/render waterfall in Chrome trace format.
use bevy::{
    platform::time::Instant,
    prelude::*,
    render::{Render, RenderApp, RenderSystems},
};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

const CAPACITY: usize = 131_072;
#[derive(Clone, serde::Serialize)]
struct TraceEvent {
    name: &'static str,
    cat: &'static str,
    ph: &'static str,
    pid: u32,
    tid: u64,
    ts: f64,
    dur: f64,
    args: Value,
    #[serde(rename = "inProgress", skip_serializing_if = "std::ops::Not::not")]
    in_progress: bool,
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot<'a> {
    trace_events: Vec<&'a TraceEvent>,
    dropped_events: usize,
    display_time_unit: &'static str,
}
struct Capture {
    origin: Instant,
    events: VecDeque<Arc<TraceEvent>>,
    active: Vec<Arc<TraceEvent>>,
    dropped: usize,
}
static CAPTURE: OnceLock<Mutex<Option<Capture>>> = OnceLock::new();
static ENABLED: AtomicBool = AtomicBool::new(false);
#[cfg(not(target_arch = "wasm32"))]
static WRITING: AtomicBool = AtomicBool::new(false);
fn capture() -> &'static Mutex<Option<Capture>> {
    CAPTURE.get_or_init(|| Mutex::new(None))
}

pub fn start() {
    *capture().lock().unwrap() = Some(Capture {
        origin: Instant::now(),
        events: VecDeque::new(),
        active: Vec::new(),
        dropped: 0,
    });
    ENABLED.store(true, Ordering::Relaxed);
}
pub fn snapshot() -> String {
    let (events, dropped) = {
        let capture = capture().lock().unwrap();
        let Some(capture) = capture.as_ref() else {
            return encode_snapshot(&[], 0);
        };
        (
            capture.snapshot_events(),
            capture.dropped,
        )
    };
    // Serialize outside the capture lock; worker/render spans never wait for
    // a trace-file write or for the full JSON document to be encoded.
    encode_snapshot(&events, dropped)
}
fn encode_snapshot(events: &[Arc<TraceEvent>], dropped: usize) -> String {
    // Serialize borrowed typed events directly. Building a generic JSON Value
    // here clones every event/key/argument and doubles capture memory first.
    serde_json::to_string(&Snapshot {
        trace_events: events.iter().map(Arc::as_ref).collect(),
        dropped_events: dropped,
        display_time_unit: "ms",
    }).expect("trace events contain JSON values and finite timestamps")
}
pub struct Span {
    start: Instant,
    event: Option<Arc<TraceEvent>>,
}
pub fn span(name: &'static str, args: impl FnOnce() -> Value) -> Option<Span> {
    if !ENABLED.load(Ordering::Relaxed) {
        return None;
    }
    let start = Instant::now();
    let args = args();
    let mut capture = capture().lock().unwrap();
    let capture = capture.as_mut()?;
    if capture.events.len() + capture.active.len() >= CAPACITY {
        capture.dropped += 1;
        if capture.events.pop_front().is_none() { return None; }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let thread = format!("{:?}", std::thread::current().id())
        .trim_start_matches("ThreadId(").trim_end_matches(')')
        .parse::<u64>().unwrap_or(1);
    #[cfg(target_arch = "wasm32")]
    let thread = 1;
    let event = Arc::new(TraceEvent {
        name, cat: "geothite", ph: "X", pid: 1, tid: thread,
        ts: start.saturating_duration_since(capture.origin).as_secs_f64() * 1_000_000.,
        dur: 0., args, in_progress: false,
    });
    capture.active.push(event.clone());
    Some(Span { start, event: Some(event) })
}
impl Capture {
    fn snapshot_events(&self) -> Vec<Arc<TraceEvent>> {
        let elapsed = self.origin.elapsed().as_secs_f64() * 1_000_000.;
        let mut events: Vec<_> = self.events.iter().cloned().collect();
        events.extend(self.active.iter().map(|event| {
            let mut event = (**event).clone();
            event.dur = (elapsed - event.ts).max(0.);
            event.in_progress = true;
            Arc::new(event)
        }));
        events
    }
    fn complete(&mut self, event: Arc<TraceEvent>, duration: f64) {
        let Some(index) = self.active.iter().position(|active| Arc::ptr_eq(active, &event)) else {
            // A new capture may have replaced the one that started this span.
            return;
        };
        self.active.swap_remove(index);
        let mut event = Arc::try_unwrap(event).unwrap_or_else(|event| (*event).clone());
        event.dur = duration;
        self.events.push_back(Arc::new(event));
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        let end = Instant::now();
        let mut capture = capture().lock().unwrap();
        let Some(capture) = capture.as_mut() else {
            return;
        };
        if let Some(event) = self.event.take() {
            capture.complete(event, end.duration_since(self.start).as_secs_f64() * 1_000_000.);
        }
    }
}
pub(crate) fn phase(current: &mut Option<Span>, name: &'static str) {
    current.take();
    *current = span(name, || json!({}));
}
pub fn flush_native() {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(path) = std::env::var_os("CRYSTAL_LOADING_TRACE") {
        while WRITING.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        if let Err(error) = std::fs::write(&path, snapshot()) {
            bevy::log::warn!("write loading trace: {error}");
        }
    }
}
fn flush_background() {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(path) = std::env::var_os("CRYSTAL_LOADING_TRACE") {
        if !WRITING.swap(true, Ordering::Relaxed) {
            std::thread::spawn(move || {
                if let Err(error) = std::fs::write(path, snapshot()) {
                    bevy::log::warn!("write loading trace: {error}");
                }
                WRITING.store(false, Ordering::Relaxed);
            });
        }
    }
}
#[derive(Resource, Default)]
struct RenderCapture {
    frame: Option<Span>,
    stage: Option<Span>,
    sequence: u64,
}
#[derive(Resource, Default)]
struct MainCapture(Option<Span>);
pub(crate) fn register(app: &mut App) {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("CRYSTAL_LOADING_TRACE").is_some() && capture().lock().unwrap().is_none() {
        start();
    }
    app.init_resource::<MainCapture>();
    app.add_systems(First, |mut trace: ResMut<MainCapture>| {
        trace.0 = span("frame.main", || json!({}));
    });
    app.add_systems(Last, |mut trace: ResMut<MainCapture>| { trace.0.take(); });
    app.add_systems(Last, |mut elapsed: Local<f64>, time: Res<Time<Real>>| {
        if time.elapsed_secs_f64() - *elapsed >= 1. {
            *elapsed = time.elapsed_secs_f64();
            flush_background();
        }
    });
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.init_resource::<RenderCapture>();
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.sequence += 1;
                trace.frame = span("render.frame", || json!({"frame": trace.sequence}));
                trace.stage = span("render.prepare_assets", || json!({}));
            })
            .before(RenderSystems::PrepareAssets),
        );
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.stage.take();
                trace.stage = span("render.prepare_meshes", || json!({}));
            })
            .after(RenderSystems::PrepareAssets)
            .before(RenderSystems::PrepareMeshes),
        );
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.stage.take();
                trace.stage = span("render.prepare_views", || json!({}));
            })
            .after(RenderSystems::PrepareMeshes)
            .before(RenderSystems::CreateViews),
        );
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.stage.take();
                trace.stage = span("render.queue", || json!({}));
            })
            .after(RenderSystems::PrepareViews)
            .before(RenderSystems::Queue),
        );
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.stage.take();
                trace.stage = span("render.sort", || json!({}));
            })
            .after(RenderSystems::Queue)
            .before(RenderSystems::PhaseSort),
        );
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.stage.take();
                trace.stage = span("render.prepare_gpu", || json!({}));
            })
            .after(RenderSystems::PhaseSort)
            .before(RenderSystems::Prepare),
        );
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.stage.take();
                trace.stage = span("render.submit", || json!({}));
            })
            .before(RenderSystems::Render)
            .after(RenderSystems::Prepare),
        );
        render.add_systems(
            Render,
            (|mut trace: ResMut<RenderCapture>| {
                trace.stage.take();
                trace.frame.take();
            })
            .after(RenderSystems::Render)
            .before(RenderSystems::Cleanup),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_snapshot_keeps_chrome_fields_arguments_and_drop_count() {
        let event = Arc::new(TraceEvent { name:"region.build_source", cat:"geothite", ph:"X",
            pid:1, tid:3, ts:123.25, dur:456.75,
            args:json!({"map":"NewBarkTown", "nested":{"trees":1303}}), in_progress:false });
        let expected = json!({"traceEvents":[{"name":"region.build_source", "cat":"geothite",
            "ph":"X", "pid":1, "tid":3, "ts":123.25, "dur":456.75,
            "args":{"map":"NewBarkTown", "nested":{"trees":1303}}}],
            "droppedEvents":7, "displayTimeUnit":"ms"});
        for _ in 0..2 {
            let encoded: Value = serde_json::from_str(&encode_snapshot(&[event.clone()],7)).unwrap();
            assert_eq!(encoded,expected);
            assert_eq!(Arc::strong_count(&event),1, "snapshot must not retain event ownership");
        }
        let empty: Value = serde_json::from_str(&encode_snapshot(&[],0)).unwrap();
        assert_eq!(empty,json!({"traceEvents":[],"droppedEvents":0,"displayTimeUnit":"ms"}));
    }
    #[test]
    fn snapshot_exposes_unfinished_stage_and_completion_replaces_it() {
        let event = Arc::new(TraceEvent { name: "region.prepare_render_meshes",
            cat: "geothite", ph: "X", pid: 1, tid: 3, ts: 0., dur: 0.,
            args: json!({"map":"AzaleaTown"}), in_progress: false });
        let mut capture = Capture { origin: Instant::now(), events: VecDeque::new(),
            active: vec![event.clone()], dropped: 0 };
        let events = capture.snapshot_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].in_progress);
        assert!(events[0].dur >= 0.);
        assert_eq!(events[0].args["map"], "AzaleaTown");
        capture.complete(event, 1200.);
        let events = capture.snapshot_events();
        assert_eq!(events.len(), 1, "complete and active events must not duplicate a stage");
        assert!(!events[0].in_progress);
        assert_eq!(events[0].dur, 1200.);
        assert!(capture.active.is_empty());
    }
}
