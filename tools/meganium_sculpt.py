#!/usr/bin/env python3
"""Continuous mature Meganium with a six-petal paper flower and paired antennae.

The quadruped surface joins the broad trunk, planted legs, sloping long neck,
and rounded muzzle. Flower petals are closed folded blades in a plane normal
to the neck, and antennae are narrow curved tubes. Neutral silhouette preserves
the legacy 1.48 authoring-unit height; physical height belongs to the registry.
"""
import math
from chikorita_sculpt import add, sub, mul, dot, cross, unit, mean, f32, linear, subdivide, mesh as _mesh
from bayleef_sculpt import face_patch as _face_patch
from skin_glb import COORDINATES

PALETTE={'body':(107/255,198/255,24/255),'petal':(173/255,8/255,82/255),
         'gold':(237/255,205/255,95/255),'ink':(.027,.035,.020),'white':(1.,1.,.94)}
FULL_HEIGHT=1.48
BODY_HEIGHT=1.322
BODY='Body / continuous mature haunch chest long neck muzzle and four legs'
TAIL='Tail / broad tapered descending sweep'
COLLAR='Flower / fitted golden collar'
JAW='Jaw / hinged rounded lower muzzle'
ANTENNA_LEFT='Antenna / left outward drooping curve'
ANTENNA_RIGHT='Antenna / right outward drooping curve'
FLOWER_CENTER=(0.,.795,.165)
FLOWER_AXIS=unit((0.,.724,.69))
FLOWER_V=unit((0.,.69,-.724))
PETAL_ANGLES=tuple(math.pi/6+i*math.tau/6 for i in range(6))


def mesh(name,color,vertices,faces):
    part=_mesh(name,'body',vertices,faces);part['base_color']=linear(PALETTE[color]);return part


def face_patch(name,color,outline,body,offset=.003):
    part=_face_patch(name,'white',outline,body,offset);part['base_color']=linear(PALETTE[color]);return part


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
    profiles=((.180,-.090,.238,.344),(.362,-.095,.310,.452),
              (.545,-.060,.297,.419),(.668,.055,.224,.295),
              (.797,.176,.137,.151),(.955,.281,.117,.134),
              (1.103,.366,.114,.132),(1.159,.418,.163,.155),
              (1.210,.455,.191,.208),(1.267,.439,.193,.202),(1.313,.410,.146,.163),
              (1.341,.394,.047,.047))
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
        cx=sx*.229;cz=.285 if sz>0 else -.355
        direction=unit((center[0],0.,center[2]+.09));tangent=(-direction[2],0.,direction[0])
        if dot(sub(vertices[ids[2]],vertices[ids[0]]),tangent)<0:tangent=mul(tangent,-1)
        angles=[math.radians(a) for a in (210,270,330,30,90,150)];previous=ids
        for yy,rx,rz in ((.217,.095,.115),(.104,.096,.125),(.028,.092,.120),(-.020,.075,.104)):
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
    return [(x,y*BODY_HEIGHT/high,z) for x,y,z in vertices],faces


def flower_basis(angle):
    return add((math.cos(angle),0.,0.),mul(FLOWER_V,math.sin(angle))),add((-math.sin(angle),0.,0.),mul(FLOWER_V,math.cos(angle)))


def flower_petal(index):
    angle=PETAL_ANGLES[index];radial,tangent=flower_basis(angle)
    # A rounded oval outline and three creases read as thick folded paper.
    # Upper petals rise behind the neck; lower petals cup in front of the chest.
    length=(.343,.342,.343,.362,.374,.362)[index]
    root=add(FLOWER_CENTER,mul(radial,.112));rings=[]
    for distance,width,fold,depth in ((0.,.027,.002,.005),(.18,.096,.011,.012),(.48,.150,.026,.015),(.76,.155,.035,.014),(.94,.103,.021,.010),(1.,.028,.006,.004)):
        center=add(root,add(mul(radial,distance*length),mul(FLOWER_AXIS,-.040*distance+.052*distance*distance)))
        # A six-point cross section creates a soft central fold and bevel rims.
        ring=[]
        for w,h in ((-1.,0.),(-.53,.42),(0.,1.),(.53,.42),(1.,0.),(0.,-.43)):
            ring.append(add(center,add(mul(tangent,w*width),mul(FLOWER_AXIS,h*(fold+depth)))))
        rings.append(ring)
    return loft('Flower / folded oval petal '+str(index+1),'petal',rings)


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
    points=[(side*x,y,z) for x,y,z in ((.076,1.299,.409),(.098,1.384,.400),(.150,1.447,.405),(.212,1.461,.425),(.265,1.423,.453),(.284,1.359,.477),(.281,1.323,.484))]
    if side<0:points=[(x,y-.007*(k/6),z-.007*(k/6)) for k,(x,y,z) in enumerate(points)]
    p=tube(ANTENNA_LEFT if side<0 else ANTENNA_RIGHT,'gold',points,(.019,.021,.021,.021,.018,.012,.004),8)
    # Keep the named silhouette datum on the antenna crown, not the head.
    high=max(p['positions'][1::3]);scale=(FULL_HEIGHT-1.299)/(high-1.299)
    p['positions']=[f32(1.299+(v-1.299)*scale) if i%3==1 else v for i,v in enumerate(p['positions'])]
    # The tiny axial correction needs fresh facet normals, preserving seams.
    verts=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
    return mesh(p['part'],'gold',verts,[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)])


