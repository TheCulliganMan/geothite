"""Standard-library closed surface helpers for original static model recipes.

Geometry only: species palettes, anatomical profiles and features live in their
own recipes. Coordinates are +Y up, +Z front. No generated mesh catalog.
"""
import math
from chikorita_sculpt import mesh,linear,add,sub,mul,dot,cross,unit

def part(name,color,vertices,faces,smooth=False):
    p=mesh(name,'body',vertices,faces,smooth)
    p['base_color']=linear(color);return p


def loft(name,color,profiles,segments=20):
    v=[(rx*math.sin(math.tau*i/segments),y,z+rz*math.cos(math.tau*i/segments))
       for y,rx,rz,z in profiles for i in range(segments)]
    f=[tuple(reversed(range(segments))),tuple((len(profiles)-1)*segments+i for i in range(segments))]
    for k in range(len(profiles)-1):
        for i in range(segments):
            a=k*segments+i;b=k*segments+(i+1)%segments
            f.append((a,b,b+segments,a+segments))
    return part(name,color,v,f,True)


def ellipsoid(name,color,center,radii,segments=16,rings=10):
    # Small nonzero polar rings retain a closed, nondegenerate authored solid.
    return translate(loft(name,color,[(center[1]+radii[1]*math.cos(math.pi*(i+.025)/(rings+.05)),radii[0]*math.sin(math.pi*(i+.025)/(rings+.05)),radii[2]*math.sin(math.pi*(i+.025)/(rings+.05)),center[2]) for i in reversed(range(rings+1))],segments),center[0],0,0)


def translate(p,x,y,z):
    p['positions']=[value+(x,y,z)[i%3] for i,value in enumerate(p['positions'])];return p


def tube(name,color,points,radii,segments=12):
    # Parallel-transport ring frame prevents abrupt striped-tail twists.
    v=[];f=[];side=None
    for k,p in enumerate(points):
        tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]))
        if side is None:side=cross(tangent,(0,1,0) if abs(tangent[1])<.9 else (1,0,0))
        side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
        v.extend(add(p,mul(add(mul(side,math.cos(i*math.tau/segments)),mul(other,math.sin(i*math.tau/segments))),radii[k])) for i in range(segments))
    f=[tuple(reversed(range(segments))),tuple((len(points)-1)*segments+i for i in range(segments))]
    for k in range(len(points)-1):
        for i in range(segments):
            a=k*segments+i;b=k*segments+(i+1)%segments;f.append((a,b,b+segments,a+segments))
    return part(name,color,v,f,True)


def fitted_patch(name,color,outline,surface,thickness=.008):
    """Closed shallow marking follows a real surface instead of floating on it.

    The XY outline must be convex. Three concentric front/back rings preserve
    the underlying curvature, and shared edges give a closed oriented solid.
    """
    outline=list(outline)
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:
        outline.reverse()
    cx=sum(x for x,y in outline)/len(outline);cy=sum(y for x,y in outline)/len(outline)
    xy=[(cx,cy)]
    for radius in (.33,.67,1.):xy.extend((cx+(x-cx)*radius,cy+(y-cy)*radius) for x,y in outline)
    front=[(x,y,surface(y,x)+thickness) for x,y in xy]
    back=[(x,y,surface(y,x)-thickness*.35) for x,y in xy]
    n=len(outline);count=len(front);faces=[]
    for i in range(n):faces.append((0,1+i,1+(i+1)%n))
    for ring in range(2):
        for i in range(n):
            a=1+ring*n+i;b=1+ring*n+(i+1)%n;faces.append((a,a+n,b+n,b))
    faces += [tuple(i+count for i in reversed(f)) for f in list(faces)]
    last=1+2*n
    faces += [(last+(i+1)%n,last+i,last+i+count,last+(i+1)%n+count) for i in range(n)]
    return part(name,color,front+back,faces)


