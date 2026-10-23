import json
from pathlib import Path
import tempfile
import unittest
from trace_stream import events, spans

class StreamingTests(unittest.TestCase):
    def test_tiny_chunks_and_wrappers(self):
        sample=[dict(ph='X',name='a'*1000,ts=0,dur=2)]
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'trace.json'
            for value in (sample,dict(traceEvents=sample)):
                path.write_text(json.dumps(value))
                self.assertEqual(list(events(path,7)),sample)
            path.write_text('[{"ph":"B"}')
            with self.assertRaisesRegex(ValueError,'Truncated'):
                list(events(path,7))

    def test_clipped_exclusive_wall_time(self):
        sample=[dict(ph='B',name='outer',ts=0),dict(ph='B',name='inner',ts=10),
                dict(ph='E',ts=60),dict(ph='E',ts=100)]
        counts={}
        rows={s['name']:s for s in spans(iter(sample),{},counts,(20,80))}
        self.assertEqual(rows['inner']['duration'],40)
        self.assertEqual(rows['outer']['self_duration'],20)
        self.assertEqual(counts['incomplete'],0)

if __name__=='__main__': unittest.main()
