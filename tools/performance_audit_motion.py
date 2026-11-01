#!/usr/bin/env python3
"""Plot rendered player/camera continuity across proven connected map handoffs."""
import argparse
import csv
import json
import math
from pathlib import Path
from performance_audit_residency import camera_valid


def motion_series(report):
    """Align source coordinates using reciprocal map placements, never tile guesses.

    Values describe the production presentation contract on the CPU clock.
    They do not measure physical display latency or infer motion from a nonzero
    natural offset. Unproven warps start a separate coordinate segment.
    """
    if not report.get('success'):
        raise ValueError('Cannot accept a failed gameplay capture')
    result = []
    origin = [0.0, 0.0]
    previous_map = None
    segment = 0
    for frame in report['frames']:
        motion = frame.get('motion')
        if motion is None:
            raise ValueError('Capture lacks source-space motion receipts; collect a fresh run')
        name = motion['map']
        if len(motion['tile_size']) != 2 or any(not math.isfinite(v) or v <= 0 for v in motion['tile_size']):
            raise ValueError('Invalid presentation tile scale')
        changed = previous_map is not None and name != previous_map
        shift = motion['connected_shift'] if changed else None
        if changed:
            if shift is None:
                segment += 1
                origin = [0.0, 0.0]
            else:
                origin[0] -= shift[0] / 2
                origin[1] += shift[1] / 2
        def aligned(point):
            if point is None:
                return None
            if len(point) != 2 or not all(math.isfinite(v) for v in point):
                raise ValueError('Invalid presentation position')
            return [point[axis] / (2 * motion['tile_size'][axis]) + origin[axis] for axis in range(2)]
        result.append(dict(
            id=frame['id'], ms=frame['start_ms'], tick=frame['simulation_tick'],
            interval_ms=frame.get('interval_ms'), main_app_wall_ms=frame.get('main_cpu_ms'),
            measured=frame['measured'], map=name, segment=segment,
            connected=changed and shift is not None, shift=shift,
            active=motion['active'] and camera_valid(motion) is True,
            player=aligned(motion['player_foot']),
            camera=aligned(None if camera_valid(motion) is not True else [motion['camera_eye'][0], -motion['camera_eye'][2]]),
            camera_height_cells=(motion['camera_eye'][1] / (2 * motion['tile_size'][1])
                                 if camera_valid(motion) is True else None),
            walk_remaining=frame['walk_remaining'], terrain_pending=frame['terrain_pending'],
            connected_terrain=motion['connected_terrain']))
        previous_map = name
    return result


def crossings(series):
    """Return each measured seam, preserving inactive/missing-foot evidence."""
    result = []
    for index, frame in enumerate(series):
        if not frame['measured'] or not frame['connected'] or index == 0:
            continue
        previous = series[index - 1]
        valid = frame['active'] and previous['active'] and all(
            row[field] is not None for row in [frame, previous] for field in ['player', 'camera'])
        delta = None if not valid else [frame['player'][a] - previous['player'][a] for a in range(2)]
        camera_delta = None if not valid else [frame['camera'][a] - previous['camera'][a] for a in range(2)]
        direction = [-frame['shift'][0], frame['shift'][1]]
        component = max(range(2), key=lambda a: abs(direction[a]))
        next_motion_ms = None
        if valid:
            for following in series[index + 1:]:
                if following['segment'] != frame['segment'] or not following['active'] or following['player'] is None:
                    break
                if any(abs(following['player'][axis] - frame['player'][axis]) > .000001 for axis in range(2)):
                    next_motion_ms = following['ms'] - frame['ms']
                    break
        result.append(dict(frame=frame['id'], ms=frame['ms'], source=previous['map'],
            destination=frame['map'], valid=valid, simulation_ticks=frame['tick'] - previous['tick'],
            frame_interval_ms=frame['interval_ms'], main_app_wall_ms=frame['main_app_wall_ms'],
            next_motion_ms=next_motion_ms,
            player_delta_cells=delta, camera_delta_cells=camera_delta,
            component=component, sign=1 if direction[component] >= 0 else -1,
            walk_remaining=frame['walk_remaining'], terrain_pending=frame['terrain_pending']))
    return result


def stationary_camera_drift(series):
    """Expose camera corrections that occur without an observed player move."""
    events = []
    vertical = []
    horizontal = []
    for before, after in zip(series, series[1:]):
        if not all(frame['measured'] and frame['active'] and frame['player'] is not None
                   and frame['camera'] is not None and frame['camera_height_cells'] is not None
                   for frame in (before, after)) or before['segment'] != after['segment']:
            continue
        if any(abs(before['player'][axis] - after['player'][axis]) > .000001 for axis in range(2)):
            continue
        dy = abs(after['camera_height_cells'] - before['camera_height_cells'])
        dxz = math.dist(after['camera'], before['camera'])
        vertical.append(dy);horizontal.append(dxz)
        if max(dy, dxz) > .0001:
            events.append(dict(before_frame=before['id'], after_frame=after['id'],
                               ms=after['ms'], vertical_cells=dy, horizontal_cells=dxz))
    return dict(observed_pairs=len(vertical), events=events,
                max_vertical_cells=max(vertical, default=None),
                max_horizontal_cells=max(horizontal, default=None))


