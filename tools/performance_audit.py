#!/usr/bin/env python3
"""Run isolated game scenarios and produce an offline CPU/GPU latency audit.

Build the Rust example first; see tools/PERFORMANCE_AUDIT.md. Outputs stay ignored.
"""
import argparse
from datetime import datetime, timezone
from collections import defaultdict
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import statistics
import subprocess
import sys

from trace_stream import events, spans, window
from metal_trace import export_gpu, union_ns
from metal_capture import record as record_metal
from host_quiet import CaptureHost
from performance_audit_residency import summarize as summarize_residency
from performance_audit_motion import motion_series, stationary_camera_drift

ANALYSIS_REVISION = 7


def frame_budget(intervals, maximum_ms=45.):
    """A hard frame ceiling uses the worst observed interval, never a percentile."""
    if not math.isfinite(maximum_ms) or maximum_ms <= 0:
        raise ValueError('Frame ceiling must be finite and positive')
    observed = (intervals or {}).get('maximum')
    valid = isinstance(observed, (int, float)) and math.isfinite(observed) and observed > 0
    return dict(passed=valid and observed <= maximum_ms, limit_ms=maximum_ms,
                maximum_ms=observed if valid else None,
                reason=('Frame observations unavailable or invalid' if not valid else
                        f'Peak {observed:.3f} ms exceeds {maximum_ms:g} ms' if observed > maximum_ms else None))


def continuous_presentation(run):
    """Separate renderer correctness from recorder quality and gameplay success."""
    residency = run.get('world_residency', {})
    camera = run.get('stationary_camera') or {}
    reasons = []
    if not residency.get('available'):
        reasons.append('Renderer residency observations unavailable')
    else:
        if residency.get('unavailable_frames', 0):
            reasons.append(f"3D unavailable for {residency['unavailable_ms']:.3f} ms")
        if residency.get('unknown_camera_frames', 0) or residency.get('unknown_intervals', 0):
            reasons.append('Incomplete camera or interval observations')
    if not camera.get('observed_pairs'):
        reasons.append('No fixed-player camera observations')
    elif any(not isinstance(camera.get(field), (int, float))
             or not math.isfinite(camera[field]) or camera[field] > .0001
             for field in ['max_vertical_cells', 'max_horizontal_cells']):
        reasons.append('Camera changed while the player remained fixed')
    budget = frame_budget(run.get('interval_ms'))
    if not budget['passed']:
        reasons.append(budget['reason'])
    return dict(passed=not reasons, reasons=reasons)


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


def digest_tree(directory):
    """Identify both the names and contents of the external artwork files."""
    result=hashlib.sha256()
    paths=sorted(path for path in directory.rglob('*') if path.is_file())
    if not paths:
        raise ValueError(f'Artwork directory is empty: {directory}')
    for path in paths:
        name=path.relative_to(directory).as_posix().encode()
        result.update(len(name).to_bytes(8,'big'))
        result.update(name)
        result.update(bytes.fromhex(digest(path)))
    return result.hexdigest()


ANALYSIS_SOURCES_SHA256 = {name:digest(Path(__file__).with_name(name))
    for name in ('performance_audit.py','metal_trace.py','trace_stream.py','metal_capture.py','instruments_scratch.py','host_quiet.py','performance_audit_residency.py','performance_audit_motion.py')}


def power_source():
    """Record macOS AC/battery source without making charge level a baseline key."""
    if sys.platform != 'darwin':
        return None
    try:
        result = subprocess.run(['pmset', '-g', 'batt'], text=True, capture_output=True,
                                timeout=3, check=True)
        return result.stdout.splitlines()[0]
    except (OSError, subprocess.SubprocessError, IndexError):
        return None


def cpu_census(trace, measured_ids):
    if not trace.exists():
        return [], 'CPU operation tracing disabled'
    selected = window(trace, measured_ids)
    grouped = defaultdict(lambda: [0., 0., 0])
    counts = {}
    for span in spans(events(trace), {}, counts, selected):
        values = grouped[span['name']]
        values[0] += span['duration']
        values[1] += span['self_duration']
        values[2] += 1
    rows = [dict(name=name,inclusive_ms=total/1000,self_ms=own/1000,calls=count)
            for name,(total,own,count) in grouped.items()]
    rows.sort(key=lambda row:row['self_ms'],reverse=True)
    return rows, f"{counts['incomplete']} incomplete spans excluded; measured frame window only"


