#!/usr/bin/env python3
"""Original seated Snorlax: broad sleeping bear, fitted cream mask and belly."""
import math
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour,translate,crease_normals
from abra_sculpt import surface
from bovine_sculpt import curved_tube,oval
from skin_glb import COORDINATES
HEIGHT=1.40
BLUE=(.25,.48,.56);CREAM=(.97,.93,.82);PAD=(.52,.47,.36)
IVORY=(.99,.98,.90);INK=(.13,.15,.16)


def sculpture():
    body=loft('Body / continuous broad seated sleeping bear',BLUE,curved_profiles(
        ((.065,.16,.14,-.045),(.13,.39,.30,-.03),(.29,.49,.37,-.015),
         (.53,.53,.40,0.),(.76,.49,.355,-.015),(.95,.36,.26,-.04),
         (1.08,.24,.18,-.045),(1.115,.06,.06,-.045)),4),32)
    p=[body];bs=surface(body)
    outline=[(-.20,.20),(-.34,.27),(-.43,.42),(-.45,.61),(-.40,.79),
             (-.28,.96),(-.13,1.005),(.13,1.005),(.28,.96),(.40,.79),
             (.45,.61),(.43,.42),(.34,.27),(.20,.20),(0.,.17)]
    p.append(crease_normals(fitted_contour('Belly / fitted broad cream bib',CREAM,outline,bs,.018,subdivisions=4),55.))
    head=loft('Head / wide blue sleeping hood',BLUE,curved_profiles(
        ((.98,.10,.11,.055),(1.02,.23,.20,.075),(1.12,.295,.245,.075),
         (1.25,.29,.235,.055),(1.35,.25,.195,.030),(1.41,.16,.125,.015),
         (1.425,.035,.03,.005)),3),32)
    p.append(head);hs=surface(head)
    mask=[(-.23,1.27),(-.18,1.335),(-.095,1.375),(0.,1.29),(.095,1.375),
          (.18,1.335),(.23,1.27),(.25,1.17),(.20,1.08),(.09,1.035),
          (-.09,1.035),(-.20,1.08),(-.25,1.17)]
    face=crease_normals(fitted_contour('Face / fitted cream notched sleeping mask',CREAM,mask,hs,.014,subdivisions=3),55.);p.append(face);fs=surface(face)
    for side in (-1,1):
        # Soft triangular ears grow out of the hood, with no exposed base cap.
        ear=loft('Ear / pointed blue hood ear '+str(side),BLUE,curved_profiles(
            ((1.28,.096,.084,.00),(1.39,.087,.070,-.012),(1.44,.067,.047,-.022),
             (1.49,.026,.024,-.028),(1.50,.002,.003,-.027)),3),20)
        ear=translate(ear,side*.226,0.,0.)
        ear['positions']=[v+side*.07*max(0.,(ear['positions'][i//3*3+1]-1.28)/.22) if i%3==0 else v for i,v in enumerate(ear['positions'])]
        p.append(ear)
        points=[]
        for i in range(13):
            x=side*(.059+.012*i);y=1.205+.019*(i/12-.5)**2*4
            points.append((x,y,fs(y,x)+.002))
        p.append(tube('Eye / closed sleepy crease '+str(side),INK,points,[.0033]*len(points),8))
        arm=[(side*.30,.955,-.015),(side*.445,.89,.020),(side*.61,.775,.080),(side*.675,.685,.115)]
        p.append(curved_tube('Arm / thick drooping forelimb '+str(side),BLUE,arm,[.129,.144,.115,.088],22))
        hand=ellipsoid('Hand / rounded blue paw '+str(side),BLUE,(side*.665,.691,.133),(.093,.086,.108),22,14);p.append(hand)
        for j,dx in enumerate((-.045,0.,.045)):
            x=side*(.67+dx)
            p.append(curved_tube('Hand claw / '+str(side)+' '+str(j),IVORY,
                [(x,.660,.206),(x+side*.006,.628,.228),(x+side*.011,.614,.245)],[.017,.013,.002],12))
        foot=ellipsoid('Foot / broad cream planted sole '+str(side),CREAM,(side*.355,.133,.29),(.181,.133,.166),28,16);p.append(foot)
        footskin=surface(foot)
        p.append(fitted_patch('Foot pad / fitted brown oval '+str(side),PAD,oval(side*.36,.105,.095,.070,28),footskin,.006))
        for j,dx in enumerate((-.12,0.,.12)):
            x=side*(.355+dx);y=.218 if j!=1 else .251;z=.365 if j!=1 else .373
            p.append(curved_tube('Foot claw / '+str(side)+' '+str(j),IVORY,
                [(x,y-.032,z-.018),(x+side*.007,y+.019,z+.008),(x+side*.012,y+.045,z+.016)],[.031,.019,.002],14))
    mouth=[]
    for i in range(23):
        x=-.10+.20*i/22;y=1.115+.018*(abs(x)/.10)**2
        mouth.append((x,y,fs(y,x)+.003))
    p.append(tube('Mouth / fitted quiet smile',INK,mouth,[.003]*len(mouth),8))
    for side in (-1,1):
        x=side*.085;y=1.128;z=fs(y,x)
        p.append(curved_tube('Fang / little upward ivory tooth '+str(side),IVORY,
            [(x,y-.003,z+.003),(x,y+.014,z+.006),(x-side*.003,y+.027,z+.007)],[.012,.008,.0015],12))
    # Broaden the seated body and compress the torso beneath the hood.
    # Apply the inverse transpose to normals under this piecewise scale.
    for q in p:
        points=[];normals=[]
        for i in range(0,len(q['positions']),3):
            x,y,z=q['positions'][i:i+3];nx,ny,nz=q['normals'][i:i+3]
            if q['part'].startswith('Ear /'):
                side=-1 if x<0 else 1
                ny-=side*(.07/.22)*nx
            sy=.90 if y<1.05 else 1.
            points.extend((x*1.15,y*.90 if y<1.05 else y-.105,z))
            n=(nx/1.15,ny/sy,nz);length=math.sqrt(sum(v*v for v in n));normals.extend(v/length for v in n)
        q['positions']=points;q['normals']=normals
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'snorlax','version':1,'coordinate_system':COORDINATES,'primitives':p}
