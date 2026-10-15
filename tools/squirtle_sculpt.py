#!/usr/bin/env python3
"""Original Squirtle: round blue turtle, plated shell and substantial curled tail."""
import math
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,fitted_contour,tube
from goldfish_sculpt import surface
from bovine_sculpt import reverse_front,oval,curved_tube
from wartortle_sculpt import ribbon
from skin_glb import COORDINATES
HEIGHT=.96
BLUE=(.42,.72,.82);BROWN=(.61,.40,.29);CREAM=(.96,.89,.68);WHITE=(.99,.98,.94)
SEAM=(.36,.29,.22);IRIS=(.48,.24,.31);INK=(.09,.10,.13);PINK=(.76,.49,.57)


def paint(name,color,xy,parent,th=.004,sub=2):
    return crease_normals(fitted_contour(name,color,xy,surface(parent),th,subdivisions=sub),50.)


def rim():
    n=64;m=12;v=[]
    for i in range(n):
        a=i*math.tau/n
        for j in range(m):
            t=j*math.tau/m;v.append(((.215+.019*math.cos(t))*math.cos(a),.325+(.248+.019*math.cos(t))*math.sin(a),-.025+.019*math.sin(t)))
    f=[(i*m+j,((i+1)%n)*m+j,((i+1)%n)*m+(j+1)%m,i*m+(j+1)%m) for i in range(n) for j in range(m)]
    return part('Shell rim / continuous rolled ivory oval',WHITE,v,f,True)


def shell_loop(parent):
    outline=[(0.,.526),(-.119,.446),(-.133,.282),(0.,.164),(.133,.282),(.119,.446)]
    # A single closed annular solid carries all six scute edges. Dense points
    # follow the actual curved shell, avoiding disconnected decal strips.
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:outline.reverse()
    xy=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):xy.extend((a[0]+(b[0]-a[0])*i/12,a[1]+(b[1]-a[1])*i/12) for i in range(12))
    n=len(xy);skin=surface(parent);v=[]
    for offset,scale in ((.004,1.012),(.004,.988),(-.002,1.012),(-.002,.988)):
        for x,y in xy:
            x*=scale;y=.345+(y-.345)*scale;v.append((x,y,skin(y,x)+offset))
    f=[]
    for i in range(n):
        j=(i+1)%n
        f.extend(((i,j,n+j,n+i),(2*n+j,2*n+i,3*n+i,3*n+j),
                  (i,2*n+i,2*n+j,j),(n+j,3*n+j,3*n+i,n+i)))
    return crease_normals(part('Shell seam / fitted closed six-sided scute loop',SEAM,v,f),40.)


