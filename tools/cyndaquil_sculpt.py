#!/usr/bin/env python3
"""Original folded-paper Cyndaquil: explicit panels, no remesh or mesh inputs.

The 30 named assemblies use the established ten-joint skin. Dimensions are
runtime units (+Y up, +Z front), retaining the 0.716729 body reference and 0.91
full silhouette. The archived Blender sculpture remains an editable historical
source; this compact recipe is the authoritative source of the new sculpture.
"""
import math
import struct

CREAM = (.94, .83, .56)
BACK = (.14, .30, .34)
INK = (.095, .12, .15)
EMBER = (.86, .18, .12)
ORANGE = (.98, .47, .075)
GOLD = (1., .78, .22)
BODY_HEIGHT = .716729
FULL_HEIGHT = .91
# z, center_y, half_width, half_height; each row is an intentional fold.
HEAD = ((-.170,.493,.107,.110),(-.095,.539,.173,.144),
        (.020,.552,.200,.156729),(.125,.540,.181,.137),
        (.220,.516,.139,.099),(.322,.486,.085,.059),
        (.416,.464,.040,.030),(.462,.455,.020,.020))
RING = 16


def add(a, b): return tuple(x+y for x,y in zip(a,b))
def sub(a, b): return tuple(x-y for x,y in zip(a,b))
def mul(a, s): return tuple(x*s for x in a)
def dot(a, b): return sum(x*y for x,y in zip(a,b))
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def unit(a): return mul(a, 1/math.sqrt(dot(a,a)))
def f32(v): return struct.unpack('<f', struct.pack('<f', v))[0]
def linear(v): return v/12.92 if v <= .04045 else ((v+.055)/1.055)**2.4


def geometry(vertices, faces):
    """Closed panel meshes, with one normal per authored polygon.

    Welding occurs only across identical positions AND identical panel normals:
    paper creases survive export rather than acquiring smoothed sphere normals.
    """
    triangles=[(face[0],face[i],face[i+1]) for face in faces for i in range(1,len(face)-1)]
    if sum(dot(vertices[a],cross(vertices[b],vertices[c])) for a,b,c in triangles)<0:
        faces=[tuple(reversed(face)) for face in faces]
    positions=[];normals=[];indices=[];lookup={}
    for face in faces:
        normal=unit(cross(sub(vertices[face[1]],vertices[face[0]]),sub(vertices[face[2]],vertices[face[0]])))
        for i in range(1,len(face)-1):
            for v in (face[0],face[i],face[i+1]):
                key=tuple(f32(x) for x in vertices[v]+normal)
                if key not in lookup:
                    lookup[key]=len(positions)//3;positions.extend(key[:3]);normals.extend(key[3:])
                indices.append(lookup[key])
    return {'positions':positions,'normals':normals,'indices':indices}


def merged(*parts):
    result={'positions':[],'normals':[],'indices':[]}
    for part in parts:
        offset=len(result['positions'])//3
        result['positions']+=part['positions'];result['normals']+=part['normals']
        result['indices'] += [i+offset for i in part['indices']]
    return result


def loft(rings):
    n=len(rings[0]);vertices=[p for ring in rings for p in ring]
    faces=[tuple(range(n-1,-1,-1)),tuple(range((len(rings)-1)*n,len(rings)*n))]
    faces += [(r*n+i,r*n+(i+1)%n,(r+1)*n+(i+1)%n,(r+1)*n+i)
              for r in range(len(rings)-1) for i in range(n)]
    return geometry(vertices,faces)


def horizontal(profiles, count=16, x=0.):
    return loft([[(x+w*math.cos(math.tau*i/count),y,z+d*math.sin(math.tau*i/count))
                  for i in range(count)] for y,z,w,d in profiles])


def head_rings():
    return [[(w*math.cos(math.tau*i/RING),y+h*math.sin(math.tau*i/RING),z)
             for i in range(RING)] for z,y,w,h in HEAD]


def shell_patch(grid, inside=(0.,-.007,0.)):
    rows=len(grid);cols=len(grid[0]);outer=[v for row in grid for v in row]
    n=len(outer);vertices=outer+[add(p,inside) for p in outer]
    faces=[]
    for r in range(rows-1):
        for c in range(cols-1):
            a=r*cols+c;b=a+1;d=a+cols;e=d+1
            faces.extend([(a,b,e,d),(d+n,e+n,b+n,a+n)])
    perimeter=list(range(cols))+[r*cols+cols-1 for r in range(1,rows)]
    perimeter+=list(range(n-2,n-cols-1,-1))+[r*cols for r in range(rows-2,0,-1)]
    faces += [(a,a+n,b+n,b) for a,b in zip(perimeter,perimeter[1:]+perimeter[:1])]
    return geometry(vertices,faces)


