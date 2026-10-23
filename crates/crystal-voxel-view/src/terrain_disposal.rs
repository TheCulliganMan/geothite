//! Release rejected CPU terrain buffers away from the native frame thread.

/// A stale completed scene can own hundreds of megabytes of vertex buffers.
/// Its destructor is work too; rejecting that scene must not run it inline.
pub(super) fn retire<T: Send + 'static>(payload: T) {
    #[cfg(not(target_arch = "wasm32"))]
    bevy::tasks::AsyncComputeTaskPool::get()
        .spawn(async move {
            let _trace = crate::loading_trace::span(
                "region.dispose_cpu_buffers",
                || serde_json::json!({"thread": "background"}),
            );
            drop(payload);
        })
        .detach();
    #[cfg(target_arch = "wasm32")]
    drop(payload);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::sync::mpsc;
    use std::thread::{self, ThreadId};
    use std::time::Duration;

    struct DropReceipt(mpsc::Sender<ThreadId>);

    impl Drop for DropReceipt {
        fn drop(&mut self) {
            self.0
                .send(thread::current().id())
                .expect("receiver is waiting");
        }
    }

    #[test]
    fn native_rejected_scene_is_destroyed_on_a_background_thread() {
        bevy::tasks::AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let (sender, receiver) = mpsc::channel();
        let caller = thread::current().id();
        super::retire(DropReceipt(sender));
        let worker = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("retired scene is released");
        assert_ne!(caller, worker);
    }
}
