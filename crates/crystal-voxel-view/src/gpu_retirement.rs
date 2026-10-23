//! Poll completed native Metal work without collecting every buffer at submit.

use bevy::prelude::{App, Commands, IntoScheduleConfigs, Res, Resource};
use bevy::render::render_resource::PollType;
use bevy::render::renderer::{
    RenderAdapterInfo, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue,
};
use bevy::render::settings::Backends;
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Default)]
struct Progress {
    submitted: u64,
    completed: u64,
    stopped: bool,
}

#[derive(Default)]
struct Shared {
    progress: Mutex<Progress>,
    changed: Condvar,
}

impl Shared {
    fn complete(&self, submission: u64) {
        let mut progress = self.progress.lock().expect("GPU completion lock");
        progress.completed = progress.completed.max(submission);
        self.changed.notify_all();
    }

    fn stop(&self) {
        self.progress.lock().expect("GPU completion lock").stopped = true;
        self.changed.notify_all();
    }

    fn wait_for_room(&self, maximum_in_flight: u64) {
        let mut progress = self.progress.lock().expect("GPU completion lock");
        while !progress.stopped
            && progress.submitted.saturating_sub(progress.completed) >= maximum_in_flight
        {
            progress = self.changed.wait(progress).expect("GPU completion lock");
        }
    }
}

struct StopOnExit(Arc<Shared>);

impl Drop for StopOnExit {
    fn drop(&mut self) {
        self.0.stop();
    }
}

#[derive(Resource)]
struct Retirement {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
    admission: Option<(u64, bool)>,
}

impl Drop for Retirement {
    fn drop(&mut self) {
        self.shared.stop();
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            if worker.join().is_err() {
                bevy::log::error!("GPU retirement worker panicked");
            }
        }
    }
}

pub(super) fn register(app: &mut App) {
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render
        .add_systems(RenderStartup, initialize)
        .add_systems(Render, admit_assets.before(RenderSystems::PrepareAssets))
        .add_systems(
            RenderGraph,
            (
                admit.before(RenderGraphSystems::Submit),
                submitted
                    .after(RenderGraphSystems::Submit)
                    .before(RenderGraphSystems::Finish),
            ),
        );
}

fn initialize(mut commands: Commands, device: Res<RenderDevice>, adapter: Res<RenderAdapterInfo>) {
    if Backends::from(adapter.backend) != Backends::METAL {
        return;
    }
    let mode = std::env::var("CRYSTAL_GPU_RETIREMENT").unwrap_or_else(|_| "bounded".into());
    if mode == "off" {
        return;
    }
    let shared = Arc::new(Shared::default());
    let worker_shared = Arc::clone(&shared);
    let device = device.clone();
    let worker = thread::Builder::new()
        .name("gpu-retirement".into())
        .spawn(move || {
            let _stop_on_exit = StopOnExit(Arc::clone(&worker_shared));
            loop {
                if worker_shared
                    .progress
                    .lock()
                    .expect("GPU completion lock")
                    .stopped
                {
                    break;
                }
                {
                    let _trace = crate::loading_trace::span(
                        "gpu.retire_completed_work",
                        || serde_json::json!({"thread": "background", "wait_for_gpu": false}),
                    );
                    if let Err(error) = device.poll(PollType::Poll) {
                        worker_shared.stop();
                        bevy::log::warn!("GPU retirement polling stopped: {error}");
                        break;
                    }
                }
                thread::park_timeout(Duration::from_millis(1));
            }
        });
    match worker {
        Ok(worker) => commands.insert_resource(Retirement {
            shared,
            worker: Some(worker),
            admission: admission_policy(&mode),
        }),
        Err(error) => bevy::log::warn!("GPU retirement worker unavailable: {error}"),
    }
}