def pack_gpu_passes(passes):
    """Columnar report payload; complete command/encoder evidence stays in audit.json."""
    origin = min((sample['start_ns'] for sample in passes), default=0)
    labels, stages, rows = {}, {}, []
    for sample in passes:
        label = labels.setdefault(sample['label'], len(labels))
        stage = stages.setdefault(sample['stage'], len(stages))
        rows.append([sample['frame'], sample['start_ns']-origin,
                     sample['end_ns']-origin, label, stage])
    return dict(origin_ns=origin, labels=list(labels), stages=list(stages), rows=rows)


def analyze(report, directory, detailed_cpu=True, measurement_revision=0):
    if report.get('schema') != 1:
        raise ValueError('Unsupported capture schema')
    frames = [frame for frame in report['frames'] if frame['measured']]
    ids = {frame['id'] for frame in frames}
    gpu = report['gpu']
    excluded = set(gpu.get('invalid_frames', []))
    samples = [sample for sample in gpu['samples'] if sample['frame'] not in excluded and sample['frame'] in ids and sample['elapsed_ms'] > 0 and sample['start_ns'] > 0 and sample['end_ns'] > sample['start_ns']]
    covered_ids = {sample['frame'] for sample in samples}
    duplicate_samples = len(samples) != len(covered_ids)
    invalid_frames = ids.intersection(gpu.get('invalid_frames', []))
    # A missing native boundary discards the entire group in the decoder. It
    # is a skipped observation, like a full asynchronous query ring, rather
    # than a bad timestamp admitted to the distribution. Ambiguous identities
    # and malformed GPU intervals remain errors. Require every diagnostic to
    # be present before classifying a truncated diagnostic list this way.
    details = gpu.get('error_details', [])
    boundary_drops = (gpu.get('backend') == 'metal_system_trace' and gpu['errors'] > 0
                      and gpu['errors'] == len(details)
                      and all(re.fullmatch(r'Unclosed source frame \d+(?: before \d+)?|Unpaired source-frame end \d+', detail)
                              for detail in details))
    measured_errors = (0 if boundary_drops else len(invalid_frames)) if 'invalid_frames' in gpu else gpu['errors']
    cpu, trace_note = cpu_census(directory / 'trace.json', ids) if detailed_cpu else ([], 'Detailed CPU tracing filtered for timings-only capture')
    pass_values = defaultdict(list)
    for diagnostic in report['render_diagnostics']:
        pass_values[diagnostic['path']].append(diagnostic['value'])
    gpu_passes = [sample for sample in gpu.get('passes', []) if sample['frame'] in ids]
    by_encoder = defaultdict(list)
    for sample in gpu_passes:
        key = (sample['frame'], sample['command_buffer'], sample['encoder'], sample['label'], sample['stage'])
        by_encoder[key].append((sample['start_ns'], sample['end_ns']))
    gpu_operations = defaultdict(list)
    for (_, _, _, label, stage), intervals in by_encoder.items():
        gpu_operations[(label, stage)].append(union_ns(intervals)/1e6)
    gpu_operations = [dict(label=label, stage=stage, duration_ms=stats(values))
                      for (label, stage), values in gpu_operations.items()]
    gpu_operations.sort(key=lambda operation: operation['duration_ms']['p95'], reverse=True)
    checkpoints = [dict(point) for point in report['checkpoints'] if not point['preparation']]
    for point in checkpoints:
        steps = report.get('scenario', {}).get('steps', [])
        expected = steps[point['step']].get('until', {}) if point['step'] < len(steps) else {}
        point['movement_leg'] = (measurement_revision >= 8
                                 and report.get('scenario', {}).get('continuous_movement', False)
                                 and point.get('renderer_waited') is False)
        point['duration_only'] = point['movement_leg'] or (
            bool(expected.get('seconds')) and not expected.get('pp_consumed') and all(
                expected.get(field) is None for field in ('map','tile','mode','battle','battle_commands')))
    issues = []
    if report.get('motion_capture', {}).get('enabled'):
        issues.append('Motion screenshot readbacks perturb timing; visual QA capture is not a benchmark')
    if not report['success']:
        issues.append(report['failure'])
    if len(frames) < 30:
        issues.append(f'Only {len(frames)} measured frames; need at least 30')
    if checkpoints and measurement_revision < 7:
        issues.append('Legacy checkpoint readiness is unverified; recapture with measurement revision 7 or newer')
    if measurement_revision >= 8 and any(point.get('renderer_waited') != (
            not report.get('scenario', {}).get('continuous_movement', False)) for point in checkpoints):
        issues.append('Checkpoint completion policy does not match the recipe')
    if measurement_revision >= 7 and any(point.get('render_settle_ms', 0) <= 0
                                          and not point['movement_leg'] for point in checkpoints):
        issues.append('Checkpoint accepted without a post-result GPU verification interval')
    if any(step.get('until', {}).get('pp_consumed') for step in report.get('scenario', {}).get('steps', [])):
        battle_frames = [frame for frame in frames if frame.get('battle')]
        if not battle_frames or not any(frame.get('dialogue')
                                        and frame.get('pp', 0) < battle_frames[0].get('pp', 0)
                                        for frame in battle_frames):
            issues.append('Battle move did not consume active-battler PP during measurement')
    if gpu['supported'] and (measured_errors or duplicate_samples or len(covered_ids) / max(1, len(frames)) < .99):
        issues.append('Incomplete GPU capture: errors or less than 99% frame coverage')
    intervals = stats([f['interval_ms'] for f in frames])
    return dict(success=not issues, issues=issues, interval_ms=intervals,
                frame_budget=frame_budget(intervals),
                frame_ceiling_exceedances=sum(f['interval_ms'] > 45. for f in frames),
                main_cpu_ms=stats([f['main_cpu_ms'] for f in frames]),
                world_residency=summarize_residency(report),
                stationary_camera=(stationary_camera_drift(motion_series(report))
                    if report['success'] and frames and all(frame.get('motion') for frame in report['frames']) else None),
                gpu_ms=stats([sample['elapsed_ms'] for sample in samples]),
                completion_latency_ms=stats([sample['completion_latency_ms'] for sample in samples if sample.get('completion_latency_ms') is not None]),
                marker_gpu_ms=stats([sample['marker_gpu_ms'] for sample in samples if sample.get('marker_gpu_ms') is not None]),
                gpu_active_ms=stats([sample['active_ms'] for sample in samples if 'active_ms' in sample]),
                budget_misses=sum(f['interval_ms'] > 1000/60 for f in frames),
                measured_frames=len(frames), gpu_coverage=len(covered_ids)/max(1,len(frames)),
                gpu_supported=gpu['supported'], gpu_reason=gpu['reason'], gpu_skipped=gpu['skipped'],
                gpu_errors=measured_errors, gpu_capture_errors=gpu['errors'],
                gpu_dropped_frames=len(invalid_frames) if boundary_drops else 0,
                checkpoints=checkpoints, cpu_operations=cpu[:80], trace_note=trace_note,
                passes={name:stats(values) for name,values in sorted(pass_values.items())},
                frames=frames, gpu_samples=samples, gpu_waterfall=pack_gpu_passes(gpu_passes), gpu_operations=gpu_operations,
                gpu_backend=gpu.get('backend', 'timestamp_query'))


