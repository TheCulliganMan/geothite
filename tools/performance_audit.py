#!/usr/bin/env python3
"""Run isolated game scenarios and produce an offline CPU/GPU latency audit.

Build the Rust example first; see tools/PERFORMANCE_AUDIT.md. Outputs stay ignored.
"""
import argparse
from collections import defaultdict
import hashlib
import html
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys

from operation_waterfall import decode


def stats(values):
    values = sorted(values)
    if not values:
        return None
    return dict(count=len(values), mean=statistics.mean(values),
                p50=statistics.median(values),
                p95=values[math.ceil(len(values) * .95) - 1],
                p99=values[math.ceil(len(values) * .99) - 1],
                maximum=values[-1])


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            result.update(block)
    return result.hexdigest()


def exclusive_census(spans, start, end):
    """Clip a window; subtract direct nested union, never sum thread wall time."""
    threads = defaultdict(list)
    for span in spans:
        low, high = max(start, span['start']), min(end, span['start'] + span['duration'])
        if high > low:
            threads[span['thread']].append(dict(span, start=low, duration=high-low))
    totals = defaultdict(lambda: [0., 0., 0])
    for thread in threads.values():
        thread.sort(key=lambda span: (span['start'], -span['duration'], span.get('depth', 0)))
        stack = []
        for span in thread:
            while stack and span['start'] >= stack[-1][0]:
                stack.pop()
            # Chrome B/E spans are properly nested per thread. Crossing X spans
            # are inclusive only: do not manufacture exclusive CPU attribution.
            high = span['start'] + span['duration']
            own = span['duration']
            if stack and high <= stack[-1][0]:
                totals[stack[-1][1]][1] -= own
            totals[span['name']][0] += own
            totals[span['name']][1] += own
            totals[span['name']][2] += 1
            stack.append((high, span['name']))
    rows = [dict(name=name, inclusive_ms=total/1000, self_ms=max(0., own)/1000, calls=count)
            for name, (total, own, count) in totals.items()]
    return sorted(rows, key=lambda row: row['self_ms'], reverse=True)


def cpu_census(trace, measured_ids):
    if not trace.exists():
        return [], 'CPU operation tracing disabled'
    source = json.loads(trace.read_text())
    events = source if isinstance(source, list) else source['traceEvents']
    # Bevy Chrome names include fields; marker args are kept separately by
    # tracing_chrome. Both encodings are accepted, with no ordinal guessing.
    markers = []
    for event in events:
        if event.get('ph') not in ('B', 'X') or not event.get('name', '').startswith('audit_frame_marker'):
            continue
        fields = event.get('args', {})
        frame = fields.get('frame_id')
        if frame is None:
            import re
            match = re.search(r'frame_id[=: ]+(\d+)', event.get('name', ''))
            frame = int(match[1]) if match else None
        if frame is not None:
            markers.append((int(frame), event['ts']))
    markers.sort()
    selected = [index for index, (frame, _) in enumerate(markers) if frame in measured_ids]
    if not selected or selected[-1]+1 >= len(markers):
        return [], 'Trace lacks frame-ID markers spanning the measured interval'
    start, end = markers[selected[0]][1], markers[selected[-1]+1][1]
    spans, _, incomplete = decode(events)
    return exclusive_census(spans, start, end), f'{incomplete} incomplete spans excluded; measured frame window only'


