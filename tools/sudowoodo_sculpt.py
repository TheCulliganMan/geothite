#!/usr/bin/env python3
"""Original Sudowoodo sculpture: forked stone trunk, branch arms and green digits.

The native skin is authored by sudowoodo_glb.py. No icon geometry is imported.
"""
import math
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals
from bovine_sculpt import curved_tube,oval
from abra_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.04
BROWN=(.67,.48,.35);CUT=(.79,.59,.43);GREEN=(.36,.68,.30)
GOLD=(.98,.91,.61);INK=(.11,.10,.10);PINK=(.86,.49,.43)
CROWN='Crown / rooted forked branch'


def sculpture():
    body=loft('Body / continuous leaning stone trunk',BROWN,curved_profiles(
        ((.11,.060,.065,-.035),(.18,.154,.130,-.035),(.30,.164,.126,-.019),
         (.43,.139,.115,.008),(.56,.111,.102,.030),(.68,.110,.099,.042),
         (.80,.115,.098,.047),(.87,.091,.074,.045),(.89,.015,.015,.044)),3),24)
    p=[crease_normals(body,45.)];skin=surface(body)
    for sign in (-1,1):
        eye=fitted_patch('Eye / fitted small ink dot '+str(sign),INK,oval(sign*.045,.770,.012,.022,18),skin,.003);p.append(eye)
        p.append(fitted_patch('Eye / tiny ivory glint '+str(sign),(.98,.97,.90),oval(sign*.045-.003,.777,.003,.004,12),surface(eye),.0015))
        p.append(curved_tube('Leg / bowed planted root '+str(sign),BROWN,
            [(sign*.098,.216,-.014),(sign*.174,.129,.003),(sign*.193,.060,.054)],[.055,.047,.040],16))
        p.append(ellipsoid('Foot / grounded flattened root pad '+str(sign),BROWN,(sign*.206,.041,.072),(.099,.041,.073),24,12))
        arm=[(sign*.093,.568,.033),(sign*.236,.601,.034),(sign*.343,.684,.035),(sign*.397,.744,.037)]
        p.append(curved_tube('Arm / raised bent branch '+str(sign),BROWN,arm,[.039,.037,.029,.027],16))
        for j,(dx,dy) in enumerate(((-.048,.020),(.027,.089),(.080,-.035))):
            tip=(sign*(.397+dx),.744+dy,.046)
            p.append(curved_tube('Finger stalk / '+str(sign)+' '+str(j),BROWN,[arm[-1],tip],[.019,.015],12))
            p.append(ellipsoid('Finger orb / '+str(sign)+' '+str(j),GREEN,tip,(.062,.061,.057),20,12))
    smile=[]
    for i in range(17):
        x=-.071+.142*i/16;y=.701+.025*(abs(x)/.071)**1.6
        smile.append((x,y,skin(y,x)+.004))
    p.append(tube('Smile / fitted pink upward curve',PINK,smile,[.0035]*len(smile),8))
    for j,(x,y,rx,ry) in enumerate(((-.036,.202,.014,.037),(.025,.308,.018,.048),
                                   (-.040,.427,.013,.032),(.031,.486,.010,.022),(.034,.560,.017,.046))):
        p.append(fitted_patch('Spot / fitted golden oval '+str(j),GOLD,oval(x,y,rx,ry,20),skin,.003))
    p.append(curved_tube(CROWN,BROWN,[(0.,.836,.034),(-.008,.916,.033),(-.048,.949,.030),(-.147,.980,.026)],[.028,.027,.026,.028],18))
    p.append(curved_tube('Crown / upward right fork',BROWN,[(-.008,.916,.033),(.050,.964,.030),(.103,1.037,.026)],[.027,.026,.026],18))
    for sign,center in ((-1,(-.147,.980,.026)),(1,(.103,1.037,.026))):
        # Small rounded cut surfaces are embedded in the twig endpoints.
        p.append(ellipsoid('Crown / pale rounded cut '+str(sign),CUT,center,(.025,.021,.022),18,10))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'sudowoodo','version':1,'coordinate_system':COORDINATES,'primitives':p}
