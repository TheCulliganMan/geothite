import copy
import json
from pathlib import Path
import tempfile
import unittest

from performance_audit import analyze, compare, cpu_census, exclusive_census, stats


class AuditTests(unittest.TestCase):
    def test_quantiles_and_empty_gpu_are_not_invented(self):
        self.assertIsNone(stats([]))
        sample = stats(range(1, 101))
        self.assertEqual((sample['p50'], sample['p95'], sample['p99']), (50.5, 95, 99))

    def test_exclusive_time_clips_nested_work_and_keeps_threads_independent(self):
        spans = [dict(name='frame', start=0, duration=100, depth=0, thread=(0, 0)),
                 dict(name='system', start=10, duration=80, depth=1, thread=(0, 0)),
                 dict(name='leaf', start=20, duration=30, depth=2, thread=(0, 0)),
                 dict(name='render', start=0, duration=100, depth=0, thread=(0, 1))]
        rows = {row['name']: row for row in exclusive_census(spans, 15, 70)}
        self.assertAlmostEqual(rows['frame']['self_ms'], 0)
        self.assertAlmostEqual(rows['system']['self_ms'], .025)
        self.assertAlmostEqual(rows['leaf']['self_ms'], .030)
        self.assertAlmostEqual(rows['render']['self_ms'], .055)

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

    def test_missing_timestamp_coverage_fails_instead_of_reporting_fast_gpu(self):
        frames = [dict(id=i, measured=True, interval_ms=10, main_cpu_ms=3) for i in range(40)]
        report = dict(success=True, failure=None, frames=frames,
                      gpu=dict(samples=[], supported=True, reason='', errors=0, skipped=40),
                      render_diagnostics=[], checkpoints=[])
        with tempfile.TemporaryDirectory() as directory:
            result = analyze(report, Path(directory))
        self.assertFalse(result['success'])
        self.assertIsNone(result['gpu_ms'])
        self.assertEqual(result['gpu_coverage'], 0)


if __name__ == '__main__':
    unittest.main()
