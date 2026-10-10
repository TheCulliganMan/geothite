#!/usr/bin/env python3
"""Original connected Ampharos sculpture; one canonical static runtime JSON.

Continuous torso/neck/head loft, fitted belly panel, tapered flippers, striped
horns and curved banded tail. No Blender, source sprites or content pack needed.
"""
import argparse,json,math
from pathlib import Path
from skin_glb import COORDINATES
from sculpt_geometry import part as _part, loft as _loft, ellipsoid as _ellipsoid, tube as _tube, translate

HEIGHT=1.35
PALETTE={'yellow':(.98,.79,.22),'belly':(1.,.96,.80),
         'ink':(.065,.055,.055),'red':(.90,.13,.17),'shine':(1.,.85,.73)}




def part(name,color,*args,**kwargs):return _part(name,PALETTE[color],*args,**kwargs)
def loft(name,color,*args,**kwargs):return _loft(name,PALETTE[color],*args,**kwargs)
def ellipsoid(name,color,*args,**kwargs):return _ellipsoid(name,PALETTE[color],*args,**kwargs)
def tube(name,color,*args,**kwargs):return _tube(name,PALETTE[color],*args,**kwargs)


def sculpture():
    body_profiles=(
        (.12,.12,.115,-.01),(.22,.205,.17,-.01),(.36,.238,.205,-.01),
        (.50,.216,.18,-.015),(.62,.157,.13,-.014),(.71,.112,.105,-.01),
        (.83,.095,.09,-.004),(.94,.095,.092,.008),
        (1.015,.139,.134,.033),(1.09,.174,.167,.047),
        (1.16,.166,.16,.05),(1.23,.125,.127,.03),(1.275,.06,.067,.022),(1.29,.015,.018,.018))
    parts=[loft('Body / continuous pear torso slender neck and rounded muzzle','yellow',body_profiles,24)]
    def surface(y,x):
        for (ya,wa,da,za),(yb,wb,db,zb) in zip(body_profiles,body_profiles[1:]):
            if ya<=y<=yb:
                t=(y-ya)/(yb-ya);w=wa+(wb-wa)*t;d=da+(db-da)*t;z=za+(zb-za)*t
                return z+d*math.sqrt(max(0.,1-(x/w)**2))
        raise ValueError('surface outside body loft')
    # Belly is a fitted closed lenticular surface, rather than a second sphere.
    v=[];rows=((.22,.04,.150),(.28,.104,.184),(.38,.142,.199),(.49,.112,.172),(.59,.055,.136),(.625,.016,.122));segments=12
    for y,w,z in rows:
        for i in range(segments+1):
            q=-1+2*i/segments;v.append((q*w,y,surface(y,q*w)+.009))
    front=len(v);v+=[(x,y,z-.012) for x,y,z in v];f=[]
    for k in range(len(rows)-1):
        for i in range(segments):
            a=k*(segments+1)+i;b=a+1;c=b+segments+1;d=a+segments+1
            f.extend(((a,b,c,d),(d+front,c+front,b+front,a+front)))
    boundary=list(range(segments+1))+[k*(segments+1)+segments for k in range(1,len(rows))]+list(range(front-2,front-segments-2,-1))+[k*(segments+1) for k in range(len(rows)-2,0,-1)]
    f.extend((b,a,a+front,b+front) for a,b in zip(boundary,boundary[1:]+boundary[:1]))
    parts.append(part('Belly / fitted tapering ivory chest panel','belly',v,f,True))
    for side in (-1,1):
        paw=ellipsoid('Foot / broad planted '+str(side),'yellow',(side*.151,.062,.069),(.10,.063,.148))
        low=min(paw['positions'][1::3]);paw=translate(paw,0,-low,0);parts.append(paw)
        # Broad flattened flippers sweep down from the shoulder into one tip.
        arm=loft('Flipper / tapered paddle '+str(side),'yellow',((.36,.02,.038,.10),(.40,.059,.045,.072),(.48,.084,.052,.03),(.57,.076,.059,-.005),(.625,.029,.038,-.01)),16)
        for i in range(0,len(arm['positions']),3):
            y=arm['positions'][i+1];arm['positions'][i]+=side*(.18+(.62-y)*.27)
        # Recalculate normals after the curved paddle offset.
        verts=[tuple(arm['positions'][i:i+3]) for i in range(0,len(arm['positions']),3)]
        faces=[tuple(arm['indices'][i:i+3]) for i in range(0,len(arm['indices']),3)]
        parts.append(part(arm['part'],'yellow',verts,faces,True))
        earpoints=[(side*.135,1.185,.008),(side*.195,1.236,-.005),(side*.258,1.295,-.02),(side*.282,1.324,-.025)]
        parts.append(tube('Horn / rounded black taper '+str(side),'ink',earpoints,[.046,.044,.032,.015]))
        parts.append(tube('Horn / fitted gold band '+str(side),'yellow',[earpoints[1],(side*.224,1.265,-.012)],[.045,.040]))
        # Fitted narrow almond eyes, without protruding white ball eyes.
        parts.append(ellipsoid('Eye / black almond '+str(side),'ink',(side*.092,1.15,.182),(.023,.035,.010),12,8))
        parts.append(ellipsoid('Eye / catchlight '+str(side),'belly',(side*.087,1.162,.192),(.007,.010,.003),10,6))
        parts.append(ellipsoid('Nostril / tiny muzzle inset '+str(side),'ink',(side*.033,1.078,.215),(.005,.004,.003),8,6))
    for y in (.835,.902):
        parts.append(loft('Neck / charcoal fitted collar '+str(y),'ink',((y-.014,.098,.093,.002),(y+.014,.098,.095,.004)),24))
    smile=[(x,1.068+.09*abs(x),surface(1.068+.09*abs(x),x)+.005) for x in (-.038,-.02,0.,.02,.038)]
    parts.append(tube('Muzzle / quiet curved smile','ink',smile,[.0025]*5,8))
    parts.append(ellipsoid('Forehead / inset red beacon','red',(0,1.232,.145),(.036,.043,.014)))
    parts.append(ellipsoid('Forehead / restrained highlight','shine',(-.010,1.247,.157),(.008,.011,.003),10,6))
    tail=[(0,.225,-.156),(.014,.245,-.29),(.029,.282,-.42),(.042,.35,-.52),(.052,.433,-.58),(.06,.49,-.601)]
    parts.append(tube('Tail / continuous rising taper','yellow',tail,[.064,.055,.047,.038,.029,.022]))
    for k in (1,3):
        a=tail[k];b=tail[k+1];start=tuple(x+(y-x)*.10 for x,y in zip(a,b));end=tuple(x+(y-x)*.57 for x,y in zip(a,b))
        radius=[.055,.038][k//2]+.0015
        parts.append(tube('Tail / fitted charcoal ring '+str(k),'ink',[start,end],[radius,radius-.003]))
    parts.append(ellipsoid('Tail / ruby lamp','red',(.06,.525,-.621),(.083,.085,.080)))
    parts.append(ellipsoid('Tail / quiet lamp highlight','shine',(.025,.558,-.557),(.013,.016,.007),12,8))
    height=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/height
    for p in parts:
        p['positions']=[round(v*scale,7) for v in p['positions']]
        # Explicit source precision avoids platform libm's near-zero residues
        # becoming different canonical normal bytes on Mac and Linux.
        p['normals']=[round(v,7) for v in p['normals']]
    return {'name':'ampharos','version':1,'coordinate_system':COORDINATES,'primitives':parts}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/ampharos.mesh.json');a=p.parse_args();model=sculpture();a.output.write_text(json.dumps(model,separators=(',',':')));print(a.output)

if __name__=='__main__':main()
