#!/usr/bin/env python3
"""Original low striped Swinub and tall shaggy Piloswine anatomy."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,translate,fitted_patch,crease_normals
from abra_sculpt import surface
from spider_sculpt import dorsal_patch,upper_skin
from skin_glb import COORDINATES

SPECIES=('swinub','piloswine');HEIGHT=.98
FUR=(.71,.55,.43);LONG_FUR=(.67,.49,.36);STRIPE=(.40,.29,.23)
PINK=(.91,.53,.60);INK=(.20,.13,.13);HOOF=(.37,.29,.24)
IVORY=(.96,.95,.85);EAR=(.72,.54,.40)

def mantle():
    rows=curved_profiles(((.10,.32,.33,-.06),(.22,.35,.36,-.045),(.40,.35,.35,-.035),(.64,.32,.33,-.055),(.83,.28,.29,-.095),(.95,.23,.26,-.105),(1.02,.14,.17,-.105),(1.045,.065,.09,-.10),(1.055,.009,.014,-.095)),3)
    v=[];n=48
    for y,w,d,z in rows:
        for i in range(n):
            a=math.tau*i/n;fade=max(0.,1-y/.70)
            hem=max(0.,1-(y-.10)/.24)*.045*math.cos(12*a)
            ripple=1+.028*math.cos(12*a+.25)*fade
            v.append((w*math.sin(a)*ripple,y+hem,z+d*math.cos(a)*ripple))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for j in range(len(rows)-1):
        for i in range(n):
            a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part('Body / continuous shaggy humped mantle',LONG_FUR,v,f),40.)

def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported fur pig')
    big=species=='piloswine';color=LONG_FUR if big else FUR
    body=mantle() if big else loft('Body / continuous low elongated fur loaf',FUR,
        curved_profiles(((.026,.25,.30,-.055),(.055,.30,.38,-.060),(.13,.335,.415,-.065),(.19,.33,.415,-.065),(.29,.30,.36,-.085),(.36,.20,.27,-.10),(.397,.015,.023,-.10)),4),40)
    p=[body];skin=surface(body)
    for side in (-1,1):
        for row,z in enumerate((.17,-.30)):
            p.append(ellipsoid(f'Leg / hidden supporting shank {side} {row}',color,(side*.20,.091,z),(.061,.085,.065),18,12))
            p.append(ellipsoid(f'Hoof / grounded oval pad {side} {row}',HOOF,(side*.20,.028,z+.015),(.079,.026,.097),20,12))
    y=.225 if big else .18
    snout=ellipsoid('Snout / rounded projecting pink oval',PINK,(0,y,skin(y,0)+.016),(.105 if big else .093,.068 if big else .057,.033),28,16);p.append(snout)
    snout_skin=surface(snout)
    for side in (-1,1):
        x=side*.037;outline=[(x-.006,y+.017),(x+.003,y+.014),(x+.010,y-.016),(x+.002,y-.020)]
        p.append(fitted_patch('Nostril / fitted tilted slit '+str(side),INK,outline,snout_skin,.003))
    if big:
        for side in (-1,1):
            p.append(tube('Tusk / downward forward ivory curve '+str(side),IVORY,
                [(side*.17,.27,.29),(side*.226,.215,.395),(side*.224,.132,.470),(side*.181,.104,.529),(side*.144,.111,.56)],
                [.059,.051,.037,.017,.0015],24))
            ear=translate(loft('Ear / long tapered drooping flap '+str(side),EAR,
                curved_profiles(((.26,.012,.018,.13),(.32,.062,.079,.14),(.45,.083,.098,.125),(.58,.066,.074,.10),(.67,.028,.035,.065)),3),24),side*.30,0,0);p.append(ear)
        for side in (-1,1):
            for j,x in enumerate((.073,.14,.215)):
                points=[]
                for yy in (.63,.49,.36,.28):
                    xx=side*(x+(.63-yy)*.025);points.append((xx,yy,skin(yy,xx)+.002))
                p.append(tube(f'Fur crease / fitted front strand {side} {j}',(.49,.35,.25),points,[.0025,.0025,.002,.001],8))
    else:
        top=upper_skin(body)
        for i,(z,w) in enumerate(((-.02,.27),(-.17,.25),(-.30,.20),(-.40,.12))):
            outline=[(-w,z-.02),(w,z-.02),(w*.78,z+.028),(0,z+.070),(-w*.78,z+.028)]
            p.append(dorsal_patch('Stripe / tapered fitted dorsal wedge '+str(i),STRIPE,outline,top))
        for side in (-1,1):
            points=[(side*x,yy,skin(yy,side*x)+.002) for x,yy in ((.09,.262),(.135,.255),(.177,.268))]
            p.append(tube('Eye / fitted sleepy arc '+str(side),INK,points,[.0028,.0032,.0024],10))
        for i,x in enumerate((-.095,-.04,.035,.09)):
            p.append(tube('Fur fringe / tapered rear wisp '+str(i),FUR,[(x,.143,-.416),(x*1.12,.117,-.486),(x*1.22,.101,-.533)],[.021,.012,.001],10))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))

if __name__=='__main__':main()
