#!/usr/bin/env python3
"""Original continuous chrysalis contours and fitted sleepy cocoon faces."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,fitted_patch,tube
from abra_sculpt import surface
from skin_glb import COORDINATES

SPECIES=('metapod','kakuna');HEIGHT={'metapod':.80,'kakuna':.80}
GREEN=(.42,.67,.25);LIGHT=(.57,.76,.33);GOLD=(.96,.73,.21)
OCHRE=(.76,.50,.12);INK=(.065,.095,.054);WHITE=(.96,.96,.79)


def shell(species):
    # Rings prescribe the full crescent profile, including the forward cheek,
    # folded lower belly and hooked crown. No duplicate sphere behind a plate.
    profiles=(((.012,.013,.01,-.025),(.12,.085,.058,-.12),(.26,.13,.025,-.186),
               (.34,.156,.13,-.209),(.435,.175,.243,-.201),(.51,.173,.199,-.193),
               (.615,.153,.129,-.172),(.72,.109,.066,-.132),(.795,.021,-.072,-.101))
              if species=='metapod' else
              ((.012,.012,.01,-.012),(.10,.078,.061,-.071),(.24,.127,.105,-.117),
               (.36,.157,.145,-.148),(.45,.181,.173,-.162),(.54,.178,.154,-.157),
               (.645,.152,.109,-.129),(.735,.091,.069,-.08),(.795,.013,.015,-.018)))
    # Octagonal front/back sections keep a deliberate hard carapace silhouette.
    v=[];f=[];segments=8
    for y,w,front,back in profiles:
        mid=(front+back)/2
        v.extend(((0,y,front),(w*.72,y,front*.76+mid*.24),(w,y,mid),
                  (w*.72,y,back*.76+mid*.24),(0,y,back),
                  (-w*.72,y,back*.76+mid*.24),(-w,y,mid),(-w*.72,y,front*.76+mid*.24)))
    f=[tuple(reversed(range(segments))),tuple((len(profiles)-1)*segments+i for i in range(segments))]
    for j in range(len(profiles)-1):
        for i in range(segments):
            a=j*segments+i;b=j*segments+(i+1)%segments
            f.append((a,b,b+segments,a+segments))
    return part('Shell / continuous crescent chrysalis' if species=='metapod' else 'Shell / continuous tapered golden cocoon',GREEN if species=='metapod' else GOLD,v,f)


def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported cocoon')
    body=shell(species);skin=surface(body);p=[body];green=species=='metapod'
    for s in (-1,1):
        if green:
            outline=[(s*.034,.578),(s*.112,.562),(s*.103,.524),(s*.04,.533)]
            iris=[(s*.065,.562),(s*.082,.558),(s*.080,.532),(s*.063,.536)]
            p.append(fitted_patch('Eye / recessed ivory slit '+str(s),WHITE,outline,skin,.003))
            p.append(fitted_patch('Eye / sleepy narrow pupil '+str(s),INK,iris,lambda y,x:skin(y,x)+.005,.003))
            brow=[(s*.029,.582),(s*.115,.566),(s*.116,.560),(s*.030,.576)]
            p.append(fitted_patch('Brow / angular fitted lid '+str(s),LIGHT,brow,lambda y,x:skin(y,x)+.009,.003))
        else:
            outline=[(s*.035,.674),(s*.112,.653),(s*.095,.596),(s*.039,.619)]
            p.append(fitted_patch('Eye / inset angular black slit '+str(s),INK,outline,skin,.004))
            shine=[(s*.045,.660),(s*.060,.655),(s*.053,.633),(s*.043,.639)]
            p.append(fitted_patch('Eye / restrained inner highlight '+str(s),WHITE,shine,lambda y,x:skin(y,x)+.006,.002))
    if green:
        for s in (-1,1):
            outline=[(s*.006,.445),(s*.125,.409),(s*.079,.331),(s*.008,.344)]
            p.append(fitted_patch('Shell fold / fitted cheek facet '+str(s),LIGHT,outline,skin,.003))
        seam=[(-.116,.337),(-.057,.311),(0,.296),(.057,.311),(.116,.337)]
        p.append(tube('Shell crease / lower folded seam',(.25,.43,.16),[(x,y,skin(y,x)+.004) for x,y in seam],[.0025]*len(seam),8))
    else:
        # Distinct overlapping folded arm plates fit the broad middle shell.
        for s in (-1,1):
            outline=[(s*.013,.504),(s*.143,.498),(s*.125,.441),(s*.015,.386)]
            p.append(fitted_patch('Arm fold / fitted golden plate '+str(s),(.91,.64,.15),outline,skin,.006))
            seam=[(s*.014,.393),(s*.066,.420),(s*.123,.453)]
            p.append(tube('Arm crease / fine ochre edge '+str(s),OCHRE,[(x,y,skin(y,x)+.009) for x,y in seam],[.002]*3,8))
        for y,w in ((.30,.111),(.22,.091)):
            seam=[(-w,y+.02),(-w*.5,y+.008),(0,y),(w*.5,y+.008),(w,y+.02)]
            p.append(tube('Shell crease / abdominal segment '+str(y),OCHRE,[(x,yy,skin(yy,x)+.004) for x,yy in seam],[.0022]*5,8))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT[species]/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))

if __name__=='__main__':main()
