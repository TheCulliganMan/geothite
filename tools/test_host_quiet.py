import unittest
from unittest.mock import patch
from host_quiet import CaptureHost, contenders, gpu_utilization


class HostPreconditionTests(unittest.TestCase):
    def test_missing_gpu_data_is_unavailable_and_zero_is_a_real_sample(self):
        self.assertIsNone(gpu_utilization([{'PerformanceStatistics':{}}]))
        self.assertEqual(gpu_utilization([{'PerformanceStatistics':{'Device Utilization %':0}}]),0)
        self.assertEqual(gpu_utilization([{'PerformanceStatistics':{'Device Utilization %':4}},
                                         {'PerformanceStatistics':{'Device Utilization %':60}}]),60)

    def test_known_jobs_are_detected_but_the_current_game_is_excluded(self):
        rows=[dict(pid=1,name='chrome-headless-shell'),dict(pid=2,name='performance_audit'),
              dict(pid=3,name='rustc'),dict(pid=4,name='Finder')]
        self.assertEqual([row['pid'] for row in contenders(rows,{2})],[1,3])

    def test_versioned_audit_binaries_are_contenders_except_owned_descendants(self):
        rows = [dict(pid=10, parent_pid=1, name='node'),
                dict(pid=11, parent_pid=10, name='candidate-progressive-v18'),
                dict(pid=12, parent_pid=1, name='candidate-progressive-v17'),
                dict(pid=13, parent_pid=1, name='baseline-progressive-v9'),
                dict(pid=14, parent_pid=1, name='candidate-progressive-v16-cpu-symbols')]
        self.assertEqual([row['pid'] for row in contenders(rows, {10})], [12, 13, 14])

    def test_disabled_guard_does_not_sample_or_start_a_watcher(self):
        with CaptureHost() as host:
            host.wait()
            self.assertIsNone(host.thread)
        self.assertEqual(host.report(),dict(policy=None,before=[],contenders_during_capture=[]))

    def test_browser_descendants_are_owned_but_another_browser_is_a_contender(self):
        rows=[dict(pid=12,parent_pid=11,name='chrome-headless-shell'),
              dict(pid=20,parent_pid=1,name='chrome-headless-shell'),
              dict(pid=11,parent_pid=10,name='chrome-headless-shell'),
              dict(pid=10,parent_pid=1,name='node')]
        self.assertEqual([row['pid'] for row in contenders(rows,{10})],[20])

    def test_new_compiler_is_recorded_and_own_client_is_excluded(self):
        host=CaptureHost(enabled=True)
        host.own_pids.add(8)
        rows=[dict(pid=8,name='performance_audit',cpu_percent=80),
              dict(pid=9,name='rustc',cpu_percent=100)]
        with patch.object(host.done,'wait',side_effect=[False,False,True]), \
             patch('host_quiet.processes',return_value=rows):
            host._watch()
        self.assertEqual([event['pid'] for event in host.events],[9])

    def test_monitoring_failure_is_recorded_instead_of_certifying_quiet(self):
        host=CaptureHost(enabled=True)
        with patch.object(host.done,'wait',return_value=False), \
             patch('host_quiet.processes',side_effect=OSError('ps failed')):
            host._watch()
        self.assertEqual(host.events,[dict(error='ps failed')])


if __name__=='__main__':
    unittest.main()