def export(before, after, output, radius_ms):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt

    output.mkdir(parents=True, exist_ok=True)
    reports = {label: json.loads(path.read_text()) for label, path in [('before', before), ('after', after)]}
    readbacks = [report.get('motion_capture', {}).get('enabled', False) for report in reports.values()]
    if readbacks[0] != readbacks[1]:
        raise ValueError('Cannot compare screenshot QA against a no-readback timing run')
    samples = {label: motion_series(report) for label, report in reports.items()}
    events = {label: crossings(series) for label, series in samples.items()}
    if not events['before'] or len(events['before']) != len(events['after']):
        raise ValueError('Both runs must contain the same nonzero number of connected crossings')
    for old, new in zip(events['before'], events['after']):
        if (old['source'], old['destination']) != (new['source'], new['destination']):
            raise ValueError('Runs crossed different connected maps')
    fig, axes = plt.subplots(len(events['before']), 3, figsize=(18, 4 * len(events['before'])), squeeze=False)
    rows = []
    for crossing, (old, new) in enumerate(zip(events['before'], events['after'])):
        for label, event, color in [('before', old, '#cf6b36'), ('after', new, '#267caa')]:
            center = next(frame for frame in samples[label] if frame['id'] == event['frame'])
            selected = [frame for frame in samples[label] if abs(frame['ms'] - event['ms']) <= radius_ms and frame['segment'] == center['segment']]
            for frame in selected:
                for field, axis in [('player', 0), ('camera', 1), ('camera_height_cells', 2)]:
                    point = frame[field]
                    # Signed distance along the actual crossing direction.
                    component, sign = event['component'], event['sign']
                    distance = None if point is None or center[field] is None else (
                        point - center[field] if field == 'camera_height_cells'
                        else (point[component] - center[field][component]) * sign)
                    rows.append(dict(build=label, crossing=crossing, frame=frame['id'],
                        relative_ms=frame['ms'] - event['ms'], field=field, distance_cells=distance,
                        tick=frame['tick'], active=frame['active'], walk_remaining=frame['walk_remaining']))
            for field, axis in [('player', 0), ('camera', 1), ('camera_height_cells', 2)]:
                values = [row for row in rows if row['build'] == label and row['crossing'] == crossing and row['field'] == field]
                axes[crossing, axis].plot([row['relative_ms'] for row in values],
                    [math.nan if row['distance_cells'] is None else row['distance_cells'] for row in values],
                    '.-', color=color, label=label, markersize=3)
                for row in values:
                    if not row['active']:
                        axes[crossing, axis].axvline(row['relative_ms'], color='#b9222a', alpha=.3, linewidth=1)
        for axis, title in zip(axes[crossing], ['Player foot', 'Camera (ground plane)', 'Camera height']):
            axis.axvline(0, color='#888888', linewidth=.8, linestyle='--')
            axis.set_title(f"{title}: {old['source']} → {old['destination']}", fontsize=11)
            axis.set(xlabel='Time from map handoff (ms)', ylabel='Travel (gameplay cells)')
            axis.grid(alpha=.2)
            axis.legend()
    note = 'visual QA readbacks enabled; timing perturbed' if readbacks[0] else 'screenshot readbacks disabled'
    fig.suptitle('Connected-map motion • CPU presentation samples • ' + note + '\nRed vertical marks: inactive 3D scene', fontsize=12)
    fig.tight_layout(rect=(0, 0, 1, .93), h_pad=2, w_pad=2)
    fig.savefig(output/'motion.png', dpi=160)
    fig.savefig(output/'motion.svg')
    plt.close(fig)
    with (output/'motion.csv').open('w', newline='') as target:
        writer = csv.DictWriter(target, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    (output/'crossings.json').write_text(json.dumps(events, indent=2) + '\n')
    (output/'stationary-camera-drift.json').write_text(json.dumps(
        {label: stationary_camera_drift(series) for label, series in samples.items()}, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path, required=True, help='Raw audit.json')
    parser.add_argument('--after', type=Path, required=True, help='Raw audit.json')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--radius-ms', type=float, default=300)
    args = parser.parse_args()
    if not math.isfinite(args.radius_ms) or args.radius_ms <= 0:
        parser.error('radius-ms must be positive and finite')
    export(args.before, args.after, args.output, args.radius_ms)


if __name__ == '__main__':
    main()
