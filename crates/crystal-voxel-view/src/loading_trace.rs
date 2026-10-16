//! Opt-in, bounded loading/render waterfall in Chrome trace format.
use bevy::{
    platform::time::Instant,
    prelude::*,
    render::{Render, RenderApp, RenderSystems},
};
use bevy::render::renderer::{RenderGraph, RenderGraphSystems};
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
/// End collection without discarding the final snapshot. A dedicated worker
/// owns its capture and must not keep tracing later unrequested jobs.
#[cfg(feature = "terrain-worker")]
pub(crate) fn enabled() -> bool { ENABLED.load(Ordering::Relaxed) }

#[cfg(feature = "terrain-worker")]
pub(crate) fn stop() {
    ENABLED.store(false, Ordering::Relaxed);
}

#[cfg(any(feature = "terrain-worker-browser", feature = "actor-worker-browser"))]
pub(crate) fn external_elapsed(name: &'static str, milliseconds: f64, args: Value) {
    if !ENABLED.load(Ordering::Relaxed) || !milliseconds.is_finite() || milliseconds < 0. { return; }
    let mut capture = capture().lock().unwrap();
    let Some(capture) = capture.as_mut() else { return; };
    if capture.events.len() + capture.active.len() >= CAPACITY {
        capture.dropped += 1;
        if capture.events.pop_front().is_none() { return; }
    }
    let end = capture.origin.elapsed().as_secs_f64() * 1_000_000.;
    capture.events.push_back(Arc::new(TraceEvent {
        name, cat: "geothite", ph: "X", pid: 1, tid: 1,
        ts: (end - milliseconds * 1000.).max(0.), dur: milliseconds * 1000.,
        args, in_progress: false,
    }));
}

