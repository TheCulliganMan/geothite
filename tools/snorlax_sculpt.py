#!/usr/bin/env python3
"""Original Crystal-seated Snorlax with a deep rounded body and fitted sleeping face."""
import math
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_triangle_contour,translate,part,crease_normals
from abra_sculpt import surface
from bovine_sculpt import curved_tube,oval
from skin_glb import COORDINATES
HEIGHT=1.40
# Full-color game pigments; Crystal's front animation governs seated posture.
BLUE=(0.,98/255,109/255);CREAM=(236/255,213/255,175/255);PAD=(126/255,67/255,18/255)
IVORY=(.99,.98,.94);INK=(.055,.050,.045)


def mark(name,color,outline,parent,thickness=.008):
    return fitted_triangle_contour(name,color,outline,parent,thickness,precision=7)


@lru_cache(maxsize=1)
def raw_sculpture():
    body=loft('Body / continuous broad seated sleeping bear',BLUE,curved_profiles(
        ((.025,.15,.20,-.11),(.09,.42,.39,-.09),(.24,.58,.50,-.055),
         (.46,.62,.55,-.020),(.69,.57,.48,-.035),(.88,.43,.34,-.065),
         (1.02,.28,.22,-.10),(1.075,.10,.10,-.10)),4),36)
    p=[body]
    outline=[(-.20,.13),(-.39,.23),(-.51,.40),(-.53,.59),(-.46,.78),
             (-.31,.95),(-.13,1.005),(.13,1.005),(.31,.95),(.46,.78),
             (.53,.59),(.51,.40),(.39,.23),(.20,.13),(0.,.105)]
    p.append(mark('Belly / fitted broad cream bib',CREAM,outline,body,.011))
    head=loft('Head / wide blue sleeping hood',BLUE,curved_profiles(
        ((.855,.15,.14,.035),(.91,.29,.25,.015),(1.02,.385,.285,-.015),
         (1.16,.405,.27,-.045),(1.28,.33,.22,-.065),(1.325,.20,.14,-.075),
         (1.34,.035,.03,-.075)),4),36);p.append(head)
    mask=[(-.31,1.225),(-.25,1.285),(-.15,1.295),(0.,1.23),(.15,1.295),
          (.25,1.285),(.31,1.225),(.345,1.135),(.32,1.025),(.24,.96),
          (.12,.915),(-.12,.915),(-.24,.96),(-.32,1.025),(-.345,1.135)]
    face=mark('Face / fitted cream notched sleeping mask',CREAM,mask,head,.008);p.append(face);fs=surface(face)
    for side in (-1,1):
        ear=loft('Ear / pointed blue hood ear '+str(side),BLUE,curved_profiles(
            ((1.19,.10,.09,-.07),(1.28,.09,.075,-.09),(1.36,.045,.04,-.10),
             (1.40,.004,.004,-.10)),3),20)
        ear=translate(ear,side*.27,0.,0.)
        ear['positions']=[v+side*.05*max(0.,(ear['positions'][i//3*3+1]-1.19)/.21) if i%3==0 else v for i,v in enumerate(ear['positions'])]
        normal=[]
        for i in range(0,len(ear['normals']),3):
            x,y,z=ear['normals'][i:i+3];n=(x,y-side*(.05/.21)*x,z);length=math.sqrt(sum(v*v for v in n));normal.extend(v/length for v in n)
        ear['normals']=normal
        p.append(ear)
        eye=[]
        for i in range(15):
            x=side*(.075+.0125*i);y=1.143+.018*(i/14-.5)**2*4
            eye.append((x,y,fs(y,x)+.0015))
        p.append(tube('Eye / closed sleepy crease '+str(side),INK,eye,[.0035]*len(eye),8))
        arm=[(side*.25,.88,-.015),(side*.53,.85,.050),(side*.62,.79,.160),(side*.58,.78,.290)]
        p.append(curved_tube('Arm / thick bent resting forelimb '+str(side),BLUE,arm,[.125,.15,.12,.10],20))
        hand=ellipsoid('Hand / rounded blue paw '+str(side),BLUE,(side*.58,.78,.290),(.107,.098,.120),24,16);p.append(hand)
        for j,dx in enumerate((-.043,0.,.043)):
            x=side*(.58+dx)
            p.append(curved_tube(f'Hand claw / {side} {j}',IVORY,
                [(x,.755,.370),(x+side*.006,.725,.405),(x+side*.010,.710,.430)],[.018,.014,.002],12))
        foot=ellipsoid('Foot / broad cream planted sole '+str(side),CREAM,(side*.37,.155,.350),(.225,.155,.225),28,18);p.append(foot)
        p.append(mark('Foot pad / fitted brown oval '+str(side),PAD,oval(side*.37,.13,.130,.105,28),foot,.006))
        for j,dx in enumerate((-.13,0.,.13)):
            x=side*(.37+dx);y=.240 if j!=1 else .280;z=.460 if j!=1 else .475
            p.append(curved_tube(f'Foot claw / {side} {j}',IVORY,
                [(x,y,z),(x+side*.007,y+.035,z+.055),(x+side*.012,y+.065,z+.085)],[.032,.020,.002],14))
    mouth=[]
    for i in range(23):
        x=-.11+.22*i/22;y=1.020+.014*(abs(x)/.11)**2
        mouth.append((x,y,fs(y,x)+.0015))
    p.append(tube('Mouth / fitted quiet smile',INK,mouth,[.003]*len(mouth),8))
    for side in (-1,1):
        x=side*.098;y=1.033;z=fs(y,x)
        p.append(curved_tube('Fang / little upward ivory tooth '+str(side),IVORY,
            [(x,y-.004,z+.002),(x,y+.012,z+.006),(x-side*.003,y+.027,z+.007)],[.010,.007,.0015],12))
    return {'name':'snorlax','version':1,'coordinate_system':COORDINATES,'primitives':p}


def sculpture():
    model=deepcopy(raw_sculpture());p=model['primitives']
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    from chikorita_sculpt import f32
    for q in p:
        fitted=q['part'].startswith(('Belly /','Face /','Foot pad /'))
        q['positions']=[f32((v-(lo if i%3==1 else 0.))*scale) if fitted else round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    from chikorita_sculpt import f32
    for i,q in enumerate(p):
        if not q['part'].startswith(('Belly /','Face /','Foot pad /')):continue
        vs=[tuple(f32(v) for v in q['positions'][j:j+3]) for j in range(0,len(q['positions']),3)]
        faces=[tuple(q['indices'][j:j+3]) for j in range(0,len(q['indices']),3)]
        rebuilt=crease_normals(part(q['part'],(0.,0.,0.),vs,faces,True),55.);rebuilt['base_color']=q['base_color'];p[i]=rebuilt
    return model
