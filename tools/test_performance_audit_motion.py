import unittest

from performance_audit_motion import crossings, motion_series, stationary_camera_drift


def frame(index, name, x, shift=None, active=True):
    return dict(id=index, start_ms=index * 16., simulation_tick=index,
        measured=True, walk_remaining=0, terrain_pending=False,
        motion=dict(map=name, active=active, player_foot=[x, 4.],
            view_center=[x + 8., 8.], tile_size=[8., 8.],
            camera_eye=[x + 8., 100., -8.],
            camera_rotation=[0., 0., 0., 1.],
            connected_shift=shift, connected_terrain=False))


class MotionTests(unittest.TestCase):
    def test_height_corrections_are_visible_even_when_ground_plane_does_not_move(self):
        first = frame(0, 'source', 2.)
        second = frame(1, 'source', 2.)
        second['motion']['camera_eye'][1] += 8.
        result = stationary_camera_drift(motion_series(dict(success=True, frames=[first, second])))
        self.assertEqual(result['observed_pairs'], 1)
        self.assertEqual(result['max_vertical_cells'], .5)
        self.assertEqual(result['max_horizontal_cells'], 0.)
        self.assertEqual(len(result['events']), 1)

    def test_camera_drift_requires_a_stationary_valid_observation(self):
        series = motion_series(dict(success=True, frames=[frame(0, 'source', 2.), frame(1, 'source', 4.)]))
        result = stationary_camera_drift(series)
        self.assertEqual(result['observed_pairs'], 0)
        self.assertIsNone(result['max_vertical_cells'])
    def test_a_non_finite_camera_height_cannot_pass_from_finite_ground_coordinates(self):
        invalid = frame(1, 'target', 644., [80, 0])
        invalid['motion']['camera_eye'][1] = None
        event, = crossings(motion_series(dict(success=True, frames=[frame(0, 'source', 2.), invalid])))
        self.assertFalse(event['valid'])
        self.assertIsNone(event['camera_delta_cells'])
    def test_reciprocal_translation_preserves_actual_two_pixel_step(self):
        series = motion_series(dict(success=True, frames=[
            frame(0, 'source', 2.), frame(1, 'target', 644., [80, 0])]))
        event, = crossings(series)
        self.assertTrue(event['valid'])
        self.assertEqual(event['player_delta_cells'], [.125, 0.])
        self.assertEqual(event['camera_delta_cells'], [.125, 0.])

    def test_unproven_warp_starts_new_segment_and_never_claims_continuity(self):
        series = motion_series(dict(success=True, frames=[
            frame(0, 'source', 2.), frame(1, 'room', 42.)]))
        self.assertEqual(series[1]['segment'], 1)
        self.assertEqual(crossings(series), [])

    def test_gameplay_cell_units_do_not_depend_on_render_scale(self):
        target = frame(1, 'target', 2576., [80, 0])
        target['motion'].update(tile_size=[32., 32.], player_foot=[2576., 16.],
                                camera_eye=[2608., 400., -32.])
        event, = crossings(motion_series(dict(success=True, frames=[frame(0, 'source', 2.), target])))
        self.assertEqual(event['player_delta_cells'], [.125, 0.])
        self.assertEqual(event['camera_delta_cells'], [.125, 0.])

    def test_inactive_scene_is_visible_in_results_and_not_accepted(self):
        series = motion_series(dict(success=True, frames=[
            frame(0, 'source', 2.), frame(1, 'target', 644., [80, 0], active=False)]))
        event, = crossings(series)
        self.assertFalse(event['valid'])
        self.assertIsNone(event['player_delta_cells'])

    def test_legacy_and_failed_captures_require_new_measurements(self):
        for report in [dict(success=False), dict(success=True, frames=[{}])]:
            with self.assertRaises(ValueError):
                motion_series(report)

    def test_boundary_cost_and_motion_gap_do_not_use_whole_sequence_latency(self):
        arrival = frame(1, 'target', 644., [80, 0])
        arrival.update(interval_ms=7., main_cpu_ms=2.)
        report = dict(success=True, checkpoints=[dict(latency_ms=1200.)], frames=[
            frame(0, 'source', 2.), arrival, frame(2, 'target', 644.),
            frame(3, 'target', 644.), frame(4, 'target', 646.)])
        event, = crossings(motion_series(report))
        self.assertEqual(event['frame_interval_ms'], 7.)
        self.assertEqual(event['main_app_wall_ms'], 2.)
        self.assertEqual(event['next_motion_ms'], 48.)

    def test_missing_future_motion_is_unknown_instead_of_a_zero_cost_handoff(self):
        report = dict(success=True, frames=[frame(0, 'source', 2.),
            frame(1, 'target', 644., [80, 0]), frame(2, 'target', 644.)])
        event, = crossings(motion_series(report))
        self.assertIsNone(event['next_motion_ms'])


if __name__ == '__main__':
    unittest.main()
