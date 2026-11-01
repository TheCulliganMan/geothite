#!/usr/bin/env python3
"""Original evolved seed saurs: supported quadrupeds, folded bud and spotted bloom.

Authoring candidates only. This module does not install runtime assets.
"""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,translate,crease_normals,fitted_adaptive_contour
from goldfish_sculpt import surface
from bovine_sculpt import oval,curved_tube
from bivalve_sculpt import top_view
from skin_glb import COORDINATES

SPECIES=('ivysaur','venusaur')
HEIGHTS={s:1.20 for s in SPECIES}
MATURE_BODY_WIDTH=1.15
IVY=(.36,.66,.64);VENUS=(.35,.59,.51);SPOT=(.13,.35,.29)
LEAF=(.28,.53,.25);VEIN=(.39,.63,.31);ROSE=(.85,.37,.46)
CORAL=(.89,.43,.48);CREAM=(.97,.91,.72);BARK=(.45,.32,.21)
WHITE=(.98,.97,.91);RED=(.76,.20,.26);INK=(.10,.14,.14);PINK=(.78,.43,.49);GOLD=(.91,.71,.28)

def paint(name,color,xy,parent,th=.003,*,error=.00065):
    return crease_normals(fitted_adaptive_contour(name,color,xy,parent,th,
        surface_at=surface(parent),initial_subdivisions=1,max_surface_error=error),50)

def blade(name,color,angle,length,width,base_y,*,petal=False,columns=9):
    """Closed folded blade, with a nonzero root and tip and rounded flower margins."""
    rows=19;vertices=[]
    def point(t,u,lower=False):
        edge=.006+width*math.sin(math.pi*t)**(.62 if petal else .76)
        if not petal:edge*=1.+.14*math.cos(8*math.pi*t)
        lateral=edge*u;along=.035+length*t
        y=base_y+(.065 if petal else .075)*math.sin(math.pi*t)-(.045 if petal else .10)*t*t
        y+=(.025 if petal else .033)*(1-u*u)*math.sin(math.pi*t)
        y-=.012 if lower else 0.
        return (math.cos(angle)*lateral+math.sin(angle)*along,y,
                -.16-math.sin(angle)*lateral+math.cos(angle)*along)
    for lower in (False,True):
        for j in range(rows):
            for i in range(columns):vertices.append(point(j/(rows-1),2*i/(columns-1)-1,lower))
    count=rows*columns;faces=[]
    for j in range(rows-1):
        for i in range(columns-1):
            a=j*columns+i;b=a+1
            faces.append((a,a+columns,b+columns,b))
    faces += [tuple(count+i for i in reversed(f)) for f in list(faces)]
    border=list(range(columns))+[j*columns+columns-1 for j in range(1,rows)]
    border+=list(range(count-2,count-columns-1,-1))+[j*columns for j in reversed(range(1,rows-1))]
    for a,b in zip(border,border[1:]+border[:1]):faces.append((a,b,b+count,a+count))
    return crease_normals(part(name,color,vertices,faces,True),55),point

def bud():
    rows=curved_profiles(((.43,.09,.10,-.16),(.52,.19,.19,-.16),(.67,.235,.215,-.17),
        (.81,.215,.195,-.18),(.94,.14,.135,-.20),(1.00,.018,.02,-.22)),3)
    segments=36;vertices=[]
    for y,w,d,z in rows:
        for j in range(segments):
            a=math.tau*j/segments;fold=1.+.075*math.cos(5*a)
            vertices.append((w*math.sin(a)*fold,y,z+d*math.cos(a)*fold))
    bottom=len(vertices);vertices.append((0.,rows[0][0],rows[0][3]))
    top=len(vertices);vertices.append((0.,rows[-1][0],rows[-1][3]))
    faces=[(bottom,(j+1)%segments,j) for j in range(segments)]
    offset=(len(rows)-1)*segments
    faces.extend((top,offset+j,offset+(j+1)%segments) for j in range(segments))
    for k in range(len(rows)-1):
        for j in range(segments):
            a=k*segments+j;b=k*segments+(j+1)%segments;faces.append((a,b,b+segments,a+segments))
    return part('Bud / closed five-fold pointed rose blossom',ROSE,vertices,faces,True)

