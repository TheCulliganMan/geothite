#!/usr/bin/env python3
"""Original Geodude rock face and two bent stone arms with clenched fists."""
import math
from sculpt_geometry import part,ellipsoid,crease_normals,fitted_contour,tube
from goldfish_sculpt import surface
from bovine_sculpt import oval
from lugia_sculpt import paddle
from chikorita_sculpt import unit,cross,sub
from skin_glb import COORDINATES
HEIGHT=.76
STONE=(.62,.62,.58);LIGHT=(.71,.71,.66);SHADE=(.50,.51,.47)
WHITE=(.98,.98,.91);INK=(.09,.10,.10)


def faceted(p,color=None):
    vertices=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
    q=part(p['part'],STONE if color is None else color,vertices,[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)],False)
    return crease_normals(q,30.)


def paint(name,color,xy,parent,th=.004):
    return crease_normals(fitted_contour(name,color,xy,surface(parent),th,subdivisions=3),40.)


def sculpture():
    base=ellipsoid('Body / irregular single grey Geodude boulder',STONE,(0.,.300,0.),(.305,.300,.245),16,10)
    vv=[]
    for i in range(0,len(base['positions']),3):
        x,y,z=base['positions'][i:i+3];a=math.atan2(z,x);q=(y-.30)/.30
        factor=1.+(.063*math.sin(5*a+.3)+.033*math.cos(3*a-.7))*max(0.,1.-q*q)
        vv.append((x*factor,y,z*factor))
    body=part(base['part'],STONE,vv,[tuple(base['indices'][i:i+3]) for i in range(0,len(base['indices']),3)],False)
    p=[body];skin=surface(body)
    for side in (-1,1):
        brow=[(side*x,y) for x,y in ((.019,.422),(.083,.473),(.161,.460),(.189,.411),(.142,.407),(.071,.393))]
        p.append(paint('Brow / fitted heavy angular rock ridge '+str(side),LIGHT,brow,body,.013))
        eye=paint('Eye / fitted ivory angular rock stare '+str(side),WHITE,
            [(side*x,y) for x,y in ((.026,.386),(.152,.405),(.161,.351),(.046,.340))],body,.005);p.append(eye)
        pupil=paint('Pupil / fitted narrow dark rock gaze '+str(side),INK,oval(side*.121,.371,.010,.023,20),eye,.002);p.append(pupil)
        p.append(paint('Eye glint / fitted small ivory rock highlight '+str(side),WHITE,oval(side*.119,.382,.003,.007,12),pupil,.001))
        p.append(faceted(paddle('Upper arm / bent grey stone shoulder '+str(side),STONE,
            [(side*.243,.349,.001),(side*.337,.304,.025),(side*.403,.268,.053)],
            [.053,.052,.050],[.055,.051,.050],segments=12,steps=4)))
        p.append(faceted(paddle('Forearm / raised grey stone wrist '+str(side),STONE,
            [(side*.403,.268,.053),(side*.470,.359,.083),(side*.493,.444,.116)],
            [.051,.056,.049],[.052,.052,.047],segments=12,steps=4)))
        fist=faceted(ellipsoid('Fist / broad clenched grey stone palm '+str(side),STONE,(side*.515,.472,.128),(.096,.090,.089),14,10));p.append(fist)
        for j,dx in enumerate((-.063,-.021,.021,.063)):
            p.append(faceted(ellipsoid('Knuckle / four clenched grey stone fingers '+str(side)+' '+str(j),LIGHT,
                (side*(.515+dx),.494+(.004 if j in (1,2) else -.009),.190),(.027,.046,.042),12,8),LIGHT))
        p.append(faceted(ellipsoid('Thumb / folded grey stone digit '+str(side),SHADE,(side*.438,.431,.183),(.037,.043,.034),12,8),SHADE))
        # One closed crease on the inside of the fist reads its thumb fold.
        fs=surface(fist);xy=[(side*.470,.451),(side*.493,.428),(side*.505,.424)]
        points=[(x,y,fs(y,x)+.002) for x,y in xy]
        p.append(tube('Fist fold / fitted dark stone crease '+str(side),SHADE,points,[.0018]*len(points),8))
    smile=[]
    for i in range(25):
        x=-.186+.372*i/24;y=.244+.035*(abs(x)/.186)**1.7+.012*math.cos(x/.186*math.pi)
        smile.append((x,y,skin(y,x)+.0025))
    p.append(tube('Mouth / fitted broad angular rock frown',INK,smile,[.0027]*len(smile),8))
    lip=[(-.128,.235),(-.080,.212),(0.,.203),(.080,.212),(.128,.235),(.077,.197),(0.,.185),(-.077,.197)]
    p.append(paint('Lip / fitted lower grey rock jaw ridge',LIGHT,lip,body,.006))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'geodude','version':1,'coordinate_system':COORDINATES,'primitives':p}