// The boolean selects asset preparation rather than the later graph submit.
// Keep the previous admission point available for same-binary comparisons.
fn admission_policy(mode: &str) -> Option<(u64, bool)> {
    match mode {
        "background" => None,
        "single" => Some((1, false)),
        "early-single" => Some((1, true)),
        "late" => Some((2, false)),
        _ => Some((2, true)),
    }
}

fn admit_assets(retirement: Option<Res<Retirement>>) {
    if let Some(retirement) = retirement
        && let Some((maximum, true)) = retirement.admission
    {
        wait_for_room(&retirement, maximum, "before_assets");
    }
}

fn admit(retirement: Option<Res<Retirement>>) {
    if let Some(retirement) = retirement
        && let Some((maximum, false)) = retirement.admission
    {
        wait_for_room(&retirement, maximum, "before_submit");
    }
}

fn wait_for_room(retirement: &Retirement, maximum: u64, stage: &'static str) {
    // Bound backlog before asset writes allocate another frame of staging
    // buffers. Admission only at submit leaves those writes ahead of the gate.
    // Use actual GPU completion, with no wall-clock or FPS pacing.
    let _trace = crate::loading_trace::span(
        "gpu.wait_for_submission_room",
        || serde_json::json!({"maximum_in_flight": maximum, "stage": stage}),
    );
    retirement.shared.wait_for_room(maximum);
}

fn submitted(retirement: Option<Res<Retirement>>, queue: Res<RenderQueue>) {
    let Some(retirement) = retirement.filter(|retirement| retirement.admission.is_some()) else {
        return;
    };
    let shared = Arc::clone(&retirement.shared);
    let submission = {
        let mut progress = shared.progress.lock().expect("GPU completion lock");
        progress.submitted += 1;
        progress.submitted
    };
    queue.on_submitted_work_done(move || shared.complete(submission));
}

#[cfg(test)]
mod tests {
    use super::{Shared, StopOnExit, admission_policy};
    use std::sync::{Arc, mpsc};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn admission_waits_before_asset_allocation_by_default_and_retains_late_controls() {
        assert_eq!(admission_policy("bounded"), Some((2, true)));
        assert_eq!(admission_policy("late"), Some((2, false)));
        assert_eq!(admission_policy("single"), Some((1, false)));
        assert_eq!(admission_policy("early-single"), Some((1, true)));
        assert_eq!(admission_policy("background"), None);
    }

    #[test]
    fn an_unwinding_poll_worker_releases_the_submission_gate() {
        let shared = Arc::new(Shared::default());
        shared.progress.lock().expect("test progress").submitted = 2;
        let worker_shared = Arc::clone(&shared);
        let worker = thread::spawn(move || {
            let _stop_on_exit = StopOnExit(worker_shared);
            panic!("simulated polling failure");
        });
        assert!(worker.join().is_err());
        assert!(shared.progress.lock().expect("test progress").stopped);
        shared.wait_for_room(2);
    }

    #[test]
    fn gpu_completion_releases_a_full_submission_window() {
        let shared = Arc::new(Shared::default());
        shared.progress.lock().expect("test progress").submitted = 2;
        let worker_shared = Arc::clone(&shared);
        let (send, receive) = mpsc::channel();
        let waiter = thread::spawn(move || {
            worker_shared.wait_for_room(2);
            send.send(()).expect("test receiver");
        });
        assert!(receive.recv_timeout(Duration::from_millis(10)).is_err());
        shared.complete(1);
        receive
            .recv_timeout(Duration::from_secs(1))
            .expect("completed GPU work opens a slot");
        waiter.join().expect("waiter finishes");
    }

    #[test]
    fn stopping_releases_waiters_even_without_a_device_completion() {
        let shared = Shared::default();
        shared.progress.lock().expect("test progress").submitted = 2;
        shared.stop();
        shared.wait_for_room(2);
        shared.complete(1);
        shared.complete(0);
        assert_eq!(shared.progress.lock().expect("test progress").completed, 1);
    }
}