def analyze(report, directory):
    frames = [frame for frame in report['frames'] if frame['measured']]
    ids = {frame['id'] for frame in frames}
    gpu = report['gpu']
    samples = [sample for sample in gpu['samples'] if sample['frame'] in ids]
    cpu, trace_note = cpu_census(directory / 'trace.json', ids)
    pass_values = defaultdict(list)
    for diagnostic in report['render_diagnostics']:
        pass_values[diagnostic['path']].append(diagnostic['value'])
    checkpoints = [point for point in report['checkpoints'] if not point['preparation']]
    issues = []
    if not report['success']:
        issues.append(report['failure'])
    if len(frames) < 30:
        issues.append(f'Only {len(frames)} measured frames; need at least 30')
    if gpu['supported'] and (gpu['errors'] or len(samples) / max(1, len(frames)) < .95):
        issues.append('Incomplete GPU capture: errors or less than 95% frame coverage')
    return dict(success=not issues, issues=issues, interval_ms=stats([f['interval_ms'] for f in frames[1:]]),
                main_cpu_ms=stats([f['main_cpu_ms'] for f in frames]),
                gpu_ms=stats([sample['elapsed_ms'] for sample in samples]),
                completion_latency_ms=stats([sample['completion_latency_ms'] for sample in samples]),
                budget_misses=sum(f['interval_ms'] > 1000/60 for f in frames[1:]),
                measured_frames=len(frames), gpu_coverage=len(samples)/max(1,len(frames)),
                gpu_supported=gpu['supported'], gpu_reason=gpu['reason'], gpu_skipped=gpu['skipped'],
                checkpoints=checkpoints, cpu_operations=cpu[:80], trace_note=trace_note,
                passes={name:stats(values) for name,values in sorted(pass_values.items())},
                frames=frames, gpu_samples=samples)


def compatibility(metadata, hardware):
    fields = ('scenario', 'pack_identity', 'os', 'arch', 'debug_assertions', 'operation_trace',
              'gpu_instrumentation', 'viewport', 'hour', 'target', 'pipeline_compilation', 'simulation', 'cache_policy')
    return dict(**{field:metadata[field] for field in fields}, hardware=hardware)


def compare(current, baseline, percent, absolute):
    if not current.get('compatibility') or current.get('compatibility') != baseline.get('compatibility'):
        return dict(comparable=False, reason='Scenario, pack, instrumentation, renderer, hardware or build settings differ', regressions=[])
    if not all(run['success'] for run in current['runs'] + baseline['runs']):
        return dict(comparable=False, reason='A run failed its scenario or measurement checks', regressions=[])
    changes, regressions = [], []
    for metric in ('interval_ms', 'main_cpu_ms', 'gpu_ms'):
        before_values = [run[metric]['p95'] for run in baseline['runs'] if run[metric]]
        after_values = [run[metric]['p95'] for run in current['runs'] if run[metric]]
        if len(before_values) != len(baseline['runs']) or len(after_values) != len(current['runs']):
            continue
        before, after = statistics.median(before_values), statistics.median(after_values)
        delta = after-before
        change = dict(metric=metric, before=before, after=after, delta_ms=delta,
                      delta_percent=100*delta/before if before else None)
        changes.append(change)
        if delta > absolute and delta > before*percent/100:
            regressions.append(change)
    # Transition/input latency is a first-class baseline metric too.
    points = len(current['runs'][0]['checkpoints'])
    if any(len(run['checkpoints']) != points for run in current['runs'] + baseline['runs']):
        return dict(comparable=False, reason='Checkpoint counts differ', regressions=[])
    for index in range(points):
        before = statistics.median(run['checkpoints'][index]['latency_ms'] for run in baseline['runs'])
        after = statistics.median(run['checkpoints'][index]['latency_ms'] for run in current['runs'])
        delta = after-before
        change = dict(metric=f'checkpoint_{index}_ms',before=before,after=after,delta_ms=delta,
                      delta_percent=100*delta/before if before else None)
        changes.append(change)
        if delta > absolute and delta > before*percent/100:
            regressions.append(change)
    return dict(comparable=True, changes=changes, regressions=regressions)


