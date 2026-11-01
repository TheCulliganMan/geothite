#!/usr/bin/env python3
"""Original alert Sneasel with angular fitted eyes and curved ivory talons."""
import argparse,json,math
from functools import lru_cache
from copy import deepcopy
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,crease_normals,part,fitted_triangle_contour
from abra_sculpt import surface
from wing_insect_sculpt import contour,membrane
from skin_glb import COORDINATES
SPECIES=('sneasel',);HEIGHT=1.11
# Full-color game pigments; Crystal's front animation governs the raised guard.
TEAL=(56/255,100/255,120/255);ROSE=(224/255,68/255,128/255);IVORY=(.95,.97,.95)
INK=(.035,.06,.08);GOLD=(248/255,196/255,32/255);RUBY=(.81,.22,.30)

def oval(cx,cy,rx,ry,n=18):return [(cx+rx*math.cos(i*math.tau/n),cy+ry*math.sin(i*math.tau/n)) for i in range(n)]


def mark(name,color,outline,parent,offset=.003):
    return fitted_triangle_contour(name,color,outline,parent,offset,precision=8)


def feather(name,root,tip,width,thickness):
    """A folded sagittal blade: broad in side view, narrow from front/back."""
    dy,dz=tip[1]-root[1],tip[2]-root[2];length=math.hypot(dy,dz)
    across=(0.,-dz/length,dy/length);vertices=[];faces=[]
    profiles=((0.,.16,.72),(.18,.65,1.),(.48,1.,1.),(.74,.76,.85),(.92,.32,.42),(1.,.015,.10))
    for f,w,h in profiles:
        center=[root[k]+f*(tip[k]-root[k]) for k in range(3)]
        center[0]+=.004*math.sin(math.pi*f)
        for a,b in ((-1.,0.),(-.50,.70),(0.,1.),(.50,.70),(1.,0.),(0.,-.55)):
            vertices.append(tuple(center[k]+across[k]*a*width*w+(thickness*h*b if k==0 else 0.) for k in range(3)))
    n=6;faces=[tuple(reversed(range(n))),tuple(range((len(profiles)-1)*n,len(profiles)*n))]
    faces += [(j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k) for j in range(len(profiles)-1) for k in range(n)]
    return part(name,ROSE,vertices,faces)


def foot_pad(side):
    q=loft('Foot / broad grounded teal pad '+str(side),TEAL,
        ((0.,.055,.077,.075),(.020,.068,.105,.080),(.055,.065,.098,.075),
         (.085,.047,.073,.050),(.108,.012,.020,.020)),20)
    q['positions']=[v+(side*.105 if i%3==0 else 0.) for i,v in enumerate(q['positions'])]
    return crease_normals(q,35.)

