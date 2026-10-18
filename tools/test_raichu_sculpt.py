"""Closed mouse anatomy, true ear notches, occupied limbs and surface-fitted features."""
import copy,unittest
import raichu_sculpt as author
from bovine_sculpt import reverse_front
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from test_seahorse_glb import occupied_overlap
import test_nidorina_sculpt as fitted

class RaichuTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.model=author.sculpture()
    def find(self,prefix,suffix=None):return next(p for p in self.model['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
    def test_closed_connected_outward_solids(self):assert_closed_models(self,{'raichu':self.model})
    def test_finite_corner_normals_and_positive_winding(self):assert_model_shading(self,{'raichu':self.model})
    def test_floor_original_height_and_existing_budgets(self):
        ps=self.model['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),.96,places=6)
        self.assertEqual(len(ps),len({p['part'] for p in ps}));self.assertLessEqual(len(ps),64);self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
    def test_mouse_features_and_long_thin_bolt_tipped_tail(self):
        for prefix,count in [('Ear /',2),('Ear inset /',2),('Ear curl /',2),('Ear curl inset /',2),('Eye /',2),('Eye glint /',2),('Cheek /',2),('Arm /',2),('Hand /',2),('Finger /',6),('Thumb /',2),('Leg /',2),('Foot /',2),('Back stripe /',2),('Belly /',1),('Tail cord /',1),('Tail bolt /',1)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.model['primitives']),count,prefix)
        tail=self.find('Tail cord /');points=[tail['positions'][i:i+3] for i in range(0,len(tail['positions']),3)];self.assertGreater(max(p[0] for p in points)-min(p[0] for p in points),.48);self.assertGreater(max(p[2] for p in points)-min(p[2] for p in points),.35)
    def attachments(self):
        yield self.find('Head /'),self.find('Body /');yield self.find('Tail cord /'),self.find('Body /');yield self.find('Tail bolt /'),self.find('Tail cord /');yield self.find('Mouth /'),self.find('Head /')
        for side in (-1,1):
            for child,parent in [('Ear /','Head /'),('Ear curl /','Ear /'),('Arm /','Body /'),('Hand /','Arm /'),('Leg /','Body /'),('Foot /','Leg /'),('Thumb /','Hand /')]:yield self.find(child,side),self.find(parent,None if parent in ('Head /','Body /') else side)
            for j in range(3):yield self.find('Finger /',f'{side} {j}'),self.find('Hand /',side)
    def test_complete_mouse_limbs_ears_smile_and_tail_have_occupied_roots(self):
        for child,parent in self.attachments():self.assertTrue(occupied_overlap(child,parent),(child['part'],parent['part']))
    def test_detached_negative_controls_are_rejected(self):
        for child,parent in self.attachments():self.assertFalse(occupied_overlap(translate(copy.deepcopy(child),3.,0.,0.),parent))
    def pairs(self):
        for p in self.model['primitives']:
            n=p['part'];side=n.split()[-1]
            if n.startswith(('Eye /','Cheek /','Nose /')):yield p,self.find('Head /')
            elif n.startswith('Eye glint /'):yield p,self.find('Eye /',side)
            elif n.startswith('Ear inset /'):yield p,self.find('Ear /',side)
            elif n.startswith('Ear curl inset /'):yield p,self.find('Ear curl /',side)
            elif n.startswith('Belly /'):yield p,self.find('Body /')
            elif n.startswith('Back stripe /'):yield reverse_front(p),reverse_front(self.find('Body /'))
    def test_all_colored_features_follow_actual_stored_parent_surfaces(self):
        h=fitted.NidorinaTests();h.pairs=self.pairs;h.test_fitted_layers_follow_actual_stored_parent();h.test_fitted_face_centers_and_edges_clear_parent()
    def test_ear_curls_have_real_open_notches(self):
        lo,k=author.authoring_transform()
        for side in (-1,1):self.assertFalse(rays(self.find('Ear /',side),side*.325*k,(.87-lo)*k),side)
    def test_both_eyes_are_visible_ahead_of_the_whole_head(self):
        head=self.find('Head /')
        for side in (-1,1):
            eye=self.find('Eye /',side);visible=0
            for i in range(0,len(eye['indices']),3):
                tri=[eye['positions'][j*3:j*3+3] for j in eye['indices'][i:i+3]];a,b,c=tri
                if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
                x,y,z=[sum(p[k] for p in tri)/3 for k in range(3)];hits=rays(head,x,y)
                if hits and z>max(hits)+.001:visible+=1
            self.assertGreaterEqual(visible,3,side)
    def test_curling_ears_limbs_tail_cord_and_bolt_do_not_cross(self):
        for p in self.model['primitives']:
            if p['part'].startswith(('Ear /','Ear inset /','Ear curl /','Ear curl inset /','Arm /','Leg /','Tail cord /','Tail bolt /')):self.assertIsNone(crossed_faces(p),p['part'])

if __name__=='__main__':unittest.main()
