from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from instruments_scratch import RecorderScratch


class NativeScratchTests(unittest.TestCase):
    def observe(self, scratch, text):
        with patch('instruments_scratch.subprocess.run',return_value=subprocess.CompletedProcess([],0,text,'')):
            scratch.observe(41)

    def test_only_exact_pid_regular_kernel_files_in_allowed_root_are_retired(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);allowed=root/'allowed';allowed.mkdir()
            own=allowed/'instrumentsABC123.ktrace';own.write_bytes(b'own')
            other=allowed/'instrumentsDEF456.ktrace';other.write_bytes(b'other pid')
            outside=root/'instrumentsGHI789.ktrace';outside.write_bytes(b'other root')
            unrelated=allowed/'notes.txt';unrelated.write_text('keep')
            link=allowed/'instrumentsJKL123.ktrace';link.symlink_to(outside)
            scratch=RecorderScratch([allowed])
            self.observe(scratch,f'p41\nn{own}\nn{outside}\nn{unrelated}\nn{link}\np99\nn{other}\n')
            report=scratch.retire(True)
            self.assertFalse(own.exists())
            self.assertTrue(all(path.exists() for path in (other,outside,unrelated,link)))
            self.assertEqual(len(report['files']),1)
            self.assertEqual(report['files'][0]['bytes'],3)
            self.assertEqual(report['errors'],[])

    def test_open_recorder_must_finish_before_retiring_its_file(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);own=root/'instrumentsABC123.ktrace';own.write_bytes(b'own')
            scratch=RecorderScratch([root]);self.observe(scratch,f'p41\nn{own}\n')
            self.assertEqual(scratch.retire(False)['files'][0]['disposition'],'recorder_still_running')
            self.assertTrue(own.exists())
            self.assertEqual(scratch.retire(True)['files'][0]['disposition'],'removed')

    def test_changed_file_identity_is_preserved(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);own=root/'instrumentsABC123.ktrace';own.write_bytes(b'own')
            scratch=RecorderScratch([root]);self.observe(scratch,f'p41\nn{own}\n')
            replacement=root/'replacement';replacement.write_bytes(b'replacement')
            replacement.replace(own)
            self.assertEqual(scratch.retire(True)['files'][0]['disposition'],'identity_changed_retained')
            self.assertEqual(own.read_bytes(),b'replacement')

    def test_already_removed_intermediate_does_not_fail_a_capture(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);own=root/'instrumentsABC123.ktrace';own.write_bytes(b'own')
            scratch=RecorderScratch([root]);self.observe(scratch,f'p41\nn{own}\n');own.unlink()
            report=scratch.retire(True)
            self.assertEqual(report['files'][0]['disposition'],'already_removed')
            self.assertEqual(report['errors'],[])

    def test_failed_process_inspection_is_reported_without_deleting_anything(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);own=root/'instrumentsABC123.ktrace';own.write_bytes(b'own')
            scratch=RecorderScratch([root])
            with patch('instruments_scratch.subprocess.run',side_effect=OSError('unavailable')):
                scratch.observe(41)
            self.assertTrue(scratch.retire(True)['errors'])
            self.assertTrue(own.exists())


if __name__=='__main__':unittest.main()
