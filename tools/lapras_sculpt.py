#!/usr/bin/env python3
"""Original Lapras sculpture: curved neck, curled ears and seated spiked shell."""
import math
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals
from bovine_sculpt import curved_tube,oval,smooth
from abra_sculpt import surface
from spider_sculpt import upper_skin
from skin_glb import COORDINATES
HEIGHT=1.50
# Verified local Crystal normal/front/back RGB555 blue (8,16,28), tan (28,21,13).
BLUE=(66/255,132/255,231/255);CREAM=(231/255,173/255,107/255);SHELL=(.68,.69,.74)
RIM=(.81,.82,.86);SPOT=tuple(v*.70 for v in BLUE);INK=(.08,.12,.17);WHITE=(.98,.98,.96)


def flipper(side,rear):
    # A broad shoulder flattens into a swept paddle, then narrows to a soft tip.
    # Blue upper and cream lower solids share the full cut boundary exactly.
    if rear:
        path=((.26,.120,-.32),(.38,.092,-.43),(.52,.072,-.55),(.66,.060,-.67))
        widths=(.080,.130,.100,.003);depths=(.040,.032,.022,.003)
    else:
        path=((.27,.145,.14),(.42,.108,.23),(.62,.078,.37),(.81,.065,.48),(.89,.061,.52))
        widths=(.095,.180,.170,.100,.004);depths=(.045,.036,.025,.015,.003)
    points,widths=smooth(path,widths,3);_,depths=smooth(path,depths,3);result=[];n=11
    for lower in (False,True):
        v=[];faces=[]
        for (x,y,z),width,depth in zip(points,widths,depths):
            for j in range(n):
                a=(math.pi if lower else 0.)+j*math.pi/(n-1)
                v.append((side*x,y+depth*math.sin(a),z+width*math.cos(a)))
        faces=[tuple(reversed(range(n))),tuple((len(points)-1)*n+j for j in range(n))]
        for k in range(len(points)-1):
            for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
        name=('Flipper underside / '+('rear ' if rear else 'fore ') if lower else ('Rear flipper / ' if rear else 'Fore flipper / '))+str(side)
        result.append(crease_normals(part(name,CREAM if lower else BLUE,v,faces,True),45.))
    return result


