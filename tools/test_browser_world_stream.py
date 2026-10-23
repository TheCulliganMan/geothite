import unittest

from browser_world_stream import crossing_samples, gpu_summary, motion_report, primary_summary
from performance_audit_motion import motion_series
from performance_audit_residency import summarize


def event(time, name='A', shift=None, active=True, eye=None):
    return dict(name='presentation.world_motion', ts=time * 1000, args=dict(
        map=name, active=active, connected_shift=shift, view_center=[0., 0.],
        camera_eye=eye or [time / 10, 2., 0.], camera_rotation=[0., 0., 0., 1.],
        player_foot=[time / 10, 0.], tile_size=[1., 1.], terrain_map=name,
        connected_terrain=shift is not None))


class BrowserWorldStreamTests(unittest.TestCase):
    def test_profiling_off_frame_cadence_does_not_claim_camera_or_renderer_continuity(self):
        report = dict(success=True, scenario='long', browser='test', workers={},
                      intervals=dict(count=120, median=16., p95=20., maximum=35.))
        result = primary_summary(report)
        self.assertEqual(result['profiling'], 'off')
        for key in ['residency', 'stationary_camera', 'crossings']:
            self.assertIsNone(result[key])
        report['intervals']['maximum'] = float('nan')
        with self.assertRaises(ValueError):
            primary_summary(report)

    def test_unavailable_interval_and_censored_tail_remain_visible(self):
        trace = dict(droppedEvents=0, traceEvents=[event(0), event(10, active=False), event(1230)])
        result = summarize(motion_report(dict(success=True), trace))
        self.assertEqual(result['unavailable_ms'], 1220)
        self.assertEqual(result['unavailable_frames'], 1)
        self.assertEqual(result['unknown_intervals'], 1)

    def test_poisoned_camera_cannot_hide_behind_active_flag(self):
        trace = dict(droppedEvents=0, traceEvents=[event(0, eye=[0., None, 0.]), event(20)])
        result = summarize(motion_report(dict(success=True), trace))
        self.assertEqual(result['invalid_camera_frames'], 1)
        self.assertEqual(result['unavailable_ms'], 20)

    def test_seam_latency_uses_presentations_without_inventing_simulation_ticks(self):
        trace = dict(droppedEvents=0, traceEvents=[event(0), event(20, 'B', [2, 0]), event(55, 'B')])
        report = motion_report(dict(success=True), trace)
        self.assertTrue(all(frame['simulation_tick'] is None for frame in report['frames']))
        seam = crossing_samples(motion_series(report))[0]
        self.assertEqual(seam['sample_interval_ms'], 20)
        self.assertEqual(seam['next_motion_ms'], 35)

    def test_failed_or_incomplete_recorder_is_rejected(self):
        for success, dropped in [(False, 0), (True, 1)]:
            with self.assertRaises(ValueError):
                motion_report(dict(success=success), dict(droppedEvents=dropped, traceEvents=[event(0)]))
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            motion_report(dict(success=True), dict(droppedEvents=0, traceEvents=[event(0), event(0)]))

    def test_invalid_seam_retains_raw_interval_but_no_fast_rendered_latency(self):
        trace = dict(droppedEvents=0, traceEvents=[event(0), event(20, 'B', [2, 0], eye=[0., None, 0.]), event(55, 'B')])
        seam = crossing_samples(motion_series(motion_report(dict(success=True), trace)))[0]
        self.assertFalse(seam['valid'])
        self.assertIsNone(seam['sample_interval_ms'])
        self.assertEqual(seam['observed_sample_interval_ms'], 20)

    def test_parallel_gpu_spans_form_a_union_not_a_sum(self):
        result = gpu_summary(dict(supported=True, frames=[dict(passes=[
            dict(startNs='1000000', endNs='5000000'), dict(startNs='3000000', endNs='7000000'),
            dict(startNs='10000000', endNs='12000000')])]))
        self.assertEqual(result['median_envelope_ms'], 11)
        self.assertEqual(result['median_active_union_ms'], 8)
        self.assertFalse(gpu_summary(None)['available'])

    def test_empty_and_quantized_gpu_passes_preserve_valid_frame_measurements(self):
        result = gpu_summary(dict(supported=True, requestedFrames=120,
                                 passCapacityReached=True, frames=[dict(passes=[
            dict(startNs='0', endNs='0'),
            dict(startNs='1000000', endNs='1000000'),
            dict(startNs='1000000', endNs='5000000')])]))
        self.assertEqual(result['median_envelope_ms'], 4)
        self.assertEqual(result['median_active_union_ms'], 4)
        self.assertEqual(result['empty_or_unwritten_passes'], 1)
        self.assertEqual(result['zero_duration_quantized_passes'], 1)
        self.assertEqual(result['frames'], 1)
        self.assertEqual(result['requested_frames'], 120)
        self.assertTrue(result['pass_capacity_reached'])

    def test_inverted_gpu_timestamp_invalidates_its_frame(self):
        result = gpu_summary(dict(supported=True, frames=[dict(passes=[
            dict(startNs='1000000', endNs='5000000'),
            dict(startNs='4000000', endNs='3000000')])]))
        self.assertFalse(result['available'])
        self.assertEqual(result['invalid_frames'], 1)
        self.assertEqual(result['invalid_passes'], 1)

    def test_truncated_gpu_submission_is_not_a_fast_complete_frame(self):
        result = gpu_summary(dict(supported=True, frames=[dict(complete=False, passes=[
            dict(startNs='1000000', endNs='2000000')])]))
        self.assertFalse(result['available'])
        self.assertEqual(result['incomplete_frames'], 1)
        self.assertEqual(result['invalid_frames'], 0)


if __name__ == '__main__':
    unittest.main()
