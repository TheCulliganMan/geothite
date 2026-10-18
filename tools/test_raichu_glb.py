"""Stored Raichu carriers, independent ear tips, rooted tail and fixed paws."""
import copy,math,os,unittest
from pathlib import Path
import raichu_glb as author
import test_raichu_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import indexed_ray_intersections as rays
ROOT=Path(os.environ.get('CRYSTAL_RAICHU_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class RaichuSkinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=(ROOT/'raichu.glb').read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
    def part(self,i,mats):
        m=self.m;return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
    def test_exact_source_packing_schema_hierarchy_and_inverse_binds(self):
        m=self.m;self.assertEqual(self.blob,author.export_raichu());self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['raichu']);self.assertEqual(author.geometry_digest(author.read_raichu(ROOT/'raichu.glb')),author.REST_DIGESTS['raichu'])
        self.assertEqual(len(m.joints),13);self.assertEqual(len(m.parts),39);self.assertEqual(m.parents[:13],[p for n,p,v in author.JOINTS['raichu']]);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime.parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blob),1024*1024)
        self.assertEqual(len(set(self.runtime.names)),len(self.runtime.names));self.assertTrue(all(0<len(n.encode())<=160 for n in self.runtime.names))
        for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
    def test_fitted_face_paws_hands_and_bolt_have_coherent_carriers(self):
        m=self.m
        for i,n in enumerate(m.names):
            j=2 if n.startswith(('Head /','Eye /','Eye glint /','Cheek /','Nose /','Mouth /')) else 1 if n.startswith(('Belly /','Back stripe /')) else 0 if n.startswith('Foot /') else 10 if n.startswith('Tail bolt /') else 3 if n.startswith(('Hand /','Thumb /')) and n.endswith('-1') or n.startswith('Finger /') and n.split()[-2]=='-1' else 4 if n.startswith(('Hand /','Thumb /','Finger /')) else None
            if j is not None:self.assertTrue(all(ids==[j,0,0,0] for ids in m.ids[i]),n);self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]),n)
        for i,n in enumerate(m.names):
            prefix='Ear /' if n.startswith('Ear inset /') else 'Ear curl /' if n.startswith('Ear curl inset /') else None
            if prefix is None:continue
            parent=next(j for j,name in enumerate(m.names) if name.startswith(prefix) and name.split()[-1]==n.split()[-1]);lookup={}
            for k,p in enumerate(m.vertices[parent]):
                key=tuple(p[:2])
                if key not in lookup or p[2]>m.vertices[parent][lookup[key]][2]:lookup[key]=k
            for k,p in enumerate(m.vertices[i]):
                self.assertIn(tuple(p[:2]),lookup,n);j=lookup[tuple(p[:2])];self.assertEqual(m.ids[i][k],m.ids[parent][j]);self.assertEqual(m.weights[i][k],m.weights[parent][j])
            triangles={tuple(sorted(tuple(m.vertices[parent][j][:2]) for j in m.indices[parent][k:k+3])) for k in range(0,len(m.indices[parent]),3)}
            for k in range(0,len(m.indices[i]),3):
                tri=[m.vertices[i][j] for j in m.indices[i][k:k+3]];a,b,c=tri
                if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])>1e-12:self.assertIn(tuple(sorted(tuple(p[:2]) for p in tri)),triangles,n)
    def test_all_controls_move_with_rotation_only_source_cues_and_neutral_endpoints(self):
        m=self.m;self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(13)))
        for j in range(1,13):
            rotations=[[0.,0.,0.,1.] for _ in m.doc['nodes']];rotations[j]=m.rotations('attack',.34)[j];mats=m.matrices(rotations=rotations)
            self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,j)
        for clip in m.clips:
            for t in (0.,1.):
                for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
        for a in m.doc['animations']:
            self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
    def test_fixed_paws_floor_positive_winding_and_bounded_strain(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,.18,.34,.46,.76,1.):
                mats=m.matrices(clip,t)
                for i,n in enumerate(m.names):
                    posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(clip,t,n))
                    if n.startswith('Foot /'):self.assertEqual(posed,m.vertices[i])
                    for k in range(0,len(m.indices[i]),3):
                        tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
                        for a,b in zip(tri,tri[1:]+tri[:1]):
                            ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b]);self.assertTrue(.8<ratio<1.2,(clip,t,n,ratio))
    def test_actual_roots_and_uncrossed_ears_limbs_and_tail_through_poses(self):
        m=self.m
        for clip in m.clips:
            for t in (.18,.34,.46,.76):
                mats=m.matrices(clip,t);h=anatomy.RaichuTests();h.model=dict(primitives=[self.part(i,mats) for i in range(len(m.parts))]);h.test_complete_mouse_limbs_ears_smile_and_tail_have_occupied_roots();h.test_curling_ears_limbs_tail_cord_and_bolt_do_not_cross()
    def test_posed_fitted_features_and_flexible_ear_linings_follow_actual_parents(self):
        m=self.m
        for clip in m.clips:
            for t in (.18,.34,.46,.76):
                mats=m.matrices(clip,t);ps=[]
                for i,n in enumerate(m.names):
                    j=1 if n.startswith(('Body /','Belly /','Back stripe /')) else 2
                    ps.append(inverse_rigid(self.part(i,mats),mats[j]))
                h=anatomy.RaichuTests();h.model=dict(primitives=ps)
                for child,parent in h.pairs():
                    for k in range(0,len(child['positions']),3):
                        x,y,z=child['positions'][k:k+3];hits=rays(parent,x,y);self.assertTrue(hits,child['part']);self.assertLess(abs(z-max(hits)),.018,child['part'])
                    caps=0
                    for k in range(0,len(child['indices']),3):
                        tri=[child['positions'][i*3:i*3+3] for i in child['indices'][k:k+3]];a,b,c=tri;u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);area=math.sqrt(sum(n*n for n in normal))
                        # Deformed closing walls may face slightly forward;
                        # planar ear caps remain strongly frontal in head space.
                        threshold=.5 if child['part'].startswith(('Ear inset /','Ear curl inset /')) else .1
                        if normal[2]<=area*threshold:continue
                        caps+=1
                        for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
                            x,y,z=[sum(tri[j][axis]*weights[j] for j in range(3)) for axis in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,child['part']);self.assertGreater(z-max(hits),.0001,(clip,t,child['part']))
                    self.assertGreater(caps,0,child['part'])
    def test_mutated_sources_bytes_and_species_are_rejected(self):
        for b in (self.blob[:-1],self.blob+b'bad',self.blob[:20]+bytes([self.blob[20]^1])+self.blob[21:]):
            with self.assertRaises(ValueError):author.decode_mouse(b,'raichu')
        with self.assertRaises(ValueError):author.decode_mouse(self.blob,'pikachu')
        model=copy.deepcopy(author.authoring_model('raichu'));model['primitives'][0]['positions'][0]+=.0001
        with self.assertRaises(ValueError):author.export_raichu(model)

if __name__=='__main__':unittest.main()
