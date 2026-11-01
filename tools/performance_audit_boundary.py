#!/usr/bin/env python3
"""Measure crossing frames and subsequent motion, independently of route duration."""
import argparse
import csv
import json
import math
from pathlib import Path
import statistics

from performance_audit import compare
from performance_audit_motion import crossings, motion_series


def match_events(before, after):
    """Keep matched seam identities and missing future observations explicit."""
    if not before or len(before) != len(after):
        raise ValueError('Both runs must contain the same nonzero number of crossings')
    result = []
    for old, new in zip(before, after):
        if (old['source'], old['destination']) != (new['source'], new['destination']):
            raise ValueError('Paired runs crossed different seams')
        pair = []
        for event in (old, new):
            interval = event['frame_interval_ms']
            if interval is None or not math.isfinite(interval) or interval <= 0:
                raise ValueError('Crossing frame has no valid measured interval')
            future = event['next_motion_ms']
            if future is not None and (not math.isfinite(future) or future < 0):
                raise ValueError('Invalid subsequent movement time')
            # Preserve the failed visual outcome and its raw CPU observation,
            # without advertising that interval as a fast rendered crossing.
            pair.append(dict(event, observed_frame_interval_ms=interval,
                             frame_interval_ms=interval if event['valid'] else None,
                             next_motion_ms=future if event['valid'] else None))
        result.append(tuple(pair))
    return result


def collect(before_path, after_path):
    summaries = {label: json.loads(path.read_text()) for label, path in [('before', before_path), ('after', after_path)]}
    paths = dict(before=before_path, after=after_path)
    previous = {scene['name']: scene for scene in summaries['before']['scenarios']}
    rows = []
    for current in summaries['after']['scenarios']:
        if not current['name'].endswith('-continuous'):
            continue
        baseline = previous.get(current['name'])
        if baseline is None or not compare(current, baseline, 10, .2)['comparable']:
            raise ValueError('Crossing scenarios are incompatible or contain rejected captures')
        runs = {label: {run['pair']: run for run in scene['runs']} for label, scene in [('before', baseline), ('after', current)]}
        if len(runs['before']) != len(baseline['runs']) or len(runs['after']) != len(current['runs']) or runs['before'].keys() != runs['after'].keys():
            raise ValueError('Missing or duplicate adjacent pair identity')
        for pair in sorted(runs['before']):
            events = {}
            for label in ('before', 'after'):
                path = (paths[label].parent / runs[label][pair]['directory'] / 'audit.json').resolve()
                report = json.loads(path.read_text())
                if report.get('motion_capture', {}).get('enabled'):
                    raise ValueError('Screenshot readbacks cannot supply primary seam timings')
                events[label] = crossings(motion_series(report))
            for old, new in match_events(events['before'], events['after']):
                for label, event in [('before', old), ('after', new)]:
                    rows.append(dict(scenario=current['name'], pair=pair, build=label,
                        source=event['source'], destination=event['destination'], frame=event['frame'],
                        presentation_valid=event['valid'],
                        observed_frame_interval_ms=event['observed_frame_interval_ms'],
                        frame_interval_ms=event['frame_interval_ms'], main_app_wall_ms=event['main_app_wall_ms'],
                        next_motion_ms=event['next_motion_ms'], landing_ticks=event['walk_remaining']))
    if not rows:
        raise ValueError('No held-movement crossing recipes selected')
    return rows


def export(rows, output):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    output.mkdir(parents=True, exist_ok=True)
    with (output/'boundary-timings.csv').open('w', newline='') as target:
        writer = csv.DictWriter(target, fieldnames=list(rows[0]))
        writer.writeheader();writer.writerows(rows)
    (output/'boundary-timings.json').write_text(json.dumps(rows, indent=2)+'\n')
    seams = list(dict.fromkeys((r['scenario'], r['source'], r['destination']) for r in rows))
    fig, axes = plt.subplots(1, 2, figsize=(13, max(5., .58 * len(seams) + 2)), sharey=True)
    for axis, metric, title in zip(axes, ['frame_interval_ms', 'next_motion_ms'], ['Crossing frame interval', 'Crossing → next observed movement']):
        largest = 0.
        for label, color in [('before', '#be6b35'), ('after', '#267caa')]:
            axis.scatter([], [], color=color, s=20, label=label)
        for index, seam in enumerate(seams):
            for label, color, offset in [('before', '#be6b35', -.10), ('after', '#267caa', .10)]:
                values = [r[metric] for r in rows if (r['scenario'], r['source'], r['destination']) == seam and r['build'] == label and r[metric] is not None]
                unavailable = sum(not r['presentation_valid'] for r in rows
                                  if (r['scenario'], r['source'], r['destination']) == seam and r['build'] == label)
                if not values:
                    if unavailable:
                        axis.annotate(f'{unavailable} unavailable', (0, index + offset),
                                      xytext=(6, 0), textcoords='offset points', va='center',
                                      fontsize=8, color='#b33d3d')
                    continue
                median = statistics.median(values)
                axis.scatter(values, [index + offset] * len(values), color=color, s=20,
                    label=None)
                axis.plot([median, median], [index + offset - .09, index + offset + .09], color=color, linewidth=2)
                note = f'{median:.1f}' + (f' · {unavailable} unavailable' if unavailable else '')
                axis.annotate(note, (max(values), index + offset), xytext=(6, 0), textcoords='offset points', va='center', fontsize=8, color=color)
                largest = max(largest, max(values))
        axis.set_title(title, fontsize=12)
        axis.set_xlabel('Milliseconds · individual seams; bars mark medians')
        axis.set_xlim(0, largest * 1.45 or 1)
        axis.grid(axis='x', alpha=.25)
        axis.legend(loc='upper right')
    identities = [(s[1], s[2]) for s in seams]
    labels = [f'{s[1]} → {s[2]}' + (f'\n{s[0]}' if identities.count((s[1], s[2])) > 1 else '') for s in seams]
    axes[0].set_yticks(range(len(seams)), labels, fontsize=9)
    axes[0].invert_yaxis()
    fig.suptitle('Boundary timings during held movement', fontsize=17)
    fig.text(.02, .02, 'Only continuous walking recipes with valid 3D at the seam. No physical display latency or route-travel time.\n'
        'Unavailable crossings remain counted; their raw CPU intervals are retained separately in the numeric table.', fontsize=9)
    fig.tight_layout(rect=(0, .09, 1, .94), w_pad=3)
    for extension in ('png', 'svg'):
        fig.savefig(output/f'boundary-timings.{extension}', dpi=160)
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--after', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    export(collect(args.before, args.after), args.output)


if __name__ == '__main__':
    main()