def lower_jaw():
    rings=[]
    for z,width,center,up,down in ((.418,.080,1.151,.020,.030),(.485,.120,1.155,.027,.046),
        (.565,.126,1.160,.032,.043),(.632,.083,1.167,.026,.031),(.647,.033,1.174,.015,.018)):
        rings.append([(width*math.cos(math.tau*i/10),center+(up if math.sin(math.tau*i/10)>0 else down)*math.sin(math.tau*i/10),z) for i in range(10)])
    return loft(JAW,'body',rings)


def toe_claw(cx,cz,index):
    x=cx+(-.038,0.,.038)[index];z=cz+.089
    vertices=[(x-.015,.024,z-.024),(x+.015,.024,z-.024),(x+.016,.013,z+.029),(x-.016,.013,z+.029),
              (x-.014,.054,z-.017),(x+.014,.054,z-.017),(x+.011,.035,z+.031),(x-.011,.035,z+.031)]
    return mesh('Claw / '+('left ' if cx<0 else 'right ')+('front ' if cz>0 else 'rear ')+str(index+1),'white',vertices,[(0,3,2,1),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7)])


def sculpt():
    body=body_cage();parts=[mesh(BODY,'body',*body)]
    for side,label in ((-1,'left'),(1,'right')):
        cx=side*.120;cy=1.249
        eye=[(cx+side*x,cy+y) for x,y in ((-.021,-.039),(-.031,-.010),(-.027,.023),(-.013,.045),(.007,.046),(.026,.029),(.031,-.007),(.019,-.036))]
        parts.append(face_patch('Eye / '+label+' fitted upright ink','ink',eye,body,.003))
        crescent=[(cx+side*x,cy+y) for x,y in ((-.019,-.021),(-.023,.011),(-.011,.032),(.001,.036),(-.002,.015),(-.006,-.020))]
        parts.append(face_patch('Eye / '+label+' pale crescent','white',crescent,body,.005))
        glint=[(cx+side*x,cy+y) for x,y in ((.006,.022),(.014,.022),(.014,.031),(.007,.032))]
        parts.append(face_patch('Eye / '+label+' catchlight','white',glint,body,.0054))
    cavity=[(-.111,1.183),(-.121,1.166),(-.100,1.133),(-.053,1.121),
            (.053,1.121),(.100,1.133),(.121,1.166),(.111,1.183),(0.,1.190)]
    parts.append(face_patch('Muzzle / fitted recessed mouth cavity','ink',cavity,body,.0032))
    parts.append(lower_jaw())
    for side,label in ((-1,'left'),(1,'right')):
        cx=side*.062;cy=1.214
        parts.append(face_patch('Muzzle / '+label+' nostril','ink',[(cx-.005,cy-.003),(cx+.006,cy-.003),(cx+.006,cy+.004),(cx-.003,cy+.005)],body,.003))
    parts += [flower_petal(i) for i in range(6)]+[flower_collar()]
    for cx in (-.229,.229):
        for cz in (.285,-.355):parts += [toe_claw(cx,cz,i) for i in range(3)]
    parts.append(tube(TAIL,'body',[(0.,.460,-.441),(.013,.492,-.567),(.035,.467,-.704),(.062,.376,-.822),(.078,.288,-.892)],(.084,.068,.050,.027,.004),10))
    parts += [antenna(-1),antenna(1)]
    return {'name':'meganium','version':1,'coordinate_system':COORDINATES,'primitives':parts}
