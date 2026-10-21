#!/usr/bin/env python3
"""A continuous pear-bodied paper Chikorita, authored without Blender.

An explicit quad cage joins all four short legs to one raised-head body. One
Catmull-Clark refinement rounds those transitions while retaining a restrained
faceted paper surface. The leaf is a thin closed, curved, folded blade, not an
extruded sprite. Runtime coordinates: +Y up, +Z front; four feet contact Y=0.
Original Crystal palette is expressed as sRGB colors, then exported linearly.
"""
import math
import struct

from skin_glb import COORDINATES

PALETTE = {'body': (193/255,214/255,124/255), 'leaf': (62/255,151/255,35/255),
           'leaf_fold': (30/255,151/255,8/255), 'ink': (.035,.043,.025),
           'white': (1.,1.,.97), 'iris': (.73,.10,.22)}
FULL_HEIGHT = 1.
BODY_HEIGHT = .688
BODY = 'Body / continuous raised head throat pear and four supporting legs'
LEAF = 'Leaf / curved asymmetric closed folded blade'
STEM = 'Leaf / short fitted petiole'
TAIL = 'Tail / blunt upturned body-colored nub'


def add(a,b): return tuple(x+y for x,y in zip(a,b))
def sub(a,b): return tuple(x-y for x,y in zip(a,b))
def mul(a,k): return tuple(x*k for x in a)
def dot(a,b): return sum(x*y for x,y in zip(a,b))
def cross(a,b): return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def unit(a): return mul(a,1/math.sqrt(dot(a,a)))
def mean(points): return tuple(sum(p[k] for p in points)/len(points) for k in range(len(points[0])))
def f32(v): return struct.unpack('<f',struct.pack('<f',v))[0]
def linear(c): return [f32(v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4) for v in c]+[1.]


def subdivide(vertices,faces):
    """One deterministic closed-manifold Catmull-Clark cage refinement."""
    centers=[mean([vertices[i] for i in f]) for f in faces]
    edges={}; incident=[[] for _ in vertices]; neighbors=[set() for _ in vertices]
    for fi,f in enumerate(faces):
        for i,j in zip(f,f[1:]+f[:1]):
            edges.setdefault(tuple(sorted((i,j))),[]).append(fi)
            incident[i].append(fi);neighbors[i].add(j);neighbors[j].add(i)
    assert all(len(fs)==2 for fs in edges.values()), 'body cage must be a closed manifold'
    out=[]
    for i,p in enumerate(vertices):
        n=len(incident[i]); face=mean([centers[f] for f in incident[i]])
        edge=mean([mul(add(p,vertices[j]),.5) for j in neighbors[i]])
        out.append(mul(add(add(face,mul(edge,2)),mul(p,n-3)),1/n))
    edgeids={}
    for edge,fs in edges.items():
        edgeids[edge]=len(out);out.append(mean([vertices[i] for i in edge]+[centers[f] for f in fs]))
    first=len(out);out+=centers;polys=[]
    for fi,f in enumerate(faces):
        for k,i in enumerate(f):polys.append((i,edgeids[tuple(sorted((i,f[(k+1)%len(f)])))],first+fi,edgeids[tuple(sorted((f[k-1],i))) ]))
    return out,polys


def mesh(name,color,vertices,faces,smooth=False):
    triangles=[(f[0],f[i],f[i+1]) for f in faces for i in range(1,len(f)-1)]
    if sum(dot(vertices[a],cross(vertices[b],vertices[c])) for a,b,c in triangles)<0:
        triangles=[(a,c,b) for a,b,c in triangles]
    norms=[(0.,0.,0.) for _ in vertices]
    if smooth:
        for a,b,c in triangles:
            n=cross(sub(vertices[b],vertices[a]),sub(vertices[c],vertices[a]))
            for i in (a,b,c):norms[i]=add(norms[i],n)
        norms=[unit(n) for n in norms]
    p=[];n=[];idx=[];lookup={}
    for tri in triangles:
        face=unit(cross(sub(vertices[tri[1]],vertices[tri[0]]),sub(vertices[tri[2]],vertices[tri[0]])))
        for i in tri:
            normal=norms[i] if smooth else face
            key=tuple(f32(x) for x in vertices[i]+normal)
            if key not in lookup:
                lookup[key]=len(p)//3;p.extend(key[:3]);n.extend(key[3:])
            idx.append(lookup[key])
    return {'part':name,'positions':p,'normals':n,'indices':idx,'base_color':linear(PALETTE[color])}


