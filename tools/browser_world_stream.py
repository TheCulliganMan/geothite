#!/usr/bin/env python3
"""Analyze browser presentation receipts without treating a bridge call as a seam."""
import argparse
import bisect
import json
import math
from pathlib import Path
import statistics

from performance_audit_motion import motion_series, stationary_camera_drift
from performance_audit_residency import summarize
from performance_audit import frame_budget


def motion_report(report, trace):
    """Keep browser samples on their CPU observation clock; no invented game ticks."""
    if not report['success'] or trace['droppedEvents']:
        raise ValueError('Failed gameplay or incomplete browser trace')
    events = sorted((event for event in trace['traceEvents']
                     if event['name'] == 'presentation.world_motion'
                     and not event.get('inProgress')), key=lambda event: event['ts'])
    if not events:
        raise ValueError('No browser motion receipts')
    main = sorted((event for event in trace['traceEvents']
                   if event['name'] == 'frame.main' and not event.get('inProgress')),
                  key=lambda event: event['ts'])
    starts = [event['ts'] for event in main]
    frames = []
    for index, event in enumerate(events):
        time = event['ts'] / 1000
        if not math.isfinite(time) or time < 0 or (frames and time <= frames[-1]['start_ms']):
            raise ValueError('Invalid or duplicate browser sample clock')
        main_index = bisect.bisect_right(starts, event['ts']) - 1
        duration = main[main_index]['dur'] / 1000 if main_index >= 0 else None
        frames.append(dict(id=index, start_ms=time, measured=True,
                           interval_ms=None if index == 0 else time - frames[-1]['start_ms'],
                           simulation_tick=None, walk_remaining=None, terrain_pending=None,
                           main_cpu_ms=duration, motion=event['args']))
    # The next presentation observation was not recorded. Never reuse the
    # preceding interval as evidence about this censored tail.
    frames[-1]['interval_ms'] = None
    return dict(success=True, frames=frames,
                clock='CPU presentation-observation timestamps; final interval censored',
                motion_capture=dict(enabled=False))


def crossing_samples(series):
    """Measure actual connected-map changes and the next observed displacement."""
    result = []
    for index, frame in enumerate(series):
        if not frame['connected'] or index == 0:
            continue
        previous = series[index - 1]
        valid = all(row['active'] and row['player'] is not None
                    and row['camera'] is not None for row in [previous, frame])
        following_ms = None
        if valid:
            for following in series[index + 1:]:
                if following['segment'] != frame['segment'] or not following['active']:
                    break
                if following['player'] is None:
                    break
                if any(abs(following['player'][axis] - frame['player'][axis]) > .000001
                       for axis in range(2)):
                    following_ms = following['ms'] - frame['ms']
                    break
        result.append(dict(source=previous['map'], destination=frame['map'],
                           frame=frame['id'], ms=frame['ms'], valid=valid,
                           observed_sample_interval_ms=frame['interval_ms'],
                           sample_interval_ms=frame['interval_ms'] if valid else None,
                           next_motion_ms=following_ms,
                           player_delta_cells=([frame['player'][axis] - previous['player'][axis]
                                                for axis in range(2)] if valid else None),
                           camera_delta_cells=([frame['camera'][axis] - previous['camera'][axis]
                                                for axis in range(2)] if valid else None)))
    return result


