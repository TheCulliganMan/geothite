import json
from pathlib import Path
import tempfile
import unittest

from performance_audit import ANALYSIS_REVISION
from performance_audit_reanalyze import refresh


class ReanalysisTests(unittest.TestCase):
    def capture(self, output, issues):
        directory = output/'00'/'run-1'
        directory.mkdir(parents=True)
        report = dict(
            schema=1, success=True, failure=None, checkpoints=[], render_diagnostics=[],
            frames=[dict(id=i, measured=True, interval_ms=10, main_cpu_ms=3) for i in range(100)],
            gpu=dict(supported=True, backend='metal_system_trace', reason='native',
                     errors=1, skipped=0, invalid_frames=[99],
                     error_details=['Unclosed source frame 99 before 100'],
                     samples=[dict(frame=i, elapsed_ms=1, start_ns=i*1000000+1,
                                   end_ns=i*1000000+1000001) for i in range(99)]))
        (directory/'audit.json').write_text(json.dumps(report))
        (directory/'metadata.json').write_text(json.dumps(dict(
            gpu_backend='metal_system_trace', operation_trace=True, trace_filter='warn')))
        summary = dict(
            schema=1, provenance={}, thresholds=dict(percent=10, absolute_ms=.2),
            scenarios=[dict(name='sample', compatibility={}, runs=[dict(
                success=False, issues=issues, directory='00/run-1',
                load_before=[1, 2, 3], power_source='AC', pair=2, pair_position=1,
                host_quiet=dict(policy=dict(stable_seconds=10),contenders_during_capture=[]))])])
        (output/'summary.json').write_text(json.dumps(summary))
        return directory, report

    def test_recomputes_dropped_frame_quality_and_preserves_original_summary(self):
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)
            self.capture(output, ['Incomplete GPU capture: errors or less than 95% frame coverage'])
            original = (output/'summary.json').read_bytes()
            result = refresh(output)
            run = result['scenarios'][0]['runs'][0]
            self.assertTrue(run['success'])
            self.assertEqual((run['gpu_coverage'], run['gpu_dropped_frames']), (.99, 1))
            self.assertEqual(run['power_source'], 'AC')
            self.assertEqual((run['pair'],run['pair_position']),(2,1))
            self.assertEqual(run['host_quiet']['policy'],dict(stable_seconds=10))
            self.assertEqual(result['scenarios'][0]['compatibility']['analysis_revision'], ANALYSIS_REVISION)
            self.assertIn('metal_trace.py', result['provenance']['analysis_sources_sha256'])
            self.assertEqual((output/'summary-before-reanalysis.json').read_bytes(), original)
            refresh(output)
            self.assertEqual((output/'summary-before-reanalysis.json').read_bytes(), original)
            self.assertTrue((output/'index.html').exists())

    def test_quality_recomputation_retains_process_and_power_failures(self):
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)
            failures = ['Client exited 7', 'Power source changed during capture']
            self.capture(output, ['Incomplete GPU capture: errors or less than 95% frame coverage']+failures)
            run = refresh(output)['scenarios'][0]['runs'][0]
            self.assertFalse(run['success'])
            self.assertEqual(run['issues'], failures)

    def test_absent_native_import_cannot_be_promoted_to_a_cpu_only_success(self):
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)
            directory, report = self.capture(output, ['Native GPU import failed'])
            report['gpu'].update(supported=False, backend='timestamp_query', errors=0)
            (directory/'audit.json').write_text(json.dumps(report))
            run = refresh(output)['scenarios'][0]['runs'][0]
            self.assertFalse(run['success'])
            self.assertEqual(run['issues'], ['Native GPU import failed'])


if __name__ == '__main__':
    unittest.main()