def recommendations(run):
    result = []
    for row in run['cpu_operations'][:8]:
        name = row['name'].lower()
        if any(word in name for word in ('terrain','mesh','grove','region')):
            advice = 'Inspect scene rebuild frequency and dirty revisions; retain geometry and prewarm adjacent regions.'
        elif any(word in name for word in ('snapshot','sync','extract')):
            advice = 'Inspect repeated snapshots, copies and full-scene extraction; cache unchanged data and update dirty entities.'
        elif any(word in name for word in ('pipeline','prepare_assets','shader')):
            advice = 'Inspect first-use pipeline/asset creation; preload at a controlled boundary and reuse GPU assets.'
        elif any(word in name for word in ('poll','wait','acquire','present','render_system')):
            advice = 'Wall time may include GPU/driver synchronization; compare GPU duration and completion latency before changing CPU code.'
        else:
            continue
        result.append(dict(operation=row['name'], self_ms=row['self_ms'], suggestion=advice))
    if run['gpu_ms'] and run['main_cpu_ms'] and run['gpu_ms']['p95'] > run['main_cpu_ms']['p95']:
        result.append(dict(operation='GPU envelope',self_ms=run['gpu_ms']['p95'],suggestion='GPU duration exceeds main-app CPU duration. Inspect pass timings, shadows, overdraw, draw calls and resolution; render-thread CPU work also needs inspection.'))
    if not run['gpu_supported']:
        result.append(dict(operation='GPU unavailable',self_ms=0,suggestion='Use an adapter with timestamp queries; CPU waits and completion callbacks cannot substitute for GPU execution time.'))
    return result


def write_dashboard(output, summary):
    data=json.dumps(summary, separators=(',',':')).replace('<','\\u003c')
    template=Path(__file__).with_name('performance_audit.html').read_text()
    (output/'index.html').write_text(template.replace('__DATA__',data))
    (output/'summary.json').write_text(json.dumps(summary,indent=2))


