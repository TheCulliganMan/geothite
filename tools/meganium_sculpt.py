#!/usr/bin/env python3
"""Continuous mature Meganium with a six-petal paper flower and paired antennae.

The quadruped surface joins the broad trunk, planted legs, sloping long neck,
and rounded muzzle. Flower petals are closed folded blades in a plane normal
to the neck, and antennae are narrow curved tubes. Neutral silhouette preserves
the legacy 1.48 authoring-unit height; physical height belongs to the registry.
"""
import math
from chikorita_sculpt import add, sub, mul, dot, cross, unit, mean, f32, linear, subdivide, gaze, front_at, mesh as _mesh
from bayleef_sculpt import face_patch as _face_patch
from skin_glb import COORDINATES

PALETTE={'body':(139/255,201/255,90/255),'petal':(.88,.28,.49),'rim':(.99,.84,.88),
         'gold':(.96,.79,.26),'iris':(.88,.52,.10),'tongue':(.86,.45,.48),
         'ink':(.027,.035,.020),'white':(1.,1.,.94)}
FULL_HEIGHT=1.48
BODY_HEIGHT=1.322
BODY='Body / continuous mature haunch chest long neck muzzle and four legs'
TAIL='Tail / thick upward tapered nub'
COLLAR='Flower / fitted golden collar'
JAW='Jaw / hinged rounded lower muzzle'
ANTENNA_LEFT='Antenna / left outward drooping curve'
ANTENNA_RIGHT='Antenna / right outward drooping curve'
FLOWER_CENTER=(0.,.795,.220)
FLOWER_AXIS=unit((0.,.724,.69))
FLOWER_V=unit((0.,.69,-.724))
PETAL_ANGLES=tuple(math.pi/6+i*math.tau/6 for i in range(6))


def mesh(name,color,vertices,faces):
    # Facet normals must describe the serialized positions, including small
    # fitted-eye triangles; computing before float32 rounding tilts slivers.
    vertices=[tuple(f32(v) for v in p) for p in vertices]
    part=_mesh(name,'body',vertices,faces);part['base_color']=linear(PALETTE[color]);return part


def face_patch(name,color,outline,body,offset=.003):
    part=_face_patch(name,'white',outline,body,offset)
    vertices=list(zip(part['positions'][::3],part['positions'][1::3],part['positions'][2::3]))
    return mesh(name,color,vertices,[tuple(part['indices'][i:i+3]) for i in range(0,len(part['indices']),3)])


def loft(name,color,rings):
    n=len(rings[0]);vertices=[p for ring in rings for p in ring]
    faces=[tuple(range(n-1,-1,-1)),tuple(range((len(rings)-1)*n,len(rings)*n))]
    faces += [(r*n+i,r*n+(i+1)%n,(r+1)*n+(i+1)%n,(r+1)*n+i) for r in range(len(rings)-1) for i in range(n)]
    return mesh(name,color,vertices,faces)


def tube(name,color,points,radii,segments=8):
    rings=[]
    for k,p in enumerate(points):
        tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]))
        seed=(1.,0.,0.) if abs(tangent[0])<.85 else (0.,0.,1.)
        u=unit(cross(tangent,seed));v=cross(tangent,u)
        rings.append([add(p,add(mul(u,radii[k]*math.cos(math.tau*i/segments)),mul(v,radii[k]*math.sin(math.tau*i/segments)))) for i in range(segments)])
    return loft(name,color,rings)


