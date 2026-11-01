#!/usr/bin/env python3
"""Original wind-borne plants with toothed leaves, petals and seed globes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_patch,part,crease_normals
from abra_sculpt import surface
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES

SPECIES=('hoppip','skiploom','jumpluff');HEIGHT=.85
PINK=(173/255,49/255,57/255);HOPPIP_LEAF=(90/255,189/255,49/255);GREEN=(.49,.73,.24);BLUE=(74/255,66/255,198/255)
SKIPLOOM_GREEN=(66/255,148/255,57/255);SKIPLOOM_GOLD=(1.,181/255,41/255)
LEAF=(.55,.74,.37);VEIN=(.28,.43,.20);YELLOW=(.97,.83,.35)
RED=(.84,.24,.31);INK=(.15,.09,.14);WHITE=(.99,.97,.86)
CREAM=(231/255,189/255,107/255);MAUVE=(.64,.38,.51);TONGUE=(.96,.65,.63)

HOPPIP_OUTLINE=[(.025,.582),(.065,.700),(.125,.683),(.125,.760),(.220,.736),(.210,.819),(.305,.810),(.298,.895),(.405,.938),(.500,1.025),(.370,1.036),(.262,.950),(.264,.892),(.188,.900),(.192,.835),(.115,.845),(.114,.782),(.051,.805),(.047,.727),(.015,.660)]

def leaf_midline_point(y,side):
    edges=[]
    for a,b in zip(HOPPIP_OUTLINE,HOPPIP_OUTLINE[1:]+HOPPIP_OUTLINE[:1]):
        if min(a[1],b[1])<=y<max(a[1],b[1]):
            edges.append(a[0]+(b[0]-a[0])*(y-a[1])/(b[1]-a[1]))
    edges.sort()
    lo,hi=max(zip(edges[::2],edges[1::2]),key=lambda pair:pair[1]-pair[0])
    x=(lo+hi)/2
    return (side*x,y,-.038-.06*x+.025*(y-.70)**2+.010),(hi-lo)/5

def skiploom_ear_point(point,side,inverse=False):
    x,y,z=point;dx=x-side*.13;dz=z-.058
    angle=math.radians(35 if inverse else -35);c=math.cos(angle);s=math.sin(angle)
    return (side*.13+c*dx+s*dz,y,.058-s*dx+c*dz)

def splayed_skiploom_ear(p,side):
    out=dict(p);out['positions']=[];out['normals']=[]
    c=math.cos(math.radians(-35));s=math.sin(math.radians(-35))
    for i in range(0,len(p['positions']),3):
        out['positions'].extend(skiploom_ear_point(p['positions'][i:i+3],side))
        x,y,z=p['normals'][i:i+3];out['normals'].extend((c*x+s*z,y,-s*x+c*z))
    return out

def jumpluff_seed_point(point,side,inverse=False):
    # Turn each complete lower assembly about its stem attachment. Y is
    # unchanged, preserving support height while separating the heads in depth.
    x,y,z=point;dx=x-side*.16;dz=z+.012
    angle=math.radians(35 if inverse else -35);c=math.cos(angle);s=math.sin(angle)
    return (side*.16+c*dx+s*dz,y,-.012-s*dx+c*dz)

def splayed_seed_part(p,side):
    # The sphere, bristles and stem use the same rigid rest-space transform.
    out=dict(p);out['positions']=[];out['normals']=[]
    c=math.cos(math.radians(-35));s=math.sin(math.radians(-35))
    for i in range(0,len(p['positions']),3):
        out['positions'].extend(jumpluff_seed_point(p['positions'][i:i+3],side))
        x,y,z=p['normals'][i:i+3];out['normals'].extend((c*x+s*z,y,-s*x+c*z))
    return out

def patch(name,color,outline,skin,depth=.006):
    dense=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):
        dense.extend((a[0]+(b[0]-a[0])*i/4,a[1]+(b[1]-a[1])*i/4) for i in range(4))
    return fitted_patch(name,color,dense,skin,depth)

def leaf_bow(y,side):
    t=max(0.,min(1.,(y-.604)/.432))
    return side*.035*math.sin(math.pi*t)

def crown_point(point,side):
    x,y,z=point;dx=x-side*.027;dz=z+.038+leaf_bow(y,side)
    c=math.cos(math.radians(-35));s=math.sin(math.radians(-35))
    return (side*.027+c*dx+s*dz,y,-.038-s*dx+c*dz)

def splayed_vein(p,side):
    out=dict(p)
    out['positions']=[v for i in range(0,len(p['positions']),3)
                      for v in crown_point(p['positions'][i:i+3],side)]
    return crease_normals(out)

def bowed_blade(p,side):
    # Refine the closed solid before bowing it. Midpoint subdivision retains
    # its serrated XY outline and thickness instead of rounding off its teeth.
    vertices=[];lookup={};faces=[]
    for i in range(0,len(p['indices']),3):
        face=[]
        for index in p['indices'][i:i+3]:
            v=tuple(p['positions'][index*3:index*3+3])
            if v not in lookup:lookup[v]=len(vertices);vertices.append(v)
            face.append(lookup[v])
        faces.append(tuple(face))
    for _ in range(2):
        mids={};refined=[]
        def midpoint(a,b):
            key=tuple(sorted((a,b)))
            if key not in mids:
                mids[key]=len(vertices)
                vertices.append(tuple((x+y)/2 for x,y in zip(vertices[a],vertices[b])))
            return mids[key]
        for a,b,c in faces:
            ab,bc,ca=midpoint(a,b),midpoint(b,c),midpoint(c,a)
            refined.extend(((a,ab,ca),(ab,b,bc),(ca,bc,c),(ab,bc,ca)))
        faces=refined
    vertices=[crown_point(v,side) for v in vertices]
    out=crease_normals(part(p['part'],(1.,1.,1.),vertices,faces))
    out['base_color']=p['base_color']
    return out

def seed_surface(name,center,radius,lower):
    # Two closed solids meet at one shared scalloped latitude. Caps lie inside
    # the puff; the visible surface retains the same spherical silhouette.
    segments=32;split=8;levels=range(0,split+1) if lower else range(split,21)
    vertices=[]
    for j in levels:
        for i in range(segments):
            angle=math.tau*i/segments
            theta=math.pi*(20-j+.025)/20.05
            if j==split:
                y=-.31+.065*math.sin(5*angle)+.035*math.sin(11*angle)
                theta=math.acos(y)
            d=(math.sin(theta)*math.sin(angle),math.cos(theta),math.sin(theta)*math.cos(angle))
            vertices.append(tuple(c+radius*v for c,v in zip(center,d)))
    faces=[tuple(reversed(range(segments))),tuple((len(levels)-1)*segments+i for i in range(segments))]
    for j in range(len(levels)-1):
        for i in range(segments):
            a=j*segments+i;b=j*segments+(i+1)%segments
            faces.append((a,b,b+segments,a+segments))
    return crease_normals(part(name,CREAM if lower else (1.,1.,1.),vertices,faces,True))

def seed_globe(name,center,radius):
    p=[seed_surface(name+' / continuous seed sphere',center,radius,False),
       seed_surface(name+' / scalloped underside shade',center,radius,True)]
    # The seed filaments share the head's rigid owner and palette shade.
    for i in range(28):
        y=1-2*(i+.5)/28;a=i*math.pi*(3-math.sqrt(5));r=math.sqrt(1-y*y)
        d=(r*math.cos(a),y,r*math.sin(a))
        points=[tuple(c+(radius+q)*v for c,v in zip(center,d)) for q in (-.005,.014)]
        p.append(tube(name+' / fine seed bristle '+str(i),CREAM,points,[.0035,.0007],6))
    return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported wind plant')
    hop=species=='hoppip';skip=species=='skiploom';color=PINK if hop else SKIPLOOM_GREEN if skip else BLUE
    body=loft('Body / continuous rounded wind-borne torso',color,curved_profiles(((.11,.035,.032,0),(.17,.15,.117,0),(.28,.235,.174,0),(.40,.252,.184,-.003),(.51,.20,.151,-.011),(.585,.105,.086,-.015),(.612,.014,.014,-.020)),3),32)
    p=[body];skin=surface(body)
    for side in (-1,1):
        p.append(tube('Foot / short rounded supporting nub '+str(side),color,[(side*.09,.17,.015),(side*.108,.072,.035),(side*.110,.027,.055),(side*.110,.009,.070)],[.041,.036,.020,.003],18))
        p.append(ellipsoid('Arm / small rounded front flipper '+str(side),color,(side*.18,.287,.145),(.049,.077,.040),20,12))
        x=side*.09;y=.432 if skip else .438;z=skin(y,x)
        p.append(ellipsoid('Eye / fitted colored oval '+str(side),YELLOW if hop else RED,(x,y,z+.006),(.020,.023,.009),20,12))
        p.append(ellipsoid('Eye / ivory highlight '+str(side),WHITE,(x+side*.004,y+.007,z+.014),(.005,.007,.003),12,8))
    if skip:
        smile=[(-.092,.345),(-.046,.322),(0,.316),(.046,.322),(.092,.345)]
        p.append(tube('Mouth / gentle fitted smile',INK,[(x,y,skin(y,x)+.003) for x,y in smile],[.005]*5,10))
    else:
        w=.069 if hop else .028;top=.355;bottom=.238 if hop else .272
        outline=[(-w,top),(w,top),(w*.65,bottom+.025),(0,bottom),(-w*.65,bottom+.025)]
        p.append(patch('Mouth / open fitted smile',INK,outline,skin))
        tongue=[(-w*.43,bottom+.016),(0,bottom+.010),(w*.43,bottom+.016),(w*.48,bottom+.035),(0,bottom+.046),(-w*.48,bottom+.035)]
        p.append(patch('Tongue / fitted lower smile',TONGUE,tongue,lambda y,x:skin(y,x)+.008,.003))
    if hop:
        outline=HOPPIP_OUTLINE
        for side in (-1,1):
            leaf=membrane('Leaf / continuous serrated crown blade '+str(side),HOPPIP_LEAF,outline,side,-.038,thickness=.008);p.append(bowed_blade(leaf,side))
            points=[(side*x,y,-.038-.06*x+.025*(y-.70)**2+.010) for x,y in ((.027,.604),(.101,.761),(.235,.894),(.449,1.01))]
            radii=[.0015,.004,.0025,.0007];dense=[];sizes=[]
            for i,(a,b) in enumerate(zip(points,points[1:])):
                for k in range(4):
                    t=k/4;point=tuple(x+(y-x)*t for x,y in zip(a,b))
                    fitted,limit=leaf_midline_point(point[1],side);dense.append(fitted)
                    sizes.append(min(radii[i]+(radii[i+1]-radii[i])*t,limit))
            fitted,limit=leaf_midline_point(points[-1][1],side);dense.append(fitted);sizes.append(min(radii[-1],limit))
            p.append(splayed_vein(tube('Leaf vein / tapered midrib '+str(side),VEIN,dense,sizes,10),side))
            ear=membrane('Ear / pointed pink shell '+str(side),PINK,[(.13,.49),(.225,.515),(.351,.645),(.223,.638),(.15,.576)],side,.038,thickness=.023);p.append(ear)
            ear_skin=surface(ear);inner=[(side*.229,.557),(side*.309,.622),(side*.249,.604)]
            p.append(patch('Ear / fitted mauve interior '+str(side),INK,inner,ear_skin,.004))
        p.append(tube('Tail / curved pink petiole',PINK,[(0,.25,-.15),(0,.27,-.26),(.03,.30,-.32)],[.031,.029,.023],16))
        p.append(ellipsoid('Tail / flattened mauve tip',MAUVE,(.035,.305,-.32),(.075,.033,.065),20,12))
    elif skip:
        for side in (-1,1):
            outline=contour([((.105,.535),(.13,.59),(.18,.61),(.24,.594)),((.24,.594),(.34,.57),(.41,.52),(.427,.471)),((.427,.471),(.39,.41),(.31,.41),(.26,.428)),((.26,.428),(.17,.45),(.11,.47),(.105,.535))],7)
            leaf=membrane('Ear leaf / broad folded green paddle '+str(side),SKIPLOOM_GREEN,outline,side,.058,thickness=.019);p.append(splayed_skiploom_ear(leaf,side))
            skin_ear=surface(leaf)
            points=[(side*x,y,skin_ear(y,side*x)+.004) for x,y in ((.135,.53),(.224,.527),(.335,.48))]
            p.append(splayed_skiploom_ear(tube('Ear fold / curved fitted crease '+str(side),VEIN,points,[.0035,.003,.0008],8),side))
        p.append(tube('Flower / short supporting stalk',SKIPLOOM_GREEN,[(0,.568,-.025),(0,.655,-.025)],[.038,.031],18))
        for i in range(8):
            a=math.tau*i/8;petal=ellipsoid('Flower petal / radial spoon '+str(i),SKIPLOOM_GOLD,(0,.665,.125),(.066,.025,.104),24,12)
            petal['positions']=[v for k in range(0,len(petal['positions']),3) for v in (petal['positions'][k]*math.cos(a)+petal['positions'][k+2]*math.sin(a),petal['positions'][k+1],-petal['positions'][k]*math.sin(a)+petal['positions'][k+2]*math.cos(a)-.025)]
            petal['normals']=[v for k in range(0,len(petal['normals']),3) for v in (petal['normals'][k]*math.cos(a)+petal['normals'][k+2]*math.sin(a),petal['normals'][k+1],-petal['normals'][k]*math.sin(a)+petal['normals'][k+2]*math.cos(a))]
            p.append(petal)
        p.append(ellipsoid('Flower / pale green central disk',(.80,.87,.61),(0,.682,-.025),(.075,.023,.075),24,12))
    else:
        for side in (-1,1):
            p.append(splayed_seed_part(tube('Seed stem / lower globe '+str(side),GREEN,[(side*.16,.275,-.012),(side*.29,.198,-.014)],[.021,.018],12),side))
            p.extend(splayed_seed_part(q,side) for q in seed_globe('Seed head / lower '+str(side),(side*.306,.166,-.014),.145))
        p.append(tube('Seed stem / tall crown peduncle',GREEN,[(0,.573,-.025),(0,.793,-.025)],[.024,.018],14))
        for side in (-1,1):
            p.append(membrane('Leaf / small crown bract '+str(side),LEAF,[(.006,.605),(.078,.655),(.105,.657),(.075,.613),(.009,.582)],side,-.020,thickness=.007))
        p.extend(seed_globe('Seed head / crown',(0,.908,-.025),.197))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low) if normalize else 1.
    for q in p:
        q['positions']=[round((v-(low if normalize and i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        # Keep exact center zeros canonical across the two authoring runtimes.
        if (species=='hoppip' and q['part'].startswith('Mouth /')) or (species=='jumpluff' and q['part'].startswith('Tongue /')):
            q['positions']=[0. if v==0. else v for v in q['positions']]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))

if __name__=='__main__':main()
