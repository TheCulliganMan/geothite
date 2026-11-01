import unittest

from performance_audit_boundary import match_events


def event(source='route', destination='town', interval=12., future=None):
    return dict(source=source, destination=destination, valid=True,
                frame_interval_ms=interval, next_motion_ms=future)


class BoundaryTests(unittest.TestCase):
    def test_censored_future_motion_remains_unknown(self):
        pair, = match_events([event()], [event(interval=8.)])
        self.assertIsNone(pair[0]['next_motion_ms'])
        self.assertEqual(pair[1]['frame_interval_ms'], 8.)

    def test_different_seams_and_missing_crossings_are_rejected(self):
        for before, after in [([], []), ([event()], []),
                ([event()], [event(destination='room')])]:
            with self.assertRaises(ValueError):
                match_events(before, after)

    def test_failed_presentation_is_retained_without_becoming_a_fast_crossing(self):
        pair, = match_events([event()], [dict(event(future=3.), valid=False)])
        self.assertFalse(pair[1]['valid'])
        self.assertEqual(pair[1]['observed_frame_interval_ms'], 12.)
        self.assertIsNone(pair[1]['frame_interval_ms'])
        self.assertIsNone(pair[1]['next_motion_ms'])

    def test_missing_or_invalid_interval_is_a_recorder_failure(self):
        for bad in [event(interval=None), event(interval=0.),
                event(interval=float('nan')), event(future=-1.)]:
            with self.assertRaises(ValueError):
                match_events([event()], [bad])


if __name__ == '__main__':
    unittest.main()