def sculpture(species,*,with_transform=False):
    if species not in SPECIES:raise ValueError('unknown evolved seed saur')
    mature=species=='venusaur';color=VENUS if mature else IVY
    body=ellipsoid('Body / heavy low mature toad torso' if mature else 'Body / growing long seed saur torso',
        color,(0.,.30 if mature else .27,-.19 if mature else -.095),(.46,.225,.46) if mature else (.285,.18,.335),32,18)
    head=loft('Head / broad low mature cheeked skull' if mature else 'Head / alert broad growing saur skull',color,
        curved_profiles(((.18,.08,.08,.36),(.23,.27 if mature else .21,.18,.36),
        (.33,.43 if mature else .31,.235 if mature else .20,.34),
        (.45,.43 if mature else .31,.23 if mature else .20,.31),
        (.55,.335 if mature else .245,.16,.26),(.60,.17,.09,.23),(.615,.035,.025,.22)),3),32)
    parts=[body,head]
    for side in (-1,1):
        ear=loft('Ear / short pointed mature hood '+str(side) if mature else 'Ear / tall alert saur hood '+str(side),color,
            curved_profiles(((.46,.085,.067,.245),(.55,.072,.055,.22),
            (.63 if mature else .68,.035,.03,.195),(.65 if mature else .72,.003,.003,.19)),3),18)
        ear=translate(ear,side*(.29 if mature else .235),0.,0.);parts.append(ear)
        if mature:
            x=side*.29
            parts.append(paint('Ear inset / fitted muted pink hood '+str(side),PINK,
                [(x-.025,.54),(x+.025,.54),(x+.014,.59),(x,.611),(x-.014,.59)],ear,.002))
        x=side*(.25 if mature else .17)
        outline=[(x+side*dx,y) for dx,y in ((-.069,.36),(.067,.37),(.079,.402 if mature else .415),(.046,.427 if mature else .505),(-.043,.445 if mature else .52),(-.073,.411 if mature else .42))]
        eye=paint('Eye / fitted ivory hooded lens '+str(side),WHITE,outline,head);parts.append(eye)
        iris=paint('Iris / fitted red vertical saur gaze '+str(side),RED,oval(x,.400 if mature else .438,.022,.026 if mature else .05,16),eye,.002);parts.append(iris)
        parts.append(paint('Pupil / fitted dark saur slit '+str(side),INK,oval(x,.400 if mature else .438,.006,.018 if mature else .035,12),iris,.0015))
        parts.append(paint('Eye glint / fitted ivory highlight '+str(side),WHITE,oval(x-.005,.411 if mature else .461,.005,.007 if mature else .009,12),iris,.002))
        if mature:
            parts.append(paint('Brow / fleshy slanted mature eye hood '+str(side),color,
                [(x+side*dx,y) for dx,y in ((-.083,.447),(.075,.425),(.087,.445),(.055,.474),(-.053,.49),(-.085,.465))],head,.007,error=.0012))
        parts.append(paint('Nostril / fitted short saur slit '+str(side),INK,oval(side*.075,.318,.009,.004,12),head,.002))
        for front in (True,False):
            z=.22 if front else -.43 if mature else -.34
            x=side*((.335 if front else .32) if mature else (.235 if front else .225))
            w=.115 if mature else .078
            leg=translate(loft(('Foreleg / ' if front else 'Hindleg / ')+str(side),color,
                curved_profiles(((.035,w*.60,.075,z+.02),(.11,w,.105,z),(.24,w*1.08,.11,z-.015),
                (.36,w*.88,.095,z-.03),(.42,.035,.04,z-.03)),3),20),x,0.,0.)
            parts.append(leg)
            foot=ellipsoid(('Forefoot / ' if front else 'Hindfoot / ')+str(side),color,
                (x,.04,z+.045),(.12,.04,.135) if mature else (.09,.04,.105),24,12);parts.append(foot)
            for j,dx in enumerate((-.063,0.,.063) if mature else (-.047,0.,.047)):
                parts.append(crease_normals(curved_tube(('Foreclaw / ' if front else 'Hindclaw / ')+str(side)+' '+str(j),WHITE,
                    [(x+dx,.042,z+(.135 if mature else .105)),(x+dx,.026,z+(.18 if mature else .15)),
                     (x+dx,.019,z+(.195 if mature else .16))],[.022 if mature else .017,.012,.002],12),60))
            parts.append(paint(('Foreleg spot / ' if front else 'Hindleg spot / ')+str(side),SPOT,
                [(x-.036,.17),(x-.022,.245),(x+.034,.265),(x+.049,.204),(x+.005,.145)],leg,.003))
    width=.255 if mature else .205
    mouth=paint('Mouth / fitted broad dark saur smile',INK,[(-width,.286),(-width*.6,.265),(0.,.253),
        (width*.6,.265),(width,.286),(width*.70,.229),(0.,.213),(-width*.70,.229)],head,.004);parts.append(mouth)
    parts.append(paint('Tongue / fitted pale mouth floor',PINK,[(-width*.6,.242),(0.,.24),(width*.6,.242),
        (width*.45,.226),(0.,.221),(-width*.45,.226)],mouth,.0015))
    hs=surface(head)
    for side in (-1,1):
        x=side*width*.77;y=.275;z=hs(y,x)
        parts.append(crease_normals(curved_tube('Fang / hanging ivory saur tooth '+str(side),WHITE,
            [(x,y+.003,z+.006),(x,y-.015,z+.009),(x+side*.003,y-.035,z+.009)],[.013,.009,.0015],12),60))
    if mature:
        for j,(x,y) in enumerate(((-.085,.55),(0.,.58),(.085,.555))):
            parts.append(ellipsoid('Head wart / rooted rounded mature skin bump '+str(j),color,
                (x,y,hs(y,x)+.003),(.018,.018,.010),8,4))
        for side in (-1,1):
            for row,y in enumerate((.365,.43)):
                radius=math.sqrt(1.-((y-.30)/.225)**2)
                for j,angle in enumerate((math.pi*.58,math.pi*.78)):
                    center=(side*.46*radius*math.sin(angle),y,-.19+.46*radius*math.cos(angle))
                    parts.append(ellipsoid('Body wart / rooted mature flank bump '+str(side)+' '+str(row)+' '+str(j),color,
                        center,(.021,.019,.014),8,4))
    else:
        for j,xy in enumerate(([(-.095,.573),(.068,.581),(.052,.544),(-.027,.514)],
                               [(-.04,.485),(-.019,.491),(-.025,.465),(-.043,.461)],
                               [(.045,.373),(.102,.385),(.099,.345),(.065,.339)])):
            parts.append(paint('Head spot / fitted angular saur marking '+str(j),SPOT,xy,head))
    for j in range(6):
        angle=math.tau*j/6+.22
        leaf,path=blade('Leaf / folded serrated back rosette '+str(j),LEAF,angle,
            .61 if mature else .43,.235 if mature else .175,.59 if mature else .50,columns=7 if mature else 9);parts.append(leaf)
        points=[path(.08+k*.10,0.) for k in range(9)]
        points=[(x,y+.003,z) for x,y,z in points]
        parts.append(crease_normals(curved_tube('Leaf midrib / rooted rosette ridge '+str(j),VEIN,points,[.004]*len(points),8),60))
    if not mature:parts.append(bud())
    else:
        parts.append(loft('Trunk / rooted woody flower stalk',BARK,
            curved_profiles(((.43,.15,.14,-.16),(.58,.12,.115,-.16),(.77,.095,.09,-.16),(.99,.12,.115,-.16)),3),24))
        for j in range(5):
            angle=math.tau*j/5+.16
            petal,path=blade('Petal / broad rounded fivefold flower '+str(j),CORAL,angle,.61,.265,.99,petal=True,columns=5);parts.append(petal)
            parent=top_view(petal)
            for k,(t,u) in enumerate(((.36,-.45),(.59,.32),(.73,-.26))):
                x,y,z=path(t,u)
                patch=paint('Petal spot / fitted cream marking '+str(j)+' '+str(k),CREAM,oval(x,-z,.027,.032,12),parent,.002)
                parts.append(top_view(patch,True))
        parts.append(ellipsoid('Flower heart / raised golden seed crown',GOLD,(0.,1.057,-.16),(.155,.075,.145),28,14))
        for j in range(9):
            a=math.tau*j/9
            parts.append(ellipsoid('Stamen / rounded golden crown tip '+str(j),CREAM,
                (.09*math.sin(a),1.108,-.16+.085*math.cos(a)),(.018,.026,.018),10,6))
    if mature:
        botanical=('Leaf /','Leaf midrib /','Trunk /','Petal /','Petal spot /','Flower heart /','Stamen /')
        for q in parts:
            if q['part'].startswith(botanical):continue
            q['positions']=[v*MATURE_BODY_WIDTH if i%3==0 else v for i,v in enumerate(q['positions'])]
            normals=[]
            for i in range(0,len(q['normals']),3):
                x,y,z=q['normals'][i:i+3];x/=MATURE_BODY_WIDTH;length=math.sqrt(x*x+y*y+z*z);normals.extend((x/length,y/length,z/length))
            q['normals']=normals
    lo=min(v for q in parts for v in q['positions'][1::3]);hi=max(v for q in parts for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
    for q in parts:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    model=dict(name=species,version=1,coordinate_system=COORDINATES,primitives=parts)
    return (model,(lo,scale)) if with_transform else model

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--only',choices=SPECIES,nargs='+',default=SPECIES);args=p.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in args.only:
        m=sculpture(s);(args.output/(s+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')))
        print(s,len(m['primitives']),sum(len(q['positions'])//3 for q in m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']),flush=True)
if __name__=='__main__':main()
