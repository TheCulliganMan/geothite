import copy
from pathlib import Path
import tempfile
import unittest

from performance_audit import stats
from performance_audit_pair_effects import export, paired_effects


def cohort(values,after=False):
    runs=[dict(success=True,pair=i+1,pair_position=2 if (i%2==0)!=after else 1,
               interval_ms=stats([value]*40),main_cpu_ms=stats([1]*40),gpu_ms=None,
               checkpoints=[]) for i,value in enumerate(values)]
    return dict(scenarios=[dict(name='sample',compatibility={'same':True},runs=runs)])


class PairEffectsTests(unittest.TestCase):
    def test_adjacent_effects_retain_common_round_drift_and_disagreement(self):
        before,after=cohort([10,20,30]),cohort([8,22,24],True)
        pairs,effects=paired_effects(after,before)
        row=next(row for row in effects if row['metric']=='interval_ms')
        # Aggregate medians rise 20 -> 22; two adjacent pairs still improve.
        self.assertEqual(row['median_paired_delta_percent'],-20)
        self.assertEqual((row['improved_pairs'],row['slower_pairs']),(2,1))
        self.assertEqual((row['minimum_paired_delta_ms'],row['maximum_paired_delta_ms']),(-6,2))
        self.assertEqual([r['before_position'] for r in pairs if r['metric']=='interval_ms'],[2,1,2])

    def test_missing_duplicate_and_invalid_order_pairs_are_refused(self):
        before,after=cohort([10,20]),cohort([9,18],True)
        for mutate,pattern in (
                (lambda runs:runs.pop(),'Incomplete'),
                (lambda runs:runs[1].update(pair=1),'duplicate'),
                (lambda runs:runs[0].update(pair_position=2),'order')):
            changed=copy.deepcopy(after)
            mutate(changed['scenarios'][0]['runs'])
            with self.assertRaisesRegex(ValueError,pattern):paired_effects(changed,before)

    def test_duration_samples_and_unavailable_gpu_are_not_input_or_zero_costs(self):
        before,after=cohort([10]),cohort([9],True)
        for summary in (before,after):
            summary['scenarios'][0]['runs'][0]['checkpoints']=[
                dict(latency_ms=5000,duration_only=True),dict(latency_ms=400,duration_only=False)]
        pairs,effects=paired_effects(after,before)
        metrics={row['metric'] for row in effects}
        self.assertNotIn('gpu_ms',metrics)
        self.assertNotIn('checkpoint_0_ms',metrics)
        self.assertIn('checkpoint_1_ms',metrics)

    def test_export_waits_for_both_collection_receipts(self):
        before,after=cohort([10]),cohort([9],True)
        after['provenance']={'paired_collection_finished_utc':'test'}
        with tempfile.TemporaryDirectory() as folder:
            output=Path(folder)/'effects'
            with self.assertRaisesRegex(ValueError,'not finished'):export(after,before,output)
            self.assertFalse(output.exists())
            before['provenance']={'paired_collection_finished_utc':'test'}
            export(after,before,output)
            self.assertTrue((output/'effects.csv').is_file())


if __name__=='__main__':unittest.main()
