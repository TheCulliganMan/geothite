"""Canonical source routing regressions; no generated assets or content pack."""
from pathlib import Path
import tempfile
import unittest

from battle_model_assets import STARTER_READERS, read_species_model, species_paths

ROOT = Path(__file__).resolve().parents[1] / 'crates/crystal-voxel-view/models'


class CanonicalSources(unittest.TestCase):
    def test_starters_resolve_production_actor_rigs(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        for filename in STARTER_READERS:
            species = filename.removeprefix('battle_').removesuffix('.glb')
            self.assertEqual(paths[species], ROOT / 'actor_props' / filename)
            model = read_species_model(paths[species])
            self.assertEqual(model['name'], 'battle_' + species)
            self.assertTrue(model['primitives'])
        self.assertEqual(len(paths), 225)

    def test_duplicate_starter_export_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            battle = root / 'battle_species'
            actors = root / 'actor_props'
            battle.mkdir()
            actors.mkdir()
            (battle / 'chikorita.mesh.json').write_text('{}')
            (actors / 'battle_chikorita.glb').write_bytes(b'fixture')
            with self.assertRaisesRegex(ValueError, 'duplicate canonical Chikorita'):
                species_paths(battle, include_starters=True)

    def test_unsupported_glb_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            (Path(directory) / 'unknown.glb').write_bytes(b'fixture')
            with self.assertRaisesRegex(ValueError, 'unsupported battle species GLB'):
                species_paths(directory)


if __name__ == '__main__':
    unittest.main()