def face_x(z,y):
    """Exact planar side of the authored head, for fitted closed-eye paper."""
    for first,second in zip(HEAD,HEAD[1:]):
        if first[0]<=z<=second[0]:
            t=(z-first[0])/(second[0]-first[0]);_,cy,w,h=[a+(b-a)*t for a,b in zip(first,second)]
            yn=(y-cy)/h
            points=[(math.cos(math.tau*i/RING),math.sin(math.tau*i/RING)) for i in range(RING//4+1)]
            for (x0,y0),(x1,y1) in zip(points,points[1:]):
                if y0<=yn<=y1:return w*(x0+(x1-x0)*(yn-y0)/(y1-y0))
            return w*math.sqrt(max(0,1-yn*yn))
    raise ValueError('eye outside head profile')


def eyelid(side):
    # The long angular V is readable from the battle flank as well as 3/4.
    line=((.039,.578,.002),(.098,.550,.008),(.175,.541,.008),(.242,.559,.002))
    vertices=[]
    for z,y,width in line:
        for depth,dy in ((.006,width),(.006,-width),(-.003,-width),(-.003,width)):
            yy=y+dy;vertices.append((side*(face_x(z,yy)+depth),yy,z))
    faces=[(3,2,1,0),(12,13,14,15)]
    faces += [(r*4+i,r*4+(i+1)%4,(r+1)*4+(i+1)%4,(r+1)*4+i) for r in range(3) for i in range(4)]
    return geometry(vertices,faces)


def flame(root, tip, width, depth, twist, bend):
    axis=sub(tip,root);direction=unit(axis)
    side=unit(sub((1.,0.,0.),mul(direction,direction[0])))
    normal=unit(cross(side,direction))
    angle=math.radians(twist);side,normal=(add(mul(side,math.cos(angle)),mul(normal,math.sin(angle))),
                                        add(mul(normal,math.cos(angle)),mul(side,-math.sin(angle))))
    # Split tongues have broad shoulders and unequal tips. The final tip is
    # an actual vertex; the fold thickness is carried into the ember socket.
    outline=((- .11,-.08),(-.72,.19),(-.76,.44),(-.38,.34),(-.39,.64),
             (bend,1.),(.12,.67),(.48,.61),(.35,.45),(.67,.29),(.11,-.07))
    ring=[add(add(root,mul(axis,t)),mul(side,x*width)) for x,t in outline]
    front=add(add(root,mul(axis,.36)),mul(normal,depth))
    back=add(add(root,mul(axis,.36)),mul(normal,-depth*.72))
    n=len(ring);faces=[(i,(i+1)%n,n) for i in range(n)]+[((i+1)%n,i,n+1) for i in range(n)]
    result={'ember silhouette':geometry(ring+[front,back],faces)}
    for label,ridge,outward,thick in (('front',front,normal,depth),('back',back,mul(normal,-1),depth*.72)):
        for layer,scale,offset in (('orange fold',.84,.0018),('golden heart',.49,.0036)):
            nested=[add(add(ridge,mul(sub(p,ridge),scale)),mul(outward,offset)) for p in ring]
            peak=add(ridge,mul(outward,offset));inner=add(ridge,mul(outward,-thick*scale-.001))
            result[label+' '+layer]=geometry(nested+[peak,inner],faces)
    return result


def sculpt():
    # Imported lazily so source generation and the skin writer remain separable.
    from cyndaquil_glb import PARTS, COORDINATES, QUILLS, source_point
    body=horizontal(((.075,-.025,.095,.09),(.125,-.035,.172,.153),(.235,-.038,.209,.185),
                     (.355,-.020,.199,.176),(.470,.003,.156,.139),(.565,.020,.094,.100)))
    pieces=[body,loft(head_rings())]
    for side in (-1,1):
        # Heel is under the haunch; the low, broad toe settles onto one plane.
        pieces.append(horizontal(((0.,.032,.061,.111),(.021,.035,.071,.125),
                                  (.058,.027,.069,.113),(.103,-.012,.057,.084),(.168,-.045,.046,.069)),12,side*.145))
        pieces.append(horizontal(((.076,-.034,.076,.094),(.130,-.049,.096,.117),
                                  (.206,-.047,.095,.118),(.271,-.027,.063,.089)),12,side*.127))
        # Explicit shoulder, elbow and folded palm; no cylindrical bead joints.
        arm=[]
        lift=.013 if side<0 else 0.
        for y,z,w,d,cx in ((.198,.177,.031,.037,.164),(.213,.188,.039,.052,.170),
                           (.245,.155,.042,.041,.178),(.282,.122,.047,.044,.185),
                           (.325,.091,.047,.044,.184),(.365,.077,.040,.041,.176)):
            curl=1-(y-.198)/(.365-.198)
            arm.append([(side*cx+w*math.cos(math.tau*i/10),y+lift*curl,
                         z-(.013 if side<0 else 0.)*curl+d*math.sin(math.tau*i/10)) for i in range(10)])
        pieces.append(loft(arm))
    body_mesh=merged(*pieces)
    # The dorsal coat follows the body's exact fold vertices, not coarse
    # chords across its convex panels. Its rear head plate meets the pear's
    # rear panels inside the neck, making the hood continuous in profile.
    cap=shell_patch([[add(p,(0.,.008,0.)) for p in ring[1:RING//2]]
                     for ring in head_rings()[:6]])
    torso_rows=((.235,-.038,.209,.185),(.355,-.020,.199,.176),
                (.470,.003,.156,.139),(.565,.020,.094,.100))
    back_grid=[]
    for y,z,w,d in torso_rows:
        back_grid.append([(w*math.cos(math.tau*i/16),y,z+d*math.sin(math.tau*i/16)-.006)
                          for i in range(8,17)])
    rear_head=head_rings()[0]
    rear_plate=loft([[add(p,(0.,0.,-.006)) for p in rear_head],
                    [add(p,(0.,0.,.008)) for p in rear_head]])
    cap=merged(cap,rear_plate,shell_patch(back_grid,(0.,0.,.012)))
    nose=loft([[(.021*math.cos(math.tau*i/8),.455+.014*math.sin(math.tau*i/8),z) for i in range(8)] for z in (.456,.473)])
    parts={PARTS[0]:(nose,INK),PARTS[1]:(eyelid(-1),INK),PARTS[2]:(eyelid(1),INK),
           PARTS[3]:(cap,BACK),PARTS[9]:(body_mesh,CREAM)}
    specs=(((-.235,.91,-.314),.126,.029,-28,-.10),
           ((.249,.875,-.302),.130,.030,27,.12),
           ((-.319,.699,-.438),.143,.033,-40,-.22),
           ((.324,.704,-.449),.140,.034,40,.22),
           ((.003,.806,-.535),.148,.035,62,-.11))
    for (name,source_root),(tip,width,depth,twist,bend) in zip(QUILLS,specs):
        root=source_point(*source_root)
        assemblies=flame(root,tip,width,depth,twist,bend)
        for suffix,mesh in assemblies.items():
            color=EMBER if suffix=='ember silhouette' else GOLD if suffix.endswith('golden heart') else ORANGE
            parts[name+' / '+suffix]=(mesh,color)
    # Height is an explicit reference, not normalisation by changing flame tips.
    all_flame=[p for name,(mesh,_) in parts.items() if 'flame quill /' in name for p in mesh['positions'][1::3]]
    highest=max(all_flame)
    if abs(highest-FULL_HEIGHT)>1e-8:
        # Blade twist can tilt the tip sideways. Scale only flame points above
        # their embedded root. Body and physical reference never change.
        for name,(mesh,_) in parts.items():
            if 'flame quill /' in name:
                root=source_point(*next(root for prefix,root in QUILLS if name.startswith(prefix+' / ')))
                for i in range(1,len(mesh['positions']),3):
                    mesh['positions'][i]=f32(root[1]+(mesh['positions'][i]-root[1])*(FULL_HEIGHT-root[1])/(highest-root[1]))
                scale=(FULL_HEIGHT-root[1])/(highest-root[1])
                for i in range(0,len(mesh['normals']),3):
                    n=mesh['normals'][i:i+3];n[1]/=scale
                    mesh['normals'][i:i+3]=[f32(v) for v in unit(n)]
    primitives=[]
    for name in PARTS:
        mesh,color=parts[name]
        primitives.append({'part':name,**mesh,'base_color':[f32(linear(v)) for v in color]+[1.]})
    return {'name':'battle_cyndaquil','version':1,'coordinate_system':COORDINATES,'primitives':primitives}