def body_cage():
    count=16
    # Height, Z-center, X-radius, Z-radius. This is one body through the
    # raised head: there is no intersecting head sphere or narrow neck join.
    profiles=((.062,-.065,.200,.290),(.171,-.060,.250,.350),
              (.340,-.055,.245,.350),(.455,.100,.210,.220),
              (.555,.160,.180,.160),(.635,.170,.126,.112),
              (.684,.167,.045,.042))
    vertices=[(w*math.cos(math.tau*i/count),y,z+d*math.sin(math.tau*i/count))
              for y,z,w,d in profiles for i in range(count)]
    faces=[tuple(range(count-1,-1,-1)),tuple(range((len(profiles)-1)*count,len(profiles)*count))]
    # Four two-panel shoulder/haunch openings are bridged into the leg cage.
    holes=(1,5,9,13)
    for row in range(len(profiles)-1):
        for i in range(count):
            if row==0 and any(i in (h,(h+1)%count) for h in holes):continue
            faces.append((row*count+i,row*count+(i+1)%count,(row+1)*count+(i+1)%count,(row+1)*count+i))
    for h in holes:
        ids=(h,(h+1)%count,(h+2)%count,(h+2)%count+count,(h+1)%count+count,h+count)
        center=mean([vertices[i] for i in ids]);sx=1 if center[0]>0 else -1;sz=1 if center[2]>0 else -1
        cx=sx*.185;cz=.174 if sz>0 else -.270
        # The initial ring faces outwards; the bottom half continues inward
        # under the belly, and the upper half rounds the shoulder down outside.
        direction=unit((center[0],0.,center[2]+.033)); tangent=(-direction[2],0.,direction[0])
        # Match the perimeter's direction to avoid a twisted leg sleeve.
        if dot(sub(vertices[ids[2]],vertices[ids[0]]),tangent)<0:tangent=mul(tangent,-1)
        angles=(math.radians(210),math.radians(270),math.radians(330),math.radians(30),math.radians(90),math.radians(150))
        previous=ids
        for yy,radx,radz in ((.055,.083,.095),(.010,.080,.092),(-.014,.061,.077)):
            ring=[]
            for a in angles:
                lateral=math.cos(a);outward=math.sin(a)
                ring.append(len(vertices));vertices.append((cx+tangent[0]*lateral*radx+direction[0]*outward*radx,
                    yy,cz+tangent[2]*lateral*radz+direction[2]*outward*radz))
            # The hole boundary continues down the sleeve; the final cap
            # closes it with the opposite directed edges to the sleeve.
            for a,b,c,d in zip(previous,previous[1:]+previous[:1],ring[1:]+ring[:1],ring):faces.append((a,b,c,d))
            previous=tuple(ring)
        faces.append(previous)
    vertices,faces=subdivide(vertices,faces)
    # Keep the low contact ring absolutely coplanar after smoothing. The
    # soles have real area; the body/leaf are never recentered by their bounds.
    vertices=[(x,0. if y<.020 else y,z) for x,y,z in vertices]
    high=max(p[1] for p in vertices);vertices=[(x,y*BODY_HEIGHT/high,z) for x,y,z in vertices]
    return vertices,faces


def loft(name,color,rings):
    n=len(rings[0]);vertices=[p for r in rings for p in r]
    faces=[tuple(range(n-1,-1,-1)),tuple(range((len(rings)-1)*n,len(rings)*n))]
    faces += [(r*n+i,r*n+(i+1)%n,(r+1)*n+(i+1)%n,(r+1)*n+i) for r in range(len(rings)-1) for i in range(n)]
    return mesh(name,color,vertices,faces)


def tube(name,color,points,radii,segments=8):
    rings=[]
    for k,p in enumerate(points):
        tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]));seed=(1.,0.,0.) if abs(tangent[0])<.85 else (0.,0.,1.)
        u=unit(cross(tangent,seed));v=cross(tangent,u)
        rings.append([add(p,add(mul(u,radii[k]*math.cos(math.tau*i/segments)),mul(v,radii[k]*math.sin(math.tau*i/segments)))) for i in range(segments)])
    return loft(name,color,rings)


