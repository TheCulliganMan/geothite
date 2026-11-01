import unittest
from performance_audit_residency import summarize


def frame(index, active=True, measured=True, interval=10.):
    return dict(id=index, start_ms=index * 10., interval_ms=interval,
                measured=measured, motion=dict(active=active,
                    camera_eye=[0., 1., 2.], camera_rotation=[0., 0., 0., 1.]))


class ResidencyTests(unittest.TestCase):
    def test_active_flag_does_not_hide_a_poisoned_camera_transform(self):
        invalid = frame(1)
        invalid['motion']['camera_eye'][1] = None
        result = summarize(dict(frames=[frame(0), invalid, frame(2)]))
        self.assertEqual(result['invalid_camera_frames'], 1)
        self.assertEqual(result['unavailable_ms'], 10.)
        self.assertEqual(result['unavailable_frames'], 1)
    def test_reports_long_gap_away_from_a_seam_without_hiding_fast_frames(self):
        result = summarize(dict(frames=[frame(0, measured=False), frame(1),
                                       frame(2, False), frame(3, False), frame(4)]))
        self.assertEqual(result['unavailable_frames'], 2)
        self.assertEqual(result['unavailable_ms'], 20.)
        self.assertEqual(result['observed_ms'], 40.)
        self.assertEqual(len(result['episodes']), 1)
        self.assertEqual(result['episodes'][0]['first_frame'], 2)

    def test_warmup_and_non_measured_gaps_are_excluded(self):
        result = summarize(dict(frames=[frame(0, False, False), frame(1, False),
                                       frame(2, False, False), frame(20, False)]))
        self.assertEqual(result['unavailable_ms'], 20.)
        self.assertEqual(len(result['episodes']), 2)

    def test_missing_last_interval_stays_censored(self):
        result = summarize(dict(frames=[frame(0), frame(1, False, interval=None)]))
        self.assertEqual(result['unavailable_ms'], 0.)
        self.assertEqual(result['unknown_intervals'], 1)
        self.assertTrue(result['episodes'][0]['interval_unknown'])

    def test_missing_motion_is_unknown_and_partial_observations_are_rejected(self):
        self.assertFalse(summarize(dict(frames=[]))['available'])
        missing = dict(frame(1), motion=None)
        self.assertFalse(summarize(dict(frames=[missing]))['available'])
        with self.assertRaisesRegex(ValueError, 'Incomplete'):
            summarize(dict(frames=[frame(0), missing]))

    def test_invalid_timestamps_are_rejected(self):
        for bad in [dict(frame(1), start_ms=-1.), dict(frame(1), start_ms=float('nan')),
                    frame(1, interval=float('inf'))]:
            with self.assertRaises(ValueError):
                summarize(dict(frames=[frame(0), bad]))


if __name__ == '__main__':
    unittest.main()