def carapace():
    # A low seated plate rises to a domed crown; scalloping fades into the dome.
    profiles=((.232,.260,.330,0.),(.275,.345,.428,.35),(.305,.365,.450,1.),
              (.350,.360,.444,.80),(.420,.328,.405,.45),(.490,.276,.342,.20),
              (.545,.211,.261,.05),(.581,.137,.173,0.),(.600,.060,.078,0.),(.605,.006,.009,0.))
    n=64;v=[];faces=[]
    for y,rx,rz,amount in profiles:
        for j in range(n):
            a=j*math.tau/n;wave=math.cos(10*a);spread=1.+.035*amount*wave
            v.append((rx*math.cos(a)*spread,y-.010*amount*wave,-.18+rz*math.sin(a)*spread))
    faces=[tuple(reversed(range(n))),tuple((len(profiles)-1)*n+j for j in range(n))]
    for k in range(len(profiles)-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
    return crease_normals(part('Shell / broad domed gray carapace',SHELL,v,faces,True),45.)


def sculpture():
    body=ellipsoid('Body / broad low blue swimming torso',BLUE,(0.,.21,-.11),(.40,.21,.55),32,18)
    p=[body,ellipsoid('Belly / seated cream underside',CREAM,(0.,.115,-.075),(.365,.115,.49),32,16)]
    neck=loft('Neck / continuous swept upright plesiosaur neck',BLUE,curved_profiles(
        ((.19,.20,.19,.27),(.32,.18,.18,.32),(.48,.14,.145,.40),
         (.69,.105,.116,.49),(.92,.098,.108,.55),(1.10,.116,.126,.58),
         (1.25,.134,.136,.59),(1.33,.08,.09,.59)),4),28)
    p.append(neck);skin=surface(neck)
    # Closed narrow inset follows the actual neck triangles through its bend.
    outline=[(-.065,.32),(-.069,.43),(-.048,.66),(-.042,.92),(-.061,1.13),
             (.061,1.13),(.042,.92),(.048,.66),(.069,.43),(.065,.32)]
    from sculpt_geometry import fitted_contour
    p.append(fitted_contour('Throat / fitted long cream bib',CREAM,outline,skin,.004,subdivisions=3))
    head=loft('Head / tapered gentle blue muzzle',BLUE,curved_profiles(
        ((1.13,.07,.10,.63),(1.16,.145,.19,.65),(1.23,.178,.235,.63),
         (1.32,.157,.182,.59),(1.39,.101,.112,.56),(1.405,.025,.024,.55)),3),28)
    p.append(head);face=surface(head)
    p.append(ellipsoid('Jaw / cream lower muzzle',CREAM,(0.,1.158,.705),(.143,.036,.172),28,12))
    for side in (-1,1):
        x=side*.118;y=1.285
        almond=[(x+side*u,y+v) for u,v in
                ((-.040,-.004),(-.025,.008),(.011,.015),(.040,.020),
                 (.031,-.008),(0.,-.016),(-.028,-.014))]
        eye=crease_normals(fitted_contour('Eye / expressive white almond '+str(side),WHITE,
                                        almond,face,.004,subdivisions=3),35.)
        p.append(eye)
        pupil=fitted_patch('Eye / dark upright pupil '+str(side),INK,oval(x+side*.009,y+.001,.010,.0115,20),surface(eye),.002);p.append(pupil)
        p.append(fitted_patch('Eye / small glint '+str(side),WHITE,oval(x+side*.0065,y+.0065,.0025,.0035,12),surface(pupil),.001))
        p.append(fitted_patch('Nostril / small muzzle inset '+str(side),INK,
                             oval(side*.057,1.211,.0065,.0042,12),face,.002))
        points=[(side*(.082+i*.006),1.294+i*.002,face(1.294+i*.002,side*(.082+i*.006))+.004) for i in range(10)]
        p.append(tube('Brow / fitted gentle upper eye crease '+str(side),SPOT,points,[.003]*len(points),8))
        # Curl is a continuous fleshy spiral rather than a sphere with a line.
        points=[];radii=[]
        for i in range(35):
            t=i/34;a=-math.pi*.60+t*math.pi*2.;r=.100*(1.-.62*t)
            points.append((side*(.165+r*math.cos(a)),1.37+r*math.sin(a),.49-.018*t));radii.append(.037-.011*t)
        p.append(ellipsoid('Ear / fleshy curled backing '+str(side),BLUE,(side*.165,1.37,.465),(.097,.099,.031),16,10))
        p.append(tube('Ear / continuous inward curl '+str(side),BLUE,points,radii,14))
        p+=flipper(side,False);p+=flipper(side,True)
        for j,(yy,xx,rx,ry) in enumerate(((.39,.125,.021,.031),(.47,.098,.014,.028),(.99,.071,.012,.018),(1.09,.078,.010,.018))):
            p.append(fitted_patch('Neck spot / '+str(side)+' '+str(j),SPOT,oval(side*xx,yy,rx,ry,16),skin,.003))
    # A shallow closed crease follows the actual muzzle instead of crossing
    # its curved surface as a straight floating chord.
    mouth=[]
    for i in range(25):
        x=-.123+.246*i/24;y=1.179+.009*(abs(x)/.123)**2
        mouth.append((x,y,face(y,x)+.0015))
    p.append(tube('Mouth / fitted gentle muzzle crease',INK,mouth,[.0022]*len(mouth),8))
    p.append(curved_tube('Horn / short forehead cone',BLUE,[(0.,1.335,.735),(0.,1.407,.754),(0.,1.475,.745)],[.029,.022,.0015],18))
    # Shell overlaps torso, with a continuous rounded lip and firmly seated spikes.
    shell=carapace();p.append(shell)
    ring=[]
    for i in range(65):
        a=i*math.tau/64;wave=math.cos(10*a);spread=1.+.035*wave
        ring.append((.365*math.cos(a)*spread,.305-.010*wave,-.18+.450*math.sin(a)*spread))
    p.append(tube('Shell rim / continuous pale rolled edge',RIM,ring,[.020]*len(ring),10));top=upper_skin(shell)
    for j,(x,z,h) in enumerate(((0.,-.20,.17),(-.20,-.14,.12),(.20,-.14,.12),
                              (-.18,-.39,.105),(.18,-.39,.105),(0.,-.51,.10),(-.25,.035,.09),(.25,.035,.09))):
        y=top(z,x)-.018
        p.append(curved_tube('Shell spike / seated blunt cone '+str(j),SHELL,[(x,y,z),(x*.98,y+h*.65,z-.012),(x*.97,y+h,z-.035)],[.060,.034,.012],18))
    p.append(curved_tube('Tail / short rear swimming taper',BLUE,[(0.,.18,-.48),(0.,.14,-.65),(0.,.09,-.78)],[.095,.069,.004],20))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'lapras','version':1,'coordinate_system':COORDINATES,'primitives':p}
