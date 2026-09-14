#!/usr/bin/env python3
"""Convert Bevy Chrome tracing into an offline waterfall and operation census.

Usage: python3 tools/operation_waterfall.py trace.json --output report.html
Only completed synchronous spans are timed; unfinished spans are reported.
Times are CPU wall durations, not GPU timestamp measurements.
"""
import argparse
from collections import defaultdict
import html
import json
import math
from pathlib import Path
import statistics


def decode(events):
    stacks = defaultdict(list)
    spans, names = [], {}
    unmatched = 0
    for event in events:
        key = (event.get('pid', 0), event.get('tid', 0))
        phase = event.get('ph')
        if phase == 'M' and event.get('name') == 'thread_name':
            names[key] = event.get('args', {}).get('name', str(key))
        elif phase == 'B':
            stacks[key].append((event, len(stacks[key])))
        elif phase == 'E':
            if not stacks[key]:
                unmatched += 1
                continue
            start, depth = stacks[key].pop()
            duration = event['ts'] - start['ts']
            if duration < 0:
                raise ValueError('Trace has a negative span duration')
            spans.append(dict(name=start.get('name', '?'), start=start['ts'],
                              duration=duration, thread=key, depth=depth))
        elif phase == 'X':
            spans.append(dict(name=event.get('name', '?'), start=event['ts'],
                              duration=event['dur'], thread=key, depth=0))
    return spans, names, unmatched + sum(map(len, stacks.values()))


def summarize(spans):
    grouped = defaultdict(list)
    for span in spans:
        grouped[span['name']].append(span['duration'])
    rows = []
    for name, durations in grouped.items():
        durations.sort()
        rows.append(dict(name=name, count=len(durations), total_ms=sum(durations)/1000,
                         median_us=statistics.median(durations),
                         p95_us=durations[math.ceil(len(durations)*.95)-1],
                         max_us=durations[-1]))
    return sorted(rows, key=lambda row: row['total_ms'], reverse=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('trace', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source = json.loads(args.trace.read_text())
    spans, names, incomplete = decode(source if isinstance(source, list) else source['traceEvents'])
    if not spans:
        raise ValueError('No completed spans in capture')
    spans.sort(key=lambda s: s['start'])
    origin = min(s['start'] for s in spans)
    end = max(s['start'] + s['duration'] for s in spans)
    threads = sorted({s['thread'] for s in spans})
    thread_ids = {key: i for i, key in enumerate(threads)}
    labels = [names.get(key, f'process {key[0]} / thread {key[1]}') for key in threads]
    operations = list(dict.fromkeys(s['name'] for s in spans))
    operation_ids = {name: i for i, name in enumerate(operations)}
    compact = [[operation_ids[s['name']], round((s['start']-origin)/1000, 4),
                round(s['duration']/1000, 4), thread_ids[s['thread']], s['depth']] for s in spans]
    frames = [(s['start']-origin)/1000 for s in spans if s['name'].rstrip(': ') == 'update']
    summary = summarize(spans)
    report = dict(source=str(args.trace), completed_spans=len(spans), incomplete_spans=incomplete,
                  duration_ms=(end-origin)/1000, threads=len(threads), operations=summary,
                  frame_intervals_ms=[b-a for a,b in zip(frames, frames[1:])])
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.with_suffix('.summary.json').write_text(json.dumps(report, indent=2))
    data = json.dumps(dict(spans=compact, names=operations, threads=labels, frames=frames,
                           end=(end-origin)/1000, summary=summary)).replace('<', '\\u003c')
    template = Path(__file__).with_name('operation_waterfall.html').read_text()
    args.output.write_text(template.replace('__DATA__', data).replace('__SOURCE__', html.escape(str(args.trace))))
    print(f'{len(spans):,} spans / {len(operations)} operations / {len(threads)} threads; {incomplete} incomplete')
    print(args.output)
    for row in summary:
        if 'system:' in row['name'] and 'crystal_' in row['name']:
            print(f"{row['total_ms']:10.2f} ms total | p95 {row['p95_us']:10.1f} us | {row['name']}")


if __name__ == '__main__':
    main()
