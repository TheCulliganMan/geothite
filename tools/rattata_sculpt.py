#!/usr/bin/env python3
"""Original Rattata: low purple quadruped, incisors, whiskers and curled tail."""
from sculpt_geometry import ellipsoid,fitted_contour,crease_normals,tube,part
from goldfish_sculpt import surface
from bovine_sculpt import oval,smooth
from skin_glb import COORDINATES
HEIGHT=.82
PURPLE=(.74,.39,.94);CREAM=(.93,.86,.67);WHITE=(.98,.97,.88)
PINK=(.71,.39,.53);INK=(.085,.065,.12);RED=(.64,.17,.22)
TAIL_POINTS=((0.,.225,-.245),(0.,.26,-.35),(.025,.36,-.43),(.075,.49,-.49),
             (.16,.565,-.49),(.245,.55,-.47),(.275,.475,-.44),(.23,.425,-.43),(.18,.448,-.44))
TAIL,RADII=smooth(TAIL_POINTS,[.025,.021,.018,.017,.016,.015,.014,.012,.007],6)


def paint(name,color,xy,parent,th=.003,subdivisions=2):
    return crease_normals(fitted_contour(name,color,xy,surface(parent),th,subdivisions=subdivisions),45.)


def tooth(name,x):
    cx,cy,cz=x,.235,.425;rx,ry,rz=.021,.034,.012
    vertices=[(cx+a*rx,cy+b*ry,cz+c*rz) for a,b,c in
        [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]]
    return part(name,WHITE,vertices,[(3,2,1,0),(4,5,6,7),(0,1,5,4),(2,3,7,6),(0,4,7,3),(1,2,6,5)])


def raw_parts():
    body=ellipsoid('Body / low purple rodent barrel',PURPLE,(0.,.235,-.075),(.145,.155,.25),28,18)
    head=ellipsoid('Head / broad purple rodent cranium',PURPLE,(0.,.325,.205),(.153,.145,.178),28,18)
    p=[body,ellipsoid('Belly / cream low rodent underside',CREAM,(0.,.14,-.035),(.116,.071,.21),24,14),
       ellipsoid('Neck / overlapping purple throat',PURPLE,(0.,.265,.105),(.125,.102,.115),24,14),head,
       ellipsoid('Mouth cavity / recessed open dark mouth',INK,(0.,.234,.375),(.073,.05,.029),24,14),
       ellipsoid('Jaw / cream lower rodent chin',CREAM,(0.,.197,.365),(.088,.031,.065),24,14)]
    for side in (-1,1):
        p.append(ellipsoid('Muzzle / cream upper cheek '+str(side),CREAM,(side*.057,.272,.358),(.065,.046,.055),24,14))
        p.append(tooth('Incisor / separate broad ivory front tooth '+str(side),side*.023))
        ear=ellipsoid('Ear / round upright purple shell '+str(side),PURPLE,(side*.132,.449,.12),(.071,.086,.032),24,16);p.append(ear)
        p.append(paint('Ear lining / fitted dusty pink bowl '+str(side),PINK,oval(side*.132,.456,.050,.061,28),ear,.004))
        eye=paint('Eye / fitted alert ivory wedge '+str(side),WHITE,
            [(side*.046,.369),(side*.113,.403),(side*.119,.352),(side*.057,.340)],head,.004);p.append(eye)
        pupil=paint('Pupil / fitted red rodent gaze '+str(side),RED,oval(side*.089,.374,.011,.014,24),eye,.003);p.append(pupil)
        p.append(paint('Eye glint / small ivory eye shine '+str(side),WHITE,oval(side*.086,.380,.003,.004,16),pupil,.0015))
        brow=[]
        skin=surface(head)
        for i in range(13):
            t=i/12;x=side*(.045+.072*t);y=.373+.034*t;brow.append((x,y,skin(y,x)+.008))
        p.append(tube('Brow / fitted dark alert upper eyelid '+str(side),INK,brow,[.003]*len(brow),8))
        for j in range(3):
            p.append(tube('Whisker / thin cream muzzle filament '+str(side)+' '+str(j),CREAM,
                [(side*.091,.270-j*.009,.39),(side*.16,.282-j*.016,.411),
                 (side*.218,.307-j*.027,.409)],[.0033,.0025,.001],8))
        for label,z in [('front',.19),('rear',-.21)]:
            p.append(ellipsoid('Leg / short purple '+label+' shank '+str(side),PURPLE,(side*.12,.098,z),(.043,.082,.05),16,12))
            p.append(ellipsoid('Paw / planted cream '+label+' foot '+str(side),CREAM,(side*.123,.025,z+.017),(.047,.026,.066),20,12))
            points=[(side*.123+dx,.021,z+.067) for dx in (-.04,-.025,-.014,0.,.014,.025,.04)]
            p.append(tube('Toes / planted cream '+label+' three-lobed toe ridge '+str(side),CREAM,
                points,[.003,.017,.011,.017,.011,.017,.003],12))
    p.append(ellipsoid('Nose / small rounded pink rodent nose',PINK,(0.,.294,.402),(.034,.021,.022),20,12))
    p.append(tube('Tail / raised purple curl with continuous taper',PURPLE,TAIL,RADII,16))
    return p


def authoring_transform(parts=None):
    p=raw_parts() if parts is None else parts
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3])
    return lo,HEIGHT/(hi-lo)


def sculpture():
    p=raw_parts();lo,scale=authoring_transform(p)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'rattata','version':1,'coordinate_system':COORDINATES,'primitives':p}
