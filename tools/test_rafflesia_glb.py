"""Stored canonical botanical skins, exact fitted carriers and occupied posed roots."""
import copy,math,os,unittest
from pathlib import Path
import rafflesia_glb as author
import test_rafflesia_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import inverse_rigid
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_RAFFLESIA_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class RafflesiaSkinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES}
        cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()}
    def part(self,m,i,mats):
        return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
    def test_exact_sources_packing_hierarchy_inverse_binds_and_original_limits(self):
        for s,m in self.logical.items():
            self.assertEqual(self.blobs[s],author.export_rafflesia(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_rafflesia(self.blobs[s],s)),author.REST_DIGESTS[s])
            self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
            self.assertEqual(len(m.joints),11 if s=='gloom' else 12);self.assertEqual(len(set(self.runtime[s].names)),len(self.runtime[s].names))
            self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
            self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000)
            for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
    def test_rigid_face_drool_and_each_petals_fitted_mottling_share_carriers(self):
        for s,m in self.logical.items():
            for i,n in enumerate(m.names):
                j=1 if n.startswith(('Eye /','Lip /','Mouth /','Drool /')) else 2 if n.startswith(('Bud /','Bud spot /','Crown /','Stem /','Rim /','Throat /')) else 7+int(n.split()[-2]) if n.startswith('Petal spot /') else 7+int(n.split()[-1]) if n.startswith(('Petal /','Sepal /')) else None
                if j is not None:
                    self.assertTrue(all(ids==[j,0,0,0] for ids in m.ids[i]),(s,n));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
    def test_every_control_moves_and_source_cues_have_neutral_endpoints(self):
        for s,m in self.logical.items():
            self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(len(m.joints))))
            for j in range(1,len(m.joints)):
                rotations=[[0.,0.,0.,1.] for _ in m.doc['nodes']];rotations[j]=m.rotations('attack',.34)[j];mats=m.matrices(rotations=rotations)
                self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
            for clip in m.clips:
                for t in (0.,1.):
                    for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
            for a in m.doc['animations']:
                self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
    def test_fixed_feet_floor_positive_posed_winding_and_bounded_strain(self):
        for s,m in self.logical.items():
            for clip in m.clips:
                for t in (0.,.18,.34,.46,.76,1.):
                    mats=m.matrices(clip,t)
                    for i,n in enumerate(m.names):
                        posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,n))
                        if n.startswith('Foot /'):self.assertEqual(posed,m.vertices[i])
                        for k in range(0,len(m.indices[i]),3):
                            tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k));self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b]);self.assertTrue(.8<ratio<1.2,(s,clip,t,n,ratio))
    def test_real_roots_and_uncrossed_limbs_petals_and_bud_through_poses(self):
        for s,m in self.logical.items():
            for clip in m.clips:
                for t in (.18,.34,.46,.76):
                    mats=m.matrices(clip,t);h=anatomy.RafflesiaTests();h.models={s:dict(primitives=[self.part(m,i,mats) for i in range(len(m.parts))])}
                    h.test_actual_roots_feet_flower_and_drool_are_attached();h.test_curved_limbs_bud_and_petal_sections_do_not_cross()
    def test_posed_fitted_surfaces_and_hollow_throat_retain_actual_support(self):
        for s,m in self.logical.items():
            for clip in m.clips:
                mats=m.matrices(clip,.34);ps=[]
                for i,n in enumerate(m.names):
                    j=7+int(n.split()[-2]) if n.startswith('Petal spot /') else 7+int(n.split()[-1]) if n.startswith(('Petal /','Sepal /')) else 2 if n.startswith(('Bud /','Bud spot /','Crown /','Stem /','Rim /','Throat /')) else 1
                    ps.append(inverse_rigid(self.part(m,i,mats),mats[j]))
                h=anatomy.RafflesiaTests();h.models={s:dict(primitives=ps)};h.test_fitted_lids_lips_gaze_and_mottling_follow_stored_surfaces()
                if s=='vileplume':h.test_flower_axis_reaches_recessed_dark_well()
    def test_mutated_sources_bytes_and_species_are_rejected(self):
        for s,b in self.blobs.items():
            for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
                with self.assertRaises(ValueError):author.decode_rafflesia(bad,s)
            with self.assertRaises(ValueError):author.decode_rafflesia(b,next(v for v in author.SPECIES if v!=s))
            model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
            with self.assertRaises(ValueError):author.export_rafflesia(s,model)

if __name__=='__main__':unittest.main()
