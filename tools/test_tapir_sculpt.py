#!/usr/bin/env python3
"""Stored tapir anatomy, continuous material seam, ruff/pendulum and grounding."""
import copy,json,os,unittest
from pathlib import Path
from tapir_sculpt import sculpture,HEIGHT
from tapir_glb import read_tapir,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
import test_magnet_sculpt as joins
ROOT=Path(os.environ.get('TAPIR_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class TapirTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:read_tapir(ROOT/(n+'.glb'),n) for n in ('drowzee','hypno')}
    def test_closed_anatomy_with_welded_body_materials(self):
        m=copy.deepcopy(self.models);a,b=m['drowzee']['primitives'][:2]
        joined={'part':'Body / joined material regions','positions':a['positions']+b['positions'],'normals':a['normals']+b['normals'],'indices':a['indices']+[i+len(a['positions'])//3 for i in b['indices']]}
        m['drowzee']['primitives']=[joined]+m['drowzee']['primitives'][2:]
        assert_closed_models(self,m)
    def test_actual_triangle_shading(self):assert_model_shading(self,self.models)
    def test_recipe_ground_and_species_identity(self):
        for n,m in self.models.items():
            self.assertEqual(geometry_digest(m),geometry_digest(sculpture(n)));names=[p['part'] for p in m['primitives']]
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLessEqual(sum(len(p['indices'])//3 for p in m['primitives']),40000)
            self.assertLessEqual(sum(len(p['positions'])//3 for p in m['primitives']),20000)
            adult=n=='hypno'
            for prefix,count in {'Ear / rounded':0 if adult else 2,'Ear / pointed':2 if adult else 0,'Ruff /':1 if adult else 0,'Pendulum /':3 if adult else 0,'Hand / curved':6,'Foot / yellow':4 if adult else 2}.items():
                self.assertEqual(sum(x.startswith(prefix) for x in names),count,(n,prefix))
            for p in m['primitives']:
                if p['part'].startswith('Eye /'):self.assertGreater(p['normals'][2],0.)
            if adult:
                body=next(p for p in m['primitives'] if p['part'].startswith('Body /'))
                self.assertGreater(body['base_color'][0],body['base_color'][2]*5)
    def test_body_seam_is_wavy_welded_and_shares_normals(self):
        a,b=self.models['drowzee']['primitives'][:2]
        def positions(p):return {tuple(p['positions'][i:i+3]):tuple(p['normals'][i:i+3]) for i in range(0,len(p['positions']),3)}
        pa,pb=positions(a),positions(b);seam=pa.keys()&pb.keys()
        self.assertEqual(len(seam),64)
        self.assertGreater(max(x[1] for x in seam)-min(x[1] for x in seam),.045)
        for v in seam:self.assertEqual(pa[v],pb[v])
        self.assertNotEqual(a['base_color'],b['base_color'])
    def test_ruff_neck_hole_and_held_pendulum(self):
        m=self.models['hypno'];parts={p['part']:p for p in m['primitives']}
        r=parts['Ruff / closed serrated ivory collar']
        ps=[r['positions'][i:i+3] for i in range(0,len(r['positions']),3)]
        self.assertGreater(min((x*x+z*z)**.5 for x,y,z in ps),.07)
        # Closed annulus has both inner and outer radial surfaces, not a disk.
        self.assertGreater(max((x*x+z*z)**.5 for x,y,z in ps),.26)
        cord=parts['Pendulum / hanging dark cord'];hand=parts['Hand / broad palm 1']
        center=[sum(cord['positions'][i:24:3])/8 for i in range(3)]
        for axis in range(3):
            self.assertGreaterEqual(center[axis],min(hand['positions'][axis::3])-.01)
            self.assertLessEqual(center[axis],max(hand['positions'][axis::3])+.01)
        self.assertGreater(max(cord['positions'][1::3])-min(cord['positions'][1::3]),.35)
        ring=parts['Pendulum / open metal ring'];points=[ring['positions'][i:i+3] for i in range(0,len(ring['positions']),3)]
        cx=(min(ring['positions'][::3])+max(ring['positions'][::3]))/2;cy=(min(ring['positions'][1::3])+max(ring['positions'][1::3]))/2
        self.assertGreater(min(((x-cx)**2+(y-cy)**2)**.5 for x,y,z in points),.035)
    def body(self,n):
        ps=self.models[n]['primitives']
        if n=='hypno':return ps[0]
        a,b=ps[:2]
        return {'part':'Body / joined material regions','positions':a['positions']+b['positions'],'indices':a['indices']+[i+len(a['positions'])//3 for i in b['indices']]}
    def find(self,ps,prefix,suffix=None):return next(p for p in ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
    def test_anatomy_has_actual_solid_roots(self):
        for n,m in self.models.items():
            ps=m['primitives'];body=self.body(n);pairs=[(body,self.find(ps,'Nose /'))]
            for sign in (-1,1):
                ear=self.find(ps,'Ear / pointed' if n=='hypno' else 'Ear / rounded',sign);upper=self.find(ps,'Arm / shoulder',sign);fore=self.find(ps,'Arm / shaped',sign);hand=self.find(ps,'Hand / broad',sign);thigh=self.find(ps,'Leg / tapered',sign);shin=self.find(ps,'Leg / planted',sign);foot=self.find(ps,'Foot / broad',sign)
                pairs += [(body,ear),(body,upper),(upper,fore),(fore,hand),(body,thigh),(thigh,shin),(shin,foot),(hand,self.find(ps,'Hand / rooted curved thumb',sign))]
                shoulder=self.find(ps,'Arm joint / rounded shoulder',sign);elbow=self.find(ps,'Arm joint / rounded elbow',sign);knee=self.find(ps,'Leg joint / rounded knee',sign)
                pairs += [(body,shoulder),(upper,shoulder),(upper,elbow),(fore,elbow),(thigh,knee),(shin,knee)]
                pairs += [(hand,self.find(ps,'Hand / curved finger',str(sign)+' '+str(j))) for j in range(3)]
                pairs += [(foot,self.find(ps,'Foot / yellow',str(sign)+' '+str(j))) for j in range(2 if n=='hypno' else 1)]
            if n=='hypno':pairs.append((body,self.find(ps,'Ruff /')))
            for a,b in pairs:self.assertTrue(joins.MagnetTests.overlap(self,a,b),(n,a['part'],b['part']))
    def test_pendulum_cord_and_dark_inner_lip_are_physically_supported(self):
        ps=self.models['hypno']['primitives'];hand=self.find(ps,'Hand / broad',1);cord=self.find(ps,'Pendulum / hanging');ring=self.find(ps,'Pendulum / open');lip=self.find(ps,'Pendulum / dark')
        for a,b in ((hand,cord),(cord,ring),(ring,lip)):self.assertTrue(joins.MagnetTests.overlap(self,a,b),(a['part'],b['part']))
    def markings(self):
        for n,m in self.models.items():
            ps=m['primitives'];body=self.body(n)
            for p in ps:
                name=p['part'];parent=None;sign=name.split()[-1]
                if name.startswith(('Eye border /','Cheek /')):parent=body
                elif name.startswith('Eye / fitted'):parent=self.find(ps,'Eye border /',sign)
                elif name.startswith('Eye / narrow'):parent=self.find(ps,'Eye / fitted',sign)
                elif name.startswith(('Ear / fitted','Ear / recessed')):parent=self.find(ps,'Ear / pointed' if n=='hypno' else 'Ear / rounded',sign)
                if parent:yield n,p,parent
    def test_marking_vertices_follow_actual_stored_parents(self):
        for n,p,parent in self.markings():
            for i in range(0,len(p['positions']),3):
                x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,(n,p['part']));self.assertLess(abs(z-max(h)),.017,(n,p['part']))
    def test_front_marking_triangles_clear_actual_stored_support(self):
        for n,p,parent in self.markings():
            for k in range(0,len(p['indices']),3):
                a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
                if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
                x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(n,p['part']))
if __name__=='__main__':unittest.main()
