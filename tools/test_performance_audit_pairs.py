import contextlib
import io
import json
from argparse import Namespace
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from performance_audit import digest, stats
from performance_audit_pairs import collect


def arguments(root, before, after, pack):
    suite=root/'suite.json'
    suite.write_text(json.dumps([dict(name='sample')]))
    return Namespace(before=before, after=after, pack=pack, suite=suite, output=root/'result',
                     scenario=None, pairs=2, cpu_only=True, gpu_backend='auto', detailed_cpu=False,
                     timeout=5, threshold_percent=10, threshold_ms=.2)


class PairedAuditTests(unittest.TestCase):
    def test_one_candidate_spike_stops_collection_despite_good_percentiles(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            before, after = root/'before', root/'after'
            before.write_text('before'); after.write_text('after')
            args = arguments(root, before, after, root/'pack')
            args.max_frame_ms = 45.
            order = []

            def capture(request):
                order.append(request.binary.name)
                request.output.mkdir(parents=True)
                peak = 500. if request.binary == before else 45.001
                run = dict(success=True, issues=[], directory='00/run-1', checkpoints=[],
                           interval_ms=stats([16.] * 1000 + [peak]),
                           main_cpu_ms=stats([4.]), gpu_ms=None)
                child = dict(schema=1, provenance=dict(binary_sha256=digest(request.binary),
                             created_at_utc='test', hardware={}), thresholds=dict(percent=10, absolute_ms=.2),
                             scenarios=[dict(name='sample', compatibility=dict(pack='same'), runs=[run])])
                (request.output/'summary.json').write_text(json.dumps(child))
                return 0

            with patch('performance_audit_pairs.run_suite', side_effect=capture), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(collect(args), 3)
            self.assertEqual(order, ['before', 'after'])
            failure = json.loads((args.output/'frame-budget-failure.json').read_text())
            self.assertEqual(failure['outcome']['maximum_ms'], 45.001)
            summary = json.loads((args.output/'after/summary.json').read_text())
            self.assertTrue(summary['scenarios'][0]['runs'][0]['success'])
            self.assertFalse(summary['scenarios'][0]['runs'][0]['frame_budget']['passed'])

    def test_renderer_failure_stops_future_pairs_without_rejecting_raw_gameplay(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            before, after = root/'before', root/'after'
            before.write_text('before'); after.write_text('after')
            args = arguments(root, before, after, root/'pack')
            args.suite.write_text(json.dumps([dict(name='walk-continuous')]))
            args.require_after_continuity = True
            args.pairs = 3
            order = []

            def capture(request):
                order.append(request.binary.name)
                request.output.mkdir(parents=True)
                run = dict(success=True, issues=[], directory='00/run-1', checkpoints=[],
                           interval_ms=stats([10]*40), main_cpu_ms=stats([4]*40), gpu_ms=None,
                           world_residency=dict(available=True, unavailable_frames=4,
                                                unavailable_ms=2500., unknown_camera_frames=0, unknown_intervals=0),
                           stationary_camera=dict(observed_pairs=2, max_vertical_cells=0., max_horizontal_cells=0.))
                child = dict(schema=1, provenance=dict(binary_sha256=digest(request.binary),
                             created_at_utc='test', hardware={}), thresholds=dict(percent=10, absolute_ms=.2),
                             scenarios=[dict(name='walk-continuous', compatibility=dict(pack='same'), runs=[run])])
                (request.output/'summary.json').write_text(json.dumps(child))
                return 0

            with patch('performance_audit_pairs.run_suite', side_effect=capture), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(collect(args), 3)
            self.assertEqual(order, ['before', 'after'])
            summary = json.loads((args.output/'after/summary.json').read_text())
            run = summary['scenarios'][0]['runs'][0]
            self.assertTrue(run['success'])
            self.assertFalse(run['presentation_continuity']['passed'])
            self.assertNotIn('paired_collection_finished_utc', summary['provenance'])
            self.assertTrue((args.output/'continuity-failure.json').exists())

    def test_alternates_order_preserves_raw_links_and_compares_aggregated_runs(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            before,after=root/'before',root/'after'
            before.write_text('before executable');after.write_text('after executable')
            args=arguments(root,before,after,root/'pack')
            order=[]

            def capture(request):
                label=request.binary.name
                order.append(label)
                request.output.mkdir(parents=True)
                raw=request.output/'00/run-1'
                raw.mkdir(parents=True)
                (raw/'audit.json').write_text('{}')
                run=dict(success=True, issues=[], directory='00/run-1', checkpoints=[],
                         interval_ms=stats([10 if label=='before' else 12]*40),
                         main_cpu_ms=stats([4]*40), gpu_ms=None)
                child=dict(schema=1, provenance=dict(binary_sha256=digest(request.binary),
                           created_at_utc='test', hardware={}),
                           thresholds=dict(percent=10,absolute_ms=.2),
                           scenarios=[dict(name='sample', compatibility=dict(pack='same'), runs=[run])])
                (request.output/'summary.json').write_text(json.dumps(child))
                return 0

            with patch('performance_audit_pairs.run_suite',side_effect=capture), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(collect(args),2)
            self.assertEqual(order,['before','after','after','before'])
            summary=json.loads((args.output/'after/summary.json').read_text())
            entry=summary['scenarios'][0]
            self.assertEqual(len(entry['runs']),2)
            self.assertEqual(entry['comparison']['regressions'][0]['metric'],'interval_ms')
            for run in entry['runs']:
                self.assertTrue((args.output/'after'/run['directory']/'audit.json').exists())
            self.assertEqual([r['pair_position'] for r in entry['runs']],[2,1])

    def test_real_failed_children_remain_failed_in_both_groups(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            program=root/'exit.py'
            program.write_text('raise SystemExit(7)\n')
            args=arguments(root,Path(sys.executable),Path(sys.executable),program)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(collect(args),1)
            for label in ('before','after'):
                summary=json.loads((args.output/label/'summary.json').read_text())
                self.assertTrue(all(not run['success'] for run in summary['scenarios'][0]['runs']))
            with self.assertRaisesRegex(ValueError,'must be empty'):
                collect(args)


if __name__=='__main__':
    unittest.main()
