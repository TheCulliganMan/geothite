#!/usr/bin/env python3
"""Deterministic, continuous long-necked papercraft Bayleef.

The body cage is one closed surface through four legs, chest, slender neck,
blunt muzzle, and crown. A folded crown has a real re-entrant notch; eight
individually proportioned rolled collar leaves sit on the neck's surface.
Coordinates are +Y up / +Z front. Neutral floor and full height are explicit.
"""
import math

from chikorita_sculpt import (add, sub, mul, dot, cross, unit, mean, f32, linear,
                              subdivide, front_at, mesh as _mesh)
from skin_glb import COORDINATES

PALETTE = {'body': (222/255,206/255,49/255), 'leaf': (90/255,132/255,16/255),
           'ink': (.027,.035,.014), 'white': (1.,1.,.94)}
FULL_HEIGHT=1.15
BODY_HEIGHT=.982
BODY='Body / continuous haunch chest long neck muzzle and four legs'
TAIL='Tail / short rising tapered curl'
STEM='Crown / fitted arched petiole'
LEAF='Crown / asymmetric folded blade with deep edge notch'


def mesh(name,color,vertices,faces):
    part=_mesh(name,'body',vertices,faces)
    part['base_color']=linear(PALETTE[color])
    return part


def face_patch(name,color,outline,body,offset=.0025):
    """Clip ink polygons to the body's exact facets, then give them closed rims.

    Every point on each visible face is on one actual underlying triangle plus
    a small uniform Z offset. This avoids the buried curved-brow chords produced
    by independently subdividing and projecting only artwork vertices.
    """
    def orient(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    def triangulate(points):
        active=list(range(len(points)));triangles=[]
        while len(active)>3:
            for k,b in enumerate(active):
                a,c=active[k-1],active[(k+1)%len(active)]
                if orient(points[a],points[b],points[c])<=1e-14:continue
                if any(all(orient(points[u],points[v],points[q])>=-1e-14 for u,v in ((a,b),(b,c),(c,a))) for q in active if q not in (a,b,c)):continue
                triangles.append((a,b,c));active.remove(b);break
            else:raise ValueError('degenerate fitted artwork polygon')
        if orient(*(points[i] for i in active))>1e-14:triangles.append(tuple(active))
        return triangles
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:outline=list(reversed(outline))
    convex=all(orient(outline[i-1],outline[i],outline[(i+1)%len(outline)])>=-1e-14 for i in range(len(outline)))
    art=[outline] if convex else [tuple(outline[i] for i in tri) for tri in triangulate(outline)]
    verts,faces=body;points=[];lookup={};triangles=[]
    xmin,xmax=min(p[0] for p in outline),max(p[0] for p in outline)
    ymin,ymax=min(p[1] for p in outline),max(p[1] for p in outline)
    def vertex(p):
        key=tuple(round(x,11) for x in p)
        if key not in lookup:lookup[key]=len(points);points.append(key)
        return lookup[key]
    for face in faces:
        for j in range(1,len(face)-1):
            a,b,c=[verts[i] for i in (face[0],face[j],face[j+1])]
            area=orient(a,b,c)
            if abs(area)<1e-13:continue
            if max(a[0],b[0],c[0])<xmin or min(a[0],b[0],c[0])>xmax:continue
            if max(a[1],b[1],c[1])<ymin or min(a[1],b[1],c[1])>ymax:continue
            center=mean((a,b,c))
            if abs(front_at(center[0],center[1],verts,faces)-center[2])>1e-7:continue
            if area<0:b,c=c,b;area=-area
            for tri in art:
                polygon=list(tri)
                for aa,bb in ((a,b),(b,c),(c,a)):
                    if not polygon:break
                    out=[]
                    for p,q in zip(polygon,polygon[1:]+polygon[:1]):
                        u=orient(aa,bb,p);v=orient(aa,bb,q)
                        if u>=-1e-13:out.append(p)
                        if (u>1e-13 and v< -1e-13) or (u< -1e-13 and v>1e-13):
                            t=u/(u-v);out.append((p[0]+t*(q[0]-p[0]),p[1]+t*(q[1]-p[1])))
                    polygon=[]
                    for p in out:
                        if not polygon or math.dist(p,polygon[-1])>1e-10:polygon.append(p)
                    if len(polygon)>1 and math.dist(polygon[0],polygon[-1])<1e-10:polygon.pop()
                if len(polygon)<3:continue
                if abs(sum(p[0]*q[1]-q[0]*p[1] for p,q in zip(polygon,polygon[1:]+polygon[:1])))<1e-13:continue
                indices=[]
                for x,y in polygon:
                    wa=orient(b,c,(x,y))/area;wb=orient(c,a,(x,y))/area;wc=1.-wa-wb
                    indices.append(vertex((x,y,wa*a[2]+wb*b[2]+wc*c[2]+offset)))
                for tri in triangulate(polygon):triangles.append(tuple(indices[i] for i in tri))
    if not triangles:raise ValueError('artwork has no body surface')
    edges={}
    for tri in triangles:
        for a,b in zip(tri,tri[1:]+tri[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
    if any(len(e)>2 for e in edges.values()):raise ValueError('overlapping fitted artwork')
    n=len(points);back=[sub(p,(0.,0.,.0012)) for p in points]
    fs=triangles+[(c+n,b+n,a+n) for a,b,c in triangles]
    fs += [(a,a+n,b+n,b) for edge in edges.values() if len(edge)==1 for a,b in edge]
    return mesh(name,color,points+back,fs)

def loft(name,color,rings):
    n=len(rings[0]);vertices=[p for ring in rings for p in ring]
    faces=[tuple(range(n-1,-1,-1)),tuple(range((len(rings)-1)*n,len(rings)*n))]
    faces += [(r*n+i,r*n+(i+1)%n,(r+1)*n+(i+1)%n,(r+1)*n+i)
              for r in range(len(rings)-1) for i in range(n)]
    return mesh(name,color,vertices,faces)


def tube(name,color,points,radii,segments=8):
    rings=[]
    for k,p in enumerate(points):
        tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]))
        seed=(1.,0.,0.) if abs(tangent[0])<.85 else (0.,0.,1.)
        u=unit(cross(tangent,seed));v=cross(tangent,u)
        rings.append([add(p,add(mul(u,radii[k]*math.cos(math.tau*i/segments)),
                               mul(v,radii[k]*math.sin(math.tau*i/segments)))) for i in range(segments)])
    return loft(name,color,rings)


