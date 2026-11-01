"""Actual-triangle botanical surfaces, occupied attachments and open flower throat."""
import copy,math,unittest
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from sculpt_geometry import translate
from test_seahorse_glb import occupied_overlap
import test_nidorina_sculpt as fitted_checks
import rafflesia_sculpt as author

class RafflesiaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:author.sculpture(s) for s in author.SPECIES}
    def find(self,s,prefix):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix))
    def test_closed_connected_outward_solids(self):assert_closed_models(self,self.models)
    def test_finite_corner_normals_and_positive_winding(self):assert_model_shading(self,self.models)
    def test_floor_original_height_unique_parts_and_existing_budgets(self):
        for s,m in self.models.items():
            ps=m['primitives'];self.assertEqual(len({p['part'] for p in ps}),len(ps));self.assertLessEqual(len(ps),64)
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),author.HEIGHTS[s],places=6)
            self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000)
            self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
    def test_species_anatomy_has_sleepy_bud_or_five_spotted_petals(self):
        for s,m in self.models.items():
            counts={prefix:sum(p['part'].startswith(prefix) for p in m['primitives']) for prefix in ('Foot /','Leg /','Arm /','Eye /','Drool /','Sepal /','Petal /','Petal spot /','Bud spot /')}
            self.assertEqual([counts[n] for n in ('Foot /','Leg /','Arm /','Eye /')],[2,2,2,2])
            self.assertEqual([counts[n] for n in ('Drool /','Sepal /','Petal /','Petal spot /','Bud spot /')],[2,4,0,0,12] if s=='gloom' else [0,0,5,15,0])
    def attachments(self,s):
        body=self.find(s,'Body /');ps=self.models[s]['primitives']
        for p in ps:
            n=p['part']
            if n.startswith(('Leg /','Arm /','Bud /','Stem /')):yield p,body
            elif n.startswith('Foot /'):yield p,next(q for q in ps if q['part'].startswith('Leg /') and q['part'].split()[-1]==n.split()[-1])
            elif n.startswith('Sepal /'):yield p,self.find(s,'Bud /')
            elif n.startswith('Drool /'):yield p,self.find(s,'Lip /')
            elif n.startswith('Crown /'):yield p,self.find(s,'Stem /')
            elif n.startswith(('Petal /','Rim /','Throat /')):yield p,self.find(s,'Crown /')
    def test_actual_roots_feet_flower_and_drool_are_attached(self):
        for s in self.models:
            for child,parent in self.attachments(s):self.assertTrue(occupied_overlap(child,parent),(s,child['part'],parent['part']))
    def test_detached_negative_controls_fail(self):
        for s in self.models:
            for child,parent in self.attachments(s):self.assertFalse(occupied_overlap(translate(copy.deepcopy(child),3.,0.,0.),parent))
    def pairs(self):
        for s,m in self.models.items():
            for child in m['primitives']:
                n=child['part']
                if n.startswith(('Eye /','Lip /','Mouth /')):
                    yield child,self.find(s,'Lip /' if s=='gloom' and n.startswith('Mouth /') else 'Body /')
                elif n.startswith('Bud spot /'):
                    angle=int(n.split()[-2])*math.tau/4+math.pi/4
                    yield author.turn(child,-angle),author.turn(self.find(s,'Bud /'),-angle)
                elif n.startswith('Petal spot /'):
                    parent=self.find(s,'Petal / five broad scarlet flower lobes '+n.split()[-2])
                    def unflower(p):
                        raw=author.sculpture(s,False)['primitives'];lo=min(v for q in raw for v in q['positions'][1::3]);hi=max(v for q in raw for v in q['positions'][1::3]);scale=author.HEIGHTS[s]/(hi-lo)
                        p=copy.deepcopy(p);v=p['positions'];c=math.cos(.27);sn=math.sin(.27);points=[]
                        for x,y,z in zip(v[::3],v[1::3],v[2::3]):
                            y=y/scale+lo-.61;z/=scale;points.extend((x/scale,.69+c*y+sn*z,-sn*y+c*z))
                        p['positions']=points;return p
                    yield author.top(unflower(child)),author.top(unflower(parent))
    def test_fitted_lids_lips_gaze_and_mottling_follow_stored_surfaces(self):
        h=fitted_checks.NidorinaTests();h.pairs=self.pairs
        h.test_fitted_layers_follow_actual_stored_parent()
        for child,parent in self.pairs():
            caps=0
            for i in range(0,len(child['indices']),3):
                tri=[child['positions'][3*j:3*j+3] for j in child['indices'][i:i+3]]
                a,b,c=tri;u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)]
                normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);area=math.sqrt(sum(n*n for n in normal))
                # Inverting a rounded rotation gives formerly vertical closing
                # walls tiny XY areas. Classify real front caps by their full
                # geometric normal, rather than treating that noise as a cap.
                if normal[2]<=area*.1:continue
                caps+=1
                for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
                    x,y,z=[sum(tri[k][axis]*weights[k] for k in range(3)) for axis in range(3)];hits=rays(parent,x,y)
                    self.assertTrue(hits,child['part']);self.assertGreater(z-max(hits),.0001,child['part'])
            self.assertGreater(caps,8,child['part'])
    def test_flower_axis_reaches_recessed_dark_well(self):
        s='vileplume';floor=rays(author.top(self.find(s,'Throat /')),0.,0.);self.assertTrue(floor)
        for prefix in ('Crown /','Rim /','Stem /','Petal /'):
            for p in self.models[s]['primitives']:
                if p['part'].startswith(prefix):
                    hits=rays(author.top(p),0.,0.);self.assertTrue(not hits or max(hits)<max(floor)-.01,p['part'])
    def test_curved_limbs_bud_and_petal_sections_do_not_cross(self):
        for s,m in self.models.items():
            for p in m['primitives']:
                if p['part'].startswith(('Leg /','Arm /','Bud /','Sepal /','Petal /','Drool /')):self.assertIsNone(crossed_faces(p),(s,p['part']))

if __name__=='__main__':unittest.main()
