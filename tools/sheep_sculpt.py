#!/usr/bin/env python3
"""Original electric sheep contours, integrated scalloped fleece and curls."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,tube,ellipsoid,fitted_patch,translate
from abra_sculpt import surface
from spider_sculpt import banded_limb
from skin_glb import COORDINATES
from bovine_sculpt import fitted_patch

SPECIES=('mareep','flaaffy');HEIGHT={'mareep':.85,'flaaffy':1.}
BLUE=(.29,.50,.64);PINK=(.89,.47,.59);CREAM=(.94,.89,.70)
WHITE=(.93,.95,.90);INK=(.085,.105,.13);GOLD=(.96,.77,.29)
ORANGE=(.95,.52,.23);BEACON=(.36,.74,.87);CURL=(.43,.42,.33)

def fleece(name,color,center,radii,segments=64,rings=32,bulges=.20):
    v=[];f=[];axes=[]
    for row,(theta,count) in enumerate(((.75,6),(1.65,7),(2.55,5))):
        for k in range(count):
            a=math.tau*(k+.5*(row%2))/count
            axes.append((math.sin(theta)*math.sin(a),math.cos(theta),math.sin(theta)*math.cos(a)))
    for j in range(rings+1):
        theta=math.pi*(j+.025)/(rings+.05)
        for k in range(segments):
            a=math.tau*k/segments
            # Continuous low-frequency bulges create scalloped wool volume;
            # no detached sphere ring surrounds an exposed blue belly.
            d=(math.sin(theta)*math.sin(a),math.cos(theta),math.sin(theta)*math.cos(a))
            r=.85
            offset=.50+bulges
            for axis in axes:
                projection=offset*sum(x*y for x,y in zip(d,axis))
                discriminant=.34**2-offset**2+projection**2
                if discriminant<0:continue
                candidate=projection+math.sqrt(discriminant)
                h=max(.025-abs(r-candidate),0)/.025
                r=max(r,candidate)+.025*h*h/4
            v.append((center[0]+radii[0]*r*math.sin(theta)*math.sin(a),
                      center[1]+radii[1]*r*math.cos(theta),
                      center[2]+radii[2]*r*math.sin(theta)*math.cos(a)))
    f=[tuple(reversed(range(segments))),tuple(rings*segments+i for i in range(segments))]
    for j in range(rings):
        for i in range(segments):
            a=j*segments+i;b=j*segments+(i+1)%segments;f.append((a,b,b+segments,a+segments))
    return part(name,color,v,f,True)

def curl(name,mesh,x,y,radius):
    skin=surface(mesh);points=[]
    for k in range(54):
        t=k/53;a=.4+math.tau*1.15*t;r=radius*(1-t)+.009*t
        xx=x+r*math.cos(a);yy=y+r*math.sin(a);points.append((xx,yy,skin(yy,xx)+.003))
    return tube(name,CURL,points,[.003]*54,8)

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported sheep')
    mid=species=='flaaffy';color=PINK if mid else BLUE;p=[]
    if mid:
        body=loft('Body / continuous pear belly and rising neck',PINK,curved_profiles(((.10,.05,.052,-.025),(.20,.18,.13,-.02),(.32,.226,.16,-.026),(.43,.20,.15,-.015),(.55,.14,.12,.00),(.66,.11,.09,.025),(.74,.055,.06,.06)),3),28)
        p.append(body)
        p.append(fleece('Fleece / continuous scalloped shoulder cape',WHITE,(0,.655,.008),(.245,.138,.19)))
        for side in (-1,1):
            p.append(tube('Arm / tapered pink flipper '+str(side),PINK,[(side*.16,.55,.03),(side*.28,.52,.07),(side*.36,.53,.09)],[.056,.041,.003],18))
            p.append(ellipsoid('Leg / rounded supporting haunch '+str(side),PINK,(side*.13,.145,.0),(.078,.122,.085),20,12))
            foot=ellipsoid('Foot / grounded pink pad '+str(side),PINK,(side*.14,.049,.085),(.095,.046,.145),24,12);p.append(foot)
            skin=surface(foot);outline=[(side*.14-.043,.032),(side*.14-.046,.060),(side*.14-.023,.076),(side*.14+.023,.076),(side*.14+.046,.060),(side*.14+.043,.032)]
            dense=[]
            for a,b in zip(outline,outline[1:]+outline[:1]):
                dense.extend((a[0]+(b[0]-a[0])*i/5,a[1]+(b[1]-a[1])*i/5) for i in range(5))
            p.append(fitted_patch('Toe / fitted ivory cap '+str(side),WHITE,dense,skin,.006))
        profiles=((.70,.04,.047,.19),(.75,.11,.10,.20),(.83,.158,.138,.17),(.91,.14,.115,.12),(.95,.07,.057,.10),(.965,.01,.012,.09))
    else:
        p.append(fleece('Fleece / full scalloped quadruped mantle',CREAM,(0,.405,-.11),(.32,.33,.375)))
        for side in (-1,1):
            for row,z in enumerate((.095,-.30)):
                p.append(translate(loft(f'Leg / short tapered blue support {side} {row}',BLUE,curved_profiles(((.01,.043,.065,z+.015),(.035,.057,.076,z+.005),(.12,.060,.066,z),(.24,.055,.058,z-.008),(.35,.045,.045,z-.013)),2),18),side*.19,0,0))
        profiles=((.45,.035,.04,.24),(.50,.115,.098,.27),(.58,.158,.14,.24),(.68,.163,.135,.195),(.745,.105,.078,.155),(.76,.015,.015,.14))
    head=loft('Head / joined cranium and tapered sheep muzzle',color,curved_profiles(profiles,3),28);p.append(head);skin=surface(head)
    for side in (-1,1):
        y=.845 if mid else .65;x=side*(.085 if mid else .087);z=skin(y,x)
        p.append(ellipsoid('Eye / black fitted oval '+str(side),INK,(x,y,z+.009),(.025,.033,.011),20,12))
        p.append(ellipsoid('Eye / outer ivory glint '+str(side),WHITE,(x+side*.007,y+.010,z+.019),(.009,.012,.006),12,8))
        yy=.78 if mid else .548;xx=side*.028
        p.append(fitted_patch('Nostril / tiny muzzle dot '+str(side),INK,[(xx-.004,yy),(xx,yy+.007),(xx+.004,yy),(xx,yy-.005)],skin,.002))
        ear_y=.883 if mid else .705
        path=[(side*.13,ear_y,.115),(side*.21,ear_y+.016,.112),(side*.29,ear_y+.020,.098),(side*.38,ear_y+.025,.085)]
        p.extend(banded_limb('Ear / '+str(side),path,(.054,.070,.058,.003),(INK,PINK if mid else GOLD,INK)))
    if mid:
        for side in (-1,1):
            tuft=fleece('Crown / paired wool curl '+str(side),WHITE,(side*.087,.970,.10),(.119,.115,.112),32,16,bulges=.08)
            p.extend((tuft,curl('Crown groove / fitted spiral '+str(side),tuft,side*.087,.972,.065)))
    else:
        tuft=fleece('Crown / single curled wool forelock',CREAM,(0,.786,.147),(.145,.142,.125),32,18,bulges=.08)
        p.extend((tuft,curl('Crown groove / fitted single spiral',tuft,0,.787,.077)))
    path=[(0,.28,-.22),(0,.30,-.39),(.055,.40,-.53),(.085,.54,-.59),(.09,.67,-.58)]
    p.extend(banded_limb('Tail / curved alternating segment',path,(.044,.038,.032,.026,.018),(INK,color if mid else GOLD,INK,color if mid else GOLD)))
    p.append(ellipsoid('Tail / round electric beacon',BEACON if mid else ORANGE,path[-1],(.073,.078,.072),24,16))
    p.append(ellipsoid('Tail / ivory beacon highlight',WHITE,(path[-1][0]-.020,path[-1][1]+.028,path[-1][2]+.058),(.017,.020,.005),12,8))
    # Local normal palettes: wool/skin and the darker source shade, white/black.
    from chikorita_sculpt import linear
    skin=(255,107,156) if mid else (82,41,156)
    wool=(255,255,255) if mid else (198,165,57)
    shade=(82,24,115) if mid else skin
    for q in p:
        n=q['part']
        if n.startswith(('Ear /','Tail / curved')):
            k=int(n.split()[-1]);rgb=(0,0,0) if k%2==0 else skin if mid else (198,165,57)
        elif n.startswith(('Fleece /','Crown /')):rgb=wool
        elif n.startswith('Crown groove /'):rgb=shade
        elif n.startswith(('Eye / outer','Toe /','Tail / ivory')):rgb=(255,255,255)
        elif n.startswith(('Eye / black','Nostril /')):rgb=(0,0,0)
        elif n.startswith('Tail / round'):rgb=shade if mid else (198,165,57)
        else:rgb=skin
        q['base_color']=linear(tuple(v/255 for v in rgb))
    if not normalize:return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT[species]/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))

if __name__=='__main__':main()