def front_at(x,y,vertices,faces):
    hits=[]
    for f in faces:
        for i in range(1,len(f)-1):
            a,b,c=[vertices[j] for j in (f[0],f[i],f[i+1])]
            det=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
            if abs(det)<1e-12:continue
            u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/det
            v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/det;w=1-u-v
            if min(u,v,w)>-1e-7:hits.append(u*a[2]+v*b[2]+w*c[2])
    if not hits:raise ValueError('facial panel lies outside the continuous head')
    return max(hits)


def collar_surface(y,angle,body):
    # Ray-cast only the authored body, so each seed sits in the local throat
    # surface. A fixed ellipse buries the rear seeds in the low pear's back.
    origin=(0.,y,.055);direction=(math.cos(angle),0.,math.sin(angle));hits=[]
    vertices,faces=body
    for face in faces:
        for i in range(1,len(face)-1):
            a,b,c=[vertices[j] for j in (face[0],face[i],face[i+1])]
            e1,e2=sub(b,a),sub(c,a);h=cross(direction,e2);det=dot(e1,h)
            if abs(det)<1e-10:continue
            inv=1/det;s=sub(origin,a);u=inv*dot(s,h)
            if not 0.<=u<=1.:continue
            q=cross(s,e1);v=inv*dot(direction,q)
            if v<0. or u+v>1.:continue
            distance=inv*dot(e2,q)
            if distance>0:hits.append(distance)
    if not hits:raise ValueError('collar seed has no body surface')
    return add(origin,mul(direction,max(hits)-.004))


def face_patch(name,color,outline,body,offset=.003):
    """Clip every artwork face to the actual stored skull triangles."""
    from sculpt_geometry import fitted_triangle_contour
    parent=mesh(BODY,'body',*body)
    p=fitted_triangle_contour(name,PALETTE[color],outline,parent,offset,precision=8)
    # Normals describe serialized positions, including narrow clipped corners.
    vertices=list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]))
    return mesh(name,color,vertices,[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)])


# Root-to-tip path and blade half widths. The upper central crease has the
# explicit height 1.0; no later bounding-box normalisation changes body scale.
LEAF_PATH=((0.,.715,.152,.013),(.003,.803,.105,.058),(.009,.895,.025,.115),
           (.018,.963,-.090,.158),(.029,.988,-.221,.151),(.042,.947,-.350,.110),
           (.053,.858,-.452,.050),(.060,.769,-.508,.002))


def gaze(side,cx,cy,outline,body,fit,kind='upright',iris='red'):
    """Five nested fitted tissues on the actual skull facets."""
    label='left' if side<0 else 'right'
    parts=[fit('Eye / '+label+' fitted '+kind+' ink','ink',outline,body,.003)]
    parts.append(fit('Eye / '+label+' fitted ivory sclera','white',
        [(cx+(x-cx)*.83,cy+(y-cy)*.85) for x,y in outline],body,.0045))
    rx=(max(x for x,y in outline)-min(x for x,y in outline))*.275
    ry=(max(y for x,y in outline)-min(y for x,y in outline))*.345
    def oval(x,y,w,h):return [(x+w*math.cos(math.tau*j/16),y+h*math.sin(math.tau*j/16)) for j in range(16)]
    ex=cx+side*.0015;ey=cy+.002
    parts.append(fit('Eye / '+label+' fitted '+iris+' iris','iris',oval(ex,ey,rx,ry),body,.0058))
    parts.append(fit('Eye / '+label+' fitted narrow pupil','ink',oval(ex,ey,rx*.45,ry*.78),body,.0070))
    parts.append(fit('Eye / '+label+' catchlight','white',oval(ex-side*rx*.16,ey+ry*.43,rx*.25,ry*.20),body,.0082))
    return parts


def small_claw(cx,cz,index):
    x=cx+(-.025,0.,.025)[index];z=cz+.068
    vertices=[(x-.009,.012,z-.014),(x+.009,.012,z-.014),(x+.009,.008,z+.018),(x-.009,.008,z+.018),
              (x-.008,.030,z-.009),(x+.008,.030,z-.009),(x+.006,.022,z+.020),(x-.006,.022,z+.020)]
    return mesh('Claw / '+('left ' if cx<0 else 'right ')+('front ' if cz>0 else 'rear ')+str(index+1),
        'white',vertices,[(0,3,2,1),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7)])


