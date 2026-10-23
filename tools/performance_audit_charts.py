#!/usr/bin/env python3
"""Export benchmark graphs and their numeric tables (optional matplotlib dependency).

Usage: python3 tools/performance_audit_charts.py SUMMARY --output DIRECTORY
Requires matplotlib only for these static exports; the auditor itself is stdlib.
"""
import argparse
import csv
import json
import re
from pathlib import Path
import statistics


def export(summary, output):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    import numpy as np

    output.mkdir(parents=True, exist_ok=True)
    plt.rcParams.update({'figure.facecolor': '#0b1016', 'axes.facecolor': '#111a24',
                        'savefig.facecolor': '#0b1016', 'text.color': '#e8f0f5',
                        'axes.labelcolor': '#9aafbf', 'xtick.color': '#9aafbf',
                        'ytick.color': '#e8f0f5', 'axes.edgecolor': '#243343',
                        'grid.color': '#243343', 'font.size': 10})
    entries = [entry for entry in summary['scenarios'] if entry['runs']
               and all(run['success'] for run in entry['runs'])]
    if not entries:
        raise ValueError('No fully successful scenarios to plot')
    med = statistics.median
    repeats = sorted({len(entry['runs']) for entry in entries})
    hardware = summary['provenance']['hardware']
    configuration = entries[0].get('compatibility', {})
    adapter = re.search(r'name: \"([^\"]+)\"', configuration.get('hardware', {}).get('adapter') or '')
    repeat_label = str(repeats[0]) if len(repeats) == 1 else f'{min(repeats)}–{max(repeats)}'
    build = 'development' if configuration.get('debug_assertions', True) else 'release'
    backend = configuration.get('gpu_backend', 'unknown backend')
    context = (f"{adapter[1] if adapter else hardware['machine']} · {build} · {backend} · {repeat_label} repeat{'s' if repeat_label != '1' else ''} · 960×864 offscreen\n"
               'Fresh processes; OS/driver caches retained.')
    if summary['provenance'].get('graph_label'):
        context=summary['provenance']['graph_label']+' · '+context
    rows = []
    metrics=[('interval_ms','Start interval','#d5e2e9'),
             ('main_cpu_ms','Main-app wall time','#5bbce8'),
             ('gpu_ms','GPU activity envelope','#bd9cff'),
             ('gpu_active_ms','GPU active union','#70d6b2')]
    metrics=[metric for metric in metrics if any(run.get(metric[0]) for entry in entries for run in entry['runs'])]
    fig, ax = plt.subplots(figsize=(13, max(5.5, (.66 if len(metrics)>3 else .52)*len(entries)+1.9)))
    y = np.arange(len(entries))
    for index, (metric, label, color) in enumerate(metrics):
        for i, entry in enumerate(entries):
            values = [run[metric]['p95'] for run in entry['runs'] if run.get(metric)]
            if len(values) != len(entry['runs']):
                continue
            center, low, high = med(values), min(values), max(values)
            position = i+(index-(len(metrics)-1)/2)*.21
            ax.barh(position, center, height=.18, color=color,
                    label=label if i == 0 else None, zorder=3)
            ax.errorbar(center, position, xerr=[[center-low], [high-center]],
                        fmt='none', ecolor=color, capsize=2, zorder=4)
            ax.text(high+max(.06, high*.008), position, f'{center:.2f}',
                    va='center', fontsize=8, color=color)
            rows.append([entry['name'], metric, 'p95 per-run median', center, low, high])
    ax.set_yticks(y, [entry['name'] for entry in entries]);ax.invert_yaxis()
    ax.set_xlabel('Milliseconds · median of per-run p95; whiskers show repeat range')
    ax.grid(axis='x', zorder=0);ax.set_axisbelow(True);ax.margins(x=.15)
    ax.legend(loc='upper right', bbox_to_anchor=(1,1.075), ncol=len(metrics), frameon=False)
    fig.suptitle('Gameplay latency across the scenario suite', fontsize=18, x=.03, ha='left')
    fig.text(.03, .015, context+'\nProfiled durations include instrumentation; these are not display FPS.',
             fontsize=9, color='#9aafbf')
    fig.tight_layout(rect=(.015,.075,.99,.95))
    for ext in ('png','svg'):fig.savefig(output/f'frame-latency.{ext}',dpi=160)
    plt.close(fig)

    points=[]
    for entry in entries:
        count=len(entry['runs'][0]['checkpoints'])
        if any(len(run['checkpoints']) != count for run in entry['runs']):
            raise ValueError('Inconsistent checkpoint count')
        for index in range(count):
            checkpoints=[run['checkpoints'][index] for run in entry['runs']]
            # A minimum-duration sampling step is not an input response latency.
            if all(checkpoint.get('duration_only', False) for checkpoint in checkpoints):
                continue
            point=checkpoints[0]
            points.append(dict(battle=entry['name'].startswith('battle'), name=f"{entry['name']} · {index+1} → {point['map']}",
                               controller=med(p['controller_ms'] for p in checkpoints),
                               settle=med(p['render_settle_ms'] for p in checkpoints),
                               total=med(p['latency_ms'] for p in checkpoints),
                               low=min(p['latency_ms'] for p in checkpoints),
                               high=max(p['latency_ms'] for p in checkpoints)))
    all_points = points
    for battle, filename, title in [(False, 'transition-latency', 'Input → authoritative result → visibly settled scene'),
                                     (True, 'battle-latency', 'Battle entry and action response · authored presentation included')]:
        points = sorted((p for p in all_points if p['battle'] == battle),
                        key=lambda point: point['total'], reverse=True)
        if not points:
            continue
        fig,ax=plt.subplots(figsize=(13,max(5.5,.43*len(points)+2)))
        y=np.arange(len(points));controller=np.array([p['controller'] for p in points])
        ax.barh(y,controller,color='#5bbce8',label='Controller / script / animation',zorder=3)
        ax.barh(y,[p['settle'] for p in points],left=controller,color='#bd9cff',label='Presentation / renderer settling',zorder=3)
        for i,p in enumerate(points):
            # Component medians need not sum to the median total.
            ax.plot([p['low'],p['high']],[i,i],color='#e8f0f5',linewidth=1.4,zorder=4)
            ax.plot(p['total'],i,'|',color='#e8f0f5',markersize=9,zorder=5)
            ax.text(max(p['high'],p['controller']+p['settle'])+max(.8,p['total']*.012),i,
                    f"{p['total']:,.1f}",va='center',fontsize=9)
            rows.append([p['name'],'checkpoint_ms','per-run median',p['total'],p['low'],p['high']])
        ax.set_yticks(y,[p['name'] for p in points]);ax.invert_yaxis();ax.grid(axis='x',zorder=0)
        ax.set_xlabel('Milliseconds · white ticks / whiskers = total median / repeat range')
        ax.set_axisbelow(True);ax.margins(x=.15);ax.legend(loc='upper right',bbox_to_anchor=(1,1.075),ncol=2,frameon=False)
        fig.suptitle(title,fontsize=18,x=.03,ha='left')
        fig.text(.03,.015,context+'\nAuthored dialogue and animation remain included. Component medians may differ from the total median.',fontsize=9,color='#9aafbf')
        fig.tight_layout(rect=(.015,.075,.99,.95))
        for ext in ('png','svg'):fig.savefig(output/f'{filename}.{ext}',dpi=160)
        plt.close(fig)
    candidates = [(sample['elapsed_ms'], entry, run, sample)
                  for entry in entries for run in entry['runs']
                  if run.get('gpu_waterfall', {}).get('rows')
                  for sample in run['gpu_samples']]
    if candidates:
        _, entry, run, sample = max(candidates, key=lambda candidate: candidate[0])
        packed = run['gpu_waterfall'];frame = sample['frame']
        intervals = [row for row in packed['rows'] if row[0] == frame]
        origin = min(row[1] for row in intervals)
        fig, ax = plt.subplots(figsize=(13,4.8))
        stages = ['Vertex', 'Fragment', 'Compute'];colors=['#5bbce8','#bd9cff','#70d6b2']
        for row in intervals:
            stage=packed['stages'][row[4]]
            if stage not in stages:continue
            lane=stages.index(stage)
            ax.broken_barh([((row[1]-origin)/1e6,(row[2]-row[1])/1e6)],
                           (lane-.28,.56),facecolors=colors[lane])
        ax.set_yticks(range(3),stages);ax.invert_yaxis();ax.set_xlabel('Milliseconds from first GPU activity in this source frame')
        ax.grid(axis='x');ax.set_axisbelow(True)
        fig.suptitle(f"Worst measured GPU frame · {entry['name']} · source {frame}",fontsize=17,x=.03,ha='left')
        fig.text(.03,.04,f"Activity envelope {sample['elapsed_ms']:.3f} ms · active union {sample.get('active_ms',0):.3f} ms\n"
                 'Native Metal activity; independent GPU clock. Parallel lanes overlap. Raw command/encoder identities retained.',
                 fontsize=10,color='#9aafbf')
        fig.tight_layout(rect=(.015,.13,.99,.93))
        for ext in ('png','svg'):fig.savefig(output/f'gpu-waterfall.{ext}',dpi=160)
        plt.close(fig)
    with (output/'numbers.csv').open('w',newline='') as file:
        writer=csv.writer(file);writer.writerow(['scenario','metric','aggregation','median_ms','repeat_min_ms','repeat_max_ms']);writer.writerows(rows)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('summary',type=Path);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--baseline',type=Path,help='Also export compatible before/after graphs')
    parser.add_argument('--label',help='Context label included in exported graph footers')
    args=parser.parse_args()
    try:
        summary=json.loads(args.summary.read_text())
        if args.label:summary['provenance']['graph_label']=args.label
        export(summary,args.output)
        if args.baseline:
            export_comparison(summary,json.loads(args.baseline.read_text()),args.output)
    except (OSError,ValueError,ImportError) as error:
        parser.exit(1,f'{error}\n')


