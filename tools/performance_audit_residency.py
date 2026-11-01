#!/usr/bin/env python3
"""Audit whole-journey 3D residency, including gaps away from logical seams."""
import argparse
import json
import math
from pathlib import Path


def camera_valid(motion):
    """Validate all transform components; absent legacy fields remain unknown."""
    missing = False
    for name, size in [('camera_eye', 3), ('camera_rotation', 4)]:
        if name not in motion:
            missing = True
            continue
        value = motion[name]
        if value is None or len(value) != size or any(
                not isinstance(component, (int, float)) or not math.isfinite(component)
                for component in value):
            return False
    return None if missing else True


def summarize(report):
    """Measure observed unavailable intervals; missing observations stay unknown.

    A frame describes the state from its CPU start to the next frame start.
    This is renderer availability, not physical display latency. Non-measured
    frames never contribute, and the final interval can remain censored.
    """
    frames = report['frames']
    measured = [frame for frame in frames if frame['measured']]
    if not measured or all(frame.get('motion') is None for frame in measured):
        return dict(available=False, reason='No measured renderer residency observations')
    if any(frame.get('motion') is None for frame in measured):
        raise ValueError('Incomplete renderer residency observations')
    episodes = []
    current = None
    missing_intervals = 0
    unavailable_ms = 0.
    observed_ms = 0.
    invalid_camera_frames = 0
    unknown_camera_frames = 0
    for index, frame in enumerate(frames):
        if not frame['measured']:
            current = None
            continue
        start = frame['start_ms']
        interval = (frames[index + 1]['start_ms'] - start
                    if index + 1 < len(frames) else frame.get('interval_ms'))
        if not math.isfinite(start) or (interval is not None and
                (not math.isfinite(interval) or interval < 0)):
            raise ValueError('Invalid residency observation interval')
        if interval is None:
            missing_intervals += 1
        else:
            observed_ms += interval
        camera = camera_valid(frame['motion'])
        invalid_camera_frames += bool(frame['motion']['active'] and camera is False)
        unknown_camera_frames += camera is None
        if frame['motion']['active'] and camera is not False:
            current = None
            continue
        if current is None:
            current = dict(first_frame=frame['id'], start_ms=start, duration_ms=0.,
                           frames=0, interval_unknown=False)
            episodes.append(current)
        current['frames'] += 1
        current['last_frame'] = frame['id']
        current['interval_unknown'] |= interval is None
        if interval is not None:
            current['duration_ms'] += interval
            unavailable_ms += interval
    return dict(available=True, measured_frames=len(measured),
                unavailable_frames=sum(episode['frames'] for episode in episodes),
                observed_ms=observed_ms, unavailable_ms=unavailable_ms,
                invalid_camera_frames=invalid_camera_frames,
                unknown_camera_frames=unknown_camera_frames,
                unknown_intervals=missing_intervals, episodes=episodes)


def export(reports, output):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from performance_audit_motion import motion_series

    output.mkdir(parents=True, exist_ok=True)
    fig, axes = plt.subplots(len(reports), 2, figsize=(14, 3.8 * len(reports)), squeeze=False)
    results = {}
    for row, (label, path) in enumerate(reports.items()):
        report = json.loads(path.read_text())
        result = summarize(report)
        if not result['available']:
            raise ValueError(result['reason'])
        results[label] = result
        series = [frame for frame in motion_series(report) if frame['measured']]
        first = series[0]['ms']
        time = [(frame['ms'] - first) / 1000 for frame in series]
        position = axes[row, 0]
        cadence = axes[row, 1]
        for segment in dict.fromkeys(frame['segment'] for frame in series):
            selected = [frame for frame in series if frame['segment'] == segment and frame['player'] is not None]
            for component, color, name in [(0, '#267caa', 'East / west'), (1, '#698a36', 'North / south')]:
                position.plot([(frame['ms'] - first) / 1000 for frame in selected],
                              [frame['player'][component] for frame in selected], color=color,
                              label=name if segment == series[0]['segment'] else None, linewidth=1.3)
        cadence.plot(time, [frame['interval_ms'] for frame in series], color='#267caa', linewidth=.8, label='Frame interval')
        cadence.plot(time, [frame['main_app_wall_ms'] for frame in series], color='#be6b35', linewidth=.8, label='Main app wall')
        for episode in result['episodes']:
            start = (episode['start_ms'] - first) / 1000
            end = start + episode['duration_ms'] / 1000
            for axis in axes[row]:
                axis.axvspan(start, end, color='#bd3030', alpha=.2)
        for frame in series:
            if frame['connected']:
                seam = (frame['ms'] - first) / 1000
                for axis in axes[row]:
                    axis.axvline(seam, color='#777777', linestyle='--', linewidth=.8)
        position.set_ylabel('Source-space position (gameplay cells)')
        cadence.set_ylabel('Milliseconds')
        position.set_title(f"{label}: {result['unavailable_ms']:.1f} ms without active 3D")
        cadence.set_title(f"{label}: whole-journey frame cadence")
        for axis in axes[row]:
            axis.set_xlabel('Seconds from measurement start')
            axis.grid(alpha=.2)
            axis.legend(loc='upper left', fontsize=8)
    fig.suptitle('Continuous world residency', fontsize=17)
    fig.text(.02, .015, 'Red bands: 3D renderer unavailable. Dashed lines: logical map changes. Includes travel between seams.\n'
             'CPU observations; offscreen throughput, not physical display latency. Diagnostic traces/readbacks may perturb timings.', fontsize=9)
    fig.tight_layout(rect=(0, .09, 1, .94), h_pad=2, w_pad=2)
    for extension in ('png', 'svg'):
        fig.savefig(output / f'residency.{extension}', dpi=160)
    plt.close(fig)
    (output / 'residency.json').write_text(json.dumps(results, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path)
    parser.add_argument('--after', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    reports = {'before': args.before} if args.before else {}
    reports['after'] = args.after
    export(reports, args.output)


if __name__ == '__main__':
    main()