def compatibility(metadata, hardware):
    fields = ('measurement_revision', 'warmup_cadence', 'native_capture_mode', 'trace_filter', 'gpu_backend', 'scenario', 'pack_identity', 'os', 'arch', 'debug_assertions', 'operation_trace',
              'gpu_instrumentation', 'viewport', 'hour', 'target', 'pipeline_compilation', 'simulation', 'cache_policy')
    return dict(**{field:metadata[field] for field in fields}, hardware=hardware,
                analysis_revision=ANALYSIS_REVISION, analysis_sources_sha256=dict(ANALYSIS_SOURCES_SHA256))


def compare(current, baseline, percent, absolute):
    if not current.get('compatibility') or current.get('compatibility') != baseline.get('compatibility'):
        return dict(comparable=False, reason='Scenario, pack, instrumentation, renderer, hardware or build settings differ', regressions=[])
    if not current['runs'] or not baseline['runs']:
        return dict(comparable=False, reason='A scenario has no completed runs', regressions=[])
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
    residency = run.get('world_residency', {})
    if residency.get('unavailable_frames', 0):
        result.append(dict(operation='World residency', self_ms=residency['unavailable_ms'],
            suggestion='3D was unavailable during measured frames. Inspect whole-journey residency before interpreting faster frame intervals as a streaming improvement; authored battle/warp presentation needs separate review.'))
    for row in run['cpu_operations']:
        name = row['name'].lower()
        if any(word in name for word in ('terrain','mesh','grove','region')):
            advice = 'Inspect scene rebuild frequency and dirty revisions; retain geometry and prewarm adjacent regions.'
        elif any(word in name for word in ('snapshot','sync','extract')):
            advice = 'Inspect repeated snapshots, copies and full-scene extraction; cache unchanged data and update dirty entities.'
        elif any(word in name for word in ('pipeline','prepare_assets','shader')):
            advice = 'Inspect first-use pipeline/asset creation; preload at a controlled boundary and reuse GPU assets.'
        elif any(word in name for word in ('poll','wait','acquire','present','render_system','queue_submit')):
            advice = 'Wall time may include GPU/driver synchronization; compare GPU duration and completion latency before changing CPU code.'
        else:
            continue
        if any(item['suggestion']==advice for item in result):
            continue
        result.append(dict(operation=row['name'], self_ms=row['self_ms'], suggestion=advice))
        if len(result)>=5:
            break
    if run['gpu_ms'] and run['main_cpu_ms'] and run['gpu_ms']['p95'] > run['main_cpu_ms']['p95']:
        result.append(dict(operation='GPU envelope',self_ms=run['gpu_ms']['p95'],suggestion='GPU duration exceeds main-app CPU duration. Inspect pass timings, shadows, overdraw, draw calls and resolution; render-thread CPU work also needs inspection.'))
    if not run['gpu_supported']:
        result.append(dict(operation='GPU unavailable',self_ms=0,suggestion='Collect native Metal activity on macOS or use an adapter with timestamp queries elsewhere. CPU waits and completion callbacks cannot substitute for GPU execution time.'))
    return result


