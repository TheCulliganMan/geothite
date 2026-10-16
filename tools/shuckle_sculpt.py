#!/usr/bin/env python3
"""Original soft-bodied Shuckle inside a truly perforated spotted shell."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,tube,fitted_patch,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from spider_sculpt import dorsal_patch,upper_skin
from skin_glb import COORDINATES
SPECIES=('shuckle',);HEIGHT=.91
# Local Crystal normal RGB555: shell (21,4,3), soft body (31,22,5).
RED=(173/255,33/255,24/255);CREAM=(1.,1.,1.);YELLOW=(1.,181/255,41/255);INK=(0.,0.,0.);DARK=INK
CENTER=(0,.285,0);RADII=(.33,.245,.285)
# Oriented cube charts share a deterministic square perimeter grid.
FACES=(((1,0,0),(0,0,-1),(0,1,0)),((-1,0,0),(0,0,1),(0,1,0)),((0,0,1),(1,0,0),(0,1,0)),((0,0,-1),(-1,0,0),(0,1,0)),((0,1,0),(1,0,0),(0,0,-1)),((0,-1,0),(1,0,0),(0,0,1)))
HOLES={(0,1,-1):'right rear limb',(1,1,-1):'left front limb',(2,1,-1):'right front limb',(3,1,-1):'left rear limb',(4,1,-1):'neck',(4,-1,-1):'empty upper opening'}

def project(face,u,v,factor=1):
    n,a,b=FACES[face];d=tuple(n[k]+a[k]*u+b[k]*v for k in range(3));length=math.sqrt(sum(x*x for x in d))
    return tuple(CENTER[k]+RADII[k]*d[k]/length*factor for k in range(3))

def perimeter(cx,cy):
    # Six matching subdivisions on every half-face edge match adjacent charts.
    p=[]
    for j in range(6):p.append((cx-.5+j/6,cy-.5))
    for j in range(6):p.append((cx+.5,cy-.5+j/6))
    for j in range(6):p.append((cx+.5-j/6,cy+.5))
    for j in range(6):p.append((cx-.5,cy+.5-j/6))
    return p

def shell():
    vertices=[];faces=[];holes=[]
    # The shell is one connected outer/inner skin. Only hole rims join the skins.
    for face in range(6):
        for sx in (-1,1):
            for sy in (-1,1):
                cx=sx*.5;cy=sy*.5;rim=perimeter(cx,cy);hole=(face,sx,sy) in HOLES;n=len(rim)
                outer_ids=[];inner_ids=[]
                for factor,target in ((1,outer_ids),(.905,inner_ids)):
                    if not hole:
                        target.append([len(vertices)]);vertices.append(project(face,cx,cy,factor))
                    for j in range(4 if hole else 3):
                        t=j/3 if hole else (j+1)/3;ids=[]
                        for u,v in rim:
                            du=u-cx;dv=v-cy;d=math.hypot(du,dv);r=.27*(1-t)+d*t if hole else d*t
                            ids.append(len(vertices));vertices.append(project(face,cx+du*r/d,cy+dv*r/d,factor))
                        target.append(ids)
                for ids,reverse in ((outer_ids,False),(inner_ids,True)):
                    start=0
                    if not hole:
                        for i in range(n):
                            q=(ids[0][0],ids[1][i],ids[1][(i+1)%n]);faces.append(tuple(reversed(q)) if reverse else q)
                        start=1
                    for j in range(start,len(ids)-1):
                        for i in range(n):
                            q=(ids[j][i],ids[j+1][i],ids[j+1][(i+1)%n],ids[j][(i+1)%n]);faces.append(tuple(reversed(q)) if reverse else q)
                if hole:
                    for i in range(n):faces.append((outer_ids[0][i],outer_ids[0][(i+1)%n],inner_ids[0][(i+1)%n],inner_ids[0][i]))
                    holes.append((face,cx,cy,HOLES[(face,sx,sy)]))
    return crease_normals(part('Shell / continuous red armor with six real openings',RED,vertices,faces),45.),holes

def collar(face,cx,cy,label):
    n=40;v=[]
    for factor in (1.016,.994):
        for radius in (.269,.475):
            for i in range(n):
                a=i*math.tau/n;v.append(project(face,cx+radius*math.cos(a),cy+radius*math.sin(a),factor))
    f=[]
    for i in range(n):
        a=i;b=(i+1)%n;f.extend(((a,b,b+n,a+n),(a+3*n,b+3*n,b+2*n,a+2*n),(b,a,a+2*n,b+2*n),(a+n,b+n,b+3*n,a+3*n)))
    return crease_normals(part('Rim / curved cream annular collar '+label,CREAM,v,f),40.)

def sculpture(species='shuckle',normalize=True):
    if species not in SPECIES:raise ValueError('unsupported mold tortoise')
    armor,holes=shell();core=ellipsoid('Interior / recessed dark soft-body core',DARK,CENTER,(.238,.177,.207),32,20);p=[armor,core]
    for face,cx,cy,label in holes:p.append(collar(face,cx,cy,label))
    for face,cx,cy,label in holes:
        if 'limb' not in label:continue
        root=project(face,cx,cy);n=FACES[face][0]
        inside=(root[0]*.65,root[1],root[2]*.65)
        elbow=(root[0]+n[0]*.11,.101,root[2]+n[2]*.11)
        tip=(root[0]+n[0]*.225,.046,root[2]+n[2]*.225)
        path=[inside,root,elbow,(tip[0],.046,tip[2])]
        p.append(curved_tube('Limb / smooth flexible yellow '+label,YELLOW,path,[.044,.043,.048,.042],22))
        p.append(ellipsoid('Foot / rounded grounded yellow '+label,YELLOW,tip,(.057,.046,.062),24,16))
    root=project(4,.5,-.5)
    neck=[(root[0]*.63,.365,root[2]*.63),root,(.13,.62,.139),(.092,.785,.173),(.058,.946,.217)]
    p.append(curved_tube('Neck / curved flexible yellow neck',YELLOW,neck,[.044,.047,.041,.044,.065],24))
    head=ellipsoid('Head / broad rounded yellow mold-tortoise head',YELLOW,(.057,1.008,.228),(.114,.089,.098),36,22);p.append(head);hs=surface(head)
    for j,x in enumerate((.019,.091)):
        p.append(fitted_patch('Eye / fitted narrow dark vertical eye '+str(j),INK,oval(x,1.024,.005,.013,18),hs,.003))
    pts=[(x,.967-.007*math.sin((x-.032)/.057*math.pi),0) for x in (.029,.041,.056,.073,.084)]
    pts=[(x,y,hs(y,x)+.0014) for x,y,z in pts]
    p.append(tube('Mouth / fitted small worried expression',INK,pts,[.0015]*len(pts),10))
    # Spots use dense small patches on the real stored shell, between openings.
    front=surface(armor)
    for j,(x,y,w,h) in enumerate(((-.10,.27,.022,.020),(.045,.40,.024,.021),(.19,.29,.021,.018))):
        p.append(fitted_patch('Spot / fitted cream front shell fleck '+str(j),CREAM,oval(x,y,w,h,24),front,.004))
    back=dict(armor);back['positions']=[-v if i%3!=1 else v for i,v in enumerate(back['positions'])];bs=surface(back)
    for j,(x,y,w,h) in enumerate(((-.10,.28,.022,.019),(.065,.39,.024,.021),(.19,.30,.021,.019))):
        q=fitted_patch('Spot / fitted cream rear shell fleck '+str(j),CREAM,oval(x,y,w,h,24),bs,.004)
        for field in ('positions','normals'):q[field]=[-v if i%3!=1 else v for i,v in enumerate(q[field])]
        p.append(q)
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    if normalize:
        for q in p:
            q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'shuckle.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