def sculpture():
    body=ellipsoid('Body / compact blue turtle mantle',BLUE,(0.,.325,.016),(.194,.229,.124),30,18)
    shell=ellipsoid('Shell / substantial domed brown turtle carapace',BROWN,(0.,.325,-.099),(.233,.265,.147),32,20)
    belly=ellipsoid('Belly / rounded cream segmented turtle breast',CREAM,(0.,.323,.066),(.181,.219,.103),32,20)
    head=loft('Head / rounded blue turtle skull and broad muzzle',BLUE,curved_profiles(
        ((.505,.033,.043,.075),(.55,.142,.127,.075),(.63,.205,.174,.058),
         (.71,.205,.171,.030),(.78,.155,.134,.01),(.82,.074,.072,0.),(.831,.014,.014,-.008)),4),32)
    p=[body,shell,belly,rim(),head]
    for j,path in enumerate(([(-.149,.421),(-.075,.406),(0.,.398),(.075,.406),(.149,.421)],
                             [(-.143,.259),(-.075,.248),(0.,.245),(.075,.248),(.143,.259)],
                             [(0.,.495),(0.,.40),(0.,.31),(0.,.245),(0.,.146)])):
        p.append(paint('Belly seam / fitted turtle plate join '+str(j),SEAM,ribbon(path,.002),belly,.0025))
    rear=reverse_front(shell);p.append(reverse_front(shell_loop(rear)))
    for j,path in enumerate(([(-.133,.282),(-.200,.205)],[(.133,.282),(.200,.205)])):
        p.append(reverse_front(paint('Shell seam / fitted lower scute branch '+str(j),SEAM,ribbon(path,.002),rear,.003)))
    hs=surface(head)
    for side in (-1,1):
        eye=paint('Eye / fitted ivory rounded turtle lens '+str(side),WHITE,
            [(side*x,y) for x,y in ((.064,.654),(.148,.655),(.167,.692),(.159,.741),(.131,.769),(.101,.749),(.077,.707))],head,.004);p.append(eye)
        iris=paint('Iris / fitted maroon turtle gaze '+str(side),IRIS,oval(side*.128,.704,.027,.045,28),eye,.0025);p.append(iris)
        pupil=paint('Pupil / fitted dark turtle oval '+str(side),INK,oval(side*.132,.712,.014,.031,24),iris,.0015);p.append(pupil)
        p.append(paint('Eye glint / fitted ivory turtle sparkle '+str(side),WHITE,oval(side*.129,.726,.005,.010,20),pupil,.001))
        p.append(paint('Nostril / fitted tiny turtle dot '+str(side),INK,oval(side*.039,.625,.004,.0025,16),head,.0018))
        p.append(curved_tube('Arm / short rounded blue turtle forelimb '+str(side),BLUE,
            [(side*.150,.466,.013),(side*.238,.418,.057),(side*.286,.370,.102)],[.060,.057,.041],20))
        p.append(ellipsoid('Hand / blue three-digit turtle palm '+str(side),BLUE,(side*.293,.362,.111),(.052,.048,.052),22,14))
        for j,dx in enumerate((-.03,0.,.03)):
            p.append(curved_tube('Finger / three blunt blue turtle digits '+str(side)+' '+str(j),BLUE,
                [(side*(.293+dx),.357,.142),(side*(.296+dx),.333,.164),(side*(.300+dx),.327,.169)],[.017,.013,.003],14))
        p.append(ellipsoid('Haunch / rounded blue turtle thigh '+str(side),BLUE,(side*.134,.139,.018),(.089,.111,.104),24,16))
        p.append(ellipsoid('Foot / planted broad blue turtle pad '+str(side),BLUE,(side*.150,.029,.077),(.075,.029,.099),24,14))
        for j,dx in enumerate((-.042,0.,.042)):
            z=.159+(.01 if j==1 else 0.)
            p.append(curved_tube('Toe / planted blunt blue turtle digit '+str(side)+' '+str(j),BLUE,
                [(side*.150+dx,.027,z-.024),(side*.150+dx,.018,z+.012),(side*.150+dx,.015,z+.024)],[.020,.016,.003],14))
    mouth=paint('Mouth / fitted gentle turtle smile',INK,[(-.137,.605),(-.077,.591),(0.,.584),(.077,.591),(.137,.605),(.099,.578),(0.,.567),(-.099,.578)],head,.003);p.append(mouth)
    p.append(paint('Tongue / fitted small pink turtle mouth floor',PINK,[(-.075,.583),(0.,.580),(.075,.583),(.054,.574),(0.,.571),(-.054,.574)],mouth,.001))
    p.append(curved_tube('Tail / substantial blue turtle curl',BLUE,
        [(0.,.188,-.139),(.105,.153,-.264),(.255,.169,-.348),(.382,.235,-.374),(.418,.343,-.382),
         (.364,.425,-.383),(.255,.424,-.375),(.198,.340,-.365),(.229,.270,-.354),(.298,.266,-.347),(.317,.316,-.340)],
        [.041,.059,.072,.070,.063,.060,.055,.048,.038,.025,.004],24))
    p.append(ellipsoid('Tail core / soft blue turtle spiral center',BLUE,(.297,.335,-.362),(.112,.117,.053),24,16))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'squirtle','version':1,'coordinate_system':COORDINATES,'primitives':p}
