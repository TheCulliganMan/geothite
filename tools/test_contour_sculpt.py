#!/usr/bin/env python3
"""Actual crease preservation, smooth skin normals and contour refinement."""
import unittest
from sculpt_geometry import part,tube,crease_normals,curved_profiles
from chikorita_sculpt import dot

class ContourTests(unittest.TestCase):
    def test_smoothing_keeps_original_triangles_and_sharp_cube_edges(self):
        vertices=[(x,y,z) for z in (-1.,1.) for y in (-1.,1.) for x in (-1.,1.)]
        faces=[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)]
        p=part('box',(1,1,1),vertices,faces);q=crease_normals(p)
        def triangles(m):
            ps=[tuple(m['positions'][i:i+3]) for i in range(0,len(m['positions']),3)]
            return [tuple(ps[j] for j in m['indices'][i:i+3]) for i in range(0,len(m['indices']),3)]
        self.assertEqual(triangles(p),triangles(q))
        for i in range(0,len(q['normals']),3):
            n=q['normals'][i:i+3];self.assertEqual(sorted(abs(v) for v in n),[0.,0.,1.])
    def test_round_tube_smooths_sides_but_keeps_cap_normals_separate(self):
        p=tube('tube',(1,1,1),[(0,0,0),(0,1,0)],[1,1],24);q=crease_normals(p)
        samples={}
        for i in range(0,len(q['positions']),3):
            v=tuple(q['positions'][i:i+3]);n=tuple(q['normals'][i:i+3]);samples.setdefault(v,set()).add(n)
        for (x,y,z),normals in samples.items():
            cap=[n for n in normals if abs(n[1])>.99];side=[n for n in normals if abs(n[1])<.01]
            self.assertTrue(cap);self.assertTrue(side)
            for n in side:self.assertGreater(dot(n,(x,0,z)),.98)
    def test_cubic_profiles_keep_knots_and_radius_bounds(self):
        p=[(0.,.01,.02,0.),(.25,.32,.24,.05),(.60,.25,.21,.02),(1.,.02,.03,0.)]
        rows=curved_profiles(p,4)
        for knot in p:self.assertIn(knot,rows)
        self.assertNotAlmostEqual(rows[5][1],p[1][1]*.75+p[2][1]*.25,places=6)
        for i in range(len(p)-1):
            for row in rows[i*4:(i+1)*4]:
                for axis in (1,2,3):
                    self.assertGreaterEqual(row[axis],min(p[i][axis],p[i+1][axis]))
                    self.assertLessEqual(row[axis],max(p[i][axis],p[i+1][axis]))

if __name__=='__main__':unittest.main()