def leaf_mesh():
    vertices=[]
    for k,(x,y,z,width) in enumerate(LEAF_PATH):
        # An asymmetric broad side and a narrower far side wrap around a
        # raised lengthwise fold; thickness is only 0.003 runtime units.
        fold=.012 if k not in (0,len(LEAF_PATH)-1) else .001
        vertices += [(x-width,y-.022,z-.009),(x,y+fold,z),
                     (x+width*.88,y-.030,z+.017)]
    n=len(vertices);vertices += [sub(p,(0.,.003,0.)) for p in vertices]
    faces=[]
    for k in range(len(LEAF_PATH)-1):
        for j in range(2):
            a=3*k+j;b=a+1;c=b+3;d=a+3
            faces += [(a,b,c,d),(a+n,d+n,c+n,b+n)]
    perimeter=[3*k for k in range(len(LEAF_PATH))]+[3*(len(LEAF_PATH)-1)+1]+[3*k+2 for k in range(len(LEAF_PATH)-1,-1,-1)]+[1]
    faces += [(a,b,b+n,a+n) for a,b in zip(perimeter,perimeter[1:]+perimeter[:1])]
    return mesh(LEAF,'leaf',vertices,faces)


def sculpt():
    # Round living skin while retaining the exact cage and sharp sole edges.
    # Local import avoids the shared geometry module's base-mesh dependency.
    from sculpt_geometry import crease_normals
    body=body_cage();parts=[crease_normals(mesh(BODY,'body',*body),35.)]
    # Narrow red irises and pupils wrap the skull, within small ivory sclerae.
    for side,label in ((-1,'left'),(1,'right')):
        cx=side*.103;cy=.540
        outline=[(cx+side*dx,cy+dy) for dx,dy in ((-.025,-.047),(-.036,-.013),(-.032,.027),(-.014,.058),(.011,.061),(.027,.042),(.031,-.013),(.023,-.046),(.0,-.057))]
        parts+=gaze(side,cx,cy,outline,body,face_patch,'tall')
    # A shallow tapered paper seam: no round tube pasted on the muzzle.
    line=((- .054,.452),(-.033,.441),(0.,.437),(.033,.441),(.054,.452))
    outline=[(x,y+.0018) for x,y in line]+[(x,y-.0037) for x,y in reversed(line)]
    parts.append(face_patch('Mouth / fitted subtle smiling seam','ink',outline,body,.0035))
    # Eight compact, seated collar buds encircle the throat. They are original
    # octagonal paper seed forms; only 20 triangles each, not spheres.
    for i in range(8):
        a=math.tau*i/8;cy=.380+.025*(1.-math.sin(a));center=collar_surface(cy,a,body)
        normal=unit((math.cos(a),.13,math.sin(a)));u=(-math.sin(a),0.,math.cos(a));v=unit(cross(normal,u))
        base=[add(add(center,mul(u,.030*math.cos(math.tau*j/6))),mul(v,.032*math.sin(math.tau*j/6))) for j in range(6)]
        outer=[add(center,add(mul(sub(p,center),.83),mul(normal,.024))) for p in base]
        rings=[base,outer];parts.append(loft('Collar / seated bud '+str(i+1),'leaf',rings))
    parts.append(tube(TAIL,'body',[(0.,.260,-.350),(0.,.280,-.395),(0.,.325,-.430),(0.,.360,-.440)],(.055,.045,.027,.002),8))
    parts.append(tube(STEM,'leaf',[(0.,.653,.165),(0.,.699,.157),(.0,.741,.139)],(.019,.017,.013),8))
    parts.append(leaf_mesh())
    for cx in (-.185,.185):
        for cz in (.174,-.270):parts += [small_claw(cx,cz,i) for i in range(3)]
    # Axis-aligned coordinates and normals can retain tiny rounding residuals.
    # Serialize geometric zero consistently without changing visible anatomy.
    for part in parts:
        for key in ('positions','normals'):
            part[key]=[0. if abs(v)<1e-12 else v for v in part[key]]
    return {'name':'battle_chikorita','version':1,'coordinate_system':COORDINATES,'primitives':parts}


if __name__=='__main__':
    import argparse,json
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True);args=parser.parse_args()
    with open(args.output,'w') as f:json.dump(sculpt(),f,separators=(',',':'))