pub fn snapshot() -> String {
    let (events, dropped) = {
        let capture = capture().lock().unwrap();
        let Some(capture) = capture.as_ref() else {
            return encode_snapshot(&[], 0);
        };
        (capture.snapshot_events(), capture.dropped)
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
    })
    .expect("trace events contain JSON values and finite timestamps")
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
        if capture.events.pop_front().is_none() {
            return None;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let thread = format!("{:?}", std::thread::current().id())
        .trim_start_matches("ThreadId(")
        .trim_end_matches(')')
        .parse::<u64>()
        .unwrap_or(1);
    #[cfg(target_arch = "wasm32")]
    let thread = 1;
    let event = Arc::new(TraceEvent {
        name,
        cat: "geothite",
        ph: "X",
        pid: 1,
        tid: thread,
        ts: start
            .saturating_duration_since(capture.origin)
            .as_secs_f64()
            * 1_000_000.,
        dur: 0.,
        args,
        in_progress: false,
    });
    capture.active.push(event.clone());
    Some(Span {
        start,
        event: Some(event),
    })
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
        let Some(index) = self
            .active
            .iter()
            .position(|active| Arc::ptr_eq(active, &event))
        else {
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
            capture.complete(
                event,
                end.duration_since(self.start).as_secs_f64() * 1_000_000.,
            );
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
/// Nested CPU schedule envelopes. GPU command execution is asynchronous, so
/// these are never GPU durations or additive to the outer render.submit span.
#[derive(Resource, Default)]
struct GraphCapture {
    frame: Option<Span>,
    stage: Option<Span>,
}
fn register_graph_boundaries(app: &mut bevy::app::SubApp) {
    app.init_resource::<GraphCapture>().add_systems(
        RenderGraph,
        (
            (|mut trace: ResMut<GraphCapture>| {
                trace.frame = span("render.graph", || json!({"timing":"cpu_schedule_envelope"}));
                trace.stage = span("render.graph_begin", || json!({}));
            }).before(RenderGraphSystems::Begin),
            (|mut trace: ResMut<GraphCapture>| {
                trace.stage.take();
                trace.stage = span("render.graph_encode", || json!({}));
            }).after(RenderGraphSystems::Begin).before(RenderGraphSystems::Render),
            (|mut trace: ResMut<GraphCapture>| {
                trace.stage.take();
                trace.stage = span("render.graph_submit", || json!({"timing":"cpu_schedule_envelope"}));
            }).after(RenderGraphSystems::Render).before(RenderGraphSystems::Submit),
            (|mut trace: ResMut<GraphCapture>| {
                trace.stage.take();
                trace.stage = span("render.graph_finish", || json!({}));
            }).after(RenderGraphSystems::Submit).before(RenderGraphSystems::Finish),
            (|mut trace: ResMut<GraphCapture>| {
                trace.stage.take();
                trace.frame.take();
            }).after(RenderGraphSystems::Finish),
        ).run_if(trace_enabled),
    );
}
#[derive(Resource, Default)]
struct MainCapture(Option<Span>);
/// Nested schedule envelopes, never additive to frame.main. Independent
/// systems may execute inside an envelope; this is not per-system self time.
#[derive(Resource, Default)]
struct PostUpdateCapture<const INDEX: usize>(Option<Span>);
const POST_UPDATE_NAMES: [&str; 4] = [
    "frame.transform_propagation",
    "frame.visibility_propagation",
    "frame.check_visibility",
    "frame.check_light_visibility",
];
fn trace_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
fn begin_post_update<const INDEX: usize>(mut trace: ResMut<PostUpdateCapture<INDEX>>) {
    trace.0 = span(POST_UPDATE_NAMES[INDEX], || {
        json!({
            "nested_in": "frame.main", "timing": "schedule_envelope",
        })
    });
}
fn end_post_update<const INDEX: usize>(mut trace: ResMut<PostUpdateCapture<INDEX>>) {
    trace.0.take();
}
fn register_post_update_boundaries(app: &mut App) {
    use bevy::camera::visibility::VisibilitySystems as V;
    use bevy::light::SimulationLightSystems as L;
    use bevy::transform::TransformSystems as T;
    app.init_resource::<PostUpdateCapture<0>>()
        .init_resource::<PostUpdateCapture<1>>()
        .init_resource::<PostUpdateCapture<2>>()
        .init_resource::<PostUpdateCapture<3>>();
    // End markers precede consumers already ordered after their measured set.
    // Do not order unrelated engine stages against one another for profiling.
    app.add_systems(
        PostUpdate,
        (
            begin_post_update::<0>.before(T::Propagate),
            end_post_update::<0>
                .after(T::Propagate)
                .before(V::VisibilityPropagate)
                .before(V::UpdateFrusta)
                .before(V::CalculateBounds)
                .before(L::UpdateBounds)
                .before(L::UpdateDirectionalLightCascades)
                .before(L::AssignLightsToClusters),
            begin_post_update::<1>
                .after(end_post_update::<0>)
                .before(V::VisibilityPropagate),
            end_post_update::<1>
                .after(V::VisibilityPropagate)
                .before(V::CheckVisibility),
            begin_post_update::<2>
                .after(end_post_update::<1>)
                .after(V::UpdateFrusta)
                .after(V::CalculateBounds)
                .before(V::CheckVisibility),
            end_post_update::<2>
                .after(V::CheckVisibility)
                .before(L::AssignLightsToClusters)
                .before(L::UpdateLightFrusta)
                .before(L::CheckLightVisibility)
                .before(V::MarkNewlyHiddenEntitiesInvisible),
            begin_post_update::<3>
                .after(end_post_update::<2>)
                .after(V::CalculateBounds)
                .after(L::UpdateLightFrusta)
                .before(L::CheckLightVisibility),
            end_post_update::<3>
                .after(L::CheckLightVisibility)
                .before(V::MarkNewlyHiddenEntitiesInvisible),
        )
            .run_if(trace_enabled),
    );
}

pub(crate) fn register(app: &mut App) {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("CRYSTAL_LOADING_TRACE").is_some() && capture().lock().unwrap().is_none() {
        start();
    }
    register_post_update_boundaries(app);
    app.init_resource::<MainCapture>();
    app.add_systems(First, |mut trace: ResMut<MainCapture>| {
        trace.0 = span("frame.main", || json!({}));
    });
    app.add_systems(Last, |mut trace: ResMut<MainCapture>| {
        trace.0.take();
    });
    app.add_systems(Last, |mut elapsed: Local<f64>, time: Res<Time<Real>>| {
        if time.elapsed_secs_f64() - *elapsed >= 1. {
            *elapsed = time.elapsed_secs_f64();
            flush_background();
        }
    });
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        // The engine graph uses weak ordering so independent stages may
        // overlap. Diagnostic envelopes need barriers, and must never install
        // those barriers in an ordinary untraced game. Runtime trace activation
        // still has the existing outer spans; opt in before app creation for
        // these finer graph boundaries.
        if trace_enabled() {
            register_graph_boundaries(render);
        }
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
    fn graph_boundaries_preserve_stock_schedule_order_and_close_spans() {
        #[derive(Resource, Default)]
        struct Witness(Vec<&'static str>);
        let mut app = bevy::app::SubApp::new();
        app.add_schedule(RenderGraph::base_schedule());
        register_graph_boundaries(&mut app);
        app.init_resource::<Witness>();
        app.add_systems(RenderGraph, (
            (|mut witness: ResMut<Witness>| witness.0.push("begin"))
                .in_set(RenderGraphSystems::Begin),
            (|mut witness: ResMut<Witness>| witness.0.push("encode"))
                .in_set(RenderGraphSystems::Render),
            (|mut witness: ResMut<Witness>| witness.0.push("submit"))
                .in_set(RenderGraphSystems::Submit),
            (|mut witness: ResMut<Witness>| witness.0.push("finish"))
                .in_set(RenderGraphSystems::Finish),
        ));
        // Repeated schedules also catch unclosed spans crossing frame boundaries.
        for _ in 0..2 {
            app.world_mut().resource_mut::<Witness>().0.clear();
            app.world_mut().run_schedule(RenderGraph);
            assert_eq!(app.world().resource::<Witness>().0,
                ["begin", "encode", "submit", "finish"]);
            let capture = app.world().resource::<GraphCapture>();
            assert!(capture.frame.is_none() && capture.stage.is_none());
        }
    }
    #[test]
    fn post_update_boundaries_preserve_engine_dependencies_and_close_nested_spans() {
        use bevy::camera::visibility::VisibilitySystems as V;
        use bevy::light::SimulationLightSystems as L;
        use bevy::transform::TransformSystems as T;
        #[derive(Resource, Default)]
        struct Witness(Vec<&'static str>);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin))
            .init_resource::<Witness>();
        register_post_update_boundaries(&mut app);
        // Reproduce the installed engine prerequisites, plus the production
        // fade->asset->bounds path, without starting a GPU or window.
        app.configure_sets(
            PostUpdate,
            (
                (V::UpdateFrusta, V::VisibilityPropagate)
                    .after(T::Propagate)
                    .before(V::CheckVisibility),
                V::CalculateBounds
                    .after(T::Propagate)
                    .after(bevy::asset::AssetEventSystems)
                    .before(V::CheckVisibility),
                V::MarkNewlyHiddenEntitiesInvisible
                    .after(V::CheckVisibility)
                    .after(L::CheckLightVisibility),
                L::UpdateBounds.after(T::Propagate).before(V::UpdateFrusta),
                L::AssignLightsToClusters
                    .after(T::Propagate)
                    .after(V::CheckVisibility),
                L::UpdateDirectionalLightCascades.after(T::Propagate),
                L::UpdateLightFrusta
                    .after(L::AssignLightsToClusters)
                    .after(L::UpdateDirectionalLightCascades)
                    .after(V::CheckVisibility),
                L::CheckLightVisibility
                    .after(V::CalculateBounds)
                    .after(T::Propagate)
                    .after(L::UpdateLightFrusta)
                    .after(V::CheckVisibility),
            ),
        );
        app.add_systems(
            PostUpdate,
            (
                (|capture: Res<PostUpdateCapture<1>>, mut witness: ResMut<Witness>| {
                    assert_eq!(capture.0.is_some(), trace_enabled());
                    witness.0.push("visibility_propagation");
                })
                .in_set(V::VisibilityPropagate),
                (|capture: Res<PostUpdateCapture<2>>, mut witness: ResMut<Witness>| {
                    assert_eq!(capture.0.is_some(), trace_enabled());
                    witness.0.push("visibility");
                })
                .in_set(V::CheckVisibility),
                (|capture: Res<PostUpdateCapture<3>>, mut witness: ResMut<Witness>| {
                    assert_eq!(capture.0.is_some(), trace_enabled());
                    witness.0.push("lights");
                })
                .in_set(L::CheckLightVisibility),
                (|| {})
                    .after(T::Propagate)
                    .before(bevy::asset::AssetEventSystems)
                    .before(V::VisibilityPropagate),
                (|| {}).after(L::CheckLightVisibility),
            ),
        );
        let root = app
            .world_mut()
            .spawn(Transform::from_xyz(17., 3., -8.))
            .id();
        let child = app.world_mut().spawn(Transform::from_xyz(2., 1., 5.)).id();
        app.world_mut().entity_mut(root).add_child(child);
        app.update();
        assert_eq!(
            app.world()
                .get::<GlobalTransform>(child)
                .unwrap()
                .translation(),
            Vec3::new(19., 4., -3.)
        );
        assert_eq!(
            app.world().resource::<Witness>().0,
            ["visibility_propagation", "visibility", "lights"]
        );
        assert!(app.world().resource::<PostUpdateCapture<0>>().0.is_none());
        assert!(app.world().resource::<PostUpdateCapture<1>>().0.is_none());
        assert!(app.world().resource::<PostUpdateCapture<2>>().0.is_none());
        assert!(app.world().resource::<PostUpdateCapture<3>>().0.is_none());
    }

    #[test]
    fn typed_snapshot_keeps_chrome_fields_arguments_and_drop_count() {
        let event = Arc::new(TraceEvent {
            name: "region.build_source",
            cat: "geothite",
            ph: "X",
            pid: 1,
            tid: 3,
            ts: 123.25,
            dur: 456.75,
            args: json!({"map":"NewBarkTown", "nested":{"trees":1303}}),
            in_progress: false,
        });
        let expected = json!({"traceEvents":[{"name":"region.build_source", "cat":"geothite",
            "ph":"X", "pid":1, "tid":3, "ts":123.25, "dur":456.75,
            "args":{"map":"NewBarkTown", "nested":{"trees":1303}}}],
            "droppedEvents":7, "displayTimeUnit":"ms"});
        for _ in 0..2 {
            let encoded: Value =
                serde_json::from_str(&encode_snapshot(&[event.clone()], 7)).unwrap();
            assert_eq!(encoded, expected);
            assert_eq!(
                Arc::strong_count(&event),
                1,
                "snapshot must not retain event ownership"
            );
        }
        let empty: Value = serde_json::from_str(&encode_snapshot(&[], 0)).unwrap();
        assert_eq!(
            empty,
            json!({"traceEvents":[],"droppedEvents":0,"displayTimeUnit":"ms"})
        );
    }
    #[test]
    fn snapshot_exposes_unfinished_stage_and_completion_replaces_it() {
        let event = Arc::new(TraceEvent {
            name: "region.prepare_render_meshes",
            cat: "geothite",
            ph: "X",
            pid: 1,
            tid: 3,
            ts: 0.,
            dur: 0.,
            args: json!({"map":"AzaleaTown"}),
            in_progress: false,
        });
        let mut capture = Capture {
            origin: Instant::now(),
            events: VecDeque::new(),
            active: vec![event.clone()],
            dropped: 0,
        };
        let events = capture.snapshot_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].in_progress);
        assert!(events[0].dur >= 0.);
        assert_eq!(events[0].args["map"], "AzaleaTown");
        capture.complete(event, 1200.);
        let events = capture.snapshot_events();
        assert_eq!(
            events.len(),
            1,
            "complete and active events must not duplicate a stage"
        );
        assert!(!events[0].in_progress);
        assert_eq!(events[0].dur, 1200.);
        assert!(capture.active.is_empty());
    }
}
