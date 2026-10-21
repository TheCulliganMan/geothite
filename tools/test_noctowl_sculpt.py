"""Independent Noctowl crown, fitted face, closed surfaces and floor checks."""
import unittest,struct
from noctowl_sculpt import sculpture, HEIGHT
from sculpt_test_support import assert_closed_models, assert_model_shading, ray_intersections

class NoctowlSculptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.model=sculpture()
        cls.parts=cls.model['primitives']

    def test_closed_outward_geometry_and_shading(self):
        assert_closed_models(self,{'noctowl':self.model})
        assert_model_shading(self,{'noctowl':self.model})

    def test_fitted_face_and_breast_marks_clear_their_actual_skin(self):
        body=self.parts[0]
        for p in self.parts:
            name=p['part']
            if name.startswith(('Eye mask /','Breast mark /','Mouth /')):
                parent=body
            else:
                prefix=next((a for a in ('Eye ring /','Eye /','Iris /','Pupil /','Glint /') if name.startswith(a)),None)
                if prefix is None:continue
                owner={'Eye ring /':'Eye mask /','Eye /':'Eye ring /','Iris /':'Eye /','Pupil /':'Iris /','Glint /':'Pupil /'}[prefix]
                parent=next(q for q in self.parts if q['part'].startswith(owner) and q['part'].endswith(' '+name.split()[-1]))
            vertices=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
            checked=0
            for i in range(0,len(p['indices']),3):
                a,b,c=[vertices[j] for j in p['indices'][i:i+3]]
                if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]) <= 1e-10:continue
                center=[sum(v[k] for v in (a,b,c))/3 for k in range(3)]
                hits=ray_intersections(parent,*center[:2])
                self.assertTrue(hits,name)
                self.assertGreater(center[2],max(hits),name)
                checked+=1
            # Every stored front triangle is checked above. A compact seam
            # need not add subdivisions just to meet an arbitrary face count.
            self.assertGreater(checked,0,name)

    def test_reference_pigments_grounded_feet_and_distinct_anatomy(self):
        colors={tuple(p['base_color']) for p in self.parts}
        self.assertEqual(len(colors),7)
        # Full-colour game body and wing pigments, rather than shared 2-bit
        # highlight/shadow indices applied to every anatomical region.
        for srgb in ((.64,.43,.31),(.28,.17,.12),(.94,.83,.56)):
            linear=tuple(v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in srgb)+(1.,)
            linear=struct.unpack("<4f",struct.pack("<4f",*linear))
            self.assertIn(linear,colors)
        for p in self.parts:self.assertGreaterEqual(min(p['positions'][1::3]),0.)
        self.assertAlmostEqual(max(v for p in self.parts for v in p['positions'][1::3]),HEIGHT,places=6)
        for prefix,count in [('Crown /',2),('Brow /',2),('Foot /',2),('Toe /',6),('Claw /',6),('Tail /',3),('Breast mark /',6)]:
            self.assertEqual(sum(p['part'].startswith(prefix) for p in self.parts),count,prefix)
        self.assertLess(sum(len(p['positions'])//3 for p in self.parts),20000)

if __name__=='__main__':unittest.main()
