#!/usr/bin/env python3
"""Original Butterfree/Beedrill anatomy with curved closed veined wing solids."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,tube,loft,curved_profiles,crease_normals
from abra_sculpt import polygon_caps
from skin_glb import COORDINATES

SPECIES=('butterfree','beedrill');HEIGHT=1.04
PURPLE=(.48,.38,.65);GOLD=(.96,.75,.20);INK=(.075,.082,.12)
WHITE=(.94,.95,.89);PALE=(.76,.85,.88);BLUE=(.38,.68,.79)
RUBY=(.82,.18,.29);GLINT=(1.,.78,.71)


def contour(curves,steps=7):
    points=[]
    for a,b,c,d in curves:
        for k in range(steps):
            t=k/steps;points.append(tuple((1-t)**3*a[i]+3*(1-t)**2*t*b[i]+3*(1-t)*t*t*c[i]+t**3*d[i] for i in range(2)))
    return points


def membrane(name,color,outline,side,root_z=-.09,inset=1.,offset=0.,thickness=.007):
    center=tuple(sum(p[k] for p in outline)/len(outline) for k in range(2))
    xy=[(side*(center[0]+(x-center[0])*inset),center[1]+(y-center[1])*inset) for x,y in outline]
    def depth(x,y):return root_z-.06*abs(x)+.025*(y-.70)**2
    v=[(x,y,depth(x,y)+offset+thickness) for x,y in xy]+[(x,y,depth(x,y)+offset-thickness) for x,y in xy]
    caps=polygon_caps(v[:len(xy)]);n=len(xy);ids=list(range(n))
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(xy,xy[1:]+xy[:1]))<0:ids.reverse()
    faces=caps+[tuple(i+n for i in reversed(f)) for f in caps]+[(a,a+n,b+n,b) for a,b in zip(ids,ids[1:]+ids[:1])]
    return crease_normals(part(name,color,v,faces))


def wings(bee):
    p=[]
    if bee:
        upper=contour([((.07,.65),(.17,.83),(.33,1.04),(.57,1.10)),
                       ((.57,1.10),(.68,1.11),(.63,.96),(.55,.84)),
                       ((.55,.84),(.39,.70),(.20,.64),(.07,.65))],10)
        lower=contour([((.09,.61),(.29,.68),(.45,.77),(.52,.80)),
                       ((.52,.80),(.63,.82),(.55,.63),(.41,.55)),
                       ((.41,.55),(.28,.50),(.17,.55),(.09,.61))],9)
    else:
        upper=contour([((.08,.57),(.14,.80),(.26,1.10),(.48,1.14)),
                       ((.48,1.14),(.68,1.20),(.85,1.06),(.82,.96)),
                       ((.82,.96),(.81,.85),(.78,.77),(.71,.71)),
                       ((.71,.71),(.73,.62),(.58,.57),(.49,.57)),
                       ((.49,.57),(.31,.54),(.15,.56),(.08,.57))],8)
        lower=contour([((.08,.55),(.26,.62),(.49,.65),(.62,.57)),
                       ((.62,.57),(.72,.52),(.66,.35),(.59,.29)),
                       ((.59,.29),(.50,.18),(.39,.16),(.31,.24)),
                       ((.31,.24),(.16,.31),(.10,.43),(.08,.55))],8)
    for side in (-1,1):
        for which,outline in (('upper',upper),('lower',lower)):
            root=-.115 if which=='lower' else -.09
            p.append(membrane('Wing / '+which+' curved perimeter '+str(side),(.48,.57,.61) if bee else INK,outline,side,root))
            p.append(membrane('Wing / '+which+' inset membrane '+str(side),PALE if bee else WHITE,outline,side,root,.94 if bee else .91,0.,.010))
        paths=([[(.12,.67),(.29,.83),(.46,.99),(.57,1.065)],[(.13,.64),(.31,.71),(.47,.80)],[(.16,.60),(.29,.60),(.43,.66)]] if bee else
               [[(.12,.62),(.27,.83),(.46,1.09)],[(.20,.75),(.39,.79),(.61,.82),(.77,.97)],
                [(.25,.83),(.44,.96),(.62,1.07)],[(.12,.58),(.31,.67),(.53,.67),(.68,.72)],
                [(.14,.49),(.33,.47),(.59,.51)],[(.33,.47),(.45,.38),(.51,.28)],[(.15,.48),(.26,.33),(.36,.25)]])
        for j,path in enumerate(paths):
            root=-.115 if j>= (2 if bee else 4) else -.09
            for face in (-1,1):
                points=[(side*x,y,root-.06*x+.025*(y-.70)**2+face*.013) for x,y in path]
                p.append(tube('Wing vein / '+str(side)+' '+str(j)+' '+str(face),(.51,.62,.65) if bee else INK,points,[.0025 if bee else .0045]*len(points),8))
    return p


def sculpture(species, *, normalize=True):
    if species not in SPECIES:raise ValueError('unsupported wing insect')
    bee=species=='beedrill';color=GOLD if bee else PURPLE;p=wings(bee)
    if bee:
        profiles=curved_profiles(((.11,.020,.028,-.04),(.17,.072,.076,-.06),(.26,.137,.123,-.073),(.34,.143,.119,-.072),(.40,.108,.093,-.063),(.445,.044,.048,-.031)),3)
        p.append(loft('Abdomen / tapered wasp gaster',GOLD,profiles,24))
        def radius(y):
            for a,b in zip(profiles,profiles[1:]):
                if a[0]<=y<=b[0]:
                    t=(y-a[0])/(b[0]-a[0]);return tuple(a[k]+t*(b[k]-a[k]) for k in range(1,4))
            raise ValueError('stripe outside abdomen')
        for j,(lo,hi) in enumerate(((.19,.228),(.313,.358))):
            rows=[]
            for y in (lo,(lo+hi)/2,hi):
                x,z,center=radius(y);rows.append((y,x+.0015,z+.0015,center))
            p.append(crease_normals(loft('Abdomen stripe / fitted charcoal band '+str(j),INK,rows,24)))
        p.append(tube('Waist / narrow articulated petiole',GOLD,[(0,.425,-.025),(0,.49,0),(0,.57,0)],[.04,.035,.06],16))
        p.append(ellipsoid('Thorax / oval wasp breast',GOLD,(0,.60,0),(.119,.127,.113),20,12))
        p.append(tube('Stinger / long tapered rear lance',WHITE,[(0,.14,-.046),(0,.065,-.086),(0,.002,-.118)],[.040,.021,.0015],16))
    else:
        p.append(ellipsoid('Abdomen / tapered violet belly',PURPLE,(0,.335,-.018),(.121,.226,.118),24,14))
        p.append(ellipsoid('Thorax / compact upper body',PURPLE,(0,.584,0),(.104,.137,.10),20,12))
        # Restrained shallow transverse abdominal folds.
        for y in (.22,.30,.38):
            z=-.018+.118*math.sqrt(1-((y-.335)/.226)**2)
            p.append(tube('Abdomen crease / '+str(y),(.36,.28,.51),[(-.064,y+.006,z-.016),(0,y,z+.002),(.064,y+.006,z-.016)],[.0024]*3,8))
    head_y=.816;head_z=.055
    p.append(ellipsoid('Head / broad compound-eye cranium',color,(0,head_y,head_z),(.173,.153,.139),24,14))
    for side in (-1,1):
        # Broad compound eye lenses embrace the head without a white eyeball.
        p.append(ellipsoid('Eye / dark fitted socket '+str(side),INK,(side*.097,.83,.166),(.075,.102,.035),20,12))
        p.append(ellipsoid('Eye / ruby compound lens '+str(side),RUBY,(side*.097,.83,.185),(.063,.090,.026),20,12))
        p.append(ellipsoid('Eye / restrained reflection '+str(side),GLINT,(side*.097-.018,.864,.210),(.014,.021,.003),12,8))
        antenna=[(side*.064,.93,.019),(side*.092,1.015,-.006),(side*.136,1.099,-.013),(side*.17,1.11,.006)]
        p.append(tube('Antenna / curved dark feeler '+str(side),INK,antenna,[.012,.011,.009,.006],12))
        if bee:
            arm=[(side*.094,.624,.029),(side*.184,.567,.071),(side*.258,.49,.119)]
            p.append(tube('Arm / angled upper limb '+str(side),GOLD,arm,[.037,.031,.028],14))
            drill=[(side*.238,.52,.11),(side*.297,.433,.146),(side*.378,.271,.22),(side*.44,.144,.268)]
            p.append(tube('Drill / long tapered forearm lance '+str(side),WHITE,drill,[.043,.076,.040,.0015],20))
            p.append(tube('Leg / angular rear tarsus '+str(side),GOLD,[(side*.09,.46,-.015),(side*.227,.40,.018),(side*.254,.282,.048),(side*.298,.25,.099)],[.023,.024,.016,.008],12))
        else:
            p.append(tube('Arm / slender reaching limb '+str(side),PURPLE,[(side*.074,.59,.039),(side*.144,.54,.069),(side*.15,.493,.091)],[.022,.019,.015],12))
            p.append(ellipsoid('Hand / blue rounded mitten '+str(side),BLUE,(side*.16,.49,.098),(.047,.057,.040),16,10))
            p.append(tube('Leg / short violet lower limb '+str(side),PURPLE,[(side*.059,.211,.018),(side*.071,.119,.031),(side*.083,.068,.063)],[.025,.023,.021],12))
            p.append(ellipsoid('Foot / long blue pad '+str(side),BLUE,(side*.084,.041,.112),(.058,.040,.112),20,10))
    if not bee:
        p.append(ellipsoid('Muzzle / short blue proboscis',BLUE,(0,.726,.169),(.032,.038,.031),16,10))
    else:
        p.append(tube('Mouth / quiet facial seam',INK,[(-.023,.75,.168),(0,.742,.172),(.023,.75,.168)],[.002]*3,8))
    if not normalize:return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    from wing_insect_glb import export_insect
    for s in SPECIES:
        (args.output/(s+'.glb')).write_bytes(export_insect(s))
        (args.output/(s+'.mesh.json')).unlink(missing_ok=True)

if __name__=='__main__':main()
