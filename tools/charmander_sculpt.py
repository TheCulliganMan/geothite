#!/usr/bin/env python3
"""Original Charmander lizard with fitted cream belly and folded tail fire."""
import math
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_contour,crease_normals
from goldfish_sculpt import surface
from bovine_sculpt import curved_tube,oval
from cyndaquil_sculpt import flame
from chikorita_sculpt import linear
from skin_glb import COORDINATES
HEIGHT=1.00
ORANGE=(.97,.60,.32);CREAM=(.98,.93,.75);WHITE=(.99,.98,.94)
BLUE=(.18,.43,.51);INK=(.09,.10,.12);PINK=(.76,.47,.58)


def paint(name,color,outline,parent,th=.004,sub=2):
    return crease_normals(fitted_contour(name,color,outline,surface(parent),th,subdivisions=sub),50.)


def sculpture():
    body=loft('Body / continuous round orange lizard pear',ORANGE,curved_profiles(
        ((.115,.046,.053,-.015),(.16,.125,.114,-.015),(.27,.184,.155,-.01),
         (.39,.178,.151,-.007),(.50,.133,.116,.005),(.60,.078,.075,.020),(.64,.025,.030,.025)),4),30)
    head=loft('Head / rounded orange lizard cranium and muzzle',ORANGE,curved_profiles(
        ((.535,.046,.060,.08),(.58,.155,.153,.08),(.66,.211,.200,.065),
         (.75,.205,.181,.041),(.83,.154,.133,.026),(.875,.080,.072,.014),(.885,.014,.014,.012)),4),32)
    p=[body,head]
    belly=[(-.047,.158),(-.103,.190),(-.135,.265),(-.125,.39),(-.084,.505),
           (-.031,.573),(0.,.59),(.031,.573),(.084,.505),(.125,.39),(.135,.265),(.103,.190),(.047,.158),(0.,.144)]
    p.append(paint('Belly / fitted broad cream lizard bib',CREAM,belly,body,.013,sub=3))
    hs=surface(head)
    for side in (-1,1):
        eye=paint('Eye / fitted ivory lizard almond '+str(side),WHITE,
            [(side*x,y) for x,y in ((.065,.692),(.145,.694),(.168,.728),(.157,.778),(.132,.813),(.094,.792),(.077,.750))],head,.004);p.append(eye)
        iris=paint('Iris / fitted tall blue lizard gaze '+str(side),BLUE,oval(side*.128,.751,.027,.048,28),eye,.0025);p.append(iris)
        pupil=paint('Pupil / fitted dark vertical lizard oval '+str(side),INK,oval(side*.132,.759,.012,.032,24),iris,.0015);p.append(pupil)
        p.append(paint('Eye glint / fitted ivory lizard sparkle '+str(side),WHITE,oval(side*.129,.775,.004,.009,20),pupil,.001))
        p.append(paint('Nostril / fitted tiny lizard slit '+str(side),INK,oval(side*.044,.666,.005,.0025,16),head,.0018))
        x=side*.137;y=.622;z=hs(y,x)
        p.append(curved_tube('Fang / tiny hanging lizard tooth '+str(side),WHITE,[(x,y+.003,z+.002),(x,y-.009,z+.008),(x-side*.001,y-.016,z+.008)],[.007,.005,.0015],12))
        arm=[(side*.127,.508,.015),(side*.211,.464,.041),(side*.263,.409,.086)]
        p.append(curved_tube('Arm / short orange lizard forelimb '+str(side),ORANGE,arm,[.046,.044,.033],18))
        hand=ellipsoid('Hand / rounded orange three-digit palm '+str(side),ORANGE,(side*.267,.400,.106),(.044,.043,.048),22,12);p.append(hand)
        for j,dx in enumerate((-.026,0.,.026)):
            x=side*(.267+dx)
            p.append(curved_tube('Finger / rooted orange lizard point '+str(side)+' '+str(j),ORANGE,[(x,.392,.137),(x+side*.004,.372,.156),(x+side*.006,.363,.162)],[.012,.008,.0015],12))
        p.append(ellipsoid('Haunch / strong orange supporting thigh '+str(side),ORANGE,(side*.133,.173,.015),(.091,.133,.101),24,16))
        foot=ellipsoid('Foot / planted long orange lizard pad '+str(side),ORANGE,(side*.142,.032,.071),(.077,.032,.119),26,14);p.append(foot)
        for j,dx in enumerate((-.046,0.,.046)):
            x=side*.142+dx;z=.168+(.010 if j==1 else 0.)
            p.append(curved_tube('Foot claw / planted ivory lizard toe '+str(side)+' '+str(j),WHITE,[(x,.033,z-.014),(x+dx*.08,.019,z+.030),(x+dx*.11,.014,z+.041)],[.020,.012,.0015],12))
    mouth=paint('Mouth / fitted open gentle lizard grin',INK,[(-.156,.625),(-.096,.608),(0.,.600),(.096,.608),(.156,.625),(.107,.582),(0.,.566),(-.107,.582)],head,.004);p.append(mouth)
    p.append(paint('Tongue / fitted soft pink lizard mouth floor',PINK,[(-.09,.590),(0.,.590),(.09,.590),(.065,.579),(0.,.574),(-.065,.579)],mouth,.0015))
    p.append(curved_tube('Tail / continuous tapering curved orange lizard tail',ORANGE,
        [(0.,.205,-.126),(.05,.164,-.258),(.183,.195,-.405),(.295,.306,-.500),(.326,.413,-.516)],[.072,.061,.043,.027,.014],24))
    for label,mesh in flame((.326,.406,-.516),(.344,.646,-.531),.125,.034,-18.,-.10).items():
        color=(.91,.27,.12) if label=='ember silhouette' else ((1.,.57,.13) if 'orange' in label else (1.,.89,.37))
        p.append(dict(mesh,part='Flame / '+label,base_color=linear(color)))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'charmander','version':1,'coordinate_system':COORDINATES,'primitives':p}