def subdivided_part(name,color,vertices,faces,steps=1,smooth=False):
    """Chamfer a closed cage with bounded Catmull-Clark subdivision.

    Operates on authored polygon faces before triangulation. This changes the
    actual silhouette, rather than just hiding facets with lighting normals.
    """
    vertices=[tuple(v) for v in vertices];faces=[tuple(f) for f in faces]
    def mean(points):return tuple(sum(p[k] for p in points)/len(points) for k in range(3))
    for _ in range(steps):
        face_points=[mean([vertices[i] for i in f]) for f in faces]
        edges={};touch=[[] for v in vertices]
        for index,f in enumerate(faces):
            for a in f:touch[a].append(index)
            for a,b in zip(f,f[1:]+f[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append(index)
        if any(len(fs)!=2 for fs in edges.values()):raise ValueError('subdivision requires a closed cage')
        neighbors=[[] for v in vertices]
        for (a,b),fs in edges.items():
            middle=mean([vertices[a],vertices[b]]);neighbors[a].append(middle);neighbors[b].append(middle)
        new=[]
        for i,v in enumerate(vertices):
            n=len(touch[i]);f=mean([face_points[j] for j in touch[i]]);r=mean(neighbors[i])
            new.append(tuple((f[k]+2*r[k]+(n-3)*v[k])/n for k in range(3)))
        edge_ids={}
        for edge,fs in edges.items():edge_ids[edge]=len(new);new.append(mean([vertices[i] for i in edge]+[face_points[j] for j in fs]))
        face_start=len(new);new.extend(face_points);next_faces=[]
        for j,f in enumerate(faces):
            for i,a in enumerate(f):
                b=f[(i+1)%len(f)];c=f[i-1]
                next_faces.append((a,edge_ids[tuple(sorted((a,b)))],face_start+j,edge_ids[tuple(sorted((a,c)))]))
        vertices,faces=new,next_faces
    return part(name,color,vertices,faces,smooth)


def curved_profiles(profiles,steps=3):
    """Monotone cubic contours retain authored knots and avoid radius overshoot."""
    profiles=list(profiles);ys=[p[0] for p in profiles];columns=[]
    for axis in range(1,4):
        values=[p[axis] for p in profiles]
        slopes=[(b-a)/(ys[i+1]-ys[i]) for i,(a,b) in enumerate(zip(values,values[1:]))]
        tangents=[slopes[0]]
        for i in range(1,len(values)-1):
            a,b=slopes[i-1:i+1];h0=ys[i]-ys[i-1];h1=ys[i+1]-ys[i]
            tangents.append(0. if a*b<=0 else (3*(h0+h1))/((2*h1+h0)/a+(h1+2*h0)/b))
        tangents.append(slopes[-1]);columns.append((values,tangents))
    out=[]
    for i in range(len(profiles)-1):
        h=ys[i+1]-ys[i]
        for j in range(steps):
            t=j/steps;row=[ys[i]+t*h]
            for values,tangents in columns:
                value=(2*t**3-3*t*t+1)*values[i]+(t**3-2*t*t+t)*h*tangents[i]+(-2*t**3+3*t*t)*values[i+1]+(t**3-t*t)*h*tangents[i+1]
                row.append(max(min(values[i:i+2]),min(max(values[i:i+2]),value)))
            out.append(tuple(row))
    return out+[profiles[-1]]


def crease_normals(p,degrees=40.):
    """Smooth connected curved faces, preserving sharp folds and exact geometry.

    Weld only for adjacency; output still splits corners across sharp edges.
    Material regions should be split after this function to avoid shading seams.
    """
    from chikorita_sculpt import f32
    ps=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
    ts=[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)]
    areas=[cross(sub(ps[b],ps[a]),sub(ps[c],ps[a])) for a,b,c in ts];normals=[unit(n) for n in areas]
    incident={};edges={}
    for i,tri in enumerate(ts):
        coords=[ps[j] for j in tri]
        for v in coords:incident.setdefault(v,set()).add(i)
        for a,b in zip(coords,coords[1:]+coords[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append(i)
    links={v:{} for v in incident};threshold=math.cos(math.radians(degrees))
    for (a,b),faces in edges.items():
        if len(faces)!=2:continue
        i,j=faces
        if dot(normals[i],normals[j])<threshold:continue
        for v in (a,b):
            links[v].setdefault(i,set()).add(j);links[v].setdefault(j,set()).add(i)
    out=dict(p);out['positions']=[];out['normals']=[];out['indices']=[];lookup={};cache={}
    for i,tri in enumerate(ts):
        for old in tri:
            v=ps[old];key=(v,i)
            if key not in cache:
                seen=set();queue=[i]
                while queue:
                    j=queue.pop()
                    if j in seen or dot(normals[i],normals[j])<threshold:continue
                    seen.add(j);queue.extend(links[v].get(j,set())-seen)
                n=unit(tuple(sum(areas[j][k] for j in sorted(seen)) for k in range(3)))
                cache[key]=tuple(f32(c) for c in n)
            n=cache[key];key=v+n
            if key not in lookup:
                lookup[key]=len(out['positions'])//3;out['positions'].extend(v);out['normals'].extend(n)
            out['indices'].append(lookup[key])
    return out
