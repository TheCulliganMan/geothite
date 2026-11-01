import unittest
import csv
from pathlib import Path
import tempfile

from performance_audit import stats
from performance_audit_scenes import export, scenes


def run(outdoor_count):
    frames=[dict(id=i,map='outside' if i<outdoor_count else 'inside',mode='Normal',
                 battle=False,visible_ready=True,interval_ms=20 if i<outdoor_count else 4,
                 main_cpu_ms=1) for i in range(1000)]
    return dict(success=True,frames=frames,gpu_samples=[],checkpoints=[],gpu_ms=None,
                interval_ms=stats([f['interval_ms'] for f in frames]),main_cpu_ms=stats([1]*1000))


class SceneCostsTests(unittest.TestCase):
    def test_aggregate_tail_can_change_while_each_scene_cost_is_unchanged(self):
        before,after=run(40),run(60)
        self.assertEqual(stats([f['interval_ms'] for f in before['frames']])['p95'],4)
        self.assertEqual(stats([f['interval_ms'] for f in after['frames']])['p95'],20)
        a,b=scenes(before),scenes(after)
        for key in a:
            self.assertEqual(a[key]['interval_ms']['p95'],b[key]['interval_ms']['p95'])
            self.assertIsNone(a[key]['gpu_ms'])

    def test_readiness_and_modes_are_separate_and_gpu_joins_use_source_id(self):
        capture=run(1)
        capture['frames']=capture['frames'][:2]
        capture['frames'][1].update(map='outside',mode='Surf',visible_ready=False,natural_offset=[.5,0])
        capture['gpu_samples']=[dict(frame=1,elapsed_ms=3,active_ms=2)]
        groups=scenes(capture)
        self.assertIsNone(groups[('outside','Normal',False,True,None)]['gpu_ms'])
        self.assertEqual(groups[('outside','Surf',False,False,True)]['gpu_active_ms']['p95'],2)

    def test_short_scene_population_is_retained_without_an_improvement_claim(self):
        before=dict(scenarios=[dict(name='transition',compatibility={'same':True},runs=[run(40)])])
        after=dict(scenarios=[dict(name='transition',compatibility={'same':True},runs=[run(2)])])
        with tempfile.TemporaryDirectory() as folder:
            output=Path(folder)/'costs.csv'
            export(after,output,before)
            with output.open() as stream:rows=list(csv.DictReader(stream))
        row=next(row for row in rows if row['map']=='outside' and row['metric']=='interval_ms')
        self.assertEqual((row['before_frames_min'],row['after_frames_min']),('40','2'))
        self.assertEqual(row['delta_percent'],'')
        self.assertIn('fewer than 30',row['comparison_quality'])

    def test_missing_offset_is_not_merged_with_a_known_zero_offset(self):
        capture=run(1000)
        capture['frames'][0]['natural_offset']=[0,0]
        summary=dict(scenarios=[dict(name='field',runs=[capture])])
        with tempfile.TemporaryDirectory() as folder:
            output=Path(folder)/'costs.csv'
            export(summary,output)
            with output.open() as stream:rows=list(csv.DictReader(stream))
        rows=[row for row in rows if row['metric']=='interval_ms']
        self.assertEqual({row['natural_offset_nonzero'] for row in rows},{'','False'})
        self.assertEqual({row['after_frames_min'] for row in rows},{'1','999'})


if __name__=='__main__':unittest.main()