def repeat_stability(entry):
    """Describe repeat spread without treating a small sample as a confidence interval."""
    result = {}
    valid = [run for run in entry['runs'] if run['success']]
    for metric in ('interval_ms', 'main_cpu_ms', 'gpu_ms'):
        values = [run[metric]['p95'] for run in valid if run.get(metric)]
        if len(values) < 2:
            continue
        median = statistics.median(values)
        spread = 100*(max(values)-min(values))/median if median else 0
        result[metric] = dict(repeats=len(values), minimum=min(values), maximum=max(values),
                              median=median, spread_percent=spread, noisy=spread>20)
    return result


def write_dashboard(output, summary):
    summary['analysis_revision'] = ANALYSIS_REVISION
    for entry in summary['scenarios']:
        entry['repeat_stability'] = repeat_stability(entry)
    summary['provenance']['analysis_sources_sha256'] = dict(ANALYSIS_SOURCES_SHA256)
    template=Path(__file__).with_name('performance_audit.html').read_text()
    summary['provenance']['dashboard_source_sha256'] = hashlib.sha256(template.encode()).hexdigest()
    data=json.dumps(summary, separators=(',',':')).replace('<','\\u003c')
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
    hardware=dict(system=platform.platform(),node=platform.node(),machine=platform.machine(),processor=platform.processor(),cpu_count=os.cpu_count())
    provenance=dict(created_at_utc=datetime.now(timezone.utc).isoformat(),binary_sha256=digest(binary),pack_sha256=digest(pack),binary=str(binary),pack=str(pack),hardware=hardware)
    model_root=getattr(args,'model_root',None)
    model_root=model_root.resolve(strict=True) if model_root is not None else None
    provenance['artwork_root']=str(model_root) if model_root else None
    provenance['artwork_sha256']=digest_tree(model_root) if model_root else None
    try:
        provenance['git_revision']=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
        provenance['git_dirty']=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except subprocess.CalledProcessError:
        pass
    summary=dict(schema=1, provenance=provenance, scenarios=[], thresholds=dict(percent=args.threshold_percent,absolute_ms=args.threshold_ms))
    baseline=json.loads(args.baseline.read_text()) if args.baseline else None
    if baseline and (baseline.get('schema') != 1 or not isinstance(baseline.get('scenarios'),list)):
        raise ValueError('Unsupported baseline summary')
    env={key:value for key,value in os.environ.items() if not key.startswith(('CRYSTAL_', 'TRACE_CHROME','WGPU_','BEVY_'))}
    timings_only=getattr(args,'timings_only',False)
    backend = getattr(args, 'gpu_backend', 'timestamps')
    metal = not args.cpu_only and (backend == 'metal' or (backend == 'auto' and sys.platform == 'darwin'))
    profiler_version = None
    if metal:
        profiler_version = subprocess.check_output(['xcrun', 'xctrace', 'version'], text=True).strip()
        provenance['native_profiler_version'] = profiler_version
    env.update(CRYSTAL_RENDER_TEST_HEADLESS='1',CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',RUST_LOG='warn' if timings_only else 'info')
    if model_root:
        env['CRYSTAL_OPEN_MODEL_ROOT']=str(model_root)
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
            elif metal:
                command.append('--metal-trace')
            capture_env=dict(env,TRACE_CHROME=str(directory/'trace.json'))
            native_trace=folder/f'metal-{repeat+1}.trace'
            client_log=folder/f'client-{repeat+1}.log'
            print(f"[{repeat+1}/{args.repeats}] {scenario['name']}",flush=True)
            host=CaptureHost(enabled=getattr(args,'quiet_host',False))
            try:
                host.wait()
                load_before=list(os.getloadavg()) if hasattr(os,"getloadavg") else None
                power_before = power_source()
                with host, (folder/f'run-{repeat+1}.log').open('w') as log:
                    if metal:
                        process=record_metal(command,capture_env,directory,native_trace,client_log,log,
                                             folder/f'run-{repeat+1}.log',args.timeout,own_pid=host.own_pid,
                                             cpu_sampling=getattr(args, 'cpu_sampling', False))
                    else:
                        with subprocess.Popen(command,env=capture_env,stdout=log,stderr=subprocess.STDOUT) as child:
                            host.own_pid(child.pid)
                            try:
                                child.wait(timeout=args.timeout)
                            except subprocess.TimeoutExpired:
                                child.kill()
                                child.wait()
                                raise
                            process=subprocess.CompletedProcess(command,child.returncode)
                load_after=list(os.getloadavg()) if hasattr(os,'getloadavg') else None
                report=json.loads((directory/'audit.json').read_text())
                metadata=json.loads((directory/'metadata.json').read_text())
                if metal:
                    (directory/'audit-rust.json').write_text(json.dumps(report))
                    report['gpu'].update(export_gpu(native_trace, directory, metadata['process_id']))
                    report['gpu']['native_trace']=str(native_trace.relative_to(output))
                    (directory/'audit.json').write_text(json.dumps(report))
                run=analyze(report,directory,detailed_cpu=not timings_only,
                            measurement_revision=metadata['measurement_revision'])
                if process.returncode:
                    run['success']=False;run['issues'].append(f'Client exited {process.returncode}')
                run['directory']=str(directory.relative_to(output))
                run['load_before']=load_before;run['load_after']=load_after
                run['power_source'] = power_before
                run['host_quiet']=host.report()
                if host.events:
                    run['success']=False;run['issues'].append('Host contention or monitoring failure during capture; see host_quiet')
                for label,path in (('binary',binary),('pack',pack)):
                    if digest(path)!=provenance[f'{label}_sha256']:
                        run['success']=False;run['issues'].append(f'{label.capitalize()} changed during collection')
                if model_root and digest_tree(model_root)!=provenance['artwork_sha256']:
                    run['success']=False;run['issues'].append('Artwork changed during collection')
                if power_source() != power_before:
                    run['success']=False;run['issues'].append('Power source changed during capture')
                # Optional detailed CPU viewer keeps raw thread lanes/nesting.
                trace=directory/'trace.json'
                if trace.exists() and not timings_only:
                    with (folder/f'waterfall-{repeat+1}.log').open('w') as log:
                        subprocess.run([sys.executable,str(Path(__file__).with_name('operation_waterfall.py')),str(trace),'--output',str(directory/'cpu.html')],stdout=log,stderr=subprocess.STDOUT,check=True)
                if metadata['operation_trace'] and not timings_only and not run['cpu_operations']:
                    run['success']=False;run['issues'].append(run['trace_note'])
                compatible=compatibility(metadata,dict(**hardware,adapter=report['adapter'],features=report['features'],power_source=power_before,profiler_version=profiler_version,
                    profiler_configuration=('Blank: Metal Application + GPU'
                        + (' + Time Profiler' if getattr(args, 'cpu_sampling', False) else '')
                        + '; attach after validated warmup') if metal else None))
                compatible['pack_sha256']=provenance['pack_sha256']
                compatible['host_quiet_policy']=host.policy
                compatible['artwork_sha256']=provenance['artwork_sha256']
                import struct
                image=directory/'warmup.png'
                if image.exists():
                    png=image.read_bytes()
                    if len(png)<24 or png[:8] != b'\x89PNG\r\n\x1a\n':
                        raise ValueError('Invalid warmup PNG receipt')
                    compatible['actual_viewport']=list(struct.unpack('>II',png[16:24]))
                elif report['success']:
                    raise ValueError('Successful capture lacks its warmup PNG receipt')
                else:
                    compatible['actual_viewport']=None
                entry=summary['scenarios'][index]
                if 'compatibility' in entry and entry['compatibility'] != compatible:
                    run['success']=False;run['issues'].append('Configuration changed between repeats')
                entry['compatibility']=compatible
                run['recommendations']=recommendations(run)
            except (OSError,ValueError,KeyError,TypeError,subprocess.SubprocessError) as error:
                detail = ''
                if metal and client_log.exists():
                    lines = client_log.read_text().splitlines()
                    failures = [line for line in lines if any(word in line for word in (' ERROR ', 'Error:', 'panicked'))]
                    detail = '\n'.join(failures[-8:] or lines[-8:])[-2400:]
                run=dict(success=False,issues=[f'{error}; see {index:02d}/run-{repeat+1}.log'+(f'; client: {detail}' if detail else '')],directory=str(directory.relative_to(output)),
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
    if any(not run['frame_budget']['passed'] for entry in summary['scenarios'] for run in entry['runs']):
        return 3
    if baseline and any(not entry['comparison']['comparable'] or entry['comparison']['regressions'] for entry in summary['scenarios']):
        return 2
    return 0


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--pack',type=Path,default=Path('content-packs/core-modular.crystalpack'))
    parser.add_argument('--model-root',type=Path,default=Path('assets/open-models'),help='External model artwork; its complete tree hash is part of baseline compatibility')
    parser.add_argument('--suite',type=Path,default=Path(__file__).with_name('performance-scenarios.json'))
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--repeats',type=int,default=3)
    parser.add_argument('--scenario',action='append')
    parser.add_argument('--baseline',type=Path)
    parser.add_argument('--cpu-only',action='store_true')
    parser.add_argument('--quiet-host',action='store_true',help='Wait for quiet macOS GPU/CPU and flag competing compiler/render jobs during capture')
    parser.add_argument('--gpu-backend',choices=('auto','timestamps','metal'),default='auto',help='Native Metal System Trace on macOS; timestamp queries elsewhere')
    parser.add_argument('--timings-only',action='store_true',help='Filter detailed CPU tracing in the same executable; retain frame, GPU and checkpoint timings')
    parser.add_argument('--cpu-sampling', action='store_true',
                        help='Add native Time Profiler call stacks to a Metal diagnostic capture')
    parser.add_argument('--timeout',type=float,default=330)
    parser.add_argument('--threshold-percent',type=float,default=10)
    parser.add_argument('--threshold-ms',type=float,default=.2)
    args=parser.parse_args()
    if args.cpu_sampling and (args.cpu_only or args.gpu_backend == 'timestamps' or sys.platform != 'darwin'):
        parser.error('--cpu-sampling requires a native Metal capture on macOS')
    if not 1 <= args.repeats <= 20 or not math.isfinite(args.timeout) or args.timeout<=0 or not math.isfinite(args.threshold_percent) or args.threshold_percent<0 or not math.isfinite(args.threshold_ms) or args.threshold_ms<0:
        parser.error('Invalid repeats, timeout or regression thresholds')
    try:
        return run_suite(args)
    except (OSError,ValueError) as error:
        parser.exit(1,f'{error}\n')


if __name__=='__main__':
    sys.exit(main())
