#!/usr/bin/env python3
"""Original Skarmory steel armor, swept blade wings and angular avian head."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals
from abra_sculpt import surface,polygon_caps
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('skarmory',);HEIGHT=1.21
STEEL=(.76,.81,.88);LIGHT=(.86,.90,.96);SHADE=(.43,.50,.62)
RED=(.72,.25,.30);DARK_RED=(.60,.18,.24);INK=(.09,.13,.20);GOLD=(.98,.83,.28)

def blade(name,color,outline,depth,thickness=.015,vertical=False):
    n=len(outline);v=[]
    for sign in (1,-1):
        for x,y in outline:
            z=depth(x,y)+sign*thickness
            v.append((-z,y,x) if vertical else (x,y,z))
    caps=polygon_caps([(x,y,0.) for x,y in outline]);ids=list(range(n))
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:ids.reverse()
    f=caps+[tuple(i+n for i in reversed(t)) for t in caps]+[(a,a+n,b+n,b) for a,b in zip(ids,ids[1:]+ids[:1])]
    return crease_normals(part(name,color,v,f),35.)

def skull():
    rows=[(-.055,.037,1.075,1.238),(.045,.106,1.055,1.263),(.15,.115,1.063,1.243),(.27,.072,1.081,1.185),(.49,.004,1.110,1.118)]
    v=[];n=8
    for z,w,low,high in rows:
        mid=(low+high)/2
        v.extend((x,y,z) for x,y in ((-w*.55,low),(w*.55,low),(w,mid),(w*.70,high-.012),(w*.38,high),(-w*.38,high),(-w*.70,high-.012),(-w,mid)))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for j in range(len(rows)-1):
        for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part('Head / joined angular skull and sharp upper bill',LIGHT,v,f),38.)

def sculpture(species='skarmory'):
    if species not in SPECIES:raise ValueError('unsupported steel bird')
    body=loft('Body / continuous steel breast shoulder and rising neck',STEEL,curved_profiles(((.22,.026,.035,-.025),(.31,.13,.13,-.01),(.47,.205,.175,-.015),(.62,.214,.19,-.025),(.73,.16,.145,.005),(.84,.09,.078,.06),(.95,.070,.071,.11),(1.07,.063,.068,.16),(1.13,.060,.062,.17)),3),12)
    body=crease_normals(body,38.);head=skull();p=[body,head]
    p.append(blade('Crest / upright swept steel blade',LIGHT,[(.115,1.205),(-.075,1.49),(-.095,1.29),(-.06,1.17)],lambda z,y:0.,.027,vertical=True))
    for side in (-1,1):
        for j in range(2):
            p.append(blade(f'Nape / layered rear steel plume {side} {j}',SHADE if j else STEEL,[(.04,1.02+j*.045),(-.13-j*.10,1.39-j*.065),(-.16-j*.06,1.18),(-.055,1.00)],lambda z,y:-side*(.04+j*.009),.009,vertical=True))
        upper=contour([((.13,.69),(.28,.96),(.68,1.30),(1.02,1.35)),((1.02,1.35),(.83,1.18),(.56,.99),(.37,.80)),((.37,.80),(.25,.72),(.18,.67),(.13,.69))],10)
        upper=[(side*x,y) for x,y in upper]
        depth=lambda x,y:-.035-.045*abs(x)+.04*(y-.70)
        p.append(blade('Wing / swept steel leading edge '+str(side),LIGHT,upper,depth,.018))
        for j in range(4):
            root=(.12+j*.008,.69-j*.014);lower=(root[0],root[1]-.052);tip=(1.055-j*.071,1.22-j*.098)
            outline=contour([(root,(.34,.85-j*.028),(.71,1.23-j*.090),tip),(tip,(.79,1.11-j*.084),(.45,.79-j*.030),lower),(lower,(root[0]-.002,root[1]-.040),(root[0]-.002,root[1]-.012),root)],8)
            outline=[(side*x,y) for x,y in outline]
            p.append(blade(f'Wing / layered red blade {side} {j}',RED if j%2==0 else DARK_RED,outline,lambda x,y:.006+j*.018-.045*abs(x),.010))
        p.append(ellipsoid('Thigh / articulated silver haunch '+str(side),STEEL,(side*.13,.30,.005),(.079,.115,.09),18,12))
        p.append(tube('Leg / bent dark steel hock '+str(side),SHADE,[(side*.13,.29,.01),(side*.15,.15,.01),(side*.16,.077,.07)],[.032,.026,.023],14))
        for j,dx in enumerate((-.032,0,.032)):
            p.append(tube(f'Claw / hooked forward steel talon {side} {j}',SHADE,[(side*.16,.071,.072),(side*.16+dx,.041,.14),(side*.16+dx*1.25,.018,.23),(side*.16+dx*1.22,.014,.24)],[.020,.015,.006,.001],12))
        p.append(tube('Claw / hooked rear talon '+str(side),SHADE,[(side*.16,.071,.052),(side*.17,.035,-.025),(side*.18,.014,-.065)],[.019,.010,.001],12))
    p.append(blade('Tail / long hooked steel spear',SHADE,[(-.07,.37),(-.25,.48),(-.72,.73),(-.55,.39),(-.44,.37),(-.48,.32),(-.21,.32)],lambda z,y:0.,.026,vertical=True))
    bs=surface(body)
    for j,y in enumerate((.82,.90,.99,1.07)):
        w=.056 if j<2 else .043
        points=[(x,y,bs(y,x)+.002) for x in (-w,-w*.66,-w*.33,0,w*.33,w*.66,w)]
        p.append(tube('Neck / fitted overlapping armor seam '+str(j),SHADE,points,[.0028]*len(points),8))
    hs=surface(head)
    for side in (-1,1):
        outline=[(side*.033,1.190),(side*.087,1.207),(side*.084,1.171),(side*.039,1.162)]
        eye=fitted_patch('Eye / fitted dark angular socket '+str(side),INK,outline,hs,.003);p.append(eye)
        cx=sum(x for x,y in outline)/4;cy=sum(y for x,y in outline)/4
        inset=[(cx+(x-cx)*.77,cy+(y-cy)*.72) for x,y in outline]
        p.append(fitted_patch('Eye / fitted yellow slit '+str(side),GOLD,inset,surface(eye),.002))
        pupil=[(cx-.004,cy-.010),(cx+.004,cy-.010),(cx+.004,cy+.010),(cx-.004,cy+.010)]
        p.append(fitted_patch('Eye / dark narrow pupil '+str(side),INK,pupil,surface(p[-1]),.001))
    # Closed lower bill plate leaves a visible dark seam under the upper edge.
    p.append(blade('Bill / tapered lower steel mandible',STEEL,[(.17,1.074),(.425,1.105),(.30,1.055),(.17,1.050)],lambda z,y:0.,.021,vertical=True))
    p.append(tube('Bill / dark lower mouth seam',INK,[(0,1.078,.20),(0,1.089,.30),(0,1.102,.41)],[.005,.004,.001],10))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'skarmory.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