def body_cage():
    count=16
    # The adult trunk is wide and long, with a high shoulder and sloping neck.
    # Head width/depth form the source's rounded muzzle, independent of Bayleef.
    profiles=((.145,-.110,.255,.380),(.325,-.110,.340,.500),
              (.525,-.075,.325,.455),(.655,.080,.240,.320),
              (.795,.235,.165,.175),(.955,.285,.130,.130),
              (1.060,.300,.125,.130),(1.100,.310,.125,.120),
              (1.140,.300,.130,.055),(1.160,.300,.140,.045),
              (1.175,.315,.145,.050),(1.190,.340,.155,.120),
              (1.215,.340,.160,.130),(1.255,.320,.160,.125),
              (1.300,.300,.125,.100),
              (1.345,.290,.060,.055),(1.355,.290,.020,.020))
    vertices=[(w*math.cos(math.tau*i/count),y,z+d*math.sin(math.tau*i/count)) for y,z,w,d in profiles for i in range(count)]
    faces=[tuple(range(count-1,-1,-1)),tuple(range((len(profiles)-1)*count,len(profiles)*count))]
    holes=(1,5,9,13)
    for row in range(len(profiles)-1):
        for i in range(count):
            if row==0 and any(i in (h,(h+1)%count) for h in holes):continue
            faces.append((row*count+i,row*count+(i+1)%count,(row+1)*count+(i+1)%count,(row+1)*count+i))
    for h in holes:
        ids=(h,(h+1)%count,(h+2)%count,(h+2)%count+count,(h+1)%count+count,h+count)
        center=mean([vertices[i] for i in ids]);sx=1 if center[0]>0 else -1;sz=1 if center[2]>-.09 else -1
        cx=sx*.250;cz=.300 if sz>0 else -.400
        direction=unit((center[0],0.,center[2]+.09));tangent=(-direction[2],0.,direction[0])
        if dot(sub(vertices[ids[2]],vertices[ids[0]]),tangent)<0:tangent=mul(tangent,-1)
        angles=[math.radians(a) for a in (210,270,330,30,90,150)];previous=ids
        for yy,rx,rz in ((.180,.105,.125),(.090,.103,.135),(.028,.098,.130),(-.020,.080,.110)):
            ring=[]
            for a in angles:
                lateral=math.cos(a);outward=math.sin(a);ring.append(len(vertices))
                vertices.append((cx+tangent[0]*lateral*rx+direction[0]*outward*rx,yy,cz+tangent[2]*lateral*rz+direction[2]*outward*rz))
            for a,b,c,d in zip(previous,previous[1:]+previous[:1],ring[1:]+ring[:1],ring):faces.append((a,b,c,d))
            previous=tuple(ring)
        faces.append(previous)
    vertices,faces=subdivide(vertices,faces)
    vertices=[(x,0. if y<.032 else y,z) for x,y,z in vertices]
    high=max(p[1] for p in vertices)
    return [(f32(x),f32(y*BODY_HEIGHT/high),f32(z)) for x,y,z in vertices],faces


def flower_basis(angle):
    return add((math.cos(angle),0.,0.),mul(FLOWER_V,math.sin(angle))),add((-math.sin(angle),0.,0.),mul(FLOWER_V,math.cos(angle)))


def flower_petal(index):
    angle=PETAL_ANGLES[index];radial,tangent=flower_basis(angle)
    # A rounded oval outline and three creases read as thick folded paper.
    # Upper petals rise behind the neck; lower petals cup in front of the chest.
    length=(.343,.342,.343,.362,.374,.362)[index]
    root=add(FLOWER_CENTER,mul(radial,.112));rings=[]
    for distance,width,fold,depth in ((0.,.035,.002,.006),(.18,.112,.011,.014),(.48,.178,.026,.018),(.76,.180,.035,.016),(.94,.120,.021,.012),(1.,.028,.006,.004)):
        center=add(root,add(mul(radial,distance*length),mul(FLOWER_AXIS,-.040*distance+.052*distance*distance)))
        # A six-point cross section creates a soft central fold and bevel rims.
        ring=[]
        for w,h in ((-1.,0.),(-.53,.42),(0.,1.),(.53,.42),(1.,0.),(0.,-.43)):
            ring.append(add(center,add(mul(tangent,w*width),mul(FLOWER_AXIS,h*(fold+depth)))))
        rings.append(ring)
    return loft('Flower / pale petal rim '+str(index+1),'rim',rings)