def run_suite(args):
    binary, pack = args.binary.resolve(strict=True), args.pack.resolve(strict=True)
    output=args.output.resolve()
    if output.exists() and any(output.iterdir()):
        raise ValueError('Output directory must be empty; each benchmark needs a fresh directory')
    output.mkdir(parents=True,exist_ok=True)
    scenarios=json.loads(args.suite.read_text())
    if args.scenario:
        selected=set(args.scenario)
        scenarios=[scenario for scenario in scenarios if scenario['name'] in selected]
        if {scenario['name'] for scenario in scenarios} != selected:
            raise ValueError('Unknown scenario name')
    if not scenarios or len({s['name'] for s in scenarios}) != len(scenarios):
        raise ValueError('Suite must contain unique scenario names')
    hardware=dict(system=platform.platform(),machine=platform.machine(),processor=platform.processor(),cpu_count=os.cpu_count())
    provenance=dict(binary_sha256=digest(binary),pack_sha256=digest(pack),binary=str(binary),pack=str(pack),hardware=hardware)
    try:
        provenance['git_revision']=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
        provenance['git_dirty']=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except subprocess.CalledProcessError:
        pass
    summary=dict(schema=1, provenance=provenance, scenarios=[], thresholds=dict(percent=args.threshold_percent,absolute_ms=args.threshold_ms))
    baseline=json.loads(args.baseline.read_text()) if args.baseline else None
    env={key:value for key,value in os.environ.items() if not key.startswith(('CRYSTAL_', 'TRACE_CHROME','WGPU_','BEVY_'))}
    env.update(CRYSTAL_RENDER_TEST_HEADLESS='1',CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',RUST_LOG='warn')
    # Repeat-major order reduces scenario-order temperature/cache bias.
    for index, scenario in enumerate(scenarios):
        folder=output/f'{index:02d}'
        folder.mkdir()
        (folder/'scenario.json').write_text(json.dumps(scenario,indent=2))
        summary['scenarios'].append(dict(name=scenario['name'],recipe=scenario,runs=[]))
    for repeat in range(args.repeats):
        for index, scenario in enumerate(scenarios):
            folder=output/f'{index:02d}'
            directory=folder/f'run-{repeat+1}'
            directory.mkdir()
            command=[str(binary),str(pack),str(folder/'scenario.json'),str(directory)]
            if args.cpu_only:
                command.append('--cpu-only')
            capture_env=dict(env,TRACE_CHROME=str(directory/'trace.json'))
            print(f"[{repeat+1}/{args.repeats}] {scenario['name']}",flush=True)
            try:
                with (folder/f'run-{repeat+1}.log').open('w') as log:
                    process=subprocess.run(command,env=capture_env,stdout=log,stderr=subprocess.STDOUT,timeout=args.timeout)
                report=json.loads((directory/'audit.json').read_text())
                run=analyze(report,directory)
                if process.returncode:
                    run['success']=False;run['issues'].append(f'Client exited {process.returncode}')
                run['directory']=str(directory.relative_to(output))
                # Optional detailed CPU viewer keeps raw thread lanes/nesting.
                trace=directory/'trace.json'
                if trace.exists():
                    with (folder/f'waterfall-{repeat+1}.log').open('w') as log:
                        subprocess.run([sys.executable,str(Path(__file__).with_name('operation_waterfall.py')),str(trace),'--output',str(directory/'cpu.html')],stdout=log,stderr=subprocess.STDOUT,check=True)
                metadata=json.loads((directory/'metadata.json').read_text())
                if metadata['operation_trace'] and not run['cpu_operations']:
                    run['success']=False;run['issues'].append(run['trace_note'])
                compatible=compatibility(metadata,dict(**hardware,adapter=report['adapter'],features=report['features']))
                compatible['pack_sha256']=provenance['pack_sha256']
                entry=summary['scenarios'][index]
                if 'compatibility' in entry and entry['compatibility'] != compatible:
                    run['success']=False;run['issues'].append('Configuration changed between repeats')
                entry['compatibility']=compatible
                run['recommendations']=recommendations(run)
            except (OSError,ValueError,subprocess.SubprocessError) as error:
                run=dict(success=False,issues=[str(error)],directory=str(directory.relative_to(output)),
                         interval_ms=None,main_cpu_ms=None,gpu_ms=None,checkpoints=[],cpu_operations=[],frames=[],gpu_samples=[],recommendations=[])
            summary['scenarios'][index]['runs'].append(run)
            write_dashboard(output,summary)
    for entry in summary['scenarios']:
        if baseline:
            previous=next((row for row in baseline['scenarios'] if row['name']==entry['name']),None)
            entry['comparison']=compare(entry,previous,args.threshold_percent,args.threshold_ms) if previous and 'compatibility' in entry else dict(comparable=False,reason='No valid matching baseline scenario',regressions=[])
    write_dashboard(output,summary)
    print(output/'index.html')
    if any(not run['success'] for entry in summary['scenarios'] for run in entry['runs']):
        return 1
    if baseline and any(not entry['comparison']['comparable'] or entry['comparison']['regressions'] for entry in summary['scenarios']):
        return 2
    return 0


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--pack',type=Path,default=Path('content-packs/core-modular.crystalpack'))
    parser.add_argument('--suite',type=Path,default=Path(__file__).with_name('performance-scenarios.json'))
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--repeats',type=int,default=3)
    parser.add_argument('--scenario',action='append')
    parser.add_argument('--baseline',type=Path)
    parser.add_argument('--cpu-only',action='store_true')
    parser.add_argument('--timeout',type=float,default=330)
    parser.add_argument('--threshold-percent',type=float,default=10)
    parser.add_argument('--threshold-ms',type=float,default=.2)
    args=parser.parse_args()
    if not 1 <= args.repeats <= 20 or not math.isfinite(args.timeout) or args.timeout<=0 or not math.isfinite(args.threshold_percent) or args.threshold_percent<0 or not math.isfinite(args.threshold_ms) or args.threshold_ms<0:
        parser.error('Invalid repeats, timeout or regression thresholds')
    try:
        return run_suite(args)
    except (OSError,ValueError) as error:
        parser.exit(1,f'{error}\n')


if __name__=='__main__':
    sys.exit(main())
