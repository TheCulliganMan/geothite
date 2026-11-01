#!/usr/bin/env python3
"""Alternate frozen baseline/candidate binaries in close, fresh-process pairs."""
import argparse
from datetime import datetime, timezone
import json
import math
from pathlib import Path
import sys

from performance_audit import compare, continuous_presentation, digest, frame_budget, run_suite, write_dashboard
from performance_audit_pair_effects import export as export_pair_effects

COLLECTOR_SOURCE_SHA256 = digest(Path(__file__))


def collect(args):
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        raise ValueError('Output directory must be empty')
    binaries = dict(before=args.before.resolve(strict=True), after=args.after.resolve(strict=True))
    hashes = {label: digest(binary) for label, binary in binaries.items()}
    suite = json.loads(args.suite.read_text())
    names = [recipe['name'] for recipe in suite if not args.scenario or recipe['name'] in args.scenario]
    if not names or (args.scenario and set(names) != set(args.scenario)):
        raise ValueError('No scenarios selected or an unknown scenario name was requested')
    if len(set(names)) != len(names):
        raise ValueError('Scenario names must be unique')
    output.mkdir(parents=True, exist_ok=True)
    groups = {}
    order = []
    for pair in range(args.pairs):
        labels = ('before', 'after') if pair % 2 == 0 else ('after', 'before')
        for index, name in enumerate(names):
            for label in labels:
                folder = output/'pairs'/f'{pair+1:02d}-{index:02d}-{label}'
                print(f'PAIR {pair+1}/{args.pairs} · {name} · {label}', flush=True)
                request = argparse.Namespace(
                    binary=binaries[label], pack=args.pack, suite=args.suite, output=folder,
                    repeats=1, scenario=[name], baseline=None, cpu_only=args.cpu_only,
                    gpu_backend=args.gpu_backend, timings_only=not args.detailed_cpu,
                    quiet_host=getattr(args,'quiet_host',False),
                    model_root=getattr(args,'model_root',None),
                    timeout=args.timeout, threshold_percent=args.threshold_percent,
                    threshold_ms=args.threshold_ms)
                run_suite(request)
                child = json.loads((folder/'summary.json').read_text())
                if child['provenance']['binary_sha256'] != hashes[label]:
                    raise ValueError(f'{label} executable changed during collection; use a frozen copy')
                if label not in groups:
                    group_output = output/label
                    group_output.mkdir()
                    groups[label] = dict(
                        schema=1, provenance=dict(child['provenance']), scenarios=[],
                        thresholds=child['thresholds'])
                    groups[label]['provenance']['paired_collector_source_sha256'] = COLLECTOR_SOURCE_SHA256
                group = groups[label]
                entry = child['scenarios'][0]
                previous = next((row for row in group['scenarios'] if row['name'] == name), None)
                run = entry['runs'][0]
                run['directory'] = '../'+str((folder/run['directory']).relative_to(output))
                run['pair'] = pair+1
                run['pair_position'] = labels.index(label)+1
                if previous is None:
                    group['scenarios'].append(entry)
                else:
                    if previous.get('compatibility') != entry.get('compatibility'):
                        run['success'] = False
                        run['issues'].append('Configuration changed between paired repeats')
                    previous['runs'].append(run)
                order.append(dict(pair=pair+1, scenario=name, label=label))
                def timing(key):
                    value=run.get(key)
                    return f"{value['p95']:.3f} ms" if value else 'unavailable'
                print(f"Captured {name} / {label}: {'verified' if run['success'] else 'FAILED'} · "
                      f"interval p95 {timing('interval_ms')} · GPU p95 {timing('gpu_ms')}",flush=True)
                for version, summary in groups.items():
                    summary['provenance']['paired_design'] = dict(
                        order=order, collection_started_utc=groups[version]['provenance']['created_at_utc'],
                        note='Each pair is adjacent; order reverses on alternate rounds. Fresh process per capture.')
                    # Each child already persists a complete capture report.
                    # Rewriting the growing GPU history after every child made
                    # long, uncapped cohorts perform quadratic serialization.
                    if (pair+1<args.pairs and index==len(names)-1 and label==labels[-1]):
                        write_dashboard(output/version, summary)
                if (getattr(args, 'require_after_continuity', False) and label == 'after'
                        and name.endswith('-continuous') and run['success']):
                    run['presentation_continuity'] = continuous_presentation(run)
                    if not run['presentation_continuity']['passed']:
                        failure = dict(pair=pair+1, scenario=name, directory=str(folder),
                                       outcome=run['presentation_continuity'],
                                       reason='Candidate renderer failure; remaining timing collection stopped')
                        (output/'continuity-failure.json').write_text(json.dumps(failure, indent=2)+'\n')
                        for version, summary in groups.items():
                            write_dashboard(output/version, summary)
                        print(json.dumps(failure), flush=True)
                        return 3
                if label == 'after' and run['success'] and getattr(args, 'max_frame_ms', None) is not None:
                    run['frame_budget'] = frame_budget(run.get('interval_ms'), args.max_frame_ms)
                    if not run['frame_budget']['passed']:
                        failure = dict(pair=pair+1, scenario=name, directory=str(folder),
                                       outcome=run['frame_budget'],
                                       reason='Candidate exceeded hard frame ceiling; remaining collection stopped')
                        (output/'frame-budget-failure.json').write_text(json.dumps(failure, indent=2)+'\n')
                        for version, summary in groups.items():
                            write_dashboard(output/version, summary)
                        print(json.dumps(failure), flush=True)
                        return 3
    before = {entry['name']: entry for entry in groups['before']['scenarios']}
    for entry in groups['after']['scenarios']:
        previous = before[entry['name']]
        entry['comparison'] = compare(entry, previous, args.threshold_percent, args.threshold_ms)
    for label, summary in groups.items():
        summary['provenance']['paired_collection_finished_utc'] = datetime.now(timezone.utc).isoformat()
        write_dashboard(output/label, summary)
    valid=all(run['success'] for group in groups.values() for entry in group['scenarios'] for run in entry['runs'])
    comparable=all(entry['comparison']['comparable'] for entry in groups['after']['scenarios'])
    if valid and comparable:
        export_pair_effects(groups['after'],groups['before'],output/'paired-effects')
    (output/'index.html').write_text(
        '<!doctype html><meta charset="utf-8"><title>Paired performance audit</title>'
        '<h1>Alternating baseline / candidate audit</h1>'
        '<p>Each pair uses adjacent fresh processes. Order reverses on alternate rounds.</p>'
        '<p><a href="before/index.html">Baseline report</a> · '
        '<a href="after/index.html">Candidate comparison</a></p>'
        + ('<p><a href="paired-effects/effects.csv">Matched-pair effects CSV</a> · '
           '<a href="paired-effects/pairs.csv">Individual pairs CSV</a></p>' if valid and comparable else ''))
    print(output/'after/index.html')
    if not valid:
        return 1
    if any(not entry['comparison']['comparable'] or entry['comparison']['regressions']
           for entry in groups['after']['scenarios']):
        return 2
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--after', type=Path, required=True)
    parser.add_argument('--pack', type=Path, default=Path('content-packs/core-modular.crystalpack'))
    parser.add_argument('--model-root',type=Path,default=Path('assets/open-models'))
    parser.add_argument('--suite', type=Path, default=Path(__file__).with_name('performance-scenarios.json'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--pairs', type=int, default=4)
    parser.add_argument('--scenario', action='append')
    parser.add_argument('--cpu-only', action='store_true')
    parser.add_argument('--quiet-host', action='store_true')
    parser.add_argument('--require-after-continuity', action='store_true',
                        help='Stop with exit 3 if a successful candidate continuous-walking recipe loses 3D or camera continuity')
    parser.add_argument('--gpu-backend', choices=('auto', 'timestamps', 'metal'), default='auto')
    parser.add_argument('--detailed-cpu', action='store_true')
    parser.add_argument('--timeout', type=float, default=330)
    parser.add_argument('--threshold-percent', type=float, default=10)
    parser.add_argument('--threshold-ms', type=float, default=.2)
    parser.add_argument('--max-frame-ms', type=float, default=45.,
                        help='Hard candidate peak interval ceiling, in milliseconds (default: 45)')
    args = parser.parse_args()
    if args.pairs < 1 or not math.isfinite(args.timeout) or args.timeout <= 0 or any(
            not math.isfinite(value) or value < 0 for value in (args.threshold_percent, args.threshold_ms)) or (
            not math.isfinite(args.max_frame_ms) or args.max_frame_ms <= 0):
        parser.error('Invalid pairs, timeout or regression thresholds')
    try:
        return collect(args)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    sys.exit(main())
