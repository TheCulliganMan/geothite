#!/usr/bin/env python3
"""Original small Pidgey: cream breast, dark cheek wedges, folded wings and pink feet."""
import math
from sculpt_geometry import ellipsoid,fitted_contour,crease_normals,tube
from goldfish_sculpt import surface
from bovine_sculpt import oval
from lugia_sculpt import paddle
from skin_glb import COORDINATES
HEIGHT=.82
BROWN=(.65,.43,.22);LIGHT=(.77,.56,.31);CREAM=(.97,.91,.66)
INK=(.13,.11,.09);WHITE=(.99,.98,.92);PINK=(.77,.48,.54)


def paint(name,color,xy,parent,th=.004,subdivisions=2):
    return crease_normals(fitted_contour(name,color,xy,surface(parent),th,subdivisions=subdivisions),40.)


def sculpture():
    body=ellipsoid('Body / round ochre tiny bird torso',BROWN,(0.,.265,-.035),(.177,.190,.157),24,16)
    head=ellipsoid('Head / small brown rounded bird head',BROWN,(0.,.482,.060),(.145,.153,.133),24,16)
    p=[body,head]
    p.append(paint('Face / fitted cream throat and central muzzle',CREAM,
        [(-.026,.546),(-.037,.493),(-.086,.455),(-.078,.395),(-.030,.347),(0.,.337),(.030,.347),(.078,.395),(.086,.455),(.037,.493),(.026,.546)],head,.006,3))
    p.append(paint('Breast / fitted broad cream feather bib',CREAM,
       [(-.063,.431),(-.125,.368),(-.154,.270),(-.133,.159),(-.081,.108),(0.,.094),(.081,.108),(.133,.159),(.154,.270),(.125,.368),(.063,.431),(0.,.450)],body,.015,3))
    for side in (-1,1):
        cheek=paint('Cheek / dark angular eye wedge '+str(side),INK,
            [(side*x,y) for x,y in ((.031,.530),(.112,.547),(.141,.493),(.103,.436),(.026,.476))],head,.004);p.append(cheek)
        eye=paint('Eye / ivory alert eye '+str(side),WHITE,
            [(side*x,y) for x,y in ((.037,.521),(.099,.530),(.105,.508),(.089,.491),(.048,.497))],cheek,.003);p.append(eye)
        p.append(paint('Pupil / small dark alert gaze '+str(side),INK,oval(side*.065,.511,.010,.011,16),eye,.002))
        p.append(paint('Brow / cream swept feather ridge '+str(side),CREAM,
            [(side*x,y) for x,y in ((.017,.541),(.049,.570),(.112,.568),(.127,.544),(.106,.546),(.053,.541))],head,.007))
        p.append(paddle('Crown / short pointed brown crest '+str(side),LIGHT,
            [(side*.027,.561,.063),(side*.053,.622,.015),(side*.065,.652,-.060)],
            [.030,.025,.002],[.048,.030,.002],segments=16,steps=4))
        wing=paddle('Wing / folded ochre shoulder '+str(side),LIGHT,
            [(side*.132,.360,-.019),(side*.200,.274,-.040),(side*.174,.167,-.129)],
            [.057,.068,.016],[.078,.072,.018],segments=20,steps=5);p.append(wing)
        for j in range(2):
            p.append(paddle('Flight feather / cream folded wing tip '+str(side)+' '+str(j),CREAM,
                [(side*(.179+.020*j),.255,-.078),(side*(.193+.016*j),.178,-.146),(side*(.178+.014*j),.113,-.207)],
                [.033,.027,.0015],[.033,.027,.0015],segments=14,steps=4))
        p.append(tube('Leg / short pink bird shank '+str(side),PINK,
            [(side*.077,.140,-.009),(side*.078,.072,.004),(side*.078,.033,.016)],[.017,.016,.018],12))
        p.append(ellipsoid('Foot / planted pink bird pad '+str(side),PINK,(side*.077,.025,.025),(.043,.022,.045),16,10))
        for j,dx in enumerate((-.029,0.,.029)):
            xx=side*.077+dx;zz=.134-(.012 if j!=1 else 0.)
            p.append(tube('Toe / three forward pink digits '+str(side)+' '+str(j),PINK,
                [(side*.077+dx*.35,.028,.040),(xx,.019,.084),(xx+dx*.25,.015,zz)],[.012,.011,.008],12))
            p.append(paddle('Claw / ivory forward talon '+str(side)+' '+str(j),WHITE,
                [(xx+dx*.25,.017,zz-.009),(xx+dx*.30,.015,zz+.014),(xx+dx*.30,.007,zz+.033)],
                [.008,.006,.001],[.009,.006,.001],segments=12,steps=3))
        p.append(tube('Rear toe / opposing pink bird digit '+str(side),PINK,
            [(side*.077,.026,.001),(side*.097,.016,-.044),(side*.101,.012,-.061)],[.012,.010,.006],12))
    p.append(paddle('Upper beak / small hooked pink bill',PINK,
        [(0.,.493,.165),(0.,.503,.226),(0.,.483,.270)],
        [.039,.032,.002],[.023,.019,.002],segments=20,steps=5))
    p.append(paddle('Lower beak / articulated pink lower bill',PINK,
        [(0.,.478,.166),(0.,.466,.220),(0.,.479,.254)],
        [.033,.024,.002],[.018,.014,.002],segments=18,steps=4))
    for j,x in enumerate((-.048,0.,.048)):
        p.append(paddle('Tail / three short brown fan feathers '+str(j),LIGHT if j==1 else BROWN,
            [(x*.45,.208,-.142),(x,.226,-.266),(x*1.35,.268,-.386)],
            [.025,.036,.008],[.020,.018,.004],segments=16,steps=5))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'pidgey','version':1,'coordinate_system':COORDINATES,'primitives':p}