def body_cage():
    count=16
    # y, z center, half width, half depth. The neck remains distinctly narrower
    # than the cheeks and haunch; the forward lower cheek creates a blunt muzzle.
    profiles=((.112,-.060,.184,.254),(.274,-.047,.254,.333),
              (.419,-.021,.241,.310),(.524,.097,.164,.205),
              (.631,.183,.100,.114),(.750,.217,.109,.121),
              (.804,.250,.154,.171),(.872,.254,.161,.179),
              (.945,.225,.128,.123),(.992,.218,.043,.041))
    vertices=[(w*math.cos(math.tau*i/count),y,z+d*math.sin(math.tau*i/count))
              for y,z,w,d in profiles for i in range(count)]
    faces=[tuple(range(count-1,-1,-1)),tuple(range((len(profiles)-1)*count,len(profiles)*count))]
    holes=(1,5,9,13)
    for row in range(len(profiles)-1):
        for i in range(count):
            if row==0 and any(i in (h,(h+1)%count) for h in holes):continue
            faces.append((row*count+i,row*count+(i+1)%count,(row+1)*count+(i+1)%count,(row+1)*count+i))
    for h in holes:
        ids=(h,(h+1)%count,(h+2)%count,(h+2)%count+count,(h+1)%count+count,h+count)
        center=mean([vertices[i] for i in ids]);sx=1 if center[0]>0 else -1;sz=1 if center[2]>-.06 else -1
        cx=sx*.178;cz=.205 if sz>0 else -.245
        direction=unit((center[0],0.,center[2]+.060));tangent=(-direction[2],0.,direction[0])
        if dot(sub(vertices[ids[2]],vertices[ids[0]]),tangent)<0:tangent=mul(tangent,-1)
        angles=[math.radians(a) for a in (210,270,330,30,90,150)];previous=ids
        for yy,rx,rz in ((.135,.084,.096),(.063,.084,.105),(.011,.079,.102),(-.014,.063,.087)):
            ring=[]
            for a in angles:
                lateral=math.cos(a);outward=math.sin(a)
                ring.append(len(vertices));vertices.append((cx+tangent[0]*lateral*rx+direction[0]*outward*rx,
                    yy,cz+tangent[2]*lateral*rz+direction[2]*outward*rz))
            for a,b,c,d in zip(previous,previous[1:]+previous[:1],ring[1:]+ring[:1],ring):faces.append((a,b,c,d))
            previous=tuple(ring)
        faces.append(previous)
    vertices,faces=subdivide(vertices,faces)
    vertices=[(x,0. if y<.018 else y,z) for x,y,z in vertices]
    high=max(p[1] for p in vertices)
    return [(x,y*BODY_HEIGHT/high,z) for x,y,z in vertices],faces


