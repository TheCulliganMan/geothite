"""Stored six-limb ladybird skins, independently moving cases and fixed feet."""
import math,os,unittest
from pathlib import Path
import ladybird_glb as author
import test_ladybird_sculpt as geometry_checks
from test_gengar_glb import Model,winding_alignment
DIRECTORY=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSETS={species:Path(os.environ.get(species.upper()+'_GLB',DIRECTORY/(species+'.glb'))) for species in author.SPECIES}

class LadybirdSkinTests(geometry_checks.LadybirdSculptTests):
    @classmethod
    def setUpClass(cls):
        cls.blobs={species:path.read_bytes() for species,path in ASSETS.items()}
        cls.skins={species:Model(blob) for species,blob in cls.blobs.items()}
        cls.models={species:author.decode_ladybird(blob,species) for species,blob in cls.blobs.items()}

    def test_canonical_bytes_geometry_and_limits(self):
        for species,blob in self.blobs.items():
            m=self.skins[species]
            self.assertEqual(blob,author.export_ladybird(species))
            self.assertEqual(author.geometry_digest(self.models[species]),author.REST_DIGESTS[species])
            self.assertEqual(len(m.parts),36 if species=='ledyba' else 38)
            self.assertEqual(sum(map(len,m.vertices)),18127 if species=='ledyba' else 19044)
            self.assertEqual(len(m.joints),14 if species=='ledyba' else 16)
            self.assertLess(len(blob),1024*1024)
            self.assertLessEqual(len(m.doc['accessors']),256)

    def test_hierarchy_and_fitted_layer_joint_owners(self):
        for species,m in self.skins.items():
            parents=[None,0,1,2,3,3,2,2,1,1,1,1,1,1]+([2,2] if species=='ledian' else [])
            self.assertEqual(m.parents[:-1],parents)
            feet=0
            for p,name in enumerate(m.names):
                side=0 if '-1' in name.split() else 1
                owner=None
                if name.startswith('Foot /'):owner=0;feet+=1
                elif name.startswith(('Arm /','Hand /','Glove seam /')):owner=(6 if name.split()[-1]=='0' else 8)+side
                elif name.startswith(('Head /','Head spot /','Face /','Eye /','Pupil /','Glint /','Mouth /')):owner=3
                elif name.startswith('Antenna /'):owner=4+side
                elif name.startswith(('Shell /','Shell spot /')):owner=12+side
                elif name.startswith('Wing /'):owner=14+side
                if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(species,name))
                for ids,weights in zip(m.ids[p],m.weights[p]):
                    self.assertTrue(all(i<len(m.joints) for i in ids))
                    self.assertAlmostEqual(sum(weights),1.,places=7)
            self.assertEqual(feet,2)

    def test_floor_fixed_feet_winding_and_edge_strain(self):
        for species,m in self.skins.items():
            for clip in m.clips:
                for step in range(9):
                    mats=m.matrices(clip,step/8)
                    for p,name in enumerate(m.names):
                        posed=m.posed(p,mats);normals=m.posed(p,mats,True)
                        self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(species,clip,step,name))
                        if name.startswith('Foot /'):
                            for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-12)
                        for i in range(0,len(m.indices[p]),3):
                            tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(species,clip,step,name,i))
                            self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
                                self.assertTrue(.7<ratio<1.3,(species,clip,step,name,ratio))

    def test_neutral_endpoints_and_face_fist_feeler_case_wing_motion(self):
        for species,m in self.skins.items():
            for clip in m.clips:
                for t in (0.,1.):
                    mats=m.matrices(clip,t)
                    for p,vs in enumerate(m.vertices):
                        for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
                mats=m.matrices(clip,.28)
                prefixes=['Eye /','Hand /','Antenna /','Shell /']+(['Wing /'] if species=='ledian' else [])
                for prefix in prefixes:
                    p=next(i for i,n in enumerate(m.names) if n.startswith(prefix))
                    self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[p],m.posed(p,mats))),.001,(species,clip,prefix))
            for animation in m.doc['animations']:
                self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
                idle=animation['name'].endswith('.idle')
                self.assertEqual(animation['extras']['loopSuggested'],idle)
                self.assertEqual(animation['extras']['cueRelative'],not idle)

    def test_wing_cases_open_outward_from_the_rear_center_seam(self):
        for species,m in self.skins.items():
            for p,name in enumerate(m.names):
                if not name.startswith('Shell /'):continue
                sign=-1 if '-1' in name.split() else 1
                seam=[i for i,v in enumerate(m.vertices[p]) if abs(v[0])<1e-7]
                self.assertTrue(seam,(species,name))
                rear=min(seam,key=lambda i:m.vertices[p][i][2])
                for clip in m.clips:
                    for step in range(9):
                        moved=m.posed(p,m.matrices(clip,step/8))[rear]
                        self.assertGreaterEqual(sign*moved[0],-1e-7,(species,clip,step,name))
                    moved=m.posed(p,m.matrices(clip,.28))[rear]
                    self.assertGreater(sign*moved[0],.0005,(species,clip,name))

if __name__=='__main__':unittest.main()
