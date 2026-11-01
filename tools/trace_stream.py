"""Bounded-memory readers for completed Chrome spans and audit frame windows."""
from collections import defaultdict
import json
import math
import re


def span_fields(event):
    fields = dict(event.get('args', {}))
    if event.get('name', '').startswith('audit_frame_marker') and 'frame_id' not in fields:
        match = re.search(r'frame_id[=: ]+(\d+)', event['name'])
        if match:
            fields['frame_id'] = int(match[1])
    return fields


def events(path, chunk_size=1024*1024):
    """Yield one object at a time; raw arrays and traceEvents wrappers work."""
    decoder = json.JSONDecoder()
    with path.open() as source:
        buffer = source.read(chunk_size)
        stripped = buffer.lstrip()
        if stripped.startswith('['):
            position = buffer.index('[') + 1
        elif stripped.startswith('{'):
            while True:
                match = re.search(r'"traceEvents"\s*:\s*\[', buffer)
                if match:
                    position = match.end()
                    break
                chunk = source.read(chunk_size)
                if not chunk or len(buffer) > 1024*1024:
                    raise ValueError('Missing traceEvents array')
                buffer += chunk
        else:
            raise ValueError('Trace must be a JSON array or traceEvents object')
        eof = False
        need_comma = False
        after_comma = False
        while True:
            while position < len(buffer) and buffer[position].isspace():
                position += 1
            if position == len(buffer):
                chunk = source.read(chunk_size)
                if not chunk:
                    raise ValueError('Truncated trace array')
                buffer = chunk
                position = 0
                continue
            char = buffer[position]
            if char == ']':
                if after_comma:
                    raise ValueError('Trailing trace comma')
                return
            if need_comma:
                if char != ',':
                    raise ValueError('Missing trace event comma')
                position += 1
                need_comma = False
                after_comma = True
                continue
            try:
                event, end = decoder.raw_decode(buffer, position)
            except json.JSONDecodeError as error:
                if eof:
                    raise ValueError('Invalid or truncated trace event') from error
                chunk = source.read(chunk_size)
                eof = not chunk
                buffer = buffer[position:] + chunk
                position = 0
                if len(buffer) > 16*1024*1024:
                    raise ValueError('Trace event exceeds 16 MiB')
                continue
            if not isinstance(event, dict):
                raise ValueError('Trace events must be objects')
            yield event
            position = end
            need_comma = True
            after_comma = False


def window(path, frame_ids):
    if not frame_ids:
        raise ValueError('No measured frame IDs')
    start = None
    seen = set()
    last = max(frame_ids)
    for event in events(path):
        if event.get('ph') not in ('B', 'X') or not event.get('name', '').startswith('audit_frame_marker'):
            continue
        frame = span_fields(event).get('frame_id')
        if frame is None:
            continue
        frame = int(frame)
        if frame in frame_ids:
            seen.add(frame)
            if start is None:
                start = event['ts']
        elif frame > last and start is not None:
            if seen != frame_ids:
                raise ValueError('Measured frame markers are missing')
            return start, event['ts']
    raise ValueError('Trace lacks frame-ID markers spanning the measured interval')


def spans(event_iterator, names, counts, selected_window=None):
    """Strict B/E nesting supplies exclusive wall duration without retaining spans."""
    stacks = defaultdict(list)
    counts.setdefault('incomplete', 0)
    for event in event_iterator:
        key = (event.get('pid', 0), event.get('tid', 0))
        phase = event.get('ph')
        if phase == 'M' and event.get('name') == 'thread_name':
            names[key] = event.get('args', {}).get('name', str(key))
        elif phase == 'B':
            stacks[key].append((event, len(stacks[key]), 0.))
        elif phase in ('E', 'X'):
            if phase == 'E':
                if not stacks[key]:
                    counts['incomplete'] += 1
                    continue
                first, depth, children = stacks[key].pop()
                duration = event['ts'] - first['ts']
            else:
                first, depth, children = event, len(stacks[key]), 0.
                duration = event['dur']
            if not math.isfinite(duration) or duration < 0:
                raise ValueError('Invalid trace duration')
            low, high = first['ts'], first['ts'] + duration
            if selected_window:
                low, high = max(low, selected_window[0]), min(high, selected_window[1])
            clipped = max(0., high-low)
            if stacks[key]:
                parent, parent_depth, parent_children = stacks[key][-1]
                stacks[key][-1] = parent, parent_depth, parent_children + clipped
            # Preserve zero-length marker spans for exact frame navigation.
            inside = not selected_window or selected_window[0] <= first['ts'] < selected_window[1]
            if clipped > 0 or (duration == 0 and inside):
                yield dict(name=first.get('name', '?'), start=low, duration=clipped,
                           self_duration=max(0., clipped-children), thread=key,
                           depth=depth, args=span_fields(first))
    counts['incomplete'] += sum(map(len, stacks.values()))
