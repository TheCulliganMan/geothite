#!/usr/bin/env python3
"""Original rounded Chinchou and Lanturn with fitted eyes and soft light stalks."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,translate,fitted_patch,tube
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES
SPECIES=('chinchou','lanturn');HEIGHT=.95
BLUE=(.39,.47,.70);PALE=(.78,.83,.92);STALK=(.29,.37,.59);YELLOW=(.98,.87,.42);LIGHT=(1.,.96,.78);INK=(.16,.17,.22);RED=(.82,.28,.25)

def dense_mask(outline,skin):
    # Twelve radial rings keep the broad mask's chords above the actual cheek.
    n=len(outline);cx=sum(x for x,y in outline)/n;cy=sum(y for x,y in outline)/n
    xy=[(cx,cy)]+[(cx+(x-cx)*r/12,cy+(y-cy)*r/12) for r in range(1,13) for x,y in outline]
    count=len(xy);v=[(x,y,skin(y,x)+offset) for offset in (.008,-.0028) for x,y in xy]
    f=[(0,1+i,1+(i+1)%n) for i in range(n)]
    for ring in range(11):
        for i in range(n):
            a=1+ring*n+i;b=1+ring*n+(i+1)%n;f.append((a,a+n,b+n,b))
    f += [tuple(i+count for i in reversed(face)) for face in list(f)]
    last=1+11*n;f += [(last+(i+1)%n,last+i,last+i+count,last+(i+1)%n+count) for i in range(n)]
    return part('Mask / broad golden eye saddle',YELLOW,v,f)

def fin(name,outline,z,color=PALE,thickness=.025):
    return membrane(name,color,outline,1,root_z=z+.06*abs(outline[0][0])-.025*(outline[0][1]-.7)**2,thickness=thickness)

def side_fins(p,adult):
    for s in (-1,1):
        y=.40 if adult else .35;x=.22 if adult else .24
        outline=contour([((s*x,y),(s*.35,y-.005),(s*.53,y+.10),(s*.49,y+.15)),((s*.49,y+.15),(s*.39,y+.19),(s*.30,y+.085),(s*x,y))],9)
        f=fin('Fin / broad rounded pectoral '+str(s),outline,.03);p.append(f);fs=surface(f)
        pts=[(s*a,y+b,fs(y+b,s*a)+.001) for a,b in ((.28,.018),(.35,.056),(.43,.097))]
        p.append(tube('Fin fold / fitted pectoral crease '+str(s),STALK,pts,[.002,.002,.001],10))
        p.append(ellipsoid('Fin / soft lower pelvic paddle '+str(s),BLUE,(s*.145,.153,.02),(.065,.095,.065),24,16))

def chinchou():
    body=ellipsoid('Body / broad oval blue lantern fish',BLUE,(0,.37,0),(.32,.215,.255),40,26);p=[body];bs=surface(body)
    side_fins(p,False)
    for s in (-1,1):
        x=s*.135;y=.394
        eye=fitted_patch('Eye / large golden oval '+str(s),YELLOW,oval(x,y,.076,.098,32),bs,.013);p.append(eye);es=surface(eye)
        # Two rounded fitted bars form the characteristic cross, without a white iris.
        for label,w,h in (('vertical',.012,.071),('horizontal',.054,.012)):
            p.append(fitted_patch('Pupil / black cross '+label+' '+str(s),INK,oval(x,y,w,h,28),es,.004))
        path=[(s*.06,.556,-.014),(s*.15,.81,.015),(s*.31,.94,.030),(s*.46,.93,.064),(s*.58,.81,.105),(s*.62,.64,.145)]
        p.append(curved_tube('Stalk / arched flexible antenna '+str(s),STALK,path,[.011,.009,.008,.007,.007,.008],14))
        bulb=translate(loft('Lure / hanging golden teardrop '+str(s),YELLOW,curved_profiles([(.366,.008,.008,.15),(.395,.062,.050,.15),(.45,.083,.067,.15),(.51,.062,.052,.15),(.59,.022,.021,.15),(.651,.008,.008,.15)],4),32),s*.62,0,0);p.append(bulb)
        p.append(fitted_patch('Lure / pale light center '+str(s),LIGHT,oval(s*.62,.439,.052,.052,28),surface(bulb),.004))
    mouth=fitted_patch('Mouth / tiny dark oval',INK,oval(0,.291,.010,.021,20),bs,.003);p.append(mouth)
    p.append(fitted_patch('Mouth / tiny pink opening',(.85,.52,.52),oval(0,.291,.006,.015,18),surface(mouth),.002))
    return p

def lanturn():
    body=ellipsoid('Body / plump blue angler fish',BLUE,(0,.39,0),(.30,.285,.405),40,28);p=[body];bs=surface(body)
    mask=dense_mask(oval(0,.505,.242,.122,64),bs);p.append(mask);ms=surface(mask)
    for s in (-1,1):
        x=s*.114;y=.526
        eye=fitted_patch('Eye / dark fitted almond socket '+str(s),INK,oval(x,y,.047,.062,28),ms,.004);p.append(eye)
        white=fitted_patch('Eye / ivory almond white '+str(s),LIGHT,oval(x,y,.041,.056,28),surface(eye),.003);p.append(white)
        iris=fitted_patch('Eye / deep red iris '+str(s),RED,oval(x+s*.009,y,.024,.046,26),surface(white),.003);p.append(iris)
        pupil=fitted_patch('Pupil / tall black pupil '+str(s),INK,oval(x+s*.011,y,.010,.031,22),surface(iris),.002);p.append(pupil)
        p.append(fitted_patch('Eye / bright small glint '+str(s),LIGHT,oval(x+s*.009-.005,y+.018,.007,.010,16),surface(iris),.005))
    mouth=fitted_patch('Mouth / soft open smile',INK,oval(0,.344,.075,.052,32),bs,.006);p.append(mouth)
    p.append(fitted_patch('Mouth / fitted rosy tongue',(.80,.47,.49),oval(.018,.325,.038,.023,24),surface(mouth),.003))
    side_fins(p,True)
    p.append(curved_tube('Tail / tapered blue caudal peduncle',BLUE,[(0,.39,-.29),(0,.39,-.46),(0,.40,-.56)],[.13,.065,.04],24))
    for label,sgn in (('upper',1),('lower',-1)):
        outline=contour([((0,.40),(-.10,.40+sgn*.10),(-.17,.40+sgn*.27),(-.105,.40+sgn*.29)),((-.105,.40+sgn*.29),(.014,.40+sgn*.22),(.077,.40+sgn*.08),(0,.40))],10)
        tail=fin('Tail fin / rounded '+label+' lobe',outline,-.53,BLUE if sgn==1 else YELLOW,.033);p.append(tail)
    p.append(curved_tube('Stalk / single arched main beacon',STALK,[(0,.637,.015),(0,.81,.065),(-.035,.95,.125),(-.16,1.00,.175),(-.29,.955,.23),(-.34,.855,.28)],[.018,.015,.013,.012,.011,.009],18))
    p.append(curved_tube('Stalk / short upper branch',STALK,[(0,.795,.065),(.085,.88,.055),(.18,.92,.06),(.235,.89,.09)],[.012,.010,.009,.008],16))
    for label,center,r in (('large',(-.34,.81,.28),.070),('small',(.245,.88,.09),.043)):
        bulb=ellipsoid('Lure / golden '+label+' orb',YELLOW,center,(r,r,r*.88),28,20);p.append(bulb)
        p.append(fitted_patch('Lure / pale '+label+' light center',LIGHT,oval(center[0],center[1],r*.78,r*.78,28),surface(bulb),.004))
    return p

def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported angler fish')
    p=chinchou() if species=='chinchou' else lanturn();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
