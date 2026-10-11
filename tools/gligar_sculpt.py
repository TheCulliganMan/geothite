#!/usr/bin/env python3
"""Original flying scorpion with hooked pincers, scalloped sails and a tongue-out grin."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals,tube
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import contour,membrane
from skin_glb import COORDINATES
HEIGHT=1.06
PINK=(.79,.51,.71);LIGHT=(.86,.59,.78);SHADE=(.62,.38,.59);BLUE=(.29,.58,.78)
INK=(.09,.10,.14);IVORY=(.98,.97,.90);TONGUE=(.91,.61,.64);GROOVE=(.64,.36,.45)

def pincer(sign):
    cx=-.45 if sign<0 else .49;cy=.82 if sign<0 else .69;rot=.27 if sign<0 else -1.10;c=math.cos(rot);s=math.sin(rot);outer=[];inner=[]
    for k in range(49):
        t=k/48;a=math.radians(155)+1.5*math.pi*t;w=.001+.046*math.sin(math.pi*t)**.72
        for pts,d in ((outer,w),(inner,-w)):
            x=(.100+d)*math.cos(a);y=(.155+d)*math.sin(a);pts.append((cx+x*c-y*s,cy+x*s+y*c))
    xy=outer+list(reversed(inner))
    return fitted_contour('Pincer / thick hooked crescent claw '+str(sign),LIGHT,xy,lambda y,x:.050-.15*((x-cx)**2+(y-cy)**2),.045,subdivisions=1)

def sculpture():
    body=loft('Body / rounded continuous pink scorpion torso',PINK,curved_profiles(((.130,.043,.040,-.025),(.207,.131,.107,-.015),(.334,.180,.139,0),(.440,.166,.124,0),(.523,.115,.096,0),(.604,.092,.074,0)),4),40)
    head=ellipsoid('Head / broad flattened pink glider face',PINK,(0,.685,.022),(.290,.170,.170),48,28);p=[body,head];hs=surface(head)
    for sign in (-1,1):
        p.append(curved_tube('Ear / long swept tapered scorpion point '+str(sign),PINK,[(sign*.175,.792,.012),(sign*.203,.907,.011),(sign*.239,1.031,.018),(sign*.259,1.095,.028)],[.071,.060,.029,.001],24))
        p.append(pincer(sign))
        path=[(-.123,.513,.008),(-.264,.578,.016),(-.409,.649,.048)] if sign<0 else [( .123,.505,.006),(.230,.551,.024),(.350,.618,.049)]
        arm=curved_tube('Arm / curved jointed pincer support '+str(sign),PINK,path,[.063,.056,.047],24);p.append(arm)
        wing=contour([((.125,.557),(.215,.586),(.286,.611),(.341,.637)),((.341,.637),(.358,.572),(.382,.534),(.414,.510)),((.414,.510),(.330,.481),(.324,.424),(.365,.377)),((.365,.377),(.283,.392),(.249,.301),(.235,.215)),((.235,.215),(.166,.221),(.124,.273),(.111,.335)),((.111,.335),(.099,.419),(.101,.513),(.125,.557))],9)
        wing=[(sign*x,y+(.019 if sign<0 else 0)) for x,y in wing];sail=membrane('Wing / scalloped blue gliding sail '+str(sign),BLUE,wing,1,root_z=-.032,thickness=.016);p.append(sail)
        p.append(curved_tube('Wing spar / pink upper sail strut '+str(sign),PINK,[(sign*.230,.579+(.019 if sign<0 else 0),.009),(sign*.294,.609+(.019 if sign<0 else 0),-.040),(sign*.333,.635+(.019 if sign<0 else 0),-.048)],[.036,.028,.020],18))
        p.append(curved_tube('Leg / bent pink rear haunch '+str(sign),PINK,[(sign*.105,.242,-.015),(sign*.148,.138,.010),(sign*.205,.070,.080)],[.052,.038,.030],22))
        p.append(ellipsoid('Foot / grounded rounded scorpion pad '+str(sign),LIGHT,(sign*.212,.037,.097),(.064,.037,.063),28,18))
        p.append(curved_tube('Foot claw / pointed curved forward talon '+str(sign),PINK,[(sign*.222,.047,.123),(sign*.242,.042,.174),(sign*.275,.026,.222)],[.035,.024,.001],18))
        x=sign*.119;eye_outline=contour([((x-.053,.664),(x-.048,.713),(x-.027,.757),(x+.001,.761)),((x+.001,.761),(x+.034,.756),(x+.052,.710),(x+.053,.664)),((x+.053,.664),(x+.020,.653),(x-.024,.650),(x-.053,.664))],10)
        eye=fitted_contour('Eye / broad ivory triangular glider eye '+str(sign),IVORY,eye_outline,hs,.006);p.append(eye)
        pupil=fitted_patch('Pupil / narrow dark vertical glider slit '+str(sign),INK,oval(x+.005,.703,.010,.025,24),surface(eye),.003);p.append(pupil)
        p.append(fitted_patch('Eye glint / tiny ivory glider sparkle '+str(sign),IVORY,oval(x+.003,.714,.003,.005,16),surface(pupil),.0015))
    smile=contour([((-.236,.628),(-.140,.596),(.151,.596),(.236,.628)),((.236,.628),(.153,.541),(-.151,.540),(-.236,.628))],14)
    mouth=fitted_contour('Mouth / fitted broad mischievous open grin',INK,smile,hs,.009,subdivisions=3);p.append(mouth)
    tongue=ellipsoid('Tongue / broad rounded protruding pink tongue',TONGUE,(0,.501,.186),(.117,.122,.041),40,24);p.append(tongue)
    p.append(fitted_contour('Tongue crease / fitted short central groove',GROOVE,[(-.003,.533),(.003,.533),(.002,.468),(-.002,.463)],surface(tongue),.0018))
    for sign in (-1,1):
        x=sign*.173;y=.591;z=hs(y,x)
        p.append(curved_tube('Fang / curved pointed ivory mouth tooth '+str(sign),IVORY,[(x,y+.009,z-.009),(sign*.164,.548,z+.015),(sign*.159,.509,z+.018)],[.024,.015,.001],18))
    p.append(curved_tube('Tail / continuous curled rear scorpion tail',PINK,[(0,.226,-.079),(-.024,.159,-.290),(-.229,.150,-.417),(-.403,.223,-.320),(-.479,.333,-.133)],[.055,.050,.045,.041,.035],24))
    p.append(ellipsoid('Tail bulb / rounded venom reservoir',SHADE,(-.486,.354,-.102),(.107,.081,.091),32,20))
    p.append(curved_tube('Stinger / swept sharp poison barb',PINK,[(-.526,.393,-.075),(-.585,.438,-.041),(-.642,.483,-.008)],[.033,.023,.001],20))
    p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Pupil /','Eye glint /','Mouth /','Tongue crease /','Pincer /')) else q for q in p]
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'gligar','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'gligar.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
