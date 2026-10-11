#!/usr/bin/env python3
"""Original painter-dog sculpture with a held looping tail and green brush."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals,translate
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import membrane,contour
from spider_sculpt import banded_limb
from skin_glb import COORDINATES
SPECIES=('smeargle',);HEIGHT=1.14
CREAM=(.91,.88,.77);BROWN=(.49,.32,.23);DARK=(.22,.16,.13);WHITE=(.98,.98,.95);GREEN=(.40,.65,.25);TONGUE=(.84,.48,.51)

def brush():
    # Exact shared scalloped paint boundary around a shaped, tufted brush solid.
    n=36;boundary=[];cream=[];green=[]
    for j in range(5):
        t=j/4
        for k in range(n):
            a=k*math.tau/n;y=.885*(1-t)+(.985+.020*math.cos(5*a))*t
            w=.031*(1-t)+.069*t;z=.142-.012*t
            cream.append((.447+w*math.sin(a),y,z+w*.64*math.cos(a)))
    for j in range(9):
        t=j/8
        for k in range(n):
            a=k*math.tau/n;y=(.985+.020*math.cos(5*a))*(1-t)+1.173*t
            w=max(.0015,(.069*(1-t)+.035*math.sin(math.pi*t))*(1-.96*t**5))
            y+=.020*math.sin(3*a)*math.sin(math.pi*t)**2
            green.append((.447+.016*t+w*math.sin(a),y,.130-.032*t+w*.64*math.cos(a)))
    def shell(name,color,v):
        rows=len(v)//n;f=[tuple(reversed(range(n))),tuple((rows-1)*n+k for k in range(n))]
        for j in range(rows-1):
            for k in range(n):a=j*n+k;b=j*n+(k+1)%n;f.append((a,b,b+n,a+n))
        return crease_normals(part(name,color,v,f),38.)
    return [shell('Brush / cream bristles beneath scalloped paint',CREAM,cream),shell('Brush / tapered green paint tuft',GREEN,green)]

def sculpture(species='smeargle'):
    if species not in SPECIES:raise ValueError('unsupported painter dog')
    body=loft('Body / continuous slim cream painter torso',CREAM,curved_profiles(((.19,.04,.05,0),(.25,.13,.115,-.012),(.34,.145,.125,-.012),(.46,.12,.105,-.008),(.57,.13,.10,0),(.64,.095,.085,.005),(.69,.033,.034,.005)),4),32)
    neck=ellipsoid('Neck / brown canine collar',BROWN,(0,.646,.002),(.106,.072,.087),28,16)
    head=loft('Head / broad cream dog crown and long tapered snout',CREAM,curved_profiles(((.643,.033,.04,.155),(.69,.125,.108,.133),(.75,.19,.17,.071),(.84,.21,.167,.021),(.919,.178,.137,-.007),(.959,.114,.10,-.015),(.979,.03,.035,-.015)),4),36)
    p=[body,neck,head];hs=surface(head)
    for side in (-1,1):
        outline=contour([((side*.14,.90),(side*.255,.899),(side*.30,.80),(side*.32,.685)),((side*.32,.685),(side*.26,.642),(side*.19,.69),(side*.17,.785)),((side*.17,.785),(side*.16,.84),(side*.14,.86),(side*.14,.90))],7)
        p.append(membrane('Ear / short floppy brown dog ear '+str(side),BROWN,outline,1,root_z=.012,thickness=.043))
        socket=fitted_patch('Eye / fitted large brown oval socket '+str(side),BROWN,oval(side*.086,.834,.062,.077,28),hs,.013);p.append(socket)
        white=fitted_patch('Eye / large ivory oval '+str(side),WHITE,oval(side*.086,.834,.046,.064,28),surface(socket),.004);p.append(white)
        p.append(fitted_patch('Eye / narrow dark curious pupil '+str(side),DARK,oval(side*.084,.843,.0055,.015,16),surface(white),.002))
        p.append(curved_tube('Leg / slim cream canine shin '+str(side),CREAM,[(side*.09,.29,-.005),(side*.11,.18,.01),(side*.13,.082,.026)],[.052,.041,.038],18))
        p.append(curved_tube('Cuff / brown ankle marking '+str(side),BROWN,[(side*.12,.142,.019),(side*.128,.102,.023)],[.043,.040],20))
        foot=ellipsoid('Foot / broad grounded cream three-toe paw '+str(side),CREAM,(side*.132,.047,.057),(.075,.047,.108),28,16);p.append(foot);fs=surface(foot)
        for j,dx in enumerate((-.025,.025)):
            x=side*.132+dx;p.append(tube(f'Toe groove / fitted paw cleft {side} {j}',BROWN,[(x,y,fs(y,x)+.001) for y in (.015,.035,.054)],[.0013,.0013,.001],8))
    muzzle=ellipsoid('Muzzle / broad projecting cream dog snout',CREAM,(0,.720,.216),(.145,.078,.092),32,18);p.append(muzzle);ms=surface(muzzle)
    for side in (-1,1):p.append(fitted_patch('Nostril / small fitted canine fleck '+str(side),DARK,oval(side*.053,.716,.009,.004,14),ms,.002))
    mouth=fitted_patch('Mouth / fitted dark smile below muzzle',DARK,[(-.048,.693),(.048,.693),(.034,.653),(-.031,.652)],hs,.005);p.append(mouth)
    p.append(ellipsoid('Tongue / soft protruding pink tongue',TONGUE,(0,.650,.211),(.039,.034,.018),24,14))
    # Broad soft cap and tiny bent crown point, rather than a brown doughnut brim.
    cap=loft('Beret / continuous soft cream painter cap',CREAM,curved_profiles(((.925,.14,.115,-.02),(.96,.226,.174,-.02),(1.01,.234,.18,-.032),(1.059,.195,.149,-.037),(1.087,.127,.104,-.028),(1.10,.032,.032,-.015)),4),40)
    # A shallow slant keeps the hat from reading as a perfect stacked ellipsoid.
    cap['positions']=[v+.035*cap['positions'][i-1] if i%3==1 else v for i,v in enumerate(cap['positions'])]
    # Recompute normals after deforming the cap, preserving its stored triangles.
    vv=[tuple(cap['positions'][i:i+3]) for i in range(0,len(cap['positions']),3)];ff=[tuple(cap['indices'][i:i+3]) for i in range(0,len(cap['indices']),3)]
    p.append(crease_normals(part('Beret / continuous soft cream painter cap',CREAM,vv,ff),40.))
    p.append(curved_tube('Beret / bent pointed crown nub',CREAM,[(.006,1.08,-.02),(.022,1.124,-.023),(.047,1.162,-.012)],[.031,.023,.0015],20))
    # Right forepaw grips the loop near the brush; left hand opens away from it.
    for side in (-1,1):
        path=[(.10,.588,0),(.205,.581,.095),(.316,.616,.245)] if side==1 else [(-.10,.565,0),(-.20,.484,.012),(-.245,.415,.083)]
        rr=[.046,.039,.036]
        pp=[];r=[]
        for j in range(7):t=j/6;pp.append(tuple(path[0][k]*(1-t)+path[1][k]*t for k in range(3)));r.append(rr[0]*(1-t)+rr[1]*t)
        for j in range(1,8):t=j/7;pp.append(tuple(path[1][k]*(1-t)+path[2][k]*t for k in range(3)));r.append(rr[1]*(1-t)+rr[2]*t)
        colors=[CREAM]*13
        for j in (5,6,9,10):colors[j]=BROWN
        p.extend(banded_limb('Arm / cream forelimb and brown wrist bands '+str(side),pp,r,colors))
        hand=ellipsoid('Hand / cream open palm '+str(side),CREAM,path[-1],(.055,.047,.041),24,14);p.append(hand)
        for j in range(3):
            dx=(j-1)*.030;x=path[-1][0]+dx;y=path[-1][1];z=path[-1][2]
            points=[(x,y+.013,z+.010),(x+side*.009,y+.048,z+.019),(x+side*.013,y+.053,z+.045)] if side==1 else [(x,y-.009,z+.01),(x+side*.013,y-.040,z+.021),(x+side*.011,y-.045,z+.040)]
            p.append(curved_tube(f'Finger / rounded canine digit {side} {j}',CREAM,points,[.020,.016,.010],14))
    tail=[(0,.29,-.09),(-.08,.34,-.22),(-.30,.45,-.22),(-.47,.56,.02),(-.42,.59,.28),(-.15,.52,.37),(.13,.53,.355),(.31,.63,.27),(.397,.77,.18),(.447,.909,.142)]
    p.append(curved_tube('Tail / continuous looping cream paintbrush handle',CREAM,tail,[.025,.026,.027,.027,.026,.025,.024,.023,.026,.033],18))
    p.extend(brush())
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'smeargle.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
