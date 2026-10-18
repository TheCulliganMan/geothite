#!/usr/bin/env python3
"""Original Raichu: cream-bellied orange mouse, curled ears and thin bolt-tipped tail."""
import argparse,math,struct
from copy import deepcopy
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals,tube,part
from bovine_sculpt import oval,curved_tube,reverse_front,smooth
from skarmory_sculpt import blade
from pitcher_sculpt import bell_fitted as fitted
from abra_sculpt import surface
from skin_glb import COORDINATES

HEIGHT=.96
ORANGE=(.94,.48,.16);BROWN=(.37,.24,.18);CREAM=(.99,.91,.64)
YELLOW=(.99,.83,.23);INK=(.035,.035,.045);WHITE=(.99,.99,.96)
TAIL=((0.,.20,-.13),(-.05,.16,-.35),(.12,.11,-.50),(.40,.18,-.56),(.52,.37,-.49),(.50,.59,-.36))

def arm_path(side):return ((side*.16,.43,.035),(side*.24,.38,.065),(side*.28,.335,.11))

def ear_section(side,curl=False):
    """Structured oval rings and a lining copied from their actual front grid.

    Shared XY coordinates give the layer exactly the parent's UNORM8 weights;
    both surfaces keep the same triangle diagonals through every bend.
    """
    path=[(.155,.770,-.041),(.235,.765,-.048),(.315,.775,-.054),(.375,.819,-.062)] if curl else [(.135,.745,-.040),(.205,.867,-.049),(.282,1.012,-.060),(.358,1.096,-.067)]
    widths=[.044,.038,.023,.0012] if curl else [.046,.050,.029,.0012]
    depths=[.025,.022,.013,.0012] if curl else [.026,.023,.014,.0012]
    path=[(side*x,y,z) for x,y,z in path];centers,widths=smooth(path,widths,5);_,depths=smooth(path,depths,5);n=16;v=[]
    for j,((x,y,z),w,d) in enumerate(zip(centers,widths,depths)):
        a,b=centers[max(0,j-1)],centers[min(len(centers)-1,j+1)];dx=b[0]-a[0];dy=b[1]-a[1];length=math.hypot(dx,dy);e=(-dy/length,dx/length)
        v.extend((x+e[0]*w*math.cos(k*math.tau/n),y+e[1]*w*math.cos(k*math.tau/n),z+d*math.sin(k*math.tau/n)) for k in range(n))
    faces=[tuple(reversed(range(n))),tuple((len(centers)-1)*n+k for k in range(n))]
    for j in range(len(centers)-1):
        for k in range(n):
            a=j*n+k;b=j*n+(k+1)%n;faces.append((a,b,b+n,a+n))
    label='Ear curl / swept brown hook ' if curl else 'Ear / pointed curling brown pinna '
    outer=crease_normals(part(label+str(side),BROWN,v,faces,True),40.)
    # part() stores f32 positions before crease splitting. Match that exact
    # representation rather than comparing the source doubles to rounded keys.
    allowed={struct.unpack('<3f',struct.pack('<3f',*v[j*n+k])) for j in range(1,len(centers)-1) for k in range(2,7)}
    ps=[tuple(outer['positions'][i:i+3]) for i in range(0,len(outer['positions']),3)];lookup={};points=[];front=[]
    for i in range(0,len(outer['indices']),3):
        tri=[ps[k] for k in outer['indices'][i:i+3]];a,b,c=tri
        if not all(p in allowed for p in tri):continue
        if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
        ids=[]
        for p in tri:
            if p not in lookup:lookup[p]=len(points);points.append(p)
            ids.append(lookup[p])
        front.append(tuple(ids))
    if not front:raise ValueError('structured Raichu ear has no lining surface')
    count=len(points);vertices=[(x,y,z+offset) for offset in (.0025,-.0006) for x,y,z in points];faces=front+[tuple(k+count for k in reversed(tri)) for tri in front];edges={}
    for tri in front:
        for a,b in zip(tri,tri[1:]+tri[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
    for uses in edges.values():
        if len(uses)==1:
            a,b=uses[0];faces.append((b,a,a+count,b+count))
    label='Ear curl inset / fitted golden hook ' if curl else 'Ear inset / fitted golden pinna '
    lining=crease_normals(part(label+str(side),YELLOW,vertices,faces,True),40.)
    return outer,lining

@lru_cache(maxsize=1)
def anatomy():
    body=loft('Body / plump orange electric mouse pear',ORANGE,curved_profiles(((.07,.06,.06,-.01),(.16,.17,.145,-.01),(.30,.24,.18,-.012),(.43,.215,.165,-.005),(.53,.135,.115,.005),(.565,.025,.025,.01)),4),28)
    head=loft('Head / broad round orange Raichu cheeks',ORANGE,curved_profiles(((.465,.04,.045,.035),(.51,.17,.135,.025),(.60,.235,.18,.012),(.69,.237,.18,-.005),(.765,.15,.135,-.018),(.80,.028,.027,-.025)),4),32)
    p=[body,head]
    p.append(fitted('Belly / fitted broad cream oval',CREAM,oval(0.,.303,.143,.159,40),body,.004))
    for side in (-1,1):
        eye=fitted('Eye / fitted dark electric mouse gaze '+str(side),INK,oval(side*.100,.687,.027,.034,28),head,.004);p.append(eye)
        p.append(fitted('Eye glint / fitted ivory sparkle '+str(side),WHITE,oval(side*.100-.007,.699,.007,.009,20),eye,.0018))
        p.append(fitted('Cheek / fitted golden electric pouch '+str(side),YELLOW,oval(side*.177,.610,.040,.041,32),head,.004))
        p.extend(ear_section(side));p.extend(ear_section(side,True))
        p.append(curved_tube('Arm / short orange electric forelimb '+str(side),ORANGE,arm_path(side),[.053,.049,.036],20))
        p.append(ellipsoid('Hand / rounded brown electric palm '+str(side),BROWN,(side*.285,.315,.125),(.052,.052,.043),20,12))
        for j in range(3):
            p.append(ellipsoid(f'Finger / three blunt brown digits {side} {j}',BROWN,(side*(.260+.023*j),.278+(.003 if j!=1 else 0.),.146),(.014,.025,.014),12,8))
        p.append(ellipsoid('Thumb / tucked brown electric digit '+str(side),BROWN,(side*.247,.315,.147),(.018,.024,.016),12,8))
        p.append(ellipsoid('Leg / rooted orange mouse haunch '+str(side),ORANGE,(side*.125,.123,.015),(.075,.102,.083),24,16))
        foot=ellipsoid('Foot / planted long brown mouse pad '+str(side),BROWN,(side*.135,.030,.080),(.080,.030,.130),28,16);p.append(foot)
    p.append(fitted('Nose / tiny fitted dark triangle',INK,[(-.011,.637),(.011,.637),(0.,.626)],head,.003))
    smile=[];hs=surface(head)
    for i in range(29):
        x=-.062+.124*i/28;y=.579+.015*(abs(x)/.062)**2+.009*math.exp(-(x/.016)**2)
        smile.append((x,y,hs(y,x)+.0018))
    p.append(tube('Mouth / fitted gentle electric mouse smile',INK,smile,[.0023]*len(smile),8))
    rear=reverse_front(body)
    for j,(y,w) in enumerate(((.337,.16),(.425,.132))):
        q=fitted('Back stripe / fitted tapered brown band '+str(j),BROWN,[(-w,y-.014),(-w*.86,y+.012),(w*.86,y+.012),(w,y-.014),(w*.76,y-.026),(-w*.76,y-.026)],rear,.003)
        p.append(reverse_front(q))
    p.append(curved_tube('Tail cord / long thin curved brown whip',BROWN,TAIL,[.020,.018,.016,.014,.013,.012],16))
    p.append(blade('Tail bolt / broad golden lightning terminal',YELLOW,[(.49,.58),(.44,.67),(.54,.75),(.48,.80),(.68,.98),(.65,.82),(.75,.84),(.59,.65),(.60,.59)],lambda x,y:-.36,.016))
    return tuple(p)

@lru_cache(maxsize=1)
def authoring_transform():
    p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);return lo,HEIGHT/(hi-lo)

def sculpture(normalize=True):
    p=deepcopy(list(anatomy()));lo,k=authoring_transform()
    for q in p:
        if normalize:q['positions']=[round((v-(lo if i%3==1 else 0.))*k,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
        for key in ('positions','normals'):q[key]=[0. if v==0 else v for v in q[key]]
    return dict(name='raichu',version=1,coordinate_system=COORDINATES,primitives=p)

def main():
    from raichu_glb import export_raichu
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'raichu.glb').write_bytes(export_raichu())

if __name__=='__main__':main()