def neck_surface(y,angle,body):
    origin=(0.,y,.171);direction=(math.cos(angle),0.,math.sin(angle));hits=[]
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
    if not hits:raise ValueError('collar leaf has no neck surface')
    return add(origin,mul(direction,max(hits)-.011))


def collar_leaf(i,body):
    # Alternating upward/sideways and drooping blades produce the source's
    # uneven rolled ruff, instead of eight identical flat radial diamonds.
    a=math.tau*(i+.5)/8;radial=(math.cos(a),0.,math.sin(a));u=(-math.sin(a),0.,math.cos(a))
    center=neck_surface(.552+.010*math.cos(2*a),a,body)
    lift=(.125,.038,-.060,.080,.120,.020,-.068,.062)[i]
    length=(.168,.170,.161,.171,.155,.167,.153,.177)[i]
    width=(.048,.051,.049,.050,.047,.051,.049,.050)[i]
    path=[]
    for travel,y,w,depth in ((0.,0.,.023,.014),(.28,.15,.84,.026),(.62,.43,1.,.030),(.91,.78,.65,.025),(1.04,1.15,.25,.013),(1.00,1.35,.025,.0015)):
        p=add(center,add(mul(radial,travel*length),(0.,y*lift,0.)))
        path.append((p,w if travel==0. else w*width,depth))
    rings=[]
    for k,(p,w,depth) in enumerate(path):
        tangent=unit(sub(path[min(k+1,len(path)-1)][0],path[max(0,k-1)][0]));v=unit(cross(tangent,u))
        rings.append([add(p,add(mul(u,w*math.cos(math.tau*j/8)),mul(v,depth*math.sin(math.tau*j/8)))) for j in range(8)])
    return loft('Collar / rolled leaf '+str(i+1),'leaf',rings)


# left/right edge widths are independent. The re-entrant right edge at station
# 6 is a true notch with closed sidewalls; it is never a painted-on dark mark.
LEAF_PATH=((0.,1.006,.287,.015,.015),(.005,1.081,.246,.070,.058),
           (.045,1.126,.134,.127,.110),(.095,1.138,-.001,.147,.130),
           (.150,1.113,-.136,.135,.122),(.199,1.068,-.246,.101,.042),
           (.232,1.037,-.314,.075,.005),(.257,1.025,-.369,.053,.093),
           (.294,1.027,-.454,.012,.020),(.309,1.043,-.478,.001,.001))