@lru_cache(maxsize=1)
def raw_sculpture(species='sneasel'):
    if species not in SPECIES:raise ValueError('unsupported claw weasel')
    body=loft('Body / slim continuous hips waist and chest',TEAL,curved_profiles(((.285,.060,.065,-.015),(.37,.150,.125,-.015),(.47,.135,.130,-.010),(.57,.105,.115,-.005),(.66,.085,.090,.005),(.74,.060,.065,.010)),4),32)
    head=loft('Head / tapered muzzle and broad slanted feline crown',TEAL,curved_profiles(((.69,.04,.055,.09),(.735,.135,.130,.055),(.80,.225,.180,.015),(.89,.245,.190,-.007),(.98,.200,.165,-.018),(1.045,.110,.095,-.021),(1.072,.022,.022,-.021)),4),36)
    p=[body,head]
    ear=[(-.105,.995),(-.18,1.16),(-.24,1.04),(-.18,.98)]
    p.append(membrane('Ear / short pointed left auricle -1',TEAL,ear,1,root_z=.005,thickness=.037))
    p.append(membrane('Ear / feather-bearing right auricle 1',TEAL,[(.10,.98),(.13,1.055),(.22,1.11),(.215,.99)],1,root_z=.005,thickness=.040))
    p.append(feather('Plume / long swept rose ear feather',(.145,1.005,.010),(.155,1.43,-.120),.065,.012))
    for j,tip in enumerate(((.012,.390,-.435),(-.006,.325,-.475),(.006,.245,-.440))):
        p.append(feather('Tail / swept rose feather '+str(j),(0.,.34,-.135),tip,(.038,.043,.037)[j],.009))
    p.append(tube('Tail / short rooted fan stem',TEAL,[(0,.35,-.065),(0,.34,-.110),(0,.34,-.150)],[.043,.038,.029],16))
    for side in (-1,1):
        p.append(tube('Leg / upright supporting hock '+str(side),TEAL,[(side*.075,.395,-.015),(side*.098,.215,-.015),(side*.105,.085,-.005)],[.055,.052,.050],20))
        p.append(foot_pad(side))
        for j,dx in enumerate((-.032,.032)):
            p.append(tube(f'Talon / paired curved foot claw {side} {j}',IVORY,[(side*.105+dx,.055,.140),(side*.105+dx,.038,.200),(side*.105+dx,.020,.245),(side*.105+dx,.017,.258)],[.027,.022,.011,.0015],18))
        palm=(-.32,1.055,.080) if side<0 else (.290,.590,.170)
        points=[(-.09,.65,0.),(-.22,.79,.02),palm] if side<0 else [(.09,.64,0.),(.20,.64,.09),palm]
        p.append(tube('Arm / curved reaching forelimb '+str(side),TEAL,points,[.057,.053,.049],20))
        hand=ellipsoid('Hand / broad claw-bearing palm '+str(side),TEAL,palm,(.069,.058,.062),24,14);p.append(hand)
        for j,dx in enumerate((-.030,.030)):
            x=palm[0]+dx;y=palm[1];z=palm[2]
            curve=[(x,y+.035,z+.030),(x+side*.012,y+.100,z+.090),(x+side*.020,y+.120,z+.140),(x+side*.015,y+.100,z+.160)] if side<0 else [(x,y+.010,z+.04),(x+side*.012,y-.008,z+.104),(x+side*.015,y-.060,z+.153),(x+side*.011,y-.094,z+.16)]
            p.append(tube(f'Talon / hooked ivory hand claw {side} {j}',IVORY,curve,[.029,.027,.017,.0015],18))
        outline=[(side*.055,.856),(side*.198,.900),(side*.184,.814),(side*.077,.793)]
        socket=mark('Eye / fitted dark slanted socket '+str(side),INK,outline,head,.003);p.append(socket)
        cx=cy=0.
        for x0,y0 in outline:cx+=x0;cy+=y0
        cx/=4;cy/=4
        inset=[(cx+(x-cx)*.82,cy+(y-cy)*.76) for x,y in outline]
        eye=mark('Eye / fitted ivory angular white '+str(side),IVORY,inset,socket,.002);p.append(eye)
        x=side*.115;y=.833
        iris=mark('Eye / narrow fitted red iris '+str(side),RUBY,oval(x,y,.008,.016),eye,.0015);p.append(iris)
        p.append(mark('Eye / vertical dark pupil '+str(side),INK,oval(x,y,.003,.013,12),iris,.001))
    for name,skin,y in (('forehead',head,.988),('chest',body,.607)):
        p.append(mark('Gem / fitted gold oval '+name,GOLD,oval(0,y,.022,.040,24),skin,.004))
    line=[(-.055,.755),(-.023,.743),(.015,.743),(.053,.761)]
    mouth=mark('Mouth / fitted narrow mischievous smirk',INK,[(x,y+.002) for x,y in line]+[(x,y-.002) for x,y in reversed(line)],head,.003);p.append(mouth)
    for side in (-1,1):
        x=side*.037
        p.append(mark('Mouth / small ivory fang '+str(side),IVORY,[(x-.007,.746),(x+.007,.748),(x,.734)],head,.004))
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(species='sneasel',normalize=True):
    model=deepcopy(raw_sculpture(species))
    if not normalize:return model
    p=model['primitives']
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    # Fitted corners can be small; their normals must describe stored f32
    # positions after normalization rather than the unrounded clipping result.
    from chikorita_sculpt import f32
    for i,q in enumerate(p):
        if not q['part'].startswith(('Eye /','Gem /','Mouth /','Plume /','Tail / swept')):continue
        v=[tuple(f32(x) for x in q['positions'][j:j+3]) for j in range(0,len(q['positions']),3)]
        faces=[tuple(q['indices'][j:j+3]) for j in range(0,len(q['indices']),3)]
        rebuilt=part(q['part'],(0.,0.,0.),v,faces);rebuilt['base_color']=q['base_color'];p[i]=rebuilt
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    from sneasel_glb import export_sneasel
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'sneasel.glb').write_bytes(export_sneasel())
if __name__=='__main__':main()
