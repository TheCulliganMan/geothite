#!/usr/bin/env python3
"""Original Delibird with feather fans, draped collar and folded tail sack."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,translate,crease_normals
from abra_sculpt import surface,polygon_caps
from skin_glb import COORDINATES
SPECIES=('delibird',);HEIGHT=1.10
RED=(.88,.25,.29);WHITE=(.96,.97,.92);SHADE=(.76,.79,.77);GOLD=(.95,.82,.36);INK=(.13,.15,.16)

def drape(name,color,outline,skin,thickness=.007,steps=2):
    # Subdivided concave footprint follows actual stored triangles throughout.
    xy=list(outline);tris=polygon_caps([(x,y,0.) for x,y in xy])
    for _ in range(steps):
        edges={};new=[]
        def midpoint(a,b):
            key=tuple(sorted((a,b)))
            if key not in edges:
                edges[key]=len(xy);xy.append(tuple((xy[a][i]+xy[b][i])/2 for i in range(2)))
            return edges[key]
        for a,b,c in tris:
            ab=midpoint(a,b);bc=midpoint(b,c);ca=midpoint(c,a)
            new.extend(((a,ab,ca),(ab,b,bc),(ca,bc,c),(ab,bc,ca)))
        tris=new
    n=len(xy);v=[(x,y,skin(y,x)+thickness) for x,y in xy]+[(x,y,skin(y,x)-thickness*.3) for x,y in xy]
    f=list(tris)+[tuple(i+n for i in reversed(t)) for t in tris];edges={}
    for t in tris:
        for a,b in zip(t,t[1:]+t[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
    for e in edges.values():
        if len(e)==1:a,b=e[0];f.append((a,a+n,b+n,b))
    return crease_normals(part(name,color,v,f),40.)

def fan(side):
    outline=[(.045,.925),(.13,1.01),(.15,1.18),(.19,1.075),(.29,1.19),(.245,1.065),(.37,1.10),(.30,1.015),(.255,.955),(.19,.94)]
    outline=[(side*x,y) for x,y in outline];n=len(outline)
    v=[(x,y,.09+.20*(y-.95)+depth) for depth in (.016,-.017) for x,y in outline]
    caps=polygon_caps(v[:n]);ids=list(range(n))
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:ids.reverse()
    f=caps+[tuple(i+n for i in reversed(t)) for t in caps]+[(a,a+n,b+n,b) for a,b in zip(ids,ids[1:]+ids[:1])]
    return crease_normals(part('Crest / three-point white fan '+str(side),WHITE,v,f),35.)

def foot(side):
    outline=[(-.06,-.04),(.06,-.04),(.09,.08),(.095,.16),(.048,.125),(.02,.195),(0.,.15),(-.035,.185),(-.040,.125),(-.10,.16),(-.085,.075)]
    n=len(outline);drawing=[(side*.12+x,z,0.) for x,z in outline];caps=polygon_caps(drawing)
    # The top slopes gently into the broad web, with a grounded flat sole.
    v=[(x,0.,z) for x,z,_ in drawing]+[(x,.046-.014*max(0.,z)/.195,z) for x,z,_ in drawing]
    f=list(caps)+[tuple(i+n for i in reversed(t)) for t in caps]
    f += [(i,i+n,(i+1)%n+n,(i+1)%n) for i in range(n)]
    return crease_normals(part('Foot / grounded three-toed web '+str(side),GOLD,v,f),35.)

def sculpture(species='delibird'):
    if species not in SPECIES:raise ValueError('unsupported feather-sack bird')
    body=loft('Body / rounded red penguin pear',RED,curved_profiles(((.105,.09,.10,-.025),(.14,.185,.18,-.025),(.24,.23,.22,-.025),(.40,.235,.225,-.025),(.56,.20,.19,-.012),(.68,.16,.15,.005),(.76,.12,.11,.01),(.80,.015,.02,.01)),4),32)
    head=ellipsoid('Head / broad white feather face',WHITE,(0,.825,.015),(.192,.182,.169),32,20)
    p=[body,head];bs=surface(body);hs=surface(head)
    def skin(y,x):
        depths=[]
        for s in (bs,hs):
            try:depths.append(s(y,x))
            except ValueError:pass
        if not depths:raise ValueError(('collar outside actual body',x,y))
        return max(depths)
    collar=[(-.125,.755),(-.19,.710),(-.155,.680),(-.205,.645),(-.155,.625),(-.17,.565),(-.105,.585),(-.10,.465),(-.052,.514),(0.,.414),(.052,.514),(.10,.465),(.105,.585),(.17,.565),(.155,.625),(.205,.645),(.155,.680),(.19,.710),(.125,.755)]
    # Shoulder roots stay within the actual tapered torso silhouette.
    collar=[(x*.72,y) for x,y in collar]
    p.append(drape('Collar / pointed surface-fitted feather bib',WHITE,collar,skin,thickness=.016,steps=3))
    p.extend((fan(-1),fan(1)))
    for side in (-1,1):
        p.append(foot(side));p.append(ellipsoid('Leg / short hidden gold support '+str(side),GOLD,(side*.12,.112,.015),(.058,.078,.063),20,12))
        profiles=curved_profiles(((.44,.006,.010,.04),(.49,.067,.036,.025),(.60,.091,.048,.006),(.71,.076,.045,-.01),(.76,.035,.027,-.02)) if side==1 else ((.65,.035,.029,-.03),(.73,.074,.042,-.02),(.85,.080,.040,-.01),(.98,.040,.025,.00),(1.055,.003,.004,.02)),4)
        wing=loft('Wing / broad tapered flipper '+str(side),RED,profiles,24)
        wing['positions']=[v+(side*((.15+(yy-.64)*.75) if side<0 else (.24+(yy-.64)*.45)) if k%3==0 else 0.) for k,v in enumerate(wing['positions']) for yy in [wing['positions'][k-k%3+1]]]
        p.append(wing)
        x=side*.088;y=.865
        surround=ellipsoid('Eye / charcoal feather surround '+str(side),INK,(x,y,hs(y,x)+.008),(.064,.080,.018),28,18);p.append(surround)
        ss=surface(surround);eye=ellipsoid('Eye / ivory oval '+str(side),WHITE,(x,y,ss(y,x)+.002),(.048,.063,.008),24,16);p.append(eye)
        es=surface(eye);pupil=ellipsoid('Eye / bright dark pupil '+str(side),INK,(x-side*.012,y-.004,es(y-.004,x-side*.012)+.002),(.020,.034,.006),20,14);p.append(pupil)
        ps=surface(pupil);p.append(ellipsoid('Eye / white glint '+str(side),WHITE,(x-side*.012-.004,y+.008,ps(y+.008,x-side*.012-.004)+.001),(.005,.010,.002),12,8))
    beak=loft('Beak / broad rounded upper bill',GOLD,curved_profiles(((.788,.100,.035,.238),(.813,.110,.066,.230),(.837,.070,.053,.219),(.85,.013,.016,.202)),4),28);p.append(beak)
    p.append(ellipsoid('Beak / lower gold bill',(.89,.75,.28),(0,.780,.229),(.087,.020,.050),28,14))
    ss=surface(beak);points=[(x,.789+.006*abs(x/.075),ss(.789+.006*abs(x/.075),x)+.001) for x in (-.075,-.05,-.025,0,.025,.05,.075)]
    p.append(tube('Beak / curved mouth seam',INK,points,[.002]*len(points),10))
    p.append(ellipsoid('Bib / single lower feather button',WHITE,(0,.375,bs(.375,0)+.005),(.019,.032,.013),20,12))
    sack=translate(loft('Tail sack / sagging rounded white bag',WHITE,curved_profiles(((.10,.024,.028,-.285),(.13,.13,.125,-.305),(.25,.205,.18,-.315),(.40,.19,.17,-.295),(.53,.12,.105,-.25),(.58,.047,.05,-.20)),4),32),.13,0,0);p.append(sack)
    p.append(tube('Tail sack / curved held neck',WHITE,[(.13,.545,-.205),(.24,.57,-.17),(.29,.535,-.065),(.29,.49,.035),(.27,.46,.08)],[.066,.057,.045,.037,.026],24))
    # Folds are fitted to the rear of the sack, visible behind the right wing.
    rear=dict(sack);rear['positions']=[(-v if i%3!=1 else v) for i,v in enumerate(sack['positions'])];rear_skin=surface(rear)
    for j,x in enumerate((-.23,-.13,-.035)):
        pts=[]
        for y in (.47,.40,.30,.22):pts.append((-x,y,-(rear_skin(y,x)+.002)))
        p.append(tube('Tail sack / fitted rear fold '+str(j),SHADE,pts,[.0025,.003,.0025,.001],8))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'delibird.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