def export_comparison(summary, baseline, output):
    """Plot compatible repeat distributions; lower durations are better."""
    import matplotlib.pyplot as plt
    from performance_audit import compare

    previous={entry['name']:entry for entry in baseline['scenarios']}
    configuration=summary['scenarios'][0].get('compatibility',{})
    repeats=sorted({len(entry['runs']) for entry in summary['scenarios']})
    repeat_label=str(repeats[0]) if len(repeats)==1 else f'{min(repeats)}–{max(repeats)}'
    context=(f"{summary['provenance'].get('graph_label','Paired benchmark')} · "
             f"{summary['provenance']['hardware']['machine']} · {configuration.get('gpu_backend','unknown backend')} · "
             f"{repeat_label} repeats · 960×864 offscreen")
    rows=[]
    for entry in summary['scenarios']:
        before=previous.get(entry['name'])
        result=compare(entry,before,10,.2) if before else dict(comparable=False,reason='Missing baseline')
        if not result['comparable']:
            raise ValueError(f"Cannot compare {entry['name']}: {result['reason']}")
        changes=list(result['changes'])
        if all(run.get('gpu_active_ms') for run in entry['runs']+before['runs']):
            active_before=statistics.median(run['gpu_active_ms']['p95'] for run in before['runs'])
            active_after=statistics.median(run['gpu_active_ms']['p95'] for run in entry['runs'])
            changes.append(dict(metric='gpu_active_ms',before=active_before,after=active_after,
                                delta_percent=100*(active_after-active_before)/active_before))
        for change in changes:
            metric=change['metric']
            if metric.startswith('checkpoint_'):
                index=int(metric.split('_')[1])
                if all(run['checkpoints'][index].get('duration_only',False) for run in entry['runs']):
                    continue
                get_value=lambda run:run['checkpoints'][index]['latency_ms']
                label=f"{entry['name']} · input {index+1}"
                group='response'
            else:
                get_value=lambda run:run[metric]['p95']
                label=entry['name']
                group=metric
            low_before,high_before=min(get_value(run) for run in before['runs']),max(get_value(run) for run in before['runs'])
            low_after,high_after=min(get_value(run) for run in entry['runs']),max(get_value(run) for run in entry['runs'])
            rows.append(dict(label=label,group=group,before=change['before'],after=change['after'],
                             delta_percent=change['delta_percent'],before_low=low_before,before_high=high_before,
                             after_low=low_after,after_high=high_after))
    for group,title in [('interval_ms','Frame-start interval p95'),('main_cpu_ms','Main-app wall time p95'),
                        ('gpu_ms','GPU activity envelope p95'),('gpu_active_ms','Correlated GPU active union p95'),
                        ('response','Input response / transition settling')]:
        values=[row for row in rows if row['group']==group]
        if not values:continue
        fig,ax=plt.subplots(figsize=(13,max(4.8,.44*len(values)+2)))
        for index,row in enumerate(values):
            ax.plot([row['before'],row['after']],[index,index],color='#526578',linewidth=2,zorder=2)
            for version,color,offset in [('before','#e7b86b',-.08),('after','#70d6b2',.08)]:
                value=row[version]
                ax.errorbar(value,index+offset,
                            xerr=[[value-row[f'{version}_low']],[row[f'{version}_high']-value]],
                            fmt='o',markersize=5,color=color,capsize=2,
                            label=version.title() if index==0 else None,zorder=3)
            extent=max(row['before_high'],row['after_high'])
            ax.annotate(f"{row['before']:.1f} → {row['after']:.1f} ms ({row['delta_percent']:+.1f}%)",
                        (extent,index),xytext=(10,0),textcoords='offset points',va='center',fontsize=8)
        ax.set_yticks(range(len(values)),[row['label'] for row in values]);ax.invert_yaxis()
        ax.grid(axis='x');ax.set_axisbelow(True)
        ax.set_xlim(0,max(max(row['before_high'],row['after_high']) for row in values)*1.8 or 1)
        ax.set_xlabel('Milliseconds · median across repeats; whiskers show repeat range')
        ax.legend(loc='upper right',bbox_to_anchor=(1,1.085),ncol=2,frameon=False)
        fig.suptitle(title+' · before / after',fontsize=17,x=.03,ha='left')
        fig.text(.03,.02,context+'\nFresh processes; OS/driver caches retained. Instrumented durations are not display FPS.\n'
                 'Repeat medians and ranges describe these captures; small changes are not significance claims.',
                 fontsize=9,color='#9aafbf')
        fig.tight_layout(rect=(.015,.10,.99,.95))
        for ext in ('png','svg'):fig.savefig(output/f'before-after-{group}.{ext}',dpi=160)
        plt.close(fig)
    with (output/'comparison.csv').open('w',newline='') as file:
        writer=csv.DictWriter(file,fieldnames=list(rows[0]) if rows else ['label'])
        writer.writeheader();writer.writerows(rows)
    if all(run.get('pair') for group in (summary,baseline)
           for entry in group['scenarios'] for run in entry['runs']):
        export_paired_changes(summary,baseline,output,context)


