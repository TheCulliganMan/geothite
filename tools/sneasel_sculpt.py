#!/usr/bin/env python3
"""Original alert Sneasel with angular fitted eyes and curved ivory talons."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals
from abra_sculpt import surface
from wing_insect_sculpt import contour,membrane
from skin_glb import COORDINATES
SPECIES=('sneasel',);HEIGHT=1.11
TEAL=(.22,.43,.50);ROSE=(.84,.37,.46);IVORY=(.95,.97,.95)
INK=(.065,.12,.16);GOLD=(.97,.78,.29);RUBY=(.81,.22,.30);TONGUE=(.92,.51,.61)

def oval(cx,cy,rx,ry,n=18):return [(cx+rx*math.cos(i*math.tau/n),cy+ry*math.sin(i*math.tau/n)) for i in range(n)]

def sculpture(species='sneasel'):
    if species not in SPECIES:raise ValueError('unsupported claw weasel')
    body=loft('Body / slim continuous hips waist and chest',TEAL,curved_profiles(((.27,.055,.055,-.015),(.34,.145,.10,-.015),(.45,.125,.115,-.015),(.57,.13,.125,-.015),(.67,.14,.12,-.005),(.74,.08,.075,.005)),4),32)
    head=loft('Head / tapered muzzle and broad slanted feline crown',TEAL,curved_profiles(((.687,.039,.040,.10),(.725,.125,.135,.055),(.78,.218,.181,.017),(.86,.245,.19,-.007),(.945,.21,.16,-.018),(1.005,.13,.105,-.021),(1.032,.019,.018,-.021)),4),36)
    p=[body,head];bs=surface(body);hs=surface(head)
    ear=[(-.10,.975),(-.19,1.205),(-.24,1.055),(-.19,.990)]
    p.append(membrane('Ear / single pointed teal ear',TEAL,ear,1,root_z=.012,thickness=.033))
    plume=contour([((.105,.955),(.13,1.17),(.29,1.40),(.45,1.43)),((.45,1.43),(.46,1.27),(.30,1.13),(.21,1.065)),((.21,1.065),(.19,1.035),(.18,1.02),(.17,1.045)),((.17,1.045),(.15,1.01),(.13,.975),(.105,.955))],8)
    p.append(membrane('Plume / long swept rose ear feather',ROSE,plume,1,root_z=.030,thickness=.015))
    tails=[contour([((-.028,.32),(-.10,.26),(-.20,.13),(-.20,.055)),((-.20,.055),(-.07,.14),(-.03,.23),(.023,.32)),((.023,.32),(.01,.34),(-.01,.34),(-.028,.32))],8),
           contour([((-.023,.31),(.015,.20),(.10,.075),(.14,.045)),((.14,.045),(.18,.16),(.095,.26),(.025,.32)),((.025,.32),(.01,.34),(-.01,.33),(-.023,.31))],8),
           contour([((-.015,.325),(.095,.38),(.20,.375),(.295,.355)),((.295,.355),(.21,.30),(.13,.28),(.012,.305)),((.012,.305),(.005,.31),(-.005,.32),(-.015,.325))],8)]
    for j,outline in enumerate(tails):p.append(membrane('Tail / swept rose feather '+str(j),ROSE,outline,1,root_z=-.090-j*.012,thickness=.014))
    p.append(tube('Tail / short rooted fan stem',TEAL,[(0,.32,-.060),(0,.30,-.108),(0,.29,-.133)],[.043,.034,.021],16))
    for side in (-1,1):
        p.append(tube('Leg / curved outward hock '+str(side),TEAL,[(side*.115,.33,-.015),(side*.16,.20,.010),(side*.22,.082,.09)],[.062,.058,.057],20))
        p.append(ellipsoid('Foot / broad grounded teal pad '+str(side),TEAL,(side*.23,.055,.12),(.09,.05,.12),24,16))
        for j,dx in enumerate((-.032,.032)):
            p.append(tube(f'Talon / paired curved foot claw {side} {j}',IVORY,[(side*.23+dx,.061,.194),(side*.23+dx,.039,.256),(side*.23+dx,.018,.303),(side*.23+dx,.013,.316)],[.031,.025,.013,.0015],18))
        palm=(-.43,.885,.085) if side<0 else (.365,.555,.145)
        points=[(side*.113,.665,-.005),(side*.25,.745,.015),palm] if side<0 else [(side*.115,.644,.0),(side*.27,.592,.075),palm]
        p.append(tube('Arm / curved reaching forelimb '+str(side),TEAL,points,[.052,.049,.047],20))
        hand=ellipsoid('Hand / broad claw-bearing palm '+str(side),TEAL,palm,(.069,.058,.062),24,14);p.append(hand)
        for j,dx in enumerate((-.030,.030)):
            x=palm[0]+dx;y=palm[1];z=palm[2]
            p.append(tube(f'Talon / hooked ivory hand claw {side} {j}',IVORY,[(x,y+.010,z+.04),(x+side*.012,y-.008,z+.104),(x+side*.015,y-.060,z+.153),(x+side*.011,y-.094,z+.16)],[.029,.027,.017,.0015],18))
        outline=[(side*.055,.856),(side*.198,.900),(side*.184,.814),(side*.077,.793)]
        socket=fitted_patch('Eye / fitted dark slanted socket '+str(side),INK,outline,hs,.004);p.append(socket)
        cx=sum(x for x,y in outline)/4;cy=sum(y for x,y in outline)/4
        inset=[(cx+(x-cx)*.82,cy+(y-cy)*.76) for x,y in outline]
        eye=fitted_patch('Eye / fitted ivory angular white '+str(side),IVORY,inset,surface(socket),.003);p.append(eye)
        x=side*.115;y=.833
        iris=fitted_patch('Eye / narrow fitted red iris '+str(side),RUBY,oval(x,y,.008,.016),surface(eye),.002);p.append(iris)
        p.append(fitted_patch('Eye / vertical dark pupil '+str(side),INK,oval(x,y,.003,.013,12),surface(iris),.001))
    for name,skin,y in (('forehead',hs,.968),('chest',bs,.607)):
        p.append(fitted_patch('Gem / fitted gold oval '+name,GOLD,oval(0,y,.022,.040,24),skin,.007))
    mouth=fitted_patch('Mouth / fitted open mischievous grin',INK,[(-.069,.745),(.069,.758),(.029,.701),(-.025,.699)],hs,.006);p.append(mouth)
    ms=surface(mouth)
    p.append(fitted_patch('Mouth / fitted pink tongue',TONGUE,[(-.023,.713),(.025,.715),(.014,.704),(-.010,.702)],ms,.002))
    for side in (-1,1):
        x=side*.037
        p.append(fitted_patch('Mouth / small ivory fang '+str(side),IVORY,[(x-.009,.740),(x+.009,.742),(x,.727)],ms,.003))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'sneasel.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