def flower_patch(index,parent,stamen=False):
    # Fit through the exact folded parent triangles in an orthonormal petal
    # frame. The buried back tissue keeps the pale edge and pink face joined.
    from sculpt_geometry import fitted_triangle_contour
    radial,tangent=flower_basis(PETAL_ANGLES[index])
    def local(point):
        delta=sub(point,FLOWER_CENTER)
        return dot(delta,radial),dot(delta,tangent),dot(delta,FLOWER_AXIS)
    # Float32 world storage adds tiny projection residuals to designed planar
    # end walls. Restore their six-decimal local datums before clipping, then
    # verify the fitted result against the unsnapped stored parent in tests.
    hull=dict(parent,positions=[round(v,6) for p in zip(parent['positions'][::3],parent['positions'][1::3],parent['positions'][2::3]) for v in local(p)])
    length=(.343,.342,.343,.362,.374,.362)[index];root=.112
    if stamen:
        outline=[(root+.025,-.004),(root+.145,-.018),(root+.160,-.010),(root+.147,-.001),(root+.025,.008)]
        name='Flower / stamen '+str(index+1);color='gold';offset=.0026
    else:
        outline=[(root+.015,-.018),(root+.18*length,-.086),(root+.48*length,-.147),
                 (root+.76*length,-.148),(root+.91*length,-.088),(root+.965*length,0.),
                 (root+.91*length,.088),(root+.76*length,.148),(root+.48*length,.147),
                 (root+.18*length,.086),(root+.015,.018)]
        name='Flower / folded oval petal '+str(index+1);color='petal';offset=.0014
    q=fitted_triangle_contour(name,PALETTE[color],outline,hull,offset,precision=7)
    points=list(zip(q['positions'][::3],q['positions'][1::3],q['positions'][2::3]))
    faces=[tuple(q['indices'][k:k+3]) for k in range(0,len(q['indices']),3)]
    if not stamen:
        # One continuous inner pink volume reaches both real petal faces.
        # A front-only pink decal would leave the entire rear pale.
        front=[];lookup={};triangles=[]
        for face in faces:
            a,b,c=[points[i] for i in face]
            if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
            ids=[]
            for p in (a,b,c):
                if p not in lookup:lookup[p]=len(front);front.append(p)
                ids.append(lookup[p])
            triangles.append(tuple(ids))
        vv=list(zip(hull['positions'][::3],hull['positions'][1::3],hull['positions'][2::3]));ff=[tuple(hull['indices'][k:k+3]) for k in range(0,len(hull['indices']),3)];inverted=[(x,y,-z) for x,y,z in vv]
        back=[(x,y,-front_at(x,y,inverted,ff)-offset) for x,y,z in front];count=len(front);edges={}
        for face in triangles:
            for a,b in zip(face,face[1:]+face[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
        faces=triangles+[tuple(i+count for i in reversed(face)) for face in triangles]
        faces += [(b,a,a+count,b+count) for uses in edges.values() if len(uses)==1 for a,b in uses]
        points=front+back
    vertices=[add(FLOWER_CENTER,add(add(mul(radial,x),mul(tangent,y)),mul(FLOWER_AXIS,z))) for x,y,z in points]
    return mesh(name,color,vertices,faces)


def flower_collar():
    vertices=[];faces=[];n=32;m=6
    for i in range(n):
        radial,_=flower_basis(math.tau*i/n)
        for j in range(m):
            a=math.tau*j/m
            vertices.append(add(FLOWER_CENTER,add(mul(radial,.128+.018*math.cos(a)),mul(FLOWER_AXIS,.018*math.sin(a)+.015))))
    for i in range(n):
        for j in range(m):faces.append((i*m+j,((i+1)%n)*m+j,((i+1)%n)*m+(j+1)%m,i*m+(j+1)%m))
    return mesh(COLLAR,'gold',vertices,faces)


def antenna(side):
    points=[(side*x,y,z) for x,y,z in ((.065,1.270,.310),(.085,1.390,.300),(.110,1.460,.230),(.135,1.465,.060),(.160,1.410,-.140),(.180,1.350,-.300),(.205,1.355,-.390))]
    if side<0:points=[(x,y-.007*(k/6),z-.007*(k/6)) for k,(x,y,z) in enumerate(points)]
    p=tube(ANTENNA_LEFT if side<0 else ANTENNA_RIGHT,'gold',points,(.010,.011,.012,.012,.010,.007,.002),8)
    # Keep the named silhouette datum on the antenna crown, not the head.
    high=max(p['positions'][1::3]);scale=(FULL_HEIGHT-1.270)/(high-1.270)
    p['positions']=[f32(1.270+(v-1.270)*scale) if i%3==1 else v for i,v in enumerate(p['positions'])]
    # The tiny axial correction needs fresh facet normals, preserving seams.
    verts=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
    return mesh(p['part'],'gold',verts,[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)])


def lower_jaw():
    rings=[]
    for z,width,center,up,down in ((.295,.065,1.105,.025,.030),(.360,.090,1.100,.025,.030),
        (.420,.090,1.105,.025,.030),(.460,.055,1.115,.018,.025),(.480,.015,1.125,.010,.015)):
        rings.append([(width*math.cos(math.tau*i/10),center+(up if math.sin(math.tau*i/10)>0 else down)*math.sin(math.tau*i/10),z) for i in range(10)])
    return loft(JAW,'body',rings)


def tongue():
    rings=[]
    for z,width,height in ((.39,.005,.002),(.405,.040,.006),(.43,.045,.007),(.45,.030,.006),(.46,.002,.001)):
        rings.append([(width*math.cos(math.tau*j/12),1.118+height*math.sin(math.tau*j/12),z) for j in range(12)])
    return loft('Tongue / seated rose tongue','tongue',rings)


def toe_claw(cx,cz,index):
    x=cx+(-.038,0.,.038)[index];z=cz+.089
    vertices=[(x-.015,.024,z-.024),(x+.015,.024,z-.024),(x+.016,.013,z+.029),(x-.016,.013,z+.029),
              (x-.014,.054,z-.017),(x+.014,.054,z-.017),(x+.011,.035,z+.031),(x-.011,.035,z+.031)]
    return mesh('Claw / '+('left ' if cx<0 else 'right ')+('front ' if cz>0 else 'rear ')+str(index+1),'white',vertices,[(0,3,2,1),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7)])


