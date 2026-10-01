#!/usr/bin/env python3
"""Pure, deterministic human walk/run curves for the canonical GLB exporter.

The one-second clips describe one distance-driven cycle, not a playback speed.
Runtime samples normalized source-travel phase and applies planted-foot IK after
blending. Every exported leg is already fully articulated for ordinary glTF
viewers; no runtime solver or procedural animation is needed to view the loops.
Run this module directly for the stdlib-only curve and interpolation checks.
"""

import math


TAU = math.tau
CYCLE_DISTANCE = 1.12
RUN_CYCLE_DISTANCE = 2.80
STRIDE_REACH = CYCLE_DISTANCE * 0.25
RUN_STRIDE_REACH = 0.20
LEG_LENGTH = 0.27
ANKLE_HEIGHT = 0.12
SAMPLES_PER_CYCLE = 256


def cycle_distance(run):
    return CYCLE_DISTANCE + (RUN_CYCLE_DISTANCE - CYCLE_DISTANCE) * run


def stance_fraction(run):
    reach = STRIDE_REACH + (RUN_STRIDE_REACH - STRIDE_REACH) * run
    return 2.0 * reach / cycle_distance(run)


def foot_target(phase, run):
    """The final runtime contact solver uses this same target in model space."""
    t = (phase % TAU) / TAU
    stance = stance_fraction(run)
    reach = STRIDE_REACH + (RUN_STRIDE_REACH - STRIDE_REACH) * run
    if t < stance:
        return reach * (1.0 - 2.0 * t / stance), ANKLE_HEIGHT
    swing = (t - stance) / (1.0 - stance)
    tangent = -2.0 * (1.0 - stance) / stance
    return_z = (-1.0 + tangent * swing + (6.0 - 3.0 * tangent) * swing**2
                + (2.0 * tangent - 4.0) * swing**3)
    lift = math.sin(math.pi * swing)**2
    walk_lift = 0.060 * lift
    run_lift = 0.170 * 1.04 * lift / (0.04 + lift)
    return reach * return_z, ANKLE_HEIGHT + walk_lift + (run_lift - walk_lift) * run


def leg_angles(hip_height, ankle_z, ankle_y):
    down = hip_height - ankle_y
    distance = min(max(math.hypot(down, ankle_z), 0.001), LEG_LENGTH * 2.0 - 0.0001)
    knee = math.pi - math.acos(min(max(
        (2.0 * LEG_LENGTH**2 - distance**2) / (2.0 * LEG_LENGTH**2), -1.0), 1.0))
    hip = math.atan2(-ankle_z, down) - math.acos(min(max(distance / (2.0 * LEG_LENGTH), -1.0), 1.0))
    return hip, knee


def quaternion_xyz(x, y=0.0, z=0.0):
    """glTF XYZW, intrinsic XYZ, matching Bevy Quat::from_euler(XYZ)."""
    sx, sy, sz = (math.sin(v * 0.5) for v in (x, y, z))
    cx, cy, cz = (math.cos(v * 0.5) for v in (x, y, z))
    return [sx * cy * cz + cx * sy * sz,
            cx * sy * cz - sx * cy * sz,
            cx * cy * sz + sx * sy * cz,
            cx * cy * cz - sx * sy * sz]


def sample_pose(phase, run, bind_translations):
    """Return absolute local TRS channels at full locomotion weight."""
    phase %= TAU
    walk_drop = 0.065 + math.cos(phase)**2 * 0.030
    run_drop = 0.068 + math.cos(phase)**2 * 0.025
    hip_drop = walk_drop + (run_drop - walk_drop) * run
    pelvis = list(bind_translations['pelvis'])
    pelvis[1] -= hip_drop
    result = {('pelvis', 'translation'): pelvis,
              ('torso', 'rotation'): quaternion_xyz(0.065 + 0.12 * run,
                  math.cos(phase) * 0.045, math.sin(phase) * 0.025),
              ('head', 'rotation'): quaternion_xyz(-0.038 - 0.07 * run,
                  -math.cos(phase) * 0.030, -math.sin(phase) * 0.018)}
    for side, suffix in ((-1.0, 'l'), (1.0, 'r')):
        leg_phase = phase + (0.0 if side < 0.0 else math.pi)
        z, y = foot_target(leg_phase, run)
        hip_height = pelvis[1] + bind_translations[f'thigh_{suffix}'][1]
        hip, knee = leg_angles(hip_height, z, y)
        swing = math.cos(leg_phase) * (0.48 + 0.12 * run)
        result[(f'upper_arm_{suffix}', 'rotation')] = quaternion_xyz(swing - 0.055, 0.0, side * 0.035)
        result[(f'forearm_{suffix}', 'rotation')] = quaternion_xyz(-0.13 - 0.65 * run - abs(min(swing, 0.0)) * 0.28)
        result[(f'hand_{suffix}', 'rotation')] = quaternion_xyz(0.045 + swing * 0.08)
        result[(f'thigh_{suffix}', 'rotation')] = quaternion_xyz(hip)
        result[(f'shin_{suffix}', 'rotation')] = quaternion_xyz(knee)
        result[(f'shoe_{suffix}', 'rotation')] = quaternion_xyz(-hip - knee)
    return result


