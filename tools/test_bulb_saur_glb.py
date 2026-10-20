"""Decoded seed saur skins: complete supports, independent foliage and fitted faces."""
import copy,math,os,unittest
from pathlib import Path
import bulb_saur_glb as author
import test_bulb_saur_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import indexed_ray_intersections as rays

ROOT=Path(os.environ.get('BULB_SAUR_SKIN_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
FACE=('Head /','Head wart /','Ear /','Ear inset /','Eye /','Iris /','Pupil /','Eye glint /','Brow /','Nostril /','Mouth /','Tongue /','Fang /','Head spot /')
FEET=('Forefoot /','Hindfoot /','Foreclaw /','Hindclaw /')
def inverse_points(p,frame):
    q=dict(p);q['positions']=[sum(frame[4*i+j]*(v[j]-frame[12+j]) for j in range(3)) for v in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for i in range(3)];return q

class BulbSaurSkinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES}
        cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
    def posed(self,m,mats):
        neutral=m.neutral_model()
        return dict(neutral,primitives=[dict(p,positions=[v for q in m.posed(i,mats) for v in q],normals=[v for q in m.posed(i,mats,True) for v in q]) for i,p in enumerate(neutral['primitives'])])
    def test_canonical_geometry_names_colors_and_existing_reader_limits(self):
        for s,m in self.models.items():
            self.assertEqual(self.blobs[s],author.export_bulb_saur(s))
            self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s])
            self.assertEqual(author.geometry_digest(author.decode_bulb_saur(self.blobs[s],s)),author.REST_DIGESTS[s])
            self.assertEqual(m.names,list(author.PARTS[s]))
            self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices)),(58,18609,29796) if s=='ivysaur' else (100,19789,31500))
            self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.joints),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
    def test_original_authoring_bounds_and_every_logical_part_survive_packing(self):
        for s,m in self.models.items():
            source=author.authoring_model(s);decoded=m.neutral_model()
            self.assertEqual(author.geometry_digest(source),author.geometry_digest(decoded))
            self.assertEqual([p['part'] for p in source['primitives']],[p['part'] for p in decoded['primitives']])
            points=[v for vs in m.vertices for v in vs];self.assertEqual(min(v[1] for v in points),0.);self.assertAlmostEqual(max(v[1] for v in points),1.20,places=6)
    def test_hierarchy_inverse_binds_and_all_controls_used(self):
        for s,m in self.models.items():
            self.assertEqual(len(m.joints),21 if s=='ivysaur' else 32)
            self.assertEqual([m.doc['nodes'][j]['name'] for j in m.joints],[s+'/'+n for n,parent,p in author.JOINTS[s]])
            parents={child:i for i,n in enumerate(m.doc['nodes']) for child in n.get('children',[])}
            self.assertEqual([parents.get(j) for j in m.joints],[p for n,p,pivot in author.JOINTS[s]])
            for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
            self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(m.joints))))
    def test_coherent_expressions_and_complete_four_fixed_feet(self):
        for s,m in self.models.items():
            feet=0
            for i,n in enumerate(m.names):
                owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
                if n.startswith(FACE):self.assertEqual(owners,{2},(s,n))
                elif n.startswith(FEET):self.assertEqual(owners,{0},(s,n));feet+=1
                elif n.startswith(('Flower heart /','Stamen /')):self.assertEqual(owners,{31},n)
            self.assertEqual(feet,16)
            for clip in m.clips:
                for t in (.125,.31,.58,.84):
                    mats=m.matrices(clip,t)
                    for i,n in enumerate(m.names):
                        if n.startswith(FEET):self.assertLess(max(math.dist(a,b) for a,b in zip(m.posed(i,mats),m.vertices[i])),1e-7,(s,clip,n))
    def test_all_controls_move_and_neighboring_leaves_and_petals_stay_independent(self):
        for s,m in self.models.items():
            mats=m.matrices('attack',.31)
            for j,mat in enumerate(mats):
                if j:self.assertGreater(max(abs(a-b) for a,b in zip(mat,IDENTITY)),.0001,(s,j))
            for prefix,base,count in (('Leaf /',9,6),('Petal /',21,5 if s=='venusaur' else 0)):
                for j in range(count):
                    matrices=[list(IDENTITY) for _ in m.joints];matrices[base+2*j][12]=.025;matrices[base+2*j+1][12]=.025
                    for i,n in enumerate(m.names):
                        if not n.startswith(prefix):continue
                        displacement=max(math.dist(a,b) for a,b in zip(m.posed(i,matrices),m.vertices[i]))
                        if int(n.split()[-1])==j:self.assertGreater(displacement,.02,(s,n,j))
                        else:self.assertLess(displacement,1e-8,(s,n,j))
    def test_floor_positive_winding_and_bounded_strain_through_all_source_clips(self):
        for s,m in self.models.items():
            for clip in m.clips:
                for t in (.125,.31,.58,.84):
                    mats=m.matrices(clip,t)
                    for i,n in enumerate(m.names):
                        posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,n))
                        for k in range(0,len(m.indices[i]),3):
                            tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k//3));self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.8<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.2,(s,clip,t,n))
    def test_actual_skull_limb_claw_bud_leaf_and_flower_roots_survive_poses(self):
        helper=anatomy.BulbSaurTests()
        for clip in ('idle','attack','hit'):
            for t in (.125,.31,.58,.84):
                helper.models={s:self.posed(m,m.matrices(clip,t)) for s,m in self.models.items()}
                for s in author.SPECIES:
                    for a,b in helper.roots(s):
                        with self.subTest(species=s,clip=clip,time=t,parent=a['part'],part=b['part']):self.assertTrue(anatomy.overlap(a,b))
    def local_marking_models(self,clip,t):
        result={}
        for s,m in self.models.items():
            mats=m.matrices(clip,t);model=self.posed(m,mats);parts=[]
            for p in model['primitives']:
                n=p['part'];frame=2 if n.startswith(FACE) else 8 if n.startswith(('Petal /','Petal spot /')) else 1
                parts.append(inverse_points(p,mats[frame]))
            result[s]=dict(model,primitives=parts)
        return result
    def test_actual_posed_marking_vertices_follow_their_deformed_parents(self):
        helper=anatomy.BulbSaurTests()
        for clip in ('idle','attack','hit'):
            for t in (.31,.58):
                helper.models=self.local_marking_models(clip,t)
                for s in author.SPECIES:
                    for p,parent in helper.markings(s):
                        for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]):
                            hits=rays(parent,x,y);self.assertTrue(hits,(s,clip,t,p['part']));self.assertLess(abs(z-max(hits)),.020,(s,clip,t,p['part']))
    def test_actual_posed_front_caps_remain_clear_of_parent_triangles(self):
        helper=anatomy.BulbSaurTests()
        helper.models={s:m.neutral_model() for s,m in self.models.items()};caps={}
        # Track the authored cap triangles. Closed marking side walls enter
        # their parent by design and can face the ray after a limb bends.
        for s in author.SPECIES:
            for p,parent in helper.markings(s):
                selected=[]
                for k in range(0,len(p['indices']),3):
                    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
                    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])>1e-10:selected.append(k)
                self.assertTrue(selected,(s,p['part']));caps[s,p['part']]=selected
        for clip in ('idle','attack','hit'):
            for t in (.31,.58):
                helper.models=self.local_marking_models(clip,t)
                for s in author.SPECIES:
                    for p,parent in helper.markings(s):
                        for k in caps[s,p['part']]:
                            a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
                            x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,(s,clip,t,p['part']));self.assertGreater(z-max(hits),.0001,(s,clip,t,p['part']))
    def test_rotation_only_neutral_endpoints_and_early_brace_with_delayed_plant_response(self):
        for s,m in self.models.items():
            for animation in m.doc['animations']:
                self.assertTrue(animation['extras']['loopSuggested'] if animation['name'].endswith('.idle') else animation['extras']['cueRelative'])
                self.assertTrue(all(c['target']['path']=='rotation' and c['target']['node']!=0 for c in animation['channels']))
            for clip in m.clips:
                for t in (0.,1.):
                    for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7,(s,clip,t))
            early=m.matrices('attack',.18);late=m.matrices('attack',.30)
            distance=lambda mat:max(abs(a-b) for a,b in zip(mat,IDENTITY))
            self.assertGreater(distance(early[1]),distance(late[1]),s)
            self.assertGreater(distance(late[8]),distance(early[8]),s)
    def test_detached_supports_are_rejected_on_decoded_geometry(self):
        helper=anatomy.BulbSaurTests();helper.models={s:m.neutral_model() for s,m in self.models.items()}
        for s in author.SPECIES:
            for prefix,parent in (('Forefoot /','Foreleg /'),('Leaf /','Trunk /' if s=='venusaur' else 'Bud /')):
                p=copy.deepcopy(helper.find(s,prefix));p['positions']=[v+(3. if i%3==0 else 0.) for i,v in enumerate(p['positions'])]
                self.assertFalse(anatomy.overlap(helper.find(s,parent),p),(s,prefix))
    def test_canonical_reader_rejects_mutation_truncation_and_unreviewed_source(self):
        for s,blob in self.blobs.items():
            for bad in (blob[:-1],blob+b'\0',blob[:500]+bytes([blob[500]^1])+blob[501:]):
                with self.assertRaises(ValueError):author.decode_bulb_saur(bad,s)
            source=copy.deepcopy(author.authoring_model(s));source['primitives'][0]['positions'][0]+=.001
            with self.assertRaises(ValueError):author.export_bulb_saur(s,source)
if __name__=='__main__':unittest.main()