def sculpt():
    # Round living skin while retaining the exact cage and sharp sole edges.
    # Local import avoids the shared geometry module's base-mesh dependency.
    from sculpt_geometry import crease_normals
    body=body_cage();parts=[crease_normals(mesh(BODY,'body',*body),35.)]
    for side,label in ((-1,'left'),(1,'right')):
        cx=side*.105;cy=1.235
        eye=[(cx+side*x*.82,cy+y*.82) for x,y in ((-.021,-.039),(-.031,-.010),(-.027,.023),(-.013,.045),(.007,.046),(.026,.029),(.031,-.007),(.019,-.036))]
        parts+=gaze(side,cx,cy,eye,body,face_patch,iris='amber')
    cavity=[(-.090,1.170),(-.100,1.155),(-.080,1.136),(-.043,1.126),
            (.043,1.126),(.080,1.136),(.100,1.155),(.090,1.170),(0.,1.177)]
    parts.append(face_patch('Muzzle / fitted recessed mouth cavity','ink',cavity,body,.0032))
    parts.append(lower_jaw());parts.append(tongue())
    for side,label in ((-1,'left'),(1,'right')):
        cx=side*.055;cy=1.194
        parts.append(face_patch('Muzzle / '+label+' nostril','ink',[(cx-.005,cy-.003),(cx+.006,cy-.003),(cx+.006,cy+.004),(cx-.003,cy+.005)],body,.003))
    petals=[flower_petal(i) for i in range(6)]
    parts+=petals+[flower_patch(i,p) for i,p in enumerate(petals)]
    parts += [flower_patch(i,p,True) for i,p in enumerate(petals)]+[flower_collar()]
    for cx in (-.250,.250):
        for cz in (.300,-.400):parts += [toe_claw(cx,cz,i) for i in range(3)]
    parts.append(tube(TAIL,'body',[(0.,.440,-.440),(0.,.530,-.660),(0.,.590,-.800),(0.,.670,-.920)],(.120,.100,.065,.002),12))
    parts += [antenna(-1),antenna(1)]
    # Stabilize sub-femtometre axis residues after topology construction.
    for part in parts:
        part['positions']=[0.0 if abs(v)<1e-15 else v for v in part['positions']]
    return {'name':'meganium','version':1,'coordinate_system':COORDINATES,'primitives':parts}