def gpu_summary(gpu):
    """GPU pass envelopes use their independent device clock, never CPU waits."""
    if not gpu or not gpu.get('supported'):
        return dict(available=False, reason=(gpu or {}).get('reason', 'Not requested'))
    envelopes = []
    unions = []
    skipped = 0
    incomplete = 0
    empty_passes = quantized_passes = invalid_passes = valid_passes = 0
    for frame in gpu['frames']:
        if gpu.get('timestamps') is False:
            continue
        if frame.get('complete') is False:
            incomplete += 1
            continue
        spans = []
        invalid = False
        for span in frame['passes']:
            start, end = int(span['startNs']), int(span['endNs'])
            if start == end == 0:
                # Empty passes can leave both query slots unwritten. Their
                # duration is unknown, not evidence that other queries failed.
                empty_passes += 1
            elif start <= 0 or end < start:
                invalid_passes += 1
                invalid = True
            else:
                quantized_passes += start == end
                valid_passes += 1
                spans.append((start, end))
        if not spans or invalid:
            skipped += 1
            continue
        spans.sort()
        envelopes.append((max(end for _, end in spans) - spans[0][0]) / 1e6)
        total = 0
        first, last = spans[0]
        for start, end in spans[1:]:
            if start <= last:
                last = max(last, end)
            else:
                total += last - first
                first, last = start, end
        unions.append((total + last - first) / 1e6)
    completions = [frame['completionCallbackLatencyMs'] for frame in gpu['frames']
                   if isinstance(frame.get('completionCallbackLatencyMs'), (int, float))
                   and math.isfinite(frame['completionCallbackLatencyMs'])
                   and frame['completionCallbackLatencyMs'] >= 0]
    return dict(available=bool(envelopes), frames=len(envelopes), invalid_frames=skipped,
                observed_frames=len(gpu['frames']), valid_passes=valid_passes,
                incomplete_frames=incomplete,
                empty_or_unwritten_passes=empty_passes, invalid_passes=invalid_passes,
                zero_duration_quantized_passes=quantized_passes,
                requested_frames=gpu.get('requestedFrames'),
                timestamps_enabled=gpu.get('timestamps', True),
                pass_capacity_reached=gpu.get('passCapacityReached'),
                query_pages=gpu.get('queryPages'), pass_page_capacity=gpu.get('passPageCapacity'),
                unframed_passes=gpu.get('unframedPasses'),
                median_envelope_ms=statistics.median(envelopes) if envelopes else None,
                maximum_envelope_ms=max(envelopes, default=None),
                median_active_union_ms=statistics.median(unions) if unions else None,
                maximum_active_union_ms=max(unions, default=None),
                completion_callback_frames=len(completions),
                completion_callback_errors=sum(bool(frame.get('completionError')) for frame in gpu['frames']),
                median_completion_callback_ms=statistics.median(completions) if completions else None,
                maximum_completion_callback_ms=max(completions, default=None),
                meaning='GPU pass timestamps for render submissions; empty/unwritten passes excluded; equal positive timestamps have zero measured duration at device resolution; query/resolve and callback overhead included; completion callback latency includes GPU queue and event-loop delivery; no CPU clock calibration or physical display timing')


def primary_summary(report):
    """Summarize profiling-off rAF intervals without inventing motion evidence."""
    intervals = report['intervals']
    if not report['success'] or intervals['count'] <= 0 or any(
            not math.isfinite(intervals[key]) or intervals[key] <= 0
            for key in ['median', 'p95', 'maximum']):
        raise ValueError('Failed gameplay or invalid browser rAF intervals')
    return dict(scenario=report['scenario'], browser=report['browser'],
                profiling='off', intervals=intervals, residency=None,
                frame_budget=frame_budget(intervals),
                stationary_camera=None, crossings=None, workers=report['workers'],
                gpu=gpu_summary(report.get('gpu')),
                clock='Browser requestAnimationFrame timestamps; no physical display timing',
                continuity_evidence='Requires separate per-frame motion diagnostic; rAF alone does not prove rendering')


def analyze(directory):
    report = json.loads((directory / 'report.json').read_text())
    if report.get('profiling') == 'off':
        result = primary_summary(report)
        (directory / 'analysis.json').write_text(json.dumps(result, indent=2) + '\n')
        return result
    trace = json.loads((directory / 'trace.json').read_text())
    motion = motion_report(report, trace)
    series = motion_series(motion)
    result = dict(scenario=report['scenario'], browser=report['browser'],
                  intervals=report['intervals'], residency=summarize(motion),
                  frame_budget=frame_budget(report['intervals']),
                  stationary_camera=stationary_camera_drift(series),
                  visual_comparison_available=report.get('pairThreeDAvailable'),
                  crossings=crossing_samples(series), workers=report['workers'],
                  gpu=gpu_summary(report.get('gpu')), clock=motion['clock'])
    (directory / 'motion-audit.json').write_text(json.dumps(motion) + '\n')
    (directory / 'analysis.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--require-frame-budget', action='store_true',
                        help='Exit with status 3 if any measured frame exceeds 45 ms')
    args = parser.parse_args()
    result = analyze(args.directory)
    print(json.dumps(result, indent=2))
    if args.require_frame_budget and not result['frame_budget']['passed']:
        raise SystemExit(3)


if __name__ == '__main__':
    main()
