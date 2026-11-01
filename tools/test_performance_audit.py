import copy
from argparse import Namespace
import contextlib
import io
import sys
import json
from pathlib import Path
import tempfile
import unittest

from performance_audit import analyze, compare, compatibility, continuous_presentation, cpu_census, digest_tree, frame_budget, repeat_stability, run_suite, stats

from trace_stream import spans as stream_spans


class AuditTests(unittest.TestCase):
    def test_source_2d_capture_does_not_require_a_3d_presentation_scale(self):
        report = dict(schema=1, renderer='2d', success=True, failure=None,
                      frames=[dict(id=i, measured=True, interval_ms=1, main_cpu_ms=.5,
                                   motion=dict(tile_size=[0., 0.])) for i in range(40)],
                      gpu=dict(samples=[], supported=False, reason='disabled', errors=0, skipped=0),
                      render_diagnostics=[], checkpoints=[])
        with tempfile.TemporaryDirectory() as directory:
            result = analyze(report, Path(directory), detailed_cpu=False)
        self.assertTrue(result['success'])
        self.assertEqual(result['renderer'], '2d')
        self.assertFalse(result['world_residency']['available'])
        self.assertIsNone(result['stationary_camera'])

    def test_source_2d_and_modeled_captures_are_not_comparable(self):
        metadata = dict.fromkeys((
            'measurement_revision', 'warmup_cadence', 'native_capture_mode',
            'trace_filter', 'gpu_backend', 'scenario', 'pack_identity', 'os',
            'arch', 'debug_assertions', 'operation_trace', 'gpu_instrumentation',
            'viewport', 'hour', 'target', 'pipeline_compilation', 'simulation',
            'cache_policy'))
        modeled = compatibility(metadata, {})
        source = compatibility(dict(metadata, renderer='2d'), {})
        self.assertEqual(modeled['renderer'], '3d')
        self.assertEqual(source['renderer'], '2d')
        self.assertNotEqual(source, modeled)
        self.assertFalse(compare(dict(compatibility=source, runs=[]),
                                 dict(compatibility=modeled, runs=[]), 10, .2)['comparable'])

    def test_a_render_stall_cannot_hide_behind_continuously_active_3d(self):
        run = dict(world_residency=dict(available=True, unavailable_frames=0,
                                       unknown_camera_frames=0, unknown_intervals=0),
                   stationary_camera=dict(observed_pairs=3, max_vertical_cells=0., max_horizontal_cells=0.),
                   interval_ms=dict(maximum=1500.))
        self.assertEqual(continuous_presentation(run), dict(passed=False,
                         reasons=['Peak 1500.000 ms exceeds 45 ms']))
        run['interval_ms']['maximum'] = 40.
        self.assertTrue(continuous_presentation(run)['passed'])

    def test_hard_frame_ceiling_rejects_one_spike_even_with_good_percentiles(self):
        self.assertFalse(frame_budget(stats([16.] * 1000 + [45.001]))['passed'])
        self.assertTrue(frame_budget(stats([16.] * 1000 + [45.]))['passed'])
        for value in [None, {}, dict(maximum=float('nan')), dict(maximum=0.)]:
            self.assertFalse(frame_budget(value)['passed'])
        for limit in [0., -1., float('inf')]:
            with self.assertRaises(ValueError):
                frame_budget(stats([16.]), limit)

    def test_unknown_or_nonfinite_camera_drift_is_not_a_continuity_pass(self):
        run = dict(world_residency=dict(available=True, unavailable_frames=0),
                   stationary_camera=dict(observed_pairs=3, max_vertical_cells=float('nan'), max_horizontal_cells=0.))
        self.assertFalse(continuous_presentation(run)['passed'])
        run['stationary_camera']['observed_pairs'] = 0
        self.assertFalse(continuous_presentation(run)['passed'])
        run['stationary_camera'] = None
        self.assertFalse(continuous_presentation(run)['passed'])

    def test_artwork_identity_includes_file_names_and_contents(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            with self.assertRaisesRegex(ValueError,'empty'):
                digest_tree(root)
            path=root/'person.glb';path.write_bytes(b'first')
            first=digest_tree(root)
            self.assertEqual(digest_tree(root),first)
            path.write_bytes(b'second')
            second=digest_tree(root)
            self.assertNotEqual(second,first)
            path.rename(root/'other.glb')
            self.assertNotEqual(digest_tree(root),second)

    def test_quantiles_and_empty_gpu_are_not_invented(self):
        self.assertIsNone(stats([]))
        sample = stats(range(1, 101))
        self.assertEqual((sample['p50'], sample['p95'], sample['p99']), (50.5, 95, 99))

    def test_repeat_spread_excludes_failed_runs_and_warns_about_noise(self):
        entry = dict(runs=[dict(success=True, interval_ms=stats([10])),
                           dict(success=True, interval_ms=stats([15])),
                           dict(success=False, interval_ms=stats([100]))])
        result = repeat_stability(entry)['interval_ms']
        self.assertEqual((result['repeats'], result['median'], result['spread_percent']), (2,12.5,40))
        self.assertTrue(result['noisy'])
        entry['runs'] = entry['runs'][:1]
        self.assertEqual(repeat_stability(entry), {})

    def test_exclusive_time_clips_nested_work_and_keeps_threads_independent(self):
        events = [dict(ph='B', name='frame', ts=0, tid=0),
                  dict(ph='B', name='render', ts=0, tid=1),
                  dict(ph='B', name='system', ts=10, tid=0),
                  dict(ph='B', name='leaf', ts=20, tid=0),
                  dict(ph='E', ts=50, tid=0), dict(ph='E', ts=90, tid=0),
                  dict(ph='E', ts=100, tid=0), dict(ph='E', ts=100, tid=1)]
        rows = {row['name']: row for row in stream_spans(iter(events), {}, {}, (15, 70))}
        self.assertAlmostEqual(rows['frame']['self_duration'], 0)
        self.assertAlmostEqual(rows['system']['self_duration'], 25)
        self.assertAlmostEqual(rows['leaf']['self_duration'], 30)
        self.assertAlmostEqual(rows['render']['self_duration'], 55)

    def test_cpu_census_excludes_startup_and_warmup_by_frame_id(self):
        events = []
        for frame in range(1, 5):
            events.extend([dict(ph='X', name='audit_frame_marker', args={'frame_id': frame}, ts=frame*100, dur=0, tid=0),
                           dict(ph='X', name='work', ts=frame*100+1, dur=50, tid=0)])
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'trace.json'
            path.write_text(json.dumps(events))
            rows, note = cpu_census(path, {2, 3})
        self.assertEqual(rows[0]['calls'], 2)
        self.assertAlmostEqual(rows[0]['inclusive_ms'], .1)
        self.assertIn('measured frame window only', note)

    def test_baseline_gates_both_absolute_and_relative_thresholds(self):
        run = dict(success=True, interval_ms=stats([10]*40), main_cpu_ms=stats([4]*40),
                   gpu_ms=None, checkpoints=[dict(latency_ms=100)])
        before = dict(compatibility={'pack': 'same'}, runs=[run, copy.deepcopy(run), copy.deepcopy(run)])
        after = copy.deepcopy(before)
        for run in after['runs']:
            run['interval_ms'] = stats([12]*40)
            run['checkpoints'][0]['latency_ms'] = 130
        result = compare(after, before, 10, .2)
        self.assertTrue(result['comparable'])
        self.assertEqual([r['metric'] for r in result['regressions']], ['interval_ms', 'checkpoint_0_ms'])
        after['compatibility']['pack'] = 'different'
        self.assertFalse(compare(after, before, 10, .2)['comparable'])
        after = copy.deepcopy(before)
        after['runs'][0]['success'] = False
        self.assertFalse(compare(after, before, 10, .2)['comparable'])
        after['runs'] = []
        self.assertFalse(compare(after, before, 10, .2)['comparable'])

    def test_failed_process_still_produces_reviewable_failure_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fake_pack = root/'exit.py'
            fake_pack.write_text('raise SystemExit(7)\n')
            suite = root/'suite.json'
            suite.write_text(json.dumps([dict(name='failure-test')]))
            args = Namespace(binary=Path(sys.executable),pack=fake_pack,output=root/'result',suite=suite,
                             scenario=None,repeats=1,baseline=None,cpu_only=False,timeout=5,
                             threshold_percent=10,threshold_ms=.2)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(run_suite(args),1)
            summary=json.loads((args.output/'summary.json').read_text())
            self.assertFalse(summary['scenarios'][0]['runs'][0]['success'])
            self.assertTrue((args.output/'index.html').exists())
            with self.assertRaisesRegex(ValueError,'must be empty'):
                run_suite(args)

    def test_missing_timestamp_coverage_fails_instead_of_reporting_fast_gpu(self):
        frames = [dict(id=i, measured=True, interval_ms=10, main_cpu_ms=3) for i in range(40)]
        report = dict(schema=1, success=True, failure=None, frames=frames,
                      gpu=dict(samples=[], supported=True, reason='', errors=0, skipped=40),
                      render_diagnostics=[], checkpoints=[])
        with tempfile.TemporaryDirectory() as directory:
            result = analyze(report, Path(directory))
        self.assertFalse(result['success'])
        self.assertIsNone(result['gpu_ms'])
        self.assertEqual(result['gpu_coverage'], 0)
        report['gpu']['samples']=[dict(frame=0,elapsed_ms=1,start_ns=1,end_ns=1000001) for _ in range(40)]
        with tempfile.TemporaryDirectory() as directory:
            duplicate = analyze(report,Path(directory))
            self.assertFalse(duplicate['success'])
            self.assertAlmostEqual(duplicate['gpu_coverage'], 1/40)
        report['gpu']['samples']=[dict(frame=i,elapsed_ms=0,start_ns=0,end_ns=0) for i in range(40)]
        with tempfile.TemporaryDirectory() as directory:
            self.assertFalse(analyze(report,Path(directory))['success'])

    def test_native_warmup_errors_are_retained_but_only_measured_frames_gate(self):
        frames = [dict(id=i, measured=True, interval_ms=10, main_cpu_ms=3) for i in range(100, 140)]
        report = dict(schema=1, success=True, failure=None, frames=frames,
                      gpu=dict(samples=[dict(frame=i, elapsed_ms=1, start_ns=i*1000000,
                                             end_ns=i*1000000+1000000) for i in range(100,140)],
                               supported=True, reason='native', errors=1, invalid_frames=[10], skipped=1),
                      render_diagnostics=[], checkpoints=[])
        with tempfile.TemporaryDirectory() as directory:
            result = analyze(report, Path(directory), detailed_cpu=False)
            self.assertTrue(result['success'])
            self.assertEqual((result['gpu_errors'], result['gpu_capture_errors']), (0,1))
            report['gpu']['invalid_frames'] = [120]
            self.assertFalse(analyze(report, Path(directory), detailed_cpu=False)['success'])

    def test_sampling_duration_is_not_charted_as_response_latency(self):
        report = dict(schema=1, success=True, failure=None,
                      frames=[dict(id=i, measured=True, interval_ms=1, main_cpu_ms=.5) for i in range(40)],
                      scenario=dict(steps=[dict(until=dict(seconds=5, map=None, tile=None, mode=None,
                                                        battle=None, battle_commands=None, pp_consumed=False))]),
                      gpu=dict(samples=[], supported=False, reason='disabled', errors=0, skipped=0),
                      render_diagnostics=[], checkpoints=[dict(step=0, preparation=False)])
        with tempfile.TemporaryDirectory() as directory:
            self.assertTrue(analyze(report, Path(directory), detailed_cpu=False)['checkpoints'][0]['duration_only'])

    def test_motion_readback_qa_cannot_be_accepted_as_a_timing_benchmark(self):
        report = dict(schema=1, success=True, failure=None,
                      frames=[dict(id=i, measured=True, interval_ms=1, main_cpu_ms=.5) for i in range(40)],
                      gpu=dict(samples=[], supported=False, reason='disabled', errors=0, skipped=0),
                      render_diagnostics=[], checkpoints=[], motion_capture=dict(enabled=True))
        with tempfile.TemporaryDirectory() as directory:
            result = analyze(report, Path(directory), detailed_cpu=False)
        self.assertFalse(result['success'])
        self.assertIn('visual QA capture is not a benchmark', result['issues'][0])

    def test_checkpoint_cannot_be_verified_in_the_same_observation_as_its_new_result(self):
        report = dict(schema=1, success=True, failure=None,
                      frames=[dict(id=i,measured=True,interval_ms=1,main_cpu_ms=.5) for i in range(40)],
                      gpu=dict(samples=[],supported=False,reason='control',errors=0,skipped=0),
                      render_diagnostics=[],checkpoints=[dict(step=0,preparation=False,render_settle_ms=0)])
        with tempfile.TemporaryDirectory() as directory:
            self.assertFalse(analyze(report,Path(directory),detailed_cpu=False,measurement_revision=7)['success'])
            report['checkpoints'][0]['render_settle_ms']=10
            self.assertTrue(analyze(report,Path(directory),detailed_cpu=False,measurement_revision=7)['success'])
            legacy=analyze(report,Path(directory),detailed_cpu=False,measurement_revision=6)
            self.assertFalse(legacy['success'])
            self.assertIn('Legacy checkpoint readiness',legacy['issues'][0])
            self.assertIsNotNone(legacy['interval_ms'])

    def test_continuous_legs_do_not_claim_renderer_settlement_or_input_latency(self):
        report = dict(schema=1, success=True, failure=None,
                      frames=[dict(id=i, measured=True, interval_ms=1, main_cpu_ms=.5) for i in range(40)],
                      gpu=dict(samples=[], supported=False, reason='control', errors=0, skipped=0),
                      render_diagnostics=[], scenario=dict(continuous_movement=True, steps=[dict(until=dict(tile=[1, 2]))]),
                      checkpoints=[dict(step=0, preparation=False, render_settle_ms=0, renderer_waited=False)])
        with tempfile.TemporaryDirectory() as directory:
            result = analyze(report, Path(directory), detailed_cpu=False, measurement_revision=8)
            self.assertTrue(result['success'])
            self.assertTrue(result['checkpoints'][0]['movement_leg'])
            self.assertTrue(result['checkpoints'][0]['duration_only'])
            report['scenario']['continuous_movement'] = False
            self.assertFalse(analyze(report, Path(directory), detailed_cpu=False, measurement_revision=8)['success'])

    def test_missing_native_boundary_is_a_visible_drop_and_bad_gpu_time_is_an_error(self):
        report = dict(schema=1, success=True, failure=None,
                      frames=[dict(id=i, measured=True, interval_ms=1, main_cpu_ms=.5) for i in range(100)],
                      gpu=dict(samples=[dict(frame=i, elapsed_ms=1, start_ns=i*1000000+1,
                                             end_ns=i*1000000+1000001) for i in range(99)],
                               supported=True, backend='metal_system_trace', reason='native',
                               errors=1, invalid_frames=[99], skipped=0,
                               error_details=['Unclosed source frame 99 before 100']),
                      render_diagnostics=[], checkpoints=[])
        with tempfile.TemporaryDirectory() as directory:
            result = analyze(report, Path(directory), detailed_cpu=False)
            self.assertTrue(result['success'])
            self.assertEqual((result['gpu_coverage'], result['gpu_dropped_frames'], result['gpu_errors']), (.99,1,0))
            report['gpu']['error_details'] = ['Invalid GPU interval for frame 99']
            self.assertFalse(analyze(report, Path(directory), detailed_cpu=False)['success'])
            report['gpu'].update(samples=report['gpu']['samples'][:98], errors=2,
                                 invalid_frames=[98,99], error_details=[
                                     'Unclosed source frame 98 before 99',
                                     'Unclosed source frame 99 before 100'])
            self.assertFalse(analyze(report, Path(directory), detailed_cpu=False)['success'])

    def test_battle_move_requires_measured_pp_change(self):
        frames = [dict(id=i, measured=True, interval_ms=10, main_cpu_ms=3,
                       pp=30, battle=True, dialogue=True) for i in range(40)]
        report = dict(schema=1, success=True, failure=None, frames=frames,
                      scenario=dict(steps=[dict(until=dict(pp_consumed=True))]),
                      gpu=dict(samples=[], supported=False, reason='disabled', errors=0, skipped=0),
                      render_diagnostics=[], checkpoints=[])
        frames[0].update(battle=False, pp=100)
        with tempfile.TemporaryDirectory() as directory:
            self.assertFalse(analyze(report, Path(directory), detailed_cpu=False)['success'])
            frames[-1]['pp'] = 29
            self.assertTrue(analyze(report, Path(directory), detailed_cpu=False)['success'])


if __name__ == '__main__':
    unittest.main()
