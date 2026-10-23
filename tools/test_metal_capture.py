import ctypes
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock
import subprocess

from metal_capture import TracingStarted, record, wait_for_saved_trace


class NativeProcessTests(unittest.TestCase):
    def test_save_wait_keeps_observing_short_lived_conversion_files(self):
        recorder=Mock(pid=41,args=['native-recorder'])
        recorder.poll.side_effect=[None,None,0]
        recorder.wait.side_effect=[subprocess.TimeoutExpired([],1),None]
        scratch=Mock()
        wait_for_saved_trace(recorder,scratch,2)
        self.assertEqual(scratch.observe.call_count,2)
        scratch.observe.assert_called_with(41)

    def test_client_failure_before_readiness_is_not_a_successful_gpu_capture(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            with (root/'recorder.log').open('w') as log:
                with self.assertRaisesRegex(ValueError,'Client exited 7 before'):
                    record([sys.executable,'-c','raise SystemExit(7)'],None,root,
                           root/'native.trace',root/'client.log',log,root/'recorder.log',2)

    def test_pid_mismatch_stops_the_waiting_client_without_attaching_a_profiler(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            program=("import json,os,time;from pathlib import Path;"
                     f"p=Path({str(root)!r});"
                     "(p/'metadata.json').write_text(json.dumps({'process_id':os.getpid()+1}));"
                     "(p/'native-ready').touch();time.sleep(60)")
            with (root/'recorder.log').open('w') as log:
                with self.assertRaisesRegex(ValueError,'PID differs'):
                    record([sys.executable,'-c',program],None,root,
                           root/'native.trace',root/'client.log',log,root/'recorder.log',2)


@unittest.skipUnless(sys.platform=='darwin','Darwin notification API')
class NativeNotificationTests(unittest.TestCase):
    def test_registration_receives_only_its_actual_posted_start_notification(self):
        with TracingStarted() as first, TracingStarted() as second:
            self.assertFalse(first.wait(0))
            self.assertFalse(second.wait(0))
            library=first.library
            library.notify_post.argtypes=[ctypes.c_char_p]
            library.notify_post.restype=ctypes.c_uint32
            self.assertEqual(library.notify_post(first.name.encode()),0)
            self.assertTrue(first.wait(1))
            self.assertFalse(second.wait(0))
            self.assertFalse(first.wait(0))


if __name__=='__main__':
    unittest.main()
