"""Canonical routing, loader budgets and negative geometry controls for roster QA."""
import copy
import importlib.util
import math
from pathlib import Path
import tempfile
import unittest

from animated_glb import parse_glb
from skin_glb_test_support import encode

TOOLS = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('check_battle_species', TOOLS/'check-battle-species.py')
check = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check)
ROOT = TOOLS.parent


def fixture():
    points = [(0.,0.,0.), (1.,0.,0.), (0.,1.,0.), (0.,0.,1.)]
    faces = [[points[i] for i in tri] for tri in [(0,2,1),(0,1,3),(0,3,2),(1,2,3)]]
    for _ in range(2):
        refined = []
        for a,b,c in faces:
            ab,ac,bc = [tuple((x+y)/2 for x,y in zip(u,v)) for u,v in [(a,b),(a,c),(b,c)]]
            refined += [[a,ab,ac],[ab,b,bc],[ac,bc,c],[ab,bc,ac]]
        faces = refined
    parts = []
    for part in range(3):
        positions,normals = [],[]
        for a,b,c in faces:
            u,v = [[p[j]-a[j] for j in range(3)] for p in [b,c]]
            n = [u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
            length = math.sqrt(sum(x*x for x in n))
            for p in [a,b,c]:
                positions += [p[0]+part*.05,p[1],p[2]]
                normals += [x/length for x in n]
        parts.append(dict(part='solid '+str(part),positions=positions,normals=normals,
                          indices=list(range(len(positions)//3)),base_color=[1.,.5,.3,1.]))
    return dict(name='fixture',version=1,coordinate_system=check.COORDINATES,primitives=parts)


class RosterValidatorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.limits = check.runtime_limits((ROOT/'crates/crystal-voxel-view/src/species_rig.rs').read_text())

    def test_current_registry_and_all_declared_sources_match(self):
        registry = check.read_registry((ROOT/'crates/crystal-voxel-view/src/battle_species_models.rs').read_text())
        authored = check.authoring_names((TOOLS/'build-battle-species.py').read_text())
        self.assertEqual(len(registry),251)
        self.assertEqual(authored,{check.roster_name(n) for n in registry.values()})
        self.assertEqual(registry['UNOWN'],'unown_i')

    def test_duplicate_registry_ids_and_wrong_species_aliases_are_rejected(self):
        for source in ['"ABRA" => "abra",\n"ABRA" => "abra",',
                       '"PIKACHU" => "eevee",', '"UNOWN" => "unown_a",']:
            with self.subTest(source=source), self.assertRaises(AssertionError):
                check.read_registry(source)

    def test_base_unown_expands_to_each_canonical_form_once(self):
        paths = check.expanded_paths({'UNOWN':'unown_i'},{'unown':Path('unown_i.glb')})
        self.assertEqual(set(paths),{'unown_'+c for c in 'abcdefghijklmnopqrstuvwxyz'})
        self.assertEqual(len(set(paths.values())),26)
        with self.assertRaisesRegex(AssertionError,'runtime files and registry differ'):
            check.expanded_paths({'UNOWN':'unown_i'},{'unown':Path('unown_i.glb'),'eevee':Path('eevee.glb')})

    def test_current_dewgong_uses_actual_runtime_budget(self):
        vertices,triangles = check.validate_storage(ROOT/'crates/crystal-voxel-view/models/battle_species/dewgong.glb',self.limits)
        self.assertGreater(triangles,22000)  # Beyond the retired static-mesh cap.
        self.assertLessEqual(triangles*3,self.limits['MAX_INDICES'])
        self.assertLessEqual(vertices,self.limits['MAX_VERTICES'])

    def test_missing_runtime_limit_is_rejected_instead_of_using_a_default(self):
        with self.assertRaisesRegex(AssertionError,'cannot read current runtime limit'):
            check.runtime_limits('const MAX_BYTES: usize = 1024;')

    def test_oversized_stored_vertex_index_and_accessor_counts_are_rejected(self):
        doc,binary = parse_glb((ROOT/'crates/crystal-voxel-view/models/battle_species/dewgong.glb').read_bytes())
        primitive = doc['meshes'][0]['primitives'][0]
        for kind in ['vertex','index','accessor','buffer-view']:
            damaged = copy.deepcopy(doc)
            if kind=='vertex':
                damaged['accessors'][primitive['attributes']['POSITION']]['count']=self.limits['MAX_VERTICES']+1
            elif kind=='index':
                damaged['accessors'][primitive['indices']]['count']=self.limits['MAX_INDICES']+3
            elif kind=='accessor':
                damaged['accessors'] += [{}]*(self.limits['MAX_ACCESSORS']+1-len(damaged['accessors']))
            else:
                damaged['bufferViews'] += [{}]*(self.limits['MAX_ACCESSORS']+1-len(damaged['bufferViews']))
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as folder:
                path=Path(folder)/'damaged.glb';path.write_bytes(encode(damaged,binary))
                with self.assertRaisesRegex(AssertionError,'runtime '+kind+' limit'):
                    check.validate_storage(path,self.limits)

    def test_valid_solids_and_wrong_identity_control(self):
        model=fixture()
        self.assertEqual(check.validate_geometry('fixture',Path('fixture.mesh.json'),model,self.limits)[1],192)
        model['name']='other_species'
        with self.assertRaisesRegex(AssertionError,'incorrect model identity'):
            check.validate_geometry('fixture',Path('fixture.mesh.json'),model,self.limits)

    def test_actor_filenames_keep_their_declared_model_identity(self):
        for species,filename,identity in [('clefairy','icon_clefairy.glb','clefairy'),
                                          ('jigglypuff','icon_jigglypuff.glb','jigglypuff'),
                                          ('chikorita','battle_chikorita.glb','battle_chikorita')]:
            model=fixture();model['name']=identity
            with self.subTest(species=species):
                check.validate_geometry(species,Path(filename),model,self.limits)

    def test_nonfinite_nonunit_and_ungrounded_models_are_rejected(self):
        for kind in ['nonfinite','nonunit','ungrounded']:
            model=fixture()
            if kind=='nonfinite':model['primitives'][0]['positions'][0]=float('nan')
            elif kind=='nonunit':model['primitives'][0]['normals'][0]=4.
            else:
                for p in model['primitives']:
                    p['positions']=[v+.1 if i%3==1 else v for i,v in enumerate(p['positions'])]
            with self.subTest(kind=kind), self.assertRaises(AssertionError):
                check.validate_geometry('fixture',Path('fixture.mesh.json'),model,self.limits)

    def test_inward_solids_fail_even_with_matching_inward_normals(self):
        model=fixture()
        for p in model['primitives']:
            p['indices']=[v for i in range(0,len(p['indices']),3) for v in reversed(p['indices'][i:i+3])]
            p['normals']=[-v for v in p['normals']]
        with self.assertRaisesRegex(AssertionError,'flat or inverted closed surface'):
            check.validate_geometry('fixture',Path('fixture.mesh.json'),model,self.limits)


if __name__ == '__main__':
    unittest.main()
