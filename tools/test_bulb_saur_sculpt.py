"""Original evolved seed saurs: actual closed supports and conforming botanical detail."""
import copy,json,os,struct,unittest
from pathlib import Path
import bulb_saur_sculpt as author
from bivalve_sculpt import top_view
from sculpt_test_support import assert_closed_models,assert_model_shading,indexed_ray_intersections as rays

def overlap(a,b):
    xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]))
    yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
    if xh<=xl or yh<=yl:return False
    for i in range(1,19):
        for j in range(1,23):
            x=xl+(xh-xl)*i/19;y=yl+(yh-yl)*j/23;aa=sorted(rays(a,x,y));bb=sorted(rays(b,x,y))
            if any(min(ah,bh)-max(al,bl)>.00025 for al,ah in zip(aa[::2],aa[1::2]) for bl,bh in zip(bb[::2],bb[1::2])):return True
    return False

class BulbSaurTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        override=os.environ.get('BULB_SAUR_TEST_MODELS')
        root=Path(override) if override else Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
        cls.models={};cls.skin_sources={}
        for s in author.SPECIES:
            cls.skin_sources[s]=(root/(s+'.glb')).exists()
            if cls.skin_sources[s]:
                from bulb_saur_glb import read_bulb_saur
                cls.models[s]=read_bulb_saur(root/(s+'.glb'),s)
            elif override:cls.models[s]=json.loads((root/(s+'.mesh.json')).read_bytes())
            else:cls.models[s]=author.sculpture(s)
    def find(self,s,prefix,suffix=None):
        return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
    def test_closed_connected_outward_solids(self):assert_closed_models(self,self.models)
    def test_finite_unit_normals_and_actual_winding(self):assert_model_shading(self,self.models)
    def test_recipe_height_floor_and_existing_vertex_triangle_budgets(self):
        for s,m in self.models.items():
            if self.skin_sources[s]:
                from bulb_saur_glb import geometry_digest
                self.assertEqual(geometry_digest(m),geometry_digest(author.sculpture(s)))
            else:self.assertEqual(m,author.sculpture(s))
            ps=m['primitives']
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),author.HEIGHTS[s],places=6)
            if s=='venusaur':
                for axis,bounds in enumerate(((-.6534906,.6754222),(0.,1.2),(-.8369827,.6095559))):
                    actual=(min(v for p in ps for v in p['positions'][axis::3]),max(v for p in ps for v in p['positions'][axis::3]))
                    self.assertEqual(struct.pack('<2f',*actual),struct.pack('<2f',*bounds))
            self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000)
            self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
    def test_complete_quadrupeds_and_distinct_bud_and_five_petal_bloom(self):
        for s,m in self.models.items():
            for prefix,count in (('Body /',1),('Head /',1),('Ear /',2),('Eye /',2),('Iris /',2),('Pupil /',2),('Nostril /',2),('Fang /',2),('Foreleg /',2),('Hindleg /',2),('Forefoot /',2),('Hindfoot /',2),('Foreclaw /',6),('Hindclaw /',6),('Leaf /',6),('Leaf midrib /',6)):
                self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),count,(s,prefix))
        for prefix,count in (('Bud /',1),('Petal /',0),('Trunk /',0)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.models['ivysaur']['primitives']),count,prefix)
        for prefix,count in (('Bud /',0),('Petal /',5),('Petal spot /',15),('Trunk /',1),('Flower heart /',1),('Stamen /',9),('Brow /',2),('Ear inset /',2),('Head wart /',3),('Body wart /',8),('Head spot /',0)):
            self.assertEqual(sum(p['part'].startswith(prefix) for p in self.models['venusaur']['primitives']),count,prefix)
    def roots(self,s):
        body=self.find(s,'Body /');head=self.find(s,'Head /');yield body,head
        for p in self.models[s]['primitives']:
            n=p['part'];end=n.split()[-1]
            if n.startswith(('Ear /','Fang /','Head wart /')):yield head,p
            elif n.startswith('Body wart /'):yield body,p
            elif n.startswith(('Foreleg /','Hindleg /')):yield body,p
            elif n.startswith('Forefoot /'):yield self.find(s,'Foreleg /',end),p
            elif n.startswith('Hindfoot /'):yield self.find(s,'Hindleg /',end),p
            elif n.startswith(('Foreclaw /','Hindclaw /')):yield self.find(s,'Forefoot /' if n.startswith('Foreclaw /') else 'Hindfoot /',n.split()[-2]),p
            elif n.startswith(('Bud /','Trunk /')):yield body,p
            elif n.startswith('Leaf /'):yield self.find(s,'Trunk /') if s=='venusaur' else self.find(s,'Bud /'),p
            elif n.startswith('Petal /'):yield self.find(s,'Trunk /'),p
            elif n.startswith('Flower heart /'):yield self.find(s,'Trunk /'),p
            elif n.startswith('Stamen /'):yield self.find(s,'Flower heart /'),p
            elif n.startswith('Leaf midrib /'):yield self.find(s,'Leaf /',end),p
    def test_actual_head_limbs_feet_claws_and_botanical_roots(self):
        for s in author.SPECIES:
            for a,b in self.roots(s):
                with self.subTest(species=s,parent=a['part'],part=b['part']):self.assertTrue(overlap(a,b))
    def markings(self,s):
        for p in self.models[s]['primitives']:
            n=p['part'];end=n.split()[-1];parent=None
            if n.startswith(('Eye /','Nostril /','Head spot /','Mouth /','Brow /')):parent=self.find(s,'Head /')
            elif n.startswith('Iris /'):parent=self.find(s,'Eye /',end)
            elif n.startswith(('Pupil /','Eye glint /')):parent=self.find(s,'Iris /',end)
            elif n.startswith('Ear inset /'):parent=self.find(s,'Ear /',end)
            elif n.startswith('Tongue /'):parent=self.find(s,'Mouth /')
            elif n.startswith('Foreleg spot /'):parent=self.find(s,'Foreleg /',end)
            elif n.startswith('Hindleg spot /'):parent=self.find(s,'Hindleg /',end)
            elif n.startswith('Petal spot /'):
                parent=top_view(self.find(s,'Petal /',n.split()[-2]));p=top_view(p)
            if parent:yield p,parent
    def test_markings_follow_actual_parent_triangles(self):
        for s in author.SPECIES:
            for p,parent in self.markings(s):
                for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]):
                    hits=rays(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertLess(abs(z-max(hits)),.020,(s,p['part']))
    def test_marking_front_triangles_clear_the_stored_parent(self):
        for s in author.SPECIES:
            for p,parent in self.markings(s):
                for k in range(0,len(p['indices']),3):
                    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
                    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
                    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,x,y)
                    self.assertTrue(hits,(s,p['part']));self.assertGreater(z-max(hits),.0001,(s,p['part']))
    def test_mature_saur_has_a_broader_body_and_lower_eye_aperture(self):
        span=lambda p,axis:max(p['positions'][axis::3])-min(p['positions'][axis::3])
        self.assertGreater(span(self.find('venusaur','Body /'),0),span(self.find('ivysaur','Body /'),0)*1.5)
        self.assertLess(span(self.find('venusaur','Eye /',1),1),span(self.find('ivysaur','Eye /',1),1)*.75)
    def test_detached_support_negative_controls_fail(self):
        for s in author.SPECIES:
            for prefix,parent in (('Forefoot /','Foreleg /'),('Leaf /','Trunk /' if s=='venusaur' else 'Bud /')):
                p=copy.deepcopy(self.find(s,prefix));p['positions']=[v+(3. if i%3==0 else 0.) for i,v in enumerate(p['positions'])]
                self.assertFalse(overlap(self.find(s,parent),p),(s,prefix))
if __name__=='__main__':unittest.main()
