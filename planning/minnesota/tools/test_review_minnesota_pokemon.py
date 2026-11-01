"""Protect catalog checkpoints and fossil choices during authoring expansion."""

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import review_minnesota_pokemon as review


class CatalogGuardTests(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads((review.ROOT/'roster.json').read_text())
        self.fossils = json.loads((review.ROOT/'fossil-choice.json').read_text())

    def test_current_catalog_is_valid(self):
        review.validate(self.catalog)

    def test_fossil_evolution_is_checked_after_pseudo_stats(self):
        fossils = copy.deepcopy(self.fossils)
        fossils['options'][0]['evolution']['level'] = 41
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'fossil-choice.json').write_text(json.dumps(fossils))
            with patch.object(review,'ROOT',root):
                with self.assertRaisesRegex(ValueError,'fossil item/evolution condition'):
                    review.validate(self.catalog)

    def test_fossil_cannot_revive_the_evolved_species(self):
        fossils = copy.deepcopy(self.fossils)
        fossils['options'][1]['revives_species_id'] = 'MN_204'
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'fossil-choice.json').write_text(json.dumps(fossils))
            with patch.object(review,'ROOT',root):
                with self.assertRaisesRegex(ValueError,'revive its first stage'):
                    review.validate(self.catalog)

    def test_new_families_cannot_rewrite_the_250_checkpoint(self):
        self.assertEqual(self.catalog['species_count'],266)
        family = next(f for f in self.catalog['families'] if f['id']=='MN_F087')
        family['theme'] = 'An unrelated replacement theme.'
        with self.assertRaisesRegex(ValueError,'approved 250 identities'):
            review.validate(self.catalog)


if __name__ == '__main__':
    unittest.main()
