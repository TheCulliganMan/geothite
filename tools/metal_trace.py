"""Join native Metal GPU work to audit frames by CPU command submission order.

GPU point markers can run concurrently with graphics. Their execution times
must never define the frame window. Only their command labels identify the
ordered submission group; the actual GPU work supplies every measured time.
"""
from collections import defaultdict
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET


BOUNDARY = re.compile(r'^geothite_audit_frame_(\d+)_(begin|end)$')


def trace_rows(path):
    """Read xctrace's interned XML values; retain only normalized row fields."""
    references = {}
    fields = None
    for event, element in ET.iterparse(path, events=('end',)):
        if element.tag == 'schema':
            fields = [column.findtext('mnemonic') for column in element.findall('col')]
        elif element.tag == 'row':
            if not fields or len(element) != len(fields):
                raise ValueError('Metal trace row does not match its schema')

            def decode(node):
                if node.get('ref'):
                    try:
                        return references[node.get('ref')]
                    except KeyError as error:
                        raise ValueError('Unresolved Metal trace XML reference') from error
                children = [decode(child) for child in node]
                value = dict(text=node.text or '', formatted=node.get('fmt', ''),
                             kind=node.tag, pid=None, label=None)
                if node.tag == 'pid':
                    value['pid'] = int(value['text'])
                else:
                    value['pid'] = next((child['pid'] for child in children
                                         if child['pid'] is not None), None)
                if node.tag == 'metal-object-label':
                    value['label'] = value['text'] or value['formatted']
                else:
                    value['label'] = next((child['label'] for child in children
                                           if child['label'] is not None), None)
                if node.get('id'):
                    references[node.get('id')] = value
                return value

            yield dict(zip(fields, (decode(child) for child in element)))
            element.clear()


def number(row, name):
    try:
        return int(row[name]['text'])
    except (KeyError, ValueError) as error:
        raise ValueError(f'Metal trace lacks numeric {name}') from error


def union_ns(intervals):
    """GPU engines overlap; count their union rather than summing stage times."""
    total, low, high = 0, None, None
    for start, end in sorted(intervals):
        if low is None:
            low, high = start, end
        elif start <= high:
            high = max(high, end)
        else:
            total += high-low
            low, high = start, end
    return total + (high-low if low is not None else 0)


def decode_gpu(submissions, activity, process_id):
    """Retain the target PID only and correlate work by command-buffer identity."""
    commands, marker_commands, complete, errors, invalid_frames = {}, {}, set(), [], set()

    def fail(frame, reason):
        invalid_frames.add(frame)
        errors.append(reason)
    commits = []
    for row in trace_rows(submissions):
        if row.get('process', {}).get('pid') != process_id:
            continue
        label = row.get('event-label', {}).get('label') or ''
        # Metal assigns this ordinal at commit. Encoding can start out of order
        # on worker threads, so neither start nor start+duration orders commits.
        commits.append((number(row, 'frame-number'), number(row, 'cmdbuffer-id'), label))
    opened = None
    for _, command, label in sorted(commits, key=lambda commit: commit[0]):
        marker = BOUNDARY.fullmatch(label)
        if marker:
            frame = int(marker[1])
            marker_commands[command] = frame
            if marker[2] == 'begin':
                if opened is not None:
                    fail(opened, f'Unclosed source frame {opened} before {frame}')
                if frame in complete:
                    fail(frame, f'Duplicate source frame {frame}')
                opened = frame
            else:
                if opened != frame:
                    fail(frame, f'Unpaired source-frame end {frame}')
                    if opened is not None:
                        fail(opened, f'Unclosed source frame {opened}')
                else:
                    complete.add(frame)
                opened = None
        elif opened is not None:
            if command in commands:
                fail(opened, f'Duplicate command-buffer identity {command}')
            commands[command] = opened
    if opened is not None:
        fail(opened, f'Unclosed source frame {opened}')
    if not complete:
        raise ValueError('Metal trace has no complete source-frame submission groups')
    passes, marker_intervals = [], defaultdict(list)
    for row in trace_rows(activity):
        if row.get('process', {}).get('pid') != process_id:
            continue
        command = number(row, 'cmdbuffer-id')
        frame = commands.get(command, marker_commands.get(command))
        if frame is None:
            continue
        state = row.get('state', {}).get('text') or row.get('state', {}).get('formatted')
        if state != 'Active':
            continue
        start, duration = number(row, 'start'), number(row, 'duration')
        if start < 0 or duration <= 0:
            fail(frame, f'Invalid GPU interval for frame {frame}')
            continue
        end = start+duration
        if command in marker_commands:
            marker_intervals[frame].append((start, end))
            continue
        if frame not in complete:
            continue
        label = row.get('event-label', {}).get('formatted', '')
        # The suffix is trace UI provenance, not an operation name.
        label = re.sub(r'\s+\([^()]*\(\d+\)\)\s+0x[0-9a-f]+$', '', label)
        names = label.split(':', 1)
        if len(names) == 2 and names[0] == names[1]:
            label = names[0]
        passes.append(dict(frame=frame, start_ns=start, end_ns=end,
                           elapsed_ms=duration/1e6, label=label,
                           stage=row.get('channel-name', {}).get('text', ''),
                           command_buffer=command, encoder=number(row, 'encoder-id'),
                           depth=number(row, 'event-depth')))
    complete -= invalid_frames
    passes = [sample for sample in passes if sample['frame'] in complete]
    grouped = defaultdict(list)
    for sample in passes:
        grouped[sample['frame']].append(sample)
    samples = []
    for frame, intervals in sorted(grouped.items()):
        start = min(sample['start_ns'] for sample in intervals)
        end = max(sample['end_ns'] for sample in intervals)
        samples.append(dict(frame=frame, start_ns=start, end_ns=end, elapsed_ms=(end-start)/1e6,
                            active_ms=union_ns((s['start_ns'],s['end_ns']) for s in intervals)/1e6,
                            completion_latency_ms=None,
                            marker_gpu_ms=union_ns(marker_intervals[frame])/1e6 if marker_intervals[frame] else None))
    return dict(supported=True, reason='Native Metal render-graph activity; command-ID and commit-ordinal correlation',
                backend='metal_system_trace', errors=len(errors), error_details=errors[:20],
                invalid_frames=sorted(invalid_frames),
                skipped=len(complete-grouped.keys()), samples=samples, passes=passes)


def export_gpu(trace, directory, process_id):
    """Export completed native captures and reject absent or incompatible data."""
    schemas = ('metal-application-command-buffer-submissions', 'metal-gpu-intervals')
    files = []
    for index, schema in enumerate(schemas):
        output = directory/f'metal-{index}.xml'
        result = subprocess.run(['xcrun', 'xctrace', 'export', '--input', str(trace),
                                 '--xpath', f'/trace-toc/run[@number="1"]/data/table[@schema="{schema}"]',
                                 '--output', str(output)], text=True, capture_output=True, timeout=120)
        if result.returncode or not output.exists():
            raise ValueError(f'Metal export failed: {result.stderr or result.stdout}')
        files.append(output)
    try:
        result = decode_gpu(*files, process_id)
    except ET.ParseError as error:
        raise ValueError(f'Invalid native Metal XML: {error}') from error
    # Native traces remain reviewable; temporary exports contain global GPU
    # metadata. Reports retain only the launched process's correlated work.
    for file in files:
        file.unlink()
    return result
