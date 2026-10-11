#!/usr/bin/env python3
"""Original Dunsparce with a continuous rounded body and grooved drill tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from spider_sculpt import upper_skin
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES
SPECIES=('dunsparce',);HEIGHT=.85
YELLOW=(.95,.85,.40);BLUE=(.35,.65,.69);IVORY=(.97,.97,.93);EYE=(.97,.91,.69);INK=(.22,.28,.26);GROOVE=(.47,.40,.18)

def trunk():
    rows=curved_profiles(((-.58,.027,.033,.205),(-.43,.12,.10,.23),(-.22,.19,.15,.25),(0,.24,.18,.26),(.21,.32,.22,.25),(.43,.35,.22,.25),(.61,.27,.17,.25),(.70,.045,.055,.25)),4)
    n=40
    # Two closed material regions share the same side seam and interior chord.
    def region(name,color,angles):
        v=[];k=len(angles)
        for z,w,h,y in rows:
            v.extend((w*math.sin(a*math.tau/n),y+h*math.cos(a*math.tau/n),z) for a in angles)
        f=[tuple(reversed(range(k))),tuple((len(rows)-1)*k+i for i in range(k))]
        for j in range(len(rows)-1):
            for i in range(k):a=j*k+i;b=j*k+(i+1)%k;f.append((a,b,b+k,a+k))
        return crease_normals(part(name,color,v,f),40.)
    return [region('Body / continuous rounded golden head and tapering trunk',YELLOW,list(range(26,41))+list(range(1,15))),region('Belly / continuous blue underside',BLUE,list(range(14,27)))]

def roof_band(name,z,width,skin):
    # Sample a rectangular surface grid; broad radial patch fans crossed the roof.
    nx=24;nz=6;v=[]
    for offset in (.004,-.0015):
        for j in range(nz+1):
            zz=z-.05+.10*j/nz;w=width+.015*j/nz
            for i in range(nx+1):
                x=w*(2*i/nx-1);v.append((x,skin(zz,x)+offset,zz))
    count=(nx+1)*(nz+1);f=[]
    for j in range(nz):
        for i in range(nx):
            a=j*(nx+1)+i;b=a+1;c=b+nx+1;d=a+nx+1;f.append((a,b,c,d));f.append((d+count,c+count,b+count,a+count))
    rim=list(range(nx+1))+[j*(nx+1)+nx for j in range(1,nz+1)]+[nz*(nx+1)+i for i in reversed(range(nx))]+[j*(nx+1) for j in reversed(range(1,nz))]
    for a,b in zip(rim,rim[1:]+rim[:1]):f.append((b,a,a+count,b+count))
    return crease_normals(part(name,BLUE,v,f),40.)

def sculpture(species='dunsparce'):
    if species not in SPECIES:raise ValueError('unsupported land snake')
    p=trunk();head=p[0];hs=surface(head);top=upper_skin(head)
    for side in (-1,1):
        x=side*.173;y=.29
        blue=fitted_patch('Eye / broad fitted blue oval ring '+str(side),BLUE,oval(x,y,.099,.102,32),hs,.014);p.append(blue)
        inside=fitted_patch('Eye / pale closed eyelid '+str(side),EYE,oval(x,y,.076,.079,32),surface(blue),.008);p.append(inside);ss=surface(inside)
        pts=[(x+dx,y-dx*.42,ss(y-dx*.42,x+dx)+.0015) for dx in (-.067,-.045,-.02,.02,.045,.067)]
        p.append(tube('Eye / fitted diagonal closed-eye crease '+str(side),INK,pts,[.002]*6,10))
        p.append(curved_tube('Jaw / pointed blue lower barb '+str(side),BLUE,[(side*.145,.16,.608),(side*.154,.105,.659),(side*.168,.047,.708)],[.047,.027,.0015],20))
        outline=contour([((side*.15,.325),(side*.18,.42),(side*.34,.56),(side*.45,.571)),((side*.45,.571),(side*.51,.562),(side*.425,.473),(side*.345,.415)),((side*.345,.415),(side*.285,.357),(side*.21,.318),(side*.15,.325))],9)
        wing=membrane('Wing / small curved ivory feather '+str(side),IVORY,outline,1,root_z=-.144,thickness=.023);p.append(wing);ws=surface(wing)
        pts=[]
        for j in range(9):
            t=j/8;xx=side*(.23+.185*t);yy=.365+.16*t;pts.append((xx,yy,ws(yy,xx)+.001))
        p.append(tube('Wing crease / fitted feather fold '+str(side),(.69,.71,.68),pts,[.0015]*9,8))
    for j,(z,w) in enumerate(((-.26,.151),(-.025,.21))):
        p.append(roof_band('Back band / fitted curved blue ribbon '+str(j),z,w,top))
    stem=[(0,.235,-.515),(0,.27,-.627),(0,.385,-.717),(0,.435,-.749)]
    p.append(curved_tube('Tail / raised tapering golden stem',YELLOW,stem,[.069,.057,.052,.064],22))
    drill=loft('Drill / tapered golden burrowing cone',YELLOW,curved_profiles(((.382,.030,.031,-.733),(.43,.115,.10,-.758),(.49,.131,.114,-.773),(.60,.108,.094,-.795),(.72,.064,.056,-.818),(.83,.022,.020,-.839),(.875,.0017,.0017,-.850)),4),40);p.append(drill)
    # Follow the authored cone profiles with a continuous three-turn spiral.
    rows=curved_profiles(((.382,.030,.031,-.733),(.43,.115,.10,-.758),(.49,.131,.114,-.773),(.60,.108,.094,-.795),(.72,.064,.056,-.818),(.83,.022,.020,-.839),(.875,.0017,.0017,-.850)),4)
    points=[]
    for j in range(145):
        t=j/144;y=.475+.359*t;a=math.tau*(3*t+.2)
        i=next(i for i in range(len(rows)-1) if rows[i][0]<=y<=rows[i+1][0]);u=(y-rows[i][0])/(rows[i+1][0]-rows[i][0]);w,d,z=[rows[i][k]*(1-u)+rows[i+1][k]*u for k in (1,2,3)]
        points.append(((w+.001)*math.sin(a),y,z+(d+.001)*math.cos(a)))
    p.append(crease_normals(tube('Drill groove / continuous three-turn spiral',GROOVE,points,[.0025*(1-.50*j/144) for j in range(145)],10),35.))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'dunsparce.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