def export_paired_changes(summary,baseline,output,context):
    """Show every adjacent-pair percentage, retaining disagreement and drift."""
    import matplotlib.pyplot as plt
    from performance_audit_pair_effects import paired_effects

    pairs,effects=paired_effects(summary,baseline)
    for metric,title in (('interval_ms','Frame-start interval p95'),('main_cpu_ms','Main-app wall time p95'),
                         ('gpu_ms','GPU activity envelope p95'),('gpu_active_ms','GPU active union p95')):
        groups=[effect for effect in effects if effect['metric']==metric
                and effect['median_paired_delta_percent'] is not None]
        if not groups:continue
        fig,ax=plt.subplots(figsize=(13,max(4.8,.46*len(groups)+2)))
        labeled=set()
        for index,effect in enumerate(groups):
            samples=[row for row in pairs if row['scenario']==effect['scenario'] and row['metric']==metric]
            changes=[row['delta_percent'] for row in samples]
            ax.plot([min(changes),max(changes)],[index,index],color='#526578',linewidth=2,zorder=2)
            for i,row in enumerate(samples):
                order='Before → after' if row['before_position']==1 else 'After → before'
                color='#e7b86b' if row['before_position']==1 else '#5bbce8'
                ax.scatter(row['delta_percent'],index+(i-(len(samples)-1)/2)*.12,
                           color=color,s=25,label=order if order not in labeled else None,zorder=3)
                labeled.add(order)
            center=effect['median_paired_delta_percent']
            ax.plot(center,index,'|',color='#e8f0f5',markersize=15,zorder=4)
            ax.annotate(f"{center:+.1f}% · {effect['improved_pairs']}/{effect['pairs']} faster",
                        (max(changes),index),xytext=(10,0),textcoords='offset points',
                        va='center',fontsize=8)
        ax.axvline(0,color='#9aafbf',linewidth=1)
        percentages=[row['delta_percent'] for row in pairs if row['metric']==metric
                     and row['delta_percent'] is not None]
        low,high=min(percentages),max(percentages)
        span=max(10,high-low)
        ax.set_xlim(min(0,low)-span*.08,max(0,high)+span*.65)
        ax.set_yticks(range(len(groups)),[group['scenario'] for group in groups]);ax.invert_yaxis()
        ax.grid(axis='x');ax.set_axisbelow(True)
        ax.set_xlabel('Change within each adjacent pair, % · negative is faster · white tick = median paired change')
        ax.legend(loc='upper right',bbox_to_anchor=(1,1.085),ncol=2,frameon=False)
        fig.suptitle(title+' · adjacent pairs',fontsize=17,x=.03,ha='left')
        fig.text(.03,.02,context+'\nAll accepted pairs shown; raw rejected attempts retained. Ranges are not confidence intervals.',
                 fontsize=9,color='#9aafbf')
        fig.tight_layout(rect=(.015,.09,.99,.95))
        for ext in ('png','svg'):fig.savefig(output/f'paired-effects-{metric}.{ext}',dpi=160)
        plt.close(fig)


if __name__=='__main__':main()
