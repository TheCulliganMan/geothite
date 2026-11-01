#!/usr/bin/env python3
"""Original Geodude: a quarried boulder, recessed face and splayed rock hands."""
import math
from sculpt_geometry import part,loft,crease_normals,fitted_contour,fitted_triangle_contour,subdivided_part
from goldfish_sculpt import surface
from bovine_sculpt import oval
from lugia_sculpt import paddle
from skin_glb import COORDINATES,smoothstep
HEIGHT=.76
STONE=(.59,.60,.54)
WHITE=(.98,.98,.91);INK=(.09,.10,.10)


def faceted(p,color=None):
    vertices=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
    q=part(p['part'],STONE if color is None else color,vertices,[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)],False)
    return crease_normals(q,30.)


def paint(name,color,xy,parent,th=.004):
    return crease_normals(fitted_contour(name,color,xy,surface(parent),th,subdivisions=3),40.)


def stone_block(name,color,center,radii):
    # Broad palm planes with rounded rock bevels.
    x,y,z=center;rx,ry,rz=radii
    vertices=[(x+a*rx,y+b*ry,z+c*rz) for a,b,c in
              ((-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),
               (-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1))]
    faces=[(0,3,2,1),(4,5,6,7),(0,1,5,4),(3,7,6,2),(0,4,7,3),(1,2,6,5)]
    return crease_normals(subdivided_part(name,color,vertices,faces,1,False),38.)


def hand_posture(side,point,normal=False):
    x,y,z=point;angle=-side*.60;c,s=math.cos(angle),math.sin(angle)
    if not normal:x-=side*.691;y-=.322
    x,y=c*x-s*y,s*x+c*y
    return (x,y,z) if normal else (x+side*.691,y+.222,z)


def sculpture(normalize=True):
    # Uneven latitudes and displaced sectors break the old regular egg. Extra
    # rings across the face give the mouth a real depression in the boulder.
    base=loft('Body / irregular single grey Geodude boulder',STONE,
        [(0.,.080,.090,-.010),(.045,.190,.170,-.010),(.100,.255,.220,-.010),
         (.130,.281,.242,-.010),(.160,.299,.264,-.012),(.195,.311,.280,-.014),
         (.235,.314,.294,-.017),(.285,.314,.300,-.018),(.405,.294,.277,-.027),
         (.495,.229,.216,-.033),(.552,.121,.130,-.040),(.566,.042,.042,-.041)],24)
    vv=[]
    for i in range(0,len(base['positions']),3):
        x,y,z=base['positions'][i:i+3];a=math.atan2(z,x);q=(y-.28)/.29
        factor=1.+(.079*math.sin(5*a+.3)+.040*math.cos(3*a-.7)+.035*math.cos(7*a-18*y))*max(0.,1.-q*q)
        twist=.055*math.sin(24*y);xx=x*math.cos(twist)-z*math.sin(twist);zz=x*math.sin(twist)+z*math.cos(twist)
        x=xx*factor;z=zz*factor
        y+=.015*math.cos(3*a+.2)*max(0.,1.-q*q)+.026*math.cos(5*a+.4)*smoothstep(.39,.52,y)
        if z>0. and .09<y<.27:
            radius=(x/.209)**2+((y-.184)/.079)**2
            z-=.090*max(0.,1.-radius)**.55
        vv.append((x,y,z))
    body=part(base['part'],STONE,vv,[tuple(base['indices'][i:i+3]) for i in range(0,len(base['indices']),3)],False)
    p=[body]
    keystone=[(-.195,.466),(-.125,.510),(-.041,.445),(0.,.415),(.041,.445),(.125,.510),(.195,.466),(.111,.462),(0.,.379),(-.111,.462)]
    p.append(paint('Brow / raised angular forehead keystone',STONE,keystone,body,.022))
    for side in (-1,1):
        brow=[(side*x,y) for x,y in ((.014,.366),(.064,.422),(.141,.456),(.217,.418),(.232,.369),(.179,.374),(.100,.351))]
        p.append(paint('Brow / fitted heavy angular rock ridge '+str(side),STONE,brow,body,.025))
        eye=paint('Eye / fitted ivory angular rock stare '+str(side),WHITE,
            [(side*x,y) for x,y in ((.030,.316),(.190,.365),(.201,.292),(.053,.274))],body,.006);p.append(eye)
        pupil=paint('Pupil / fitted narrow dark rock gaze '+str(side),INK,oval(side*.142,.324,.014,.022,20),eye,.003);p.append(pupil)
        p.append(paint('Eye glint / fitted small ivory rock highlight '+str(side),WHITE,oval(side*.139,.329,.0035,.008,12),pupil,.001))
        p.append(faceted(paddle('Upper arm / bent grey stone shoulder '+str(side),STONE,
            [(side*.258,.270,.005),(side*.414,.160,.016),(side*.531,.172,.037)],
            [.084,.078,.070],[.080,.074,.070],segments=8,steps=2)))
        p.append(faceted(paddle('Forearm / raised grey stone wrist '+str(side),STONE,
            [(side*.510,.166,.032),(side*.628,.180,.060),(side*.691,.222,.090)],
            [.072,.084,.078],[.070,.078,.068],segments=8,steps=2)))
        p.append(stone_block('Hand / broad open angular palm '+str(side),STONE,
                            (side*.721,.377,.091),(.111,.113,.065)))
        # Four separate, bent fingers fan out above the palm. The thumb opens
        # toward the body. These cages are authored here, with no imported mesh.
        fingers=[[(.657,.416,.107),(.623,.508,.127),(.592,.561,.148)],
                 [(.694,.449,.091),(.692,.568,.111),(.683,.622,.136)],
                 [(.740,.442,.082),(.785,.545,.098),(.814,.583,.120)],
                 [(.776,.403,.073),(.855,.473,.076),(.892,.495,.094)]]
        for j,path in enumerate(fingers):
            p.append(faceted(paddle('Finger / splayed three-plane rock digit '+str(side)+' '+str(j),STONE,
                [(side*x,y,z) for x,y,z in path],[.042,.038,.031],[.038,.035,.029],segments=6,steps=1)))
        p.append(faceted(paddle('Thumb / opened opposing rock digit '+str(side),STONE,
            [(side*.661,.349,.119),(side*.594,.390,.166),(side*.573,.443,.186)],
            [.045,.037,.029],[.042,.035,.028],segments=6,steps=1)))
    mouth=[(-.190,.218),(-.120,.242),(.120,.242),(.190,.218),(.180,.140),(.100,.112),(-.100,.112),(-.180,.140)]
    p.append(crease_normals(fitted_triangle_contour('Mouth / recessed angular oral chamber',(.21,.13,.13),mouth,body,.002),30.))
    lip=[(-.169,.134),(-.092,.115),(0.,.110),(.092,.115),(.169,.134),(.104,.082),(0.,.071),(-.104,.082)]
    p.append(paint('Lip / fitted lower grey rock jaw ridge',STONE,lip,body,.015))
    for q in p:
        if q['part'].startswith(('Hand /','Finger /','Thumb /')):
            side=int(q['part'].split()[-2] if q['part'].startswith('Finger /') else q['part'].split()[-1])
            for field in ('positions','normals'):
                q[field]=[v for k in range(0,len(q[field]),3) for v in hand_posture(side,q[field][k:k+3],field=='normals')]
    if not normalize:
        return {'name':'geodude','version':1,'coordinate_system':COORDINATES,'primitives':p}
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'geodude','version':1,'coordinate_system':COORDINATES,'primitives':p}
