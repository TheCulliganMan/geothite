#!/usr/bin/env python3
"""Original Bulbasaur: squat spotted quadruped and a genuinely lobed back bulb."""
import math
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals,translate
from goldfish_sculpt import surface
from bovine_sculpt import curved_tube,oval
from skin_glb import COORDINATES
HEIGHT=.95
TEAL=(.48,.76,.69);SPOT=(.19,.43,.34);GREEN=(.43,.67,.35)
WHITE=(.98,.98,.91);RED=(.80,.28,.35);INK=(.10,.14,.14);PINK=(.74,.44,.56)


def paint(name,color,outline,parent,th=.004):
    return crease_normals(fitted_contour(name,color,outline,surface(parent),th,subdivisions=3),50.)


def bulb():
    rows=curved_profiles(((.36,.065,.09,-.155),(.42,.225,.27,-.145),(.535,.28,.29,-.135),
                         (.64,.25,.25,-.145),(.713,.176,.18,-.174),(.762,.105,.106,-.205),
                         (.812,.052,.056,-.225),(.834,.003,.004,-.233)),3)
    n=48;v=[]
    for y,w,d,z in rows:
        for i in range(n):
            a=i*math.tau/n;fold=1.+.095*math.cos(6*a)
            v.append((w*math.sin(a)*fold,y+.007*math.sin(3*a)*math.sin(math.pi*(y-.36)/.474),z+d*math.cos(a)*fold))
    # Radial cap fans close the scalloped contours without crossing concave
    # valleys. The cap rings are planar; fold height vanishes at both ends.
    bottom=len(v);v.append((0.,rows[0][0],rows[0][3]));top=len(v);v.append((0.,rows[-1][0],rows[-1][3]))
    f=[(bottom,(i+1)%n,i) for i in range(n)]
    offset=(len(rows)-1)*n
    f.extend((top,offset+i,offset+(i+1)%n) for i in range(n))
    for j in range(len(rows)-1):
        for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
    return part('Bulb / continuous six-lobed folded green seed',GREEN,v,f,True)


def sculpture():
    body=ellipsoid('Body / low substantial teal quadruped torso',TEAL,(0.,.295,-.115),(.255,.178,.340),32,20)
    head=loft('Head / broad flat cheeked teal saur skull',TEAL,curved_profiles(
        ((.235,.065,.070,.28),(.27,.21,.14,.28),(.36,.275,.185,.28),
         (.47,.28,.185,.26),(.56,.235,.15,.22),(.61,.175,.102,.19),(.63,.06,.045,.18)),3),32)
    p=[body,head,bulb()];hs=surface(head)
    for side in (-1,1):
        ear=loft('Ear / short pointed saur hood '+str(side),TEAL,curved_profiles(
            ((.49,.069,.053,.205),(.57,.065,.048,.191),(.64,.038,.030,.173),(.675,.002,.003,.167)),3),20)
        p.append(translate(ear,side*.207,0.,0.))
        eye=paint('Eye / fitted ivory triangular saur lens '+str(side),WHITE,
            [(side*x,y) for x,y in ((.083,.410),(.172,.410),(.188,.458),(.164,.526),(.135,.541),(.098,.492))],head,.004);p.append(eye)
        iris=paint('Iris / fitted tall red saur gaze '+str(side),RED,oval(side*.145,.468,.024,.049,28),eye,.0025);p.append(iris)
        p.append(paint('Eye glint / fitted ivory saur oval '+str(side),WHITE,oval(side*.145-.004,.485,.007,.018,20),iris,.0015))
        p.append(fitted_patch('Nostril / fitted tiny saur slit '+str(side),INK,oval(side*.052,.358,.005,.0025,16),hs,.0018))
        x=side*.136;y=.332;z=hs(y,x)
        p.append(curved_tube('Fang / short hanging ivory saur tooth '+str(side),WHITE,[(x,y+.004,z+.003),(x,y-.008,z+.006),(x+side*.002,y-.018,z+.007)],[.008,.006,.0015],12))
        for front in (True,False):
            z=.215 if front else -.30;x=side*(.211 if front else .195)
            leg=loft(('Foreleg / ' if front else 'Hindleg / ')+str(side),TEAL,curved_profiles(
                ((.035,.050,.060,z+.023),(.10,.080,.079,z+.010),(.19,.083,.079,z),(.27,.064,.070,z-.008),(.32,.027,.031,z-.008)),3),22)
            leg=translate(leg,x,0.,0.);p.append(leg)
            foot=ellipsoid(('Forefoot / ' if front else 'Hindfoot / ')+str(side),TEAL,(x,.030,z+.039),(.083,.030,.095),24,12);p.append(foot)
            for j,dx in enumerate((-.046,0.,.046)):
                p.append(curved_tube(('Foreclaw / ' if front else 'Hindclaw / ')+str(side)+' '+str(j),WHITE,
                    [(x+dx,.033,z+.099),(x+dx,.022,z+.131),(x+dx,.018,z+.142)],[.017,.012,.0015],12))
            # Each leg gets a genuine fitted diamond, not a floating flat patch.
            xy=[(x-.033,.155),(x-.020,.204),(x+.025,.217),(x+.039,.174),(x+.008,.135)]
            p.append(paint(('Foreleg spot / ' if front else 'Hindleg spot / ')+str(side),SPOT,xy,leg,.003))
    mouth=paint('Mouth / fitted broad dark saur smile',INK,[(-.177,.344),(-.12,.322),(0.,.313),(.12,.322),(.177,.344),(.13,.292),(0.,.276),(-.13,.292)],head,.004);p.append(mouth)
    p.append(paint('Tongue / fitted pale saur mouth floor',PINK,[(-.112,.306),(0.,.304),(.112,.306),(.080,.293),(0.,.286),(-.080,.293)],mouth,.0015))
    for j,xy in enumerate(([(-.075,.591),(.051,.602),(.046,.578),(-.018,.544)],
                           [(-.040,.534),(-.026,.539),(-.028,.516),(-.041,.515)],
                           [(.030,.484),(.080,.499),(.084,.450),(.051,.439)])):
        p.append(paint('Head spot / fitted angular saur marking '+str(j),SPOT,xy,head,.003))
    # Squat the lower body beneath a broad head and low seated bulb.
    for q in p:
        points=[];normals=[]
        for i in range(0,len(q['positions']),3):
            x,y,z=q['positions'][i:i+3];nx,ny,nz=q['normals'][i:i+3];sy=.8 if y<.32 else 1.
            points.extend((x,y*.8 if y<.32 else y-.064,z))
            n=(nx,ny/sy,nz);length=math.sqrt(sum(v*v for v in n));normals.extend(v/length for v in n)
        q['positions']=points;q['normals']=normals
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'bulbasaur','version':1,'coordinate_system':COORDINATES,'primitives':p}
