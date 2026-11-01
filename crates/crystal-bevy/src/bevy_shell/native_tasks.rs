//! Native preparation uses CPU workers rather than reserving a quarter of the
//! machine for the small amount of asynchronous filesystem work in this game.
use bevy::app::{TaskPoolOptions, TaskPoolPlugin};

pub(super) fn plugin() -> TaskPoolPlugin {
    TaskPoolPlugin {
        task_pool_options: options(bevy::tasks::available_parallelism()),
    }
}

fn options(available: usize) -> TaskPoolOptions {
    let available = available.max(1);
    let asynchronous = (available / 2).max(1);
    let compute = available.saturating_sub(asynchronous + 1).max(1);
    let mut options = TaskPoolOptions::with_num_threads(available);
    options.io.min_threads = 1;
    options.io.max_threads = 1;
    options.async_compute.min_threads = asynchronous;
    options.async_compute.max_threads = asynchronous;
    options.compute.min_threads = compute;
    options.compute.max_threads = compute;
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_preparation_scales_without_taking_frame_compute_workers_on_ten_cores() {
        let options = options(10);
        assert_eq!(options.io.min_threads, 1);
        assert_eq!(options.async_compute.min_threads, 5);
        assert_eq!(options.compute.min_threads, 4);
    }

    #[test]
    fn pool_assignments_are_nonzero_and_bounded_by_available_cpu_capacity() {
        for available in [0,1,2,3,4,8,10,16,32,64,128] {
            let options = options(available);
            let pools = [&options.io,&options.async_compute,&options.compute];
            for pool in pools {
                assert!(pool.min_threads >= 1);
                assert_eq!(pool.min_threads,pool.max_threads);
            }
            // Bevy requires all three pools, including on one/two-core hosts.
            let total = pools.iter().map(|pool| pool.min_threads).sum::<usize>();
            assert_eq!(total,available.max(3));
        }
    }
}
