#!/usr/bin/env python3
"""Original rounded quadrupedal Cyndaquil, authored without mesh inputs.

The 34 named assemblies use a twelve-joint skin with four planted legs. Dimensions are
runtime units (+Y up, +Z front), retaining the 0.716729 body reference and 1.25
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
FULL_HEIGHT = 1.25
# z, center_y, half_width, half_height; each row is an intentional fold.
HEAD = ((-.075,.45,.20,.19),(.015,.48,.245,.23),
        (.13,.477,.255,.231729),(.25,.43,.225,.22),
        (.35,.355,.168,.17),(.45,.265,.095,.10),
        (.54,.205,.035,.037),(.565,.194,.014,.016))
RING = 32
TORSO = ((-.57,.29,.045,.06),(-.52,.32,.16,.18),
         (-.42,.335,.235,.235),(-.28,.345,.255,.245),
         (-.12,.36,.24,.24),(.02,.40,.19,.22),(.10,.43,.13,.17))


def curved_profiles(profiles):
    """Cubic silhouette curves rather than a stack of straight taper sections."""
    result=[]
    for i in range(len(profiles)-1):
        a,b,c,d=profiles[max(0,i-1)],profiles[i],profiles[i+1],profiles[min(len(profiles)-1,i+2)]
        for step in range(2):
            t=step/2.
            result.append(tuple(.5*((2*y)+(-x+z)*t+(2*x-5*y+4*z-w)*t*t+(-x+3*y-3*z+w)*t*t*t)
                                for x,y,z,w in zip(a,b,c,d)))
    return tuple(result)+(profiles[-1],)


HEAD=curved_profiles(HEAD)
TORSO=curved_profiles(TORSO)


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


def rounded(mesh):
    """Area-weighted surface normals, welded over the authored loft seams."""
    points=list(zip(*[iter(mesh['positions'])]*3));sums={}
    for a,b,c in zip(*[iter(mesh['indices'])]*3):
        normal=cross(sub(points[b],points[a]),sub(points[c],points[a]))
        for i in (a,b,c):sums[points[i]]=add(sums.get(points[i],(0.,0.,0.)),normal)
    normals=[]
    for i,p in enumerate(points):
        n=unit(sums[p]);original=tuple(mesh['normals'][i*3:i*3+3])
        normals.extend(f32(v) for v in (n if dot(n,original)>.25 else original))
    mesh['normals']=normals
    return mesh


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
    line=((.09,.445,.002),(.145,.475,.005),(.205,.48,.005),(.265,.455,.002))
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
    torso_rings=[[(w*math.cos(math.tau*i/RING),y+h*math.sin(math.tau*i/RING),z)
                   for i in range(RING)] for z,y,w,h in TORSO]
    body_mesh=merged(rounded(loft(torso_rings)),rounded(loft(head_rings())))
    legs={}
    for rear,z in ((False,.04),(True,-.36)):
        for side in (-1,1):
            # Broad flat soles, rounded toes, a curved ankle, and a shoulder
            # embedded in the low body. Four separate supports span its length.
            profiles=((0.,z+.03,.060,.08),(.013,z+.045,.064,.086),
                      (.04,z+.025,.06,.076),(.075,z-.005,.057,.062),
                      (.13,z-.017,.075,.075),(.21,z-.02,.081,.08),
                      (.25,z-.02,.045,.05))
            name=('Hindleg' if rear else 'Foreleg')+' / '+('left' if side<0 else 'right')
            legs[name]=(rounded(horizontal(curved_profiles(profiles),32,side*(.19 if rear else .16))),CREAM)
    # The dorsal coat follows the body's exact fold vertices, not coarse
    # chords across its convex panels. Its rear head plate meets the pear's
    # rear panels inside the neck, making the hood continuous in profile.
    cap_grid=[]
    for z,y,w,h in HEAD:
        edge=.65*(1.-max(0.,min(1.,(z-.30)/.24)))+.13*max(0.,min(1.,(z-.30)/.24))
        cap_grid.append([(w*math.cos(edge+(math.pi-2*edge)*i/24),
                          y+h*math.sin(edge+(math.pi-2*edge)*i/24)+.006,z) for i in range(25)])
    cap=shell_patch(cap_grid)
    back_grid=[[(w*math.cos(math.pi*i/16),y+h*math.sin(math.pi*i/16)+.006,z)
                for i in range(17)] for z,y,w,h in TORSO]
    cap=rounded(merged(cap,shell_patch(back_grid,(0.,-.008,0.))))
    nose=rounded(loft([[(.009*math.cos(math.tau*i/16),.194+.007*math.sin(math.tau*i/16),z) for i in range(16)] for z in (.562,.574)]))
    parts={PARTS[0]:(nose,INK),PARTS[1]:(eyelid(-1),INK),PARTS[2]:(eyelid(1),INK),
           PARTS[3]:(cap,BACK),PARTS[9]:(body_mesh,CREAM),**legs}
    specs=(((-.28,1.12,-.35),.23,.04,-28,-.10),
           ((.28,1.10,-.37),.24,.04,27,.12),
           ((-.34,1.17,-.65),.24,.045,-40,-.22),
           ((.34,1.15,-.67),.24,.045,40,.22),
           ((.003,1.25,-.80),.26,.045,62,-.11))
    for (name,source_root),(tip,width,depth,twist,bend) in zip(QUILLS,specs):
        root=source_point(*source_root)
        assemblies=flame(root,tip,width,depth,twist,bend)
        for suffix,mesh in assemblies.items():
            color=EMBER if suffix=='ember silhouette' else GOLD if suffix.endswith('golden heart') else ORANGE
            parts[name+' / '+suffix]=(mesh,color)
    # Keep the established metre datum while shortening the four supports
    # relative to the deep, low belly. Fire remains independently sized.
    body_scale=BODY_HEIGHT/max(v for name,(mesh,_) in parts.items() if 'flame quill /' not in name for v in mesh['positions'][1::3])
    for name,(mesh,_) in parts.items():
        if 'flame quill /' in name:continue
        for i in range(1,len(mesh['positions']),3):mesh['positions'][i]=f32(mesh['positions'][i]*body_scale)
        for i in range(0,len(mesh['normals']),3):
            n=mesh['normals'][i:i+3];n[1]/=body_scale
            mesh['normals'][i:i+3]=[f32(v) for v in unit(n)]
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
