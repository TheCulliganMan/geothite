#!/usr/bin/env python3
"""Report adjacent-pair effects without replacing the primary regression gate."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import statistics

from performance_audit import ANALYSIS_SOURCES_SHA256, compare


def paired_effects(current, baseline):
    """Return raw pairs and effect ranges; require complete, compatible pairs."""
    previous={entry['name']:entry for entry in baseline['scenarios']}
    pairs=[]
    effects=[]
    for entry in current['scenarios']:
        before=previous.get(entry['name'])
        if before is None or not compare(entry,before,10,.2)['comparable']:
            raise ValueError(f"Incompatible or failed scenario: {entry['name']}")
        groups=[]
        for side in (before,entry):
            group={}
            for run in side['runs']:
                pair=run.get('pair')
                if type(pair) is not int or pair<1 or pair in group:
                    raise ValueError(f"Missing or duplicate pair identity: {entry['name']}")
                group[pair]=run
            groups.append(group)
        if groups[0].keys()!=groups[1].keys():
            raise ValueError(f"Incomplete pairs: {entry['name']}")
        for pair in groups[0]:
            if {group[pair].get('pair_position') for group in groups}!={1,2}:
                raise ValueError(f"Invalid pair order: {entry['name']}, pair {pair}")
        metrics=[(metric,lambda run,key=metric:run.get(key,{}).get('p95') if run.get(key) else None)
                 for metric in ('interval_ms','main_cpu_ms','gpu_ms','gpu_active_ms')]
        for index,checkpoint in enumerate(entry['runs'][0]['checkpoints']):
            if checkpoint.get('duration_only',False):
                continue
            metrics.append((f'checkpoint_{index}_ms',
                            lambda run,index=index:run['checkpoints'][index]['latency_ms']))
        for metric,value in metrics:
            rows=[]
            for pair in sorted(groups[0]):
                b,a=(value(group[pair]) for group in groups)
                if b is None or a is None:
                    rows=[]
                    break
                rows.append(dict(scenario=entry['name'],metric=metric,pair=pair,
                                 before_position=groups[0][pair]['pair_position'],
                                 after_position=groups[1][pair]['pair_position'],
                                 before_ms=b,after_ms=a,delta_ms=a-b,
                                 delta_percent=100*(a-b)/b if b else None))
            if not rows:
                continue
            pairs.extend(rows)
            deltas=[row['delta_ms'] for row in rows]
            percentages=[row['delta_percent'] for row in rows]
            effects.append(dict(scenario=entry['name'],metric=metric,pairs=len(rows),
                                median_paired_delta_ms=statistics.median(deltas),
                                minimum_paired_delta_ms=min(deltas),maximum_paired_delta_ms=max(deltas),
                                median_paired_delta_percent=statistics.median(percentages)
                                if all(value is not None for value in percentages) else None,
                                improved_pairs=sum(delta<0 for delta in deltas),
                                slower_pairs=sum(delta>0 for delta in deltas)))
    return pairs,effects


def export(current, baseline, output):
    if not all(summary.get('provenance',{}).get('paired_collection_finished_utc')
               for summary in (current,baseline)):
        raise ValueError('Paired collection is not finished')
    pairs,effects=paired_effects(current,baseline)
    output.mkdir(parents=True,exist_ok=True)
    for name,rows in (('pairs',pairs),('effects',effects)):
        with (output/f'{name}.csv').open('w',newline='') as stream:
            writer=csv.DictWriter(stream,fieldnames=list(rows[0]) if rows else ['scenario'])
            writer.writeheader();writer.writerows(rows)
    metadata=dict(source_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  analysis_sources_sha256=dict(ANALYSIS_SOURCES_SHA256),
                  current_provenance=current.get('provenance',{}),
                  baseline_provenance=baseline.get('provenance',{}),
                  note='Differences within adjacent pairs. Ranges describe these pairs, not confidence '
                       'intervals. They supplement the unchanged primary median-p95 gate. '
                       'Unavailable metrics and duration-only sampling checkpoints are omitted.')
    (output/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('summary',type=Path)
    parser.add_argument('--baseline',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    try:
        export(json.loads(args.summary.read_text()),json.loads(args.baseline.read_text()),args.output)
    except (OSError,ValueError,KeyError,TypeError) as error:
        parser.exit(1,f'{error}\n')
    print(args.output/'effects.csv')


if __name__=='__main__':main()
