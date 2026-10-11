#!/usr/bin/env python3
"""Original wind-borne plants with toothed leaves, petals and seed globes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_patch
from abra_sculpt import surface
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES

SPECIES=('hoppip','skiploom','jumpluff');HEIGHT=.85
PINK=(.95,.51,.65);GREEN=(.49,.73,.24);BLUE=(.40,.54,.77)
LEAF=(.55,.74,.37);VEIN=(.28,.43,.20);YELLOW=(.97,.83,.35)
RED=(.84,.24,.31);INK=(.15,.09,.14);WHITE=(.99,.97,.86)
CREAM=(.96,.91,.70);MAUVE=(.64,.38,.51);TONGUE=(.96,.65,.63)

def patch(name,color,outline,skin,depth=.006):
    dense=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):
        dense.extend((a[0]+(b[0]-a[0])*i/4,a[1]+(b[1]-a[1])*i/4) for i in range(4))
    return fitted_patch(name,color,dense,skin,depth)

def seed_globe(name,center,radius):
    p=[ellipsoid(name+' / continuous seed sphere',CREAM,center,(radius,radius,radius),32,20)]
    # Sparse short bristles retain the spherical seed-head silhouette.
    for i in range(28):
        y=1-2*(i+.5)/28;a=i*math.pi*(3-math.sqrt(5));r=math.sqrt(1-y*y)
        d=(r*math.cos(a),y,r*math.sin(a))
        points=[tuple(c+(radius+q)*v for c,v in zip(center,d)) for q in (-.005,.012)]
        p.append(tube(name+' / fine seed bristle '+str(i),CREAM,points,[.0025,.0007],6))
    return p

def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported wind plant')
    hop=species=='hoppip';skip=species=='skiploom';color=PINK if hop else GREEN if skip else BLUE
    body=loft('Body / continuous rounded wind-borne torso',color,curved_profiles(((.11,.035,.032,0),(.17,.15,.117,0),(.28,.235,.174,0),(.40,.252,.184,-.003),(.51,.20,.151,-.011),(.585,.105,.086,-.015),(.612,.014,.014,-.020)),3),32)
    p=[body];skin=surface(body)
    for side in (-1,1):
        p.append(tube('Foot / short rounded supporting nub '+str(side),color,[(side*.09,.17,.015),(side*.108,.072,.035),(side*.110,.027,.055),(side*.110,.009,.070)],[.041,.036,.020,.003],18))
        p.append(ellipsoid('Arm / small rounded front flipper '+str(side),color,(side*.18,.287,.145),(.049,.077,.040),20,12))
        x=side*.09;y=.432 if skip else .438;z=skin(y,x)
        p.append(ellipsoid('Eye / fitted colored oval '+str(side),YELLOW if hop else RED,(x,y,z+.006),(.020,.023,.009),20,12))
        p.append(ellipsoid('Eye / ivory highlight '+str(side),WHITE,(x+side*.004,y+.007,z+.014),(.005,.007,.003),12,8))
    if skip:
        smile=[(-.092,.345),(-.046,.322),(0,.316),(.046,.322),(.092,.345)]
        p.append(tube('Mouth / gentle fitted smile',INK,[(x,y,skin(y,x)+.003) for x,y in smile],[.005]*5,10))
    else:
        w=.069 if hop else .028;top=.355;bottom=.238 if hop else .272
        outline=[(-w,top),(w,top),(w*.65,bottom+.025),(0,bottom),(-w*.65,bottom+.025)]
        p.append(patch('Mouth / open fitted smile',INK,outline,skin))
        tongue=[(-w*.43,bottom+.016),(0,bottom+.010),(w*.43,bottom+.016),(w*.48,bottom+.035),(0,bottom+.046),(-w*.48,bottom+.035)]
        p.append(patch('Tongue / fitted lower smile',TONGUE,tongue,lambda y,x:skin(y,x)+.008,.003))
    if hop:
        outline=[(.025,.582),(.065,.700),(.125,.683),(.125,.760),(.220,.736),(.210,.819),(.305,.810),(.298,.895),(.405,.938),(.500,1.025),(.370,1.036),(.262,.950),(.264,.892),(.188,.900),(.192,.835),(.115,.845),(.114,.782),(.051,.805),(.047,.727),(.015,.660)]
        for side in (-1,1):
            leaf=membrane('Leaf / continuous serrated crown blade '+str(side),LEAF,outline,side,-.038,thickness=.008);p.append(leaf)
            points=[(side*x,y,-.038-.06*x+.025*(y-.70)**2+.010) for x,y in ((.027,.604),(.101,.761),(.235,.894),(.449,1.01))]
            p.append(tube('Leaf vein / tapered midrib '+str(side),VEIN,points,[.006,.005,.003,.0008],10))
            ear=membrane('Ear / pointed pink shell '+str(side),PINK,[(.13,.49),(.225,.515),(.351,.645),(.223,.638),(.15,.576)],side,.038,thickness=.023);p.append(ear)
            ear_skin=surface(ear);inner=[(side*.229,.557),(side*.309,.622),(side*.249,.604)]
            p.append(patch('Ear / fitted mauve interior '+str(side),INK,inner,ear_skin,.004))
        p.append(tube('Tail / curved pink petiole',PINK,[(0,.25,-.15),(0,.27,-.26),(.03,.30,-.32)],[.031,.029,.023],16))
        p.append(ellipsoid('Tail / flattened mauve tip',MAUVE,(.035,.305,-.32),(.075,.033,.065),20,12))
    elif skip:
        for side in (-1,1):
            outline=contour([((.105,.535),(.13,.59),(.18,.61),(.24,.594)),((.24,.594),(.34,.57),(.41,.52),(.427,.471)),((.427,.471),(.39,.41),(.31,.41),(.26,.428)),((.26,.428),(.17,.45),(.11,.47),(.105,.535))],7)
            leaf=membrane('Ear leaf / broad folded green paddle '+str(side),GREEN,outline,side,.058,thickness=.019);p.append(leaf)
            skin_ear=surface(leaf)
            points=[(side*x,y,skin_ear(y,side*x)+.004) for x,y in ((.135,.53),(.224,.527),(.335,.48))]
            p.append(tube('Ear fold / curved fitted crease '+str(side),VEIN,points,[.0035,.003,.0008],8))
        p.append(tube('Flower / short supporting stalk',GREEN,[(0,.568,-.025),(0,.655,-.025)],[.038,.031],18))
        for i in range(8):
            a=math.tau*i/8;petal=ellipsoid('Flower petal / radial spoon '+str(i),YELLOW,(0,.665,.125),(.066,.025,.104),24,12)
            petal['positions']=[v for k in range(0,len(petal['positions']),3) for v in (petal['positions'][k]*math.cos(a)+petal['positions'][k+2]*math.sin(a),petal['positions'][k+1],-petal['positions'][k]*math.sin(a)+petal['positions'][k+2]*math.cos(a)-.025)]
            petal['normals']=[v for k in range(0,len(petal['normals']),3) for v in (petal['normals'][k]*math.cos(a)+petal['normals'][k+2]*math.sin(a),petal['normals'][k+1],-petal['normals'][k]*math.sin(a)+petal['normals'][k+2]*math.cos(a))]
            p.append(petal)
        p.append(ellipsoid('Flower / pale green central disk',(.80,.87,.61),(0,.682,-.025),(.075,.023,.075),24,12))
    else:
        for side in (-1,1):
            p.append(tube('Seed stem / lower globe '+str(side),GREEN,[(side*.16,.275,-.012),(side*.29,.198,-.014)],[.021,.018],12))
            p.extend(seed_globe('Seed head / lower '+str(side),(side*.306,.166,-.014),.145))
        p.append(tube('Seed stem / tall crown peduncle',GREEN,[(0,.573,-.025),(0,.793,-.025)],[.024,.018],14))
        for side in (-1,1):
            p.append(membrane('Leaf / small crown bract '+str(side),LEAF,[(.006,.605),(.078,.655),(.105,.657),(.075,.613),(.009,.582)],side,-.020,thickness=.007))
        p.extend(seed_globe('Seed head / crown',(0,.908,-.025),.197))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))

if __name__=='__main__':main()