def locomotion_clips(bind_translations):
    """Return exporter-ready walk/run dictionaries with absolute local values.

    bind_translations maps canonical joint names to parent-relative XYZ lists.
    Tracks use {'joint', 'path', 'values'} with flattened XYZW/XYZ values. End
    samples are exact copies of the first, guaranteeing a bit-exact loop seam.
    Extra contact keys include both feet's toe-off, beyond uniform 1/256 keys.
    """
    clips = []
    for name, run in (('walk', 0.0), ('run', 1.0)):
        stance = stance_fraction(run)
        times = sorted({i / SAMPLES_PER_CYCLE for i in range(SAMPLES_PER_CYCLE + 1)}
                       | {stance, (stance + 0.5) % 1.0})
        poses = [sample_pose(time * TAU, run, bind_translations) for time in times[:-1]]
        poses.append(poses[0])
        tracks = [{'joint': joint, 'path': path,
                   'values': [value for pose in poses for value in pose[joint, path]]}
                  for joint, path in poses[0]]
        clips.append({'name': name, 'times': times, 'tracks': tracks,
                      'extras': {'purpose': 'production source-distance locomotion',
                                 'loopSuggested': True, 'cycleDistance': cycle_distance(run),
                                 'stanceFraction': stance, 'phaseSource': 'authoritative traveled distance',
                                 'completeLegAnimation': True}})
    return clips


def _self_test():
    import bisect
    import unittest

    # The exporter passes its actual binds; these are the fixed canonical leg pivots.
    binds = {'pelvis': [0.0, 0.66, 0.0], 'thigh_l': [-0.105, 0.0, 0.0],
             'thigh_r': [0.105, 0.0, 0.0]}

    class LocomotionTests(unittest.TestCase):
        def test_complete_channels_exact_loop_and_unit_quaternions(self):
            for clip in locomotion_clips(binds):
                self.assertEqual(clip['times'][0], 0.0)
                self.assertEqual(clip['times'][-1], 1.0)
                self.assertGreaterEqual(len(clip['times']), 257)
                self.assertEqual(len(clip['tracks']), 15)
                channels = {(t['joint'], t['path']) for t in clip['tracks']}
                for suffix in ('l', 'r'):
                    for segment in ('thigh', 'shin', 'shoe'):
                        self.assertIn((f'{segment}_{suffix}', 'rotation'), channels)
                for track in clip['tracks']:
                    width = 4 if track['path'] == 'rotation' else 3
                    values = track['values']
                    self.assertEqual(len(values), width * len(clip['times']))
                    self.assertEqual(values[:width], values[-width:])
                    for i in range(0, len(values), width):
                        self.assertTrue(all(math.isfinite(v) for v in values[i:i + width]))
                        if width == 4:
                            self.assertAlmostEqual(sum(v*v for v in values[i:i + width]), 1.0)

        def test_standalone_leg_channels_hit_ankles_and_keep_soles_level(self):
            for run in (0.0, 1.0):
                for i in range(1024):
                    phase = i / 1024.0 * TAU
                    pose = sample_pose(phase, run, binds)
                    for side, offset in (('l', 0.0), ('r', math.pi)):
                        angle = lambda name: 2.0 * math.atan2(pose[name, 'rotation'][0], pose[name, 'rotation'][3])
                        hip, knee, shoe = (angle(f'{part}_{side}') for part in ('thigh', 'shin', 'shoe'))
                        y = pose['pelvis', 'translation'][1] - LEG_LENGTH * (math.cos(hip) + math.cos(hip + knee))
                        z = -LEG_LENGTH * (math.sin(hip) + math.sin(hip + knee))
                        target_z, target_y = foot_target(phase + offset, run)
                        self.assertAlmostEqual(y, target_y, places=10)
                        self.assertAlmostEqual(z, target_z, places=10)
                        self.assertAlmostEqual(hip + knee + shoe, 0.0, places=10)

        def test_linear_keys_keep_standalone_feet_within_submillimeter_error(self):
            for clip, run in zip(locomotion_clips(binds), (0.0, 1.0)):
                tracks = {(t['joint'], t['path']): t['values'] for t in clip['tracks']}
                maximum = 0.0
                for i in range(2048):
                    time = (i + 0.5) / 2048.0
                    index = bisect.bisect_right(clip['times'], time) - 1
                    a, b = clip['times'][index:index + 2]
                    fraction = (time - a) / (b - a)
                    pelvis = tracks['pelvis', 'translation']
                    height = pelvis[index * 3 + 1] * (1.0 - fraction) + pelvis[(index + 1) * 3 + 1] * fraction
                    for side, offset in (('l', 0.0), ('r', math.pi)):
                        angles = []
                        for part in ('thigh', 'shin'):
                            values = tracks[f'{part}_{side}', 'rotation']
                            # Single-axis shortest-path SLERP is linear in angle.
                            angles.append(sum(2.0 * math.atan2(values[key * 4], values[key * 4 + 3]) * weight
                                              for key, weight in ((index, 1.0 - fraction), (index + 1, fraction))))
                        hip, knee = angles
                        y = height - LEG_LENGTH * (math.cos(hip) + math.cos(hip + knee))
                        z = -LEG_LENGTH * (math.sin(hip) + math.sin(hip + knee))
                        target_z, target_y = foot_target(time * TAU + offset, run)
                        maximum = max(maximum, math.hypot(z - target_z, y - target_y))
                self.assertLess(maximum, 0.0005, f'{clip["name"]} interpolation ankle error: {maximum}')

        def test_stance_cancels_root_travel_at_low_and_high_frame_rates(self):
            for hz in (9, 30, 60):
                for run in (0.0, 0.25, 0.5, 0.75, 1.0):
                    stance = stance_fraction(run)
                    previous = None
                    for i in range(hz + 1):
                        time = stance * i / hz
                        z, y = foot_target(time * TAU, run)
                        foot = cycle_distance(run) * time + z
                        self.assertAlmostEqual(y, ANKLE_HEIGHT, places=10)
                        if previous is not None:
                            self.assertAlmostEqual(foot, previous, places=10)
                        previous = foot

    suite = unittest.defaultTestLoader.loadTestsFromTestCase(LocomotionTests)
    if not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful():
        raise SystemExit(1)


if __name__ == '__main__':
    _self_test()
