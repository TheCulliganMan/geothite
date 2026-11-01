"""Canonical source routing regressions; no generated assets or content pack."""
from pathlib import Path
import tempfile
import unittest

from battle_model_assets import STARTER_READERS, read_species_model, species_paths

ROOT = Path(__file__).resolve().parents[1] / 'crates/crystal-voxel-view/models'


class CanonicalSources(unittest.TestCase):
    def test_quagsire_resolves_one_canonical_skin(self):
        paths=species_paths(ROOT / 'battle_species',include_starters=True)
        self.assertEqual(paths['quagsire'],ROOT / 'battle_species/quagsire.glb')
        self.assertFalse((ROOT / 'battle_species/quagsire.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['quagsire'])['primitives']),22)
        self.assertEqual(len(paths),251)

    def test_spiders_resolve_one_canonical_skin_each(self):
        paths=species_paths(ROOT / 'battle_species',include_starters=True)
        for species,count in [('spinarak',43),('ariados',41)]:
            self.assertEqual(paths[species],ROOT / ('battle_species/'+species+'.glb'))
            self.assertFalse((ROOT / ('battle_species/'+species+'.mesh.json')).exists())
            self.assertEqual(len(read_species_model(paths[species])['primitives']),count)
        self.assertEqual(len(paths),251)

    def test_ladybirds_resolve_distinct_canonical_skins(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        for species, count in [('ledyba',36),('ledian',38)]:
            self.assertEqual(paths[species], ROOT / ('battle_species/'+species+'.glb'))
            self.assertFalse((ROOT / ('battle_species/'+species+'.mesh.json')).exists())
            self.assertEqual(len(read_species_model(paths[species])['primitives']), count)
        self.assertEqual(len(paths),251)

    def test_noctowl_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['noctowl'], ROOT / 'battle_species/noctowl.glb')
        self.assertFalse((ROOT / 'battle_species/noctowl.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['noctowl'])['primitives']), 52)
        self.assertEqual(len(paths), 251)

    def test_furret_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['furret'], ROOT / 'battle_species/furret.glb')
        self.assertFalse((ROOT / 'battle_species/furret.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['furret'])['primitives']), 33)
        self.assertEqual(len(paths), 251)

    def test_entei_shares_one_canonical_battle_and_world_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['entei'], ROOT / 'battle_species/entei.glb')
        self.assertFalse((ROOT / 'actor_props/entei.mesh.json').exists())
        self.assertFalse((ROOT / 'battle_species/entei.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['entei'])['primitives']), 59)

    def test_raikou_shares_one_canonical_battle_and_world_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['raikou'], ROOT / 'battle_species/raikou.glb')
        self.assertFalse((ROOT / 'actor_props/raikou.mesh.json').exists())
        self.assertFalse((ROOT / 'battle_species/raikou.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['raikou'])['primitives']), 62)

    def test_suicune_shares_one_canonical_battle_and_world_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['suicune'], ROOT / 'battle_species/suicune.glb')
        self.assertFalse((ROOT / 'actor_props/suicune.mesh.json').exists())
        self.assertFalse((ROOT / 'battle_species/suicune.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['suicune'])['primitives']), 47)

    def test_unown_forms_share_the_complete_canonical_family(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['unown'], ROOT / 'battle_species/unown_i.glb')
        self.assertFalse((ROOT / 'actor_props/icon_unown.mesh.json').exists())
        self.assertFalse((ROOT / 'battle_species/unown.mesh.json').exists())
        self.assertFalse(any(name.startswith('unown_') for name in paths))
        shapes=set()
        for letter in 'abcdefghijklmnopqrstuvwxyz':
            path=ROOT / 'battle_species' / ('unown_'+letter+'.glb')
            model=read_species_model(path)
            self.assertEqual(model['name'], 'unown_'+letter)
            shapes.add(tuple(model['primitives'][0]['positions']))
        self.assertGreater(len(shapes), 1)

    def test_incomplete_unown_form_publication_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            (root/'unown_i.glb').write_bytes(b'fixture')
            with self.assertRaisesRegex(ValueError, 'incomplete canonical Unown'):
                species_paths(root)

    def test_slowpoke_resolves_one_canonical_skin_for_battle_and_world(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['slowpoke'], ROOT / 'battle_species/slowpoke.glb')
        for folder, name in [('battle_species', 'slowpoke'), ('actor_props', 'slowpoke'), ('actor_props', 'icon_slowpoke')]:
            self.assertFalse((ROOT / folder / (name + '.mesh.json')).exists())
        self.assertEqual(len(read_species_model(paths['slowpoke'])['primitives']), 40)

    def test_diglett_resolves_one_new_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['diglett'], ROOT / 'battle_species/diglett.glb')
        self.assertFalse((ROOT / 'battle_species/diglett.mesh.json').exists())
        self.assertFalse((ROOT / 'actor_props/icon_diglett.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['diglett'])['primitives']), 15)

    def test_onix_resolves_one_new_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['onix'], ROOT / 'battle_species/onix.glb')
        self.assertFalse((ROOT / 'battle_species/onix.mesh.json').exists())
        self.assertFalse((ROOT / 'actor_props/big_onix.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['onix'])['primitives']), 36)

    def test_voltorb_resolves_one_new_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['voltorb'], ROOT / 'battle_species/voltorb.glb')
        self.assertFalse((ROOT / 'battle_species/voltorb.mesh.json').exists())
        self.assertFalse((ROOT / 'actor_props/icon_voltorb.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['voltorb'])['primitives']), 8)

    def test_staryu_resolves_one_new_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['staryu'], ROOT / 'battle_species/staryu.glb')
        self.assertFalse((ROOT / 'battle_species/staryu.mesh.json').exists())
        self.assertFalse((ROOT / 'actor_props/icon_staryu.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['staryu'])['primitives']), 10)

    def test_sentret_resolves_one_new_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['sentret'], ROOT / 'battle_species/sentret.glb')
        self.assertFalse((ROOT / 'battle_species/sentret.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['sentret'])['primitives']), 38)

    def test_hoothoot_resolves_one_new_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['hoothoot'], ROOT / 'battle_species/hoothoot.glb')
        self.assertFalse((ROOT / 'battle_species/hoothoot.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['hoothoot'])['primitives']), 38)

    def test_wing_insects_resolve_one_canonical_skin_each(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        for name, parts in [('butterfree', 59), ('beedrill', 42)]:
            self.assertEqual(paths[name], ROOT / 'battle_species' / (name + '.glb'))
            self.assertFalse((ROOT / 'battle_species' / (name + '.mesh.json')).exists())
            self.assertEqual(len(read_species_model(paths[name])['primitives']), parts)

    def test_bats_resolve_one_canonical_skin_each(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        for name, parts in [('zubat', 28), ('golbat', 32), ('crobat', 28)]:
            self.assertEqual(paths[name], ROOT / 'battle_species' / (name + '.glb'))
            self.assertFalse((ROOT / 'battle_species' / (name + '.mesh.json')).exists())
            self.assertEqual(len(read_species_model(paths[name])['primitives']), parts)

    def test_ducks_resolve_one_canonical_skin_each(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        for name,count in [('psyduck',28),('golduck',45)]:
            self.assertEqual(paths[name], ROOT / 'battle_species' / (name+'.glb'))
            self.assertFalse((ROOT / 'battle_species' / (name+'.mesh.json')).exists())
            self.assertEqual(len(read_species_model(paths[name])['primitives']), count)

    def test_raticate_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['raticate'], ROOT / 'battle_species/raticate.glb')
        self.assertFalse((ROOT / 'battle_species/raticate.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['raticate'])['primitives']), 63)

    def test_rattata_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['rattata'], ROOT / 'battle_species/rattata.glb')
        self.assertFalse((ROOT / 'battle_species/rattata.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['rattata'])['primitives']), 42)

    def test_jigglypuff_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['jigglypuff'], ROOT / 'actor_props/icon_jigglypuff.glb')
        self.assertFalse((ROOT / 'actor_props/icon_jigglypuff.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['jigglypuff'])['primitives']), 22)

    def test_clefairy_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['clefairy'], ROOT / 'actor_props/icon_clefairy.glb')
        self.assertFalse((ROOT / 'actor_props/icon_clefairy.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['clefairy'])['primitives']), 35)

    def test_poliwag_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['poliwag'], ROOT / 'battle_species/poliwag.glb')
        self.assertFalse((ROOT / 'battle_species/poliwag.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['poliwag'])['primitives']), 19)

    def test_oddish_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['oddish'], ROOT / 'battle_species/oddish.glb')
        self.assertFalse((ROOT / 'battle_species/oddish.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['oddish'])['primitives']), 19)

    def test_pidgey_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['pidgey'], ROOT / 'battle_species/pidgey.glb')
        self.assertFalse((ROOT / 'battle_species/pidgey.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['pidgey'])['primitives']), 43)

    def test_geodude_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['geodude'], ROOT / 'battle_species/geodude.glb')
        self.assertFalse((ROOT / 'battle_species/geodude.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['geodude'])['primitives']), 29)

    def test_squirtle_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['squirtle'], ROOT / 'battle_species/squirtle.glb')
        self.assertFalse((ROOT / 'battle_species/squirtle.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['squirtle'])['primitives']), 45)

    def test_charmander_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['charmander'], ROOT / 'battle_species/charmander.glb')
        self.assertFalse((ROOT / 'battle_species/charmander.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['charmander'])['primitives']), 43)

    def test_bulbasaur_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['bulbasaur'], ROOT / 'battle_species/bulbasaur.glb')
        self.assertFalse((ROOT / 'battle_species/bulbasaur.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['bulbasaur'])['primitives']), 44)

    def test_pikachu_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['pikachu'], ROOT / 'battle_species/pikachu.glb')
        self.assertFalse((ROOT / 'battle_species/pikachu.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['pikachu'])['primitives']), 40)

    def test_snorlax_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['snorlax'], ROOT / 'battle_species/snorlax.glb')
        self.assertFalse((ROOT / 'battle_species/snorlax.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['snorlax'])['primitives']), 31)

    def test_lapras_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['lapras'], ROOT / 'battle_species/lapras.glb')
        self.assertFalse((ROOT / 'battle_species/lapras.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['lapras'])['primitives']), 42)

    def test_sudowoodo_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['sudowoodo'], ROOT / 'battle_species/sudowoodo.glb')
        self.assertFalse((ROOT / 'battle_species/sudowoodo.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['sudowoodo'])['primitives']), 33)

    def test_gyarados_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['gyarados'], ROOT / 'battle_species/gyarados.glb')
        self.assertFalse((ROOT / 'battle_species/gyarados.mesh.json').exists())
        self.assertEqual(len(read_species_model(paths['gyarados'])['primitives']), 40)

    def test_wooper_resolves_one_canonical_skin(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        self.assertEqual(paths['wooper'], ROOT / 'battle_species/wooper.glb')
        self.assertFalse((ROOT / 'battle_species/wooper.mesh.json').exists())
        model = read_species_model(paths['wooper'])
        self.assertEqual(model['name'], 'wooper')
        self.assertEqual(len(model['primitives']), 22)

    def test_starters_resolve_production_actor_rigs(self):
        paths = species_paths(ROOT / 'battle_species', include_starters=True)
        for filename in STARTER_READERS:
            species = filename.removeprefix('battle_').removesuffix('.glb')
            self.assertEqual(paths[species], ROOT / 'actor_props' / filename)
            model = read_species_model(paths[species])
            self.assertEqual(model['name'], 'battle_' + species)
            self.assertTrue(model['primitives'])
        self.assertEqual(len(paths), 251)

    def test_duplicate_shared_icon_json_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);battle=root/'battle_species';actors=root/'actor_props'
            battle.mkdir();actors.mkdir()
            (actors/'icon_clefairy.glb').write_bytes(b'fixture')
            (actors/'icon_clefairy.mesh.json').write_text('{}')
            with self.assertRaisesRegex(ValueError,'duplicate canonical Clefairy'):
                species_paths(battle,include_starters=True)

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
