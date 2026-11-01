#!/usr/bin/env python3
"""Refresh offline audit analysis/UI from retained raw captures, without rerunning gameplay."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path

from performance_audit import ANALYSIS_REVISION, ANALYSIS_SOURCES_SHA256, analyze, compare, digest, recommendations, write_dashboard


def refresh(output, baseline=None):
    summary_path = output/'summary.json'
    summary = json.loads(summary_path.read_text())
    summary['provenance'].setdefault('collection_analysis_sources_sha256',
                                    summary['provenance'].get('analysis_sources_sha256',{}))
    previous = output/'summary-before-reanalysis.json'
    if not previous.exists():
        previous.write_bytes(summary_path.read_bytes())
    for entry in summary['scenarios']:
        for index, old in enumerate(entry['runs']):
            directory = output/old['directory']
            path = directory/'audit.json'
            if not path.exists():
                continue
            report = json.loads(path.read_text())
            metadata = json.loads((directory/'metadata.json').read_text())
            if metadata['gpu_backend'] == 'metal_system_trace' and report['gpu'].get('backend') != 'metal_system_trace':
                continue  # An absent native import must remain a failed capture.
            run = analyze(report, directory, detailed_cpu=metadata['operation_trace'] and metadata['trace_filter'] != 'warn',
                          measurement_revision=metadata.get('measurement_revision', 0))
            # Preserve exit-status, power-source and configuration failures.
            # Only the old GPU-quality diagnosis is replaced by recomputation.
            retained = [issue for issue in old['issues'] if not issue.startswith('Incomplete GPU capture:')]
            run['issues'] = list(dict.fromkeys(run['issues']+retained))
            run['success'] = not run['issues']
            for key in ('directory','load_before','load_after','power_source','host_quiet','pair','pair_position'):
                if key in old:
                    run[key] = old[key]
            run['recommendations'] = recommendations(run)
            entry['runs'][index] = run
        if 'compatibility' in entry:
            entry['compatibility'].setdefault('host_quiet_policy',None)
            entry['compatibility'].setdefault('artwork_sha256',None)
            entry['compatibility']['analysis_revision'] = ANALYSIS_REVISION
            entry['compatibility']['analysis_sources_sha256'] = dict(ANALYSIS_SOURCES_SHA256)
        if baseline:
            before = next((e for e in baseline['scenarios'] if e['name'] == entry['name']), None)
            thresholds = summary['thresholds']
            entry['comparison'] = compare(entry,before,thresholds['percent'],thresholds['absolute_ms']) if before else dict(comparable=False,reason='No matching baseline',regressions=[])
    summary['provenance']['reanalyzed_at_utc'] = datetime.now(timezone.utc).isoformat()
    summary['provenance']['reanalysis_source_sha256'] = digest(Path(__file__))
    write_dashboard(output, summary)
    return summary


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path);parser.add_argument('--baseline',type=Path)
    args=parser.parse_args()
    try:
        summary=refresh(args.output,json.loads(args.baseline.read_text()) if args.baseline else None)
    except (OSError,ValueError,KeyError) as error:
        parser.exit(1,f'{error}\n')
    print(args.output/'index.html')
    if any(not run['success'] for entry in summary['scenarios'] for run in entry['runs']):
        return 1
    if any(not entry['comparison']['comparable'] or entry['comparison']['regressions']
           for entry in summary['scenarios'] if 'comparison' in entry):
        return 2
    return 0


if __name__=='__main__':raise SystemExit(main())
