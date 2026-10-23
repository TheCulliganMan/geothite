#!/usr/bin/env python3
"""Export per-scene costs so changing transition populations remain visible."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import statistics

from performance_audit import ANALYSIS_SOURCES_SHA256, compare, stats

METRICS=('interval_ms','main_cpu_ms','gpu_ms','gpu_active_ms')


def scenes(run):
    """Group source-frame observations; readiness does not mean idle gameplay."""
    gpu={sample['frame']:sample for sample in run['gpu_samples']}
    grouped={}
    for frame in run['frames']:
        offset=frame.get('natural_offset')
        off_center=any(component!=0 for component in offset) if offset is not None else None
        key=(frame['map'],frame['mode'],bool(frame['battle']),bool(frame['visible_ready']),off_center)
        values=grouped.setdefault(key,{metric:[] for metric in METRICS})
        values['interval_ms'].append(frame['interval_ms'])
        values['main_cpu_ms'].append(frame['main_cpu_ms'])
        sample=gpu.get(frame['id'])
        if sample is not None:
            values['gpu_ms'].append(sample['elapsed_ms'])
            if sample.get('active_ms') is not None:
                values['gpu_active_ms'].append(sample['active_ms'])
    return {key:{metric:stats(values) for metric,values in group.items()}
            for key,group in grouped.items()}


def export(summary, output, baseline=None):
    previous={entry['name']:entry for entry in baseline['scenarios']} if baseline else {}
    rows=[]
    for entry in summary['scenarios']:
        if not entry['runs'] or not all(run['success'] for run in entry['runs']):
            raise ValueError(f"Failed or empty scenario: {entry['name']}")
        before=previous.get(entry['name'])
        if baseline and (before is None or not compare(entry,before,10,.2)['comparable']):
            raise ValueError(f"Incompatible baseline: {entry['name']}")
        groups=dict(after=[scenes(run) for run in entry['runs']],
                    before=[scenes(run) for run in before['runs']] if before else [])
        keys=sorted({key for runs in groups.values() for run in runs for key in run},
                    key=lambda key:(*key[:4],-1 if key[4] is None else int(key[4])))
        for key in keys:
            for metric in METRICS:
                row=dict(scenario=entry['name'],map=key[0],mode=key[1],battle=key[2],
                         renderer_ready=key[3],natural_offset_nonzero=key[4],metric=metric)
                complete=True
                for label,runs in groups.items():
                    values=[run[key][metric] for run in runs if key in run and run[key][metric]]
                    counts=[value['count'] for value in values]
                    row[label+'_repeats']=len(values)
                    row[label+'_expected_repeats']=len(runs)
                    row[label+'_frames_min']=min(counts) if counts else None
                    row[label+'_frames_max']=max(counts) if counts else None
                    row[label+'_p95_median_ms']=statistics.median(v['p95'] for v in values) if values else None
                    if not values or len(values)!=len(runs) or min(counts)<30:
                        complete=False
                a,b=row['after_p95_median_ms'],row['before_p95_median_ms']
                row['delta_percent']=100*(a-b)/b if baseline and complete and b else None
                row['comparison_quality']=('matching groups; at least 30 samples per repeat' if complete
                                           else 'inspect population: missing group or fewer than 30 samples')
                if not baseline:
                    row['comparison_quality']='baseline not requested; inspect sample counts'
                rows.append(row)
    output.parent.mkdir(parents=True,exist_ok=True)
    with output.open('w',newline='') as stream:
        writer=csv.DictWriter(stream,fieldnames=list(rows[0]) if rows else ['scenario'])
        writer.writeheader();writer.writerows(rows)
    provenance=dict(source_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                    analysis_sources_sha256=dict(ANALYSIS_SOURCES_SHA256),
                    summary_provenance=summary.get('provenance',{}),
                    baseline_provenance=baseline.get('provenance',{}) if baseline else None,
                    note='Frame-weighted per-scene p95 values. Renderer readiness is not inactivity. '
                         'A nonzero natural offset can persist at rest and does not classify motion. '
                         'Missing/short populations have no percentage comparison. Primary gates are unchanged.')
    output.with_suffix('.metadata.json').write_text(json.dumps(provenance,indent=2)+'\n')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('summary',type=Path)
    parser.add_argument('--baseline',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    try:
        summary=json.loads(args.summary.read_text())
        baseline=json.loads(args.baseline.read_text()) if args.baseline else None
        export(summary,args.output,baseline)
    except (OSError,ValueError,KeyError,TypeError) as error:
        parser.exit(1,f'{error}\n')
    print(args.output)


if __name__=='__main__':main()
