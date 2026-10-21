#!/usr/bin/env python3
"""Check the stored Ampharos sculpture's topology, shading and anatomy."""
import json,math,os,unittest
from collections import Counter,defaultdict
from pathlib import Path
from electric_sheep_glb import read_ampharos,geometry_digest
from ampharos_sculpt import sculpture,HEIGHT
from chikorita_sculpt import sub,cross,dot

ASSET=Path(os.environ.get('CRYSTAL_ELECTRIC_SHEEP_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))/'ampharos.glb'

class AmpharosTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.model=read_ampharos(ASSET);cls.parts=cls.model['primitives']

    def test_canonical_recipe_and_grounded_physical_datum(self):
        self.assertEqual(geometry_digest(self.model),geometry_digest(sculpture()))
        points=[tuple(p['positions'][i:i+3]) for p in self.parts for i in range(0,len(p['positions']),3)]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),HEIGHT,places=6)
        self.assertEqual(self.model['name'],'ampharos')
        self.assertLess(len(ASSET.read_bytes()),650000)
        self.assertLess(sum(len(p['indices'])//3 for p in self.parts),6000)

    def test_each_anatomy_part_is_one_closed_oriented_solid(self):
        for p in self.parts:
            points=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
            edges=Counter();directions=defaultdict(int);adj=defaultdict(set);volume=0.
            for i in range(0,len(p['indices']),3):
                a,b,c=[points[j] for j in p['indices'][i:i+3]]
                n=cross(sub(b,a),sub(c,a));self.assertGreater(dot(n,n),1e-22,p['part'])
                volume+=dot(a,cross(b,c))
                for x,y in ((a,b),(b,c),(c,a)):
                    edge=tuple(sorted((x,y)));edges[edge]+=1;directions[edge]+=1 if x<y else -1;adj[x].add(y);adj[y].add(x)
            self.assertTrue(all(n==2 for n in edges.values()),p['part'])
            self.assertTrue(all(n==0 for n in directions.values()),p['part'])
            self.assertGreater(volume,1e-11,p['part'])
            seen=set();queue=[next(iter(adj))]
            while queue:
                a=queue.pop()
                if a in seen:continue
                seen.add(a);queue.extend(adj[a]-seen)
            self.assertEqual(len(seen),len(adj),p['part'])

    def test_unit_normals_agree_with_real_triangle_winding(self):
        for p in self.parts:
            ps=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
            ns=[tuple(p['normals'][i:i+3]) for i in range(0,len(p['normals']),3)]
            self.assertTrue(all(math.isfinite(v) for q in ps+ns for v in q))
            for n in ns:self.assertAlmostEqual(dot(n,n),1.,places=5)
            for i in range(0,len(p['indices']),3):
                ids=p['indices'][i:i+3];a,b,c=[ps[j] for j in ids];n=cross(sub(b,a),sub(c,a))
                normal=tuple(sum(ns[j][k] for j in ids) for k in range(3))
                self.assertGreaterEqual(dot(n,normal),-1e-8,(p['part'],i//3))

    def test_connected_body_and_distinct_fitted_details(self):
        names=[p['part'] for p in self.parts];self.assertEqual(len(names),len(set(names)))
        self.assertEqual(sum(n.startswith('Body /') for n in names),1)
        for prefix,count in [('Foot /',2),('Haunch /',2),('Flipper /',2),('Neck /',3),('Horn / rounded',2),('Horn / fitted',2),('Eye / black',2),('Tail / fitted',3)]:
            self.assertEqual(sum(n.startswith(prefix) for n in names),count)
        belly=next(p for p in self.parts if p['part'].startswith('Belly /'))
        # The front of the lenticular panel faces outward. An inverted closed
        # panel can pass volume checks while hiding all of the ivory chest.
        ids=belly['indices'][:3];normal=[sum(belly['normals'][j*3+k] for j in ids) for k in range(3)]
        self.assertGreater(normal[2],0.)
        feet=[p for p in self.parts if p['part'].startswith('Foot /')]
        self.assertTrue(all(min(p['positions'][1::3])==0 for p in feet))

    def test_projecting_muzzle_long_chest_and_two_haunches(self):
        nostrils=[p for p in self.parts if p['part'].startswith('Nostril /')]
        for p in nostrils:self.assertGreater(sum(p['positions'][2::3])/(len(p['positions'])//3),.17*HEIGHT)
        belly=next(p for p in self.parts if p['part'].startswith('Belly /'))
        self.assertGreater(max(belly['positions'][1::3])-min(belly['positions'][1::3]),.33*HEIGHT)
        for side in (-1,1):
            haunch=next(p for p in self.parts if p['part'].startswith('Haunch /') and p['part'].endswith(' '+str(side)))
            self.assertGreater(max(haunch['positions'][1::3])-min(haunch['positions'][1::3]),.22*HEIGHT)

    def test_upright_haunches_and_flat_feet_under_hips(self):
        from sculpt_test_support import horizontal_section_bounds
        for side in (-1,1):
            haunch=next(p for p in self.parts if p['part'].startswith('Haunch /') and p['part'].endswith(' '+str(side)))
            sections=[horizontal_section_bounds(haunch,y*HEIGHT) for y in (.085,.15,.23)]
            depths=[(lo[1]+hi[1])/2 for lo,hi in sections]
            # In the game profiles the thigh bulges outward, while the ankle
            # stays beneath it. A toe's forward reach must not tilt the leg.
            self.assertLess(max(depths)-min(depths),.02*HEIGHT)
            self.assertLessEqual(depths[0]-depths[-1],.012*HEIGHT)
            foot=next(p for p in self.parts if p['part'].startswith('Foot /') and p['part'].endswith(' '+str(side)))
            self.assertGreater(sum(foot['positions'][2::3])/(len(foot['positions'])//3)-depths[0],.02*HEIGHT)
            self.assertLess(max(foot['positions'][1::3])-min(foot['positions'][1::3]),.075*HEIGHT)

    def test_dropped_muzzle_lateral_eyes_and_low_banded_ears(self):
        from sculpt_test_support import indexed_ray_intersections
        body=next(p for p in self.parts if p['part'].startswith('Body /'))
        def center(p):return tuple(sum(p['positions'][axis::3])/(len(p['positions'])//3) for axis in range(3))
        smile=next(p for p in self.parts if p['part'].startswith('Muzzle /'))
        for side in (-1,1):
            eye=next(p for p in self.parts if p['part'].startswith('Eye / black') and p['part'].endswith(' '+str(side)))
            nose=next(p for p in self.parts if p['part'].startswith('Nostril /') and p['part'].endswith(' '+str(side)))
            e,n=center(eye),center(nose)
            self.assertGreater(e[1]-n[1],.10*HEIGHT)
            self.assertGreater(n[2]-e[2],.075*HEIGHT)
            self.assertGreater(abs(e[0]),.08*HEIGHT)
            # Probe actual occupied muzzle sections, rather than treating a
            # tiny nose decal projected in front of the neck as enough depth.
            for y in (n[1],center(smile)[1]):
                hits=sorted({round(z,8) for z in indexed_ray_intersections(body,0.,y)})
                self.assertGreaterEqual(len(hits),2)
                self.assertGreater(hits[-1]-hits[-2],.05*HEIGHT)
            ear=next(p for p in self.parts if p['part'].startswith('Horn / rounded') and p['part'].endswith(' '+str(side)))
            self.assertLessEqual(max(ear['positions'][1::3]),max(body['positions'][1::3])+.003*HEIGHT)

if __name__=='__main__':unittest.main()
