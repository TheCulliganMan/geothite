"""Independent stored-mesh topology and shading checks shared by sculpture tests."""
import math
from collections import Counter,defaultdict
from chikorita_sculpt import cross,sub,dot

def assert_closed_models(self,models):
    for name,m in models.items():
        for p in m['primitives']:
            with self.subTest(species=name,part=p['part']):
                points=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
                edges=Counter();directions=defaultdict(int);adj=defaultdict(set);volume=0
                for i in range(0,len(p['indices']),3):
                    a,b,c=[points[j] for j in p['indices'][i:i+3]];n=cross(sub(b,a),sub(c,a))
                    self.assertGreater(dot(n,n),1e-22);volume+=dot(a,cross(b,c))
                    for x,y in ((a,b),(b,c),(c,a)):
                        edge=tuple(sorted((x,y)));edges[edge]+=1;directions[edge]+=1 if x<y else -1
                        adj[x].add(y);adj[y].add(x)
                self.assertTrue(all(n==2 for n in edges.values()))
                self.assertTrue(all(n==0 for n in directions.values()))
                self.assertGreater(volume,1e-11)
                seen=set();queue=[next(iter(adj))]
                while queue:
                    a=queue.pop()
                    if a in seen:continue
                    seen.add(a);queue.extend(adj[a]-seen)
                self.assertEqual(len(seen),len(adj))


def assert_model_shading(self,models):
    for name,m in models.items():
        for p in m['primitives']:
            with self.subTest(species=name,part=p['part']):
                ps=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
                ns=[tuple(p['normals'][i:i+3]) for i in range(0,len(p['normals']),3)]
                self.assertTrue(all(math.isfinite(v) for q in ps+ns for v in q))
                for n in ns:self.assertAlmostEqual(dot(n,n),1,places=5)
                for i in range(0,len(p['indices']),3):
                    ids=p['indices'][i:i+3];a,b,c=[ps[j] for j in ids];n=cross(sub(b,a),sub(c,a))
                    avg=tuple(sum(ns[j][k] for j in ids) for k in range(3))
                    self.assertGreaterEqual(dot(n,avg),-1e-8,(name,p['part'],i//3))




def ray_intersections(part, x, y):
    """Independent +Z ray hits on stored triangles, without authoring surface math."""
    positions = [part['positions'][i:i+3] for i in range(0, len(part['positions']), 3)]
    hits = []
    for i in range(0, len(part['indices']), 3):
        a, b, c = [positions[j] for j in part['indices'][i:i+3]]
        denominator = (b[1]-c[1])*(a[0]-c[0]) + (c[0]-b[0])*(a[1]-c[1])
        if abs(denominator) < 1e-12:
            continue
        u = ((b[1]-c[1])*(x-c[0]) + (c[0]-b[0])*(y-c[1])) / denominator
        v = ((c[1]-a[1])*(x-c[0]) + (a[0]-c[0])*(y-c[1])) / denominator
        if min(u, v, 1-u-v) >= -1e-7:
            hits.append(u*a[2] + v*b[2] + (1-u-v)*c[2])
    return hits
