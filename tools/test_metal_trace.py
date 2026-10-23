from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET

from metal_trace import decode_gpu, union_ns


def document(path, fields, rows):
    root = ET.Element('trace-query-result')
    node = ET.SubElement(root, 'node')
    schema = ET.SubElement(node, 'schema')
    for field in fields:
        ET.SubElement(ET.SubElement(schema, 'col'), 'mnemonic').text = field
    known = set()
    for values in rows:
        row = ET.SubElement(node, 'row')
        for field, value in zip(fields, values):
            if field == 'process':
                if value in known:
                    ET.SubElement(row, 'process', ref=f'process-{value}')
                else:
                    process = ET.SubElement(row, 'process', id=f'process-{value}')
                    ET.SubElement(process, 'pid').text = str(value)
                    known.add(value)
            elif field == 'event-label' and isinstance(value, tuple):
                event = ET.SubElement(row, 'narrative')
                ET.SubElement(event, 'metal-object-label').text = value[0]
            elif field == 'event-label':
                ET.SubElement(row, 'formatted-label', fmt=value)
            else:
                ET.SubElement(row, 'value').text = str(value)
    ET.ElementTree(root).write(path)


class MetalTraceTests(unittest.TestCase):
    def test_submission_identity_survives_gpu_marker_reordering_and_other_processes(self):
        commits = [
            (10, 1, ('geothite_audit_frame_42_begin',), 100),
            (20, 2, ('main opaque',), 100),
            (30, 3, ('geothite_audit_frame_42_end',), 100),
            (40, 4, ('geothite_audit_frame_43_begin',), 100),
            (50, 5, ('shadows',), 100),
            (60, 6, ('geothite_audit_frame_43_end',), 100)]
        activity = [
            # End marker executes before graphics finish. A timestamp window
            # would underreport this frame; command identity must retain it.
            (1000, 1000, 'Vertex', 2, 20, 0, 'Active', 'main opaque', 100),
            (1500, 3500, 'Fragment', 2, 20, 0, 'Active', 'main opaque', 100),
            (1200, 10, 'Compute', 3, 30, 0, 'Active', 'marker', 100),
            (900, 10, 'Compute', 1, 10, 0, 'Active', 'marker', 100),
            # Same clock range and numeric command ID in another PID is noise.
            (1000, 1000000000, 'Compute', 2, 20, 0, 'Active', 'unrelated', 999),
            (2000, 500, 'Fragment', 5, 50, 0, 'Active', 'shadows', 100)]
        with tempfile.TemporaryDirectory() as folder:
            submissions, gpu = Path(folder)/'submissions.xml', Path(folder)/'gpu.xml'
            document(submissions, ('frame-number', 'start', 'cmdbuffer-id', 'event-label', 'process'),
                     [(ordinal, 1000-ordinal, command, label, pid)
                      for ordinal, command, label, pid in reversed(commits)])
            document(gpu, ('start', 'duration', 'channel-name', 'cmdbuffer-id', 'encoder-id',
                           'event-depth', 'state', 'event-label', 'process'), activity)
            result = decode_gpu(submissions, gpu, 100)
        self.assertEqual(result['errors'], 0)
        self.assertEqual(len(result['passes']), 3)
        samples = {sample['frame']: sample for sample in result['samples']}
        self.assertAlmostEqual(samples[42]['elapsed_ms'], .004)
        self.assertAlmostEqual(samples[42]['active_ms'], .004)
        self.assertAlmostEqual(samples[43]['elapsed_ms'], .0005)
        self.assertIsNone(samples[42]['completion_latency_ms'])

    def test_unclosed_group_never_donates_work_to_the_following_frame(self):
        with tempfile.TemporaryDirectory() as folder:
            submissions, gpu = Path(folder)/'submissions.xml', Path(folder)/'gpu.xml'
            document(submissions, ('frame-number', 'cmdbuffer-id', 'event-label', 'process'), [
                (1, 1, ('geothite_audit_frame_10_begin',), 100),
                (2, 2, ('unfinished',), 100),
                (3, 3, ('geothite_audit_frame_11_begin',), 100),
                (4, 4, ('complete',), 100),
                (5, 5, ('geothite_audit_frame_11_end',), 100)])
            document(gpu, ('start', 'duration', 'channel-name', 'cmdbuffer-id', 'encoder-id',
                           'event-depth', 'state', 'event-label', 'process'), [
                (100, 9999999, 'Fragment', 2, 20, 0, 'Active', 'unfinished', 100),
                (200, 100, 'Fragment', 4, 40, 0, 'Active', 'complete', 100)])
            result = decode_gpu(submissions, gpu, 100)
        self.assertEqual(result['invalid_frames'], [10])
        self.assertEqual([sample['frame'] for sample in result['samples']], [11])
        self.assertAlmostEqual(result['samples'][0]['elapsed_ms'], .0001)

    def test_overlap_union_is_not_sum_of_parallel_gpu_stages(self):
        self.assertEqual(union_ns([(10, 40), (20, 50), (60, 70)]), 50)
        self.assertEqual(union_ns([]), 0)

    def test_absent_source_boundaries_are_a_capture_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder)/'empty.xml'
            document(path, ('frame-number', 'cmdbuffer-id', 'event-label', 'process'), [])
            with self.assertRaisesRegex(ValueError, 'no complete source-frame'):
                decode_gpu(path, path, 100)


if __name__ == '__main__':
    unittest.main()
