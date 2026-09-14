import unittest
from operation_waterfall import decode, summarize


class TraceTests(unittest.TestCase):
    def test_nested_spans_keep_thread_local_stacks(self):
        spans, names, incomplete = decode([
            dict(ph='M', pid=1, tid=2, name='thread_name', args={'name': 'render'}),
            dict(ph='B', pid=1, tid=1, name='update', ts=10),
            dict(ph='B', pid=1, tid=2, name='draw', ts=11),
            dict(ph='B', pid=1, tid=1, name='system', ts=12),
            dict(ph='E', pid=1, tid=2, ts=15),
            dict(ph='E', pid=1, tid=1, ts=16),
            dict(ph='E', pid=1, tid=1, ts=20),
        ])
        self.assertEqual([(s['name'], s['duration'], s['depth']) for s in spans],
                         [('draw', 4, 0), ('system', 4, 1), ('update', 10, 0)])
        self.assertEqual(names[(1, 2)], 'render')
        self.assertEqual(incomplete, 0)

    def test_unfinished_work_is_reported_not_invented(self):
        spans, _, incomplete = decode([
            dict(ph='E', ts=0), dict(ph='B', name='unfinished', ts=1),
            dict(ph='X', name='complete', ts=2, dur=5),
        ])
        self.assertEqual(len(spans), 1)
        self.assertEqual(incomplete, 2)
        self.assertEqual(summarize(spans)[0]['total_ms'], .005)

    def test_statistics_include_all_calls(self):
        spans = [dict(name='draw', duration=n) for n in range(1, 101)]
        row = summarize(spans)[0]
        self.assertEqual(row['count'], 100)
        self.assertEqual(row['total_ms'], 5.05)
        self.assertEqual(row['median_us'], 50.5)
        self.assertEqual(row['p95_us'], 95)


if __name__ == '__main__':
    unittest.main()
