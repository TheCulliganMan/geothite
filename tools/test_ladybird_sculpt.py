"""Independent Ledyba/Ledian shell spots, fitted skin and six-limb anatomy."""
import struct, unittest
from ladybird_sculpt import sculpture, SPECIES, HEIGHTS
from bovine_sculpt import reverse_front
from sculpt_test_support import assert_closed_models, assert_model_shading, ray_intersections

class LadybirdSculptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={species:sculpture(species) for species in SPECIES}

    def test_closed_outward_geometry_and_shading(self):
        assert_closed_models(self,self.models)
        assert_model_shading(self,self.models)

    def test_closed_shell_halves_share_the_dome_seam_and_exterior_normals(self):
        for species,model in self.models.items():
            shells=[p for p in model['primitives'] if p['part'].startswith('Shell /')]
            self.assertEqual(len(shells),2)
            def field(p):
                result={}
                for i in range(0,len(p['positions']),3):
                    result.setdefault(tuple(p['positions'][i:i+3]),set()).add(tuple(p['normals'][i:i+3]))
                return result
            a,b=map(field,shells);shared=a.keys()&b.keys()
            self.assertGreaterEqual(len(shared),40,species)
            for point in shared:self.assertTrue(a[point]&b[point],(species,point))

    def test_fitted_faces_gloves_and_rear_spots_clear_actual_parent_skins(self):
        for species,model in self.models.items():
            parts=model['primitives']
            for p in parts:
                name=p['part'];rear=False
                if name.startswith(('Face /','Eye /')):parent=parts[1]
                elif name.startswith('Head spot /'):parent=reverse_front(parts[1]);p=reverse_front(p);rear=True
                elif name.startswith('Mouth /'):parent=next(q for q in parts if q['part'].startswith('Face /'))
                elif name.startswith('Breast seam /'):parent=parts[0]
                elif name.startswith(('Pupil /','Glint /')):
                    prefix='Eye /' if name.startswith('Pupil /') else 'Pupil /'
                    parent=next(q for q in parts if q['part'].startswith(prefix) and q['part'].endswith(' '+name.split()[-1]))
                elif name.startswith('Glove seam /'):
                    parent=next(q for q in parts if q['part']=='Hand / '+name.split('finger ')[-1])
                elif name.startswith('Shell spot /'):
                    side='-1' if '-1' in name.split() else '1'
                    parent=next(q for q in parts if q['part'].startswith('Shell /') and q['part'].endswith(' '+side))
                    p=reverse_front(p);parent=reverse_front(parent);rear=True
                else:continue
                vertices=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
                checked=0
                for i in range(0,len(p['indices']),3):
                    a,b,c=[vertices[j] for j in p['indices'][i:i+3]]
                    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
                    center=[sum(v[k] for v in (a,b,c))/3 for k in range(3)]
                    hits=ray_intersections(parent,*center[:2])
                    self.assertTrue(hits,(species,name,rear))
                    self.assertGreater(center[2],max(hits),(species,name,rear))
                    checked+=1
                self.assertGreater(checked,8,(species,name))

    def test_pack_palette_floor_budget_and_distinct_spot_counts(self):
        for species,model in self.models.items():
            parts=model['primitives'];colors={tuple(p['base_color']) for p in parts}
            self.assertEqual(len(colors),4)
            for rgb555 in (((26,19,8) if species=='ledyba' else (31,22,4)),(27,6,5)):
                srgb=[((v<<3)|(v>>2))/255 for v in rgb555]
                linear=tuple(v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in srgb)+(1.,)
                self.assertIn(struct.unpack('<4f',struct.pack('<4f',*linear)),colors)
            for p in parts:self.assertGreaterEqual(min(p['positions'][1::3]),0.)
            feet=[p for p in parts if p['part'].startswith('Foot /')]
            self.assertEqual(len(feet),2)
            for p in feet:self.assertEqual(min(p['positions'][1::3]),0.)
            self.assertEqual(sum(p['part'].startswith('Arm /') for p in parts),4)
            self.assertEqual(sum(p['part'].startswith('Leg /') for p in parts),2)
            self.assertEqual(sum(p['part'].startswith('Hand /') for p in parts),4)
            self.assertEqual(sum(p['part'].startswith('Head spot /') for p in parts),0 if species=='ledyba' else 1)
            spots=[p for p in parts if p['part'].startswith('Shell spot /')]
            self.assertEqual(len(spots),6 if species=='ledyba' else 4)
            self.assertEqual(sum('fifth joined cap' in p['part'] for p in spots),2 if species=='ledyba' else 0)
            self.assertEqual(sum(p['part'].startswith('Wing /') for p in parts),0 if species=='ledyba' else 2)
            self.assertAlmostEqual(max(v for p in parts for v in p['positions'][1::3]),HEIGHTS[species],places=6)
            self.assertLess(sum(len(p['positions'])//3 for p in parts),20000)

if __name__=='__main__':unittest.main()