def leaf_mesh():
    vertices=[]
    for k,(x,y,z,left,right) in enumerate(LEAF_PATH):
        fold=.012 if k not in (0,len(LEAF_PATH)-1) else .001
        vertices += [(x-left,y-.018,z-.005),(x,y+fold,z),(x+right,y-.010-.70*right,z+.015)]
    n=len(vertices);vertices += [sub(p,(0.,.0035,0.)) for p in vertices];faces=[]
    for k in range(len(LEAF_PATH)-1):
        for j in range(2):
            a=3*k+j;b=a+1;c=b+3;d=a+3
            faces += [(a,b,c,d),(a+n,d+n,c+n,b+n)]
    perimeter=[3*k for k in range(len(LEAF_PATH))]+[3*(len(LEAF_PATH)-1)+1]+[3*k+2 for k in range(len(LEAF_PATH)-1,-1,-1)]+[1]
    faces += [(a,b,b+n,a+n) for a,b in zip(perimeter,perimeter[1:]+perimeter[:1])]
    return mesh(LEAF,'leaf',vertices,faces)


def toe_claw(cx,cz,index):
    # Twelve short cream paper nail wedges are seated in the foot fronts.
    # No spherical beads: their bottom and heel disappear into the foot.
    x=cx+(-.032,0.,.032)[index];z=cz+.076
    vertices=[(x-.013,.020,z-.022),(x+.013,.020,z-.022),(x+.014,.012,z+.024),(x-.014,.012,z+.024),
              (x-.012,.046,z-.014),(x+.012,.046,z-.014),(x+.010,.031,z+.027),(x-.010,.031,z+.027)]
    return mesh('Claw / '+('left ' if cx<0 else 'right ')+('front ' if cz>0 else 'rear ')+str(index+1),
                'white',vertices,[(0,3,2,1),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7)])


def sculpt():
    body=body_cage();parts=[mesh(BODY,'body',*body)]
    for side,label in ((-1,'left'),(1,'right')):
        cx=side*.107;cy=.857
        eye=[(cx+side*x,cy+y) for x,y in ((-.022,-.041),(-.030,-.009),(-.026,.028),(-.010,.052),(.013,.050),(.028,.027),(.029,-.017),(.014,-.043))]
        parts.append(face_patch('Eye / '+label+' fitted upright ink','ink',eye,body,.003))
        crescent=[(cx+side*x,cy+y) for x,y in ((-.020,-.024),(-.023,.014),(-.010,.039),(.002,.040),(-.002,.018),(-.006,-.020))]
        parts.append(face_patch('Eye / '+label+' pale crescent','white',crescent,body,.005))
        highlight=[(cx+side*x,cy+y) for x,y in ((.008,.025),(.017,.025),(.017,.035),(.009,.035))]
        parts.append(face_patch('Eye / '+label+' catchlight','white',highlight,body,.0054))
    # A restrained mouth follows the broad lower muzzle. Two inset-looking
    # nostril facets echo the source's wedge muzzle without button geometry.
    line=((- .074,.801),(-.047,.789),(0.,.784),(.047,.789),(.074,.801))
    parts.append(face_patch('Muzzle / fitted shallow mouth','ink',[(x,y+.0013) for x,y in line]+[(x,y-.0025) for x,y in reversed(line)],body,.0025))
    for side,label in ((-1,'left'),(1,'right')):
        cx=side*.059;cy=.834
        outline=[(cx-.004,cy-.002),(cx+.005,cy-.002),(cx+.006,cy+.004),(cx-.003,cy+.005)]
        parts.append(face_patch('Muzzle / '+label+' nostril','ink',outline,body,.0027))
    parts += [collar_leaf(i,body) for i in range(8)]
    for cx in (-.178,.178):
        for cz in (.205,-.245):parts += [toe_claw(cx,cz,i) for i in range(3)]
    parts.append(tube(TAIL,'body',[(0.,.336,-.313),(.007,.343,-.382),(.015,.385,-.454),(.020,.448,-.486),(.022,.465,-.475)],(.051,.044,.031,.014,.003),8))
    parts.append(tube(STEM,'leaf',[(0.,.962,.245),(0.,.994,.282),(.002,1.032,.278)],(.018,.017,.014),8))
    parts.append(leaf_mesh())
    return {'name':'bayleef','version':1,'coordinate_system':COORDINATES,'primitives':parts}


if __name__=='__main__':
    import argparse,json
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True);args=parser.parse_args()
    with open(args.output,'w') as output:json.dump(sculpt(),output,separators=(',',':'))
