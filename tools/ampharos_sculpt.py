#!/usr/bin/env python3
"""Original connected Ampharos authoring sculpture for the canonical standard skin.

Continuous torso/neck/head loft, fitted belly panel, tapered flippers, striped
horns and curved banded tail. No Blender, source sprites or content pack needed.
"""
import argparse,json,math
from pathlib import Path
from skin_glb import COORDINATES
from sculpt_geometry import part as _part, loft as _loft, ellipsoid as _ellipsoid, tube as _tube, translate,curved_profiles,fitted_triangle_contour,crease_normals
from goldfish_sculpt import surface as stored_surface
from bovine_sculpt import oval

HEIGHT=1.35
# Local normal RGB555: yellow (31,31,2), lamp (21,10,1).
PALETTE={'yellow':(1.,1.,16/255),'belly':(1.,1.,1.),'ink':(0.,0.,0.),'red':(173/255,82/255,8/255),'shine':(1.,1.,1.)}




def part(name,color,*args,**kwargs):return _part(name,PALETTE[color],*args,**kwargs)
def loft(name,color,*args,**kwargs):return _loft(name,PALETTE[color],*args,**kwargs)
def ellipsoid(name,color,*args,**kwargs):return _ellipsoid(name,PALETTE[color],*args,**kwargs)
def tube(name,color,*args,**kwargs):return _tube(name,PALETTE[color],*args,**kwargs)


def sculpture(normalize=True):
    body_profiles=(
        (.105,.16,.14,-.01),(.22,.205,.17,-.01),(.36,.238,.205,-.01),
        (.50,.216,.18,-.015),(.62,.157,.13,-.014),(.71,.112,.105,-.01),
        (.83,.095,.09,-.004),(.94,.095,.092,.008),
        (1.015,.139,.134,.033),(1.09,.174,.167,.047),
        (1.16,.166,.16,.05),(1.23,.125,.127,.03),(1.275,.06,.067,.022),(1.29,.015,.018,.018))
    body=loft('Body / continuous pear torso slender neck and rounded muzzle','yellow',curved_profiles(body_profiles,2),32)
    parts=[body];surface=stored_surface(body)
    def fitted(name,color,outline,parent=body,thickness=.0015):
        return crease_normals(fitted_triangle_contour(name,PALETTE[color],outline,parent,thickness),45.)
    # Clip every marking to stored triangles, rather than an analytic surface
    # that can bridge the real body or leave small facial solids floating.
    outline=[(.137*math.sin(a)*(1-.25*math.cos(a)),.423+.201*math.cos(a))
        for a in (math.tau*i/40 for i in range(40))]
    parts.append(fitted('Belly / fitted tapering ivory chest panel','belly',outline))
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
        eye=fitted('Eye / black almond '+str(side),'ink',oval(side*.092,1.15,.023,.035,24));parts.append(eye)
        parts.append(fitted('Eye / catchlight '+str(side),'belly',oval(side*.087,1.162,.007,.010,16),eye,.0007))
        parts.append(fitted('Nostril / tiny muzzle inset '+str(side),'ink',oval(side*.033,1.078,.005,.004,16),thickness=.0008))
    for y in (.835,.902):
        parts.append(loft('Neck / charcoal fitted collar '+str(y),'ink',((y-.014,.098,.093,.002),(y+.014,.098,.095,.004)),24))
    smile=[]
    for i in range(17):
        x=-.038+.076*i/16;y=1.068+.0034*(x/.038)**2
        smile.append((x,y,surface(y,x)+.0008))
    parts.append(tube('Muzzle / quiet curved smile','ink',smile,[.0025]*len(smile),8))
    # Keep the outline off the exact 1.275 body-ring tangency, which otherwise
    # produces a sliver that collapses when the stored float32 mesh is rounded.
    beacon=fitted('Forehead / inset red beacon','red',oval(0,1.233,.036,.043,28),thickness=.0025);parts.append(beacon)
    parts.append(fitted('Forehead / restrained highlight','shine',oval(-.010,1.247,.008,.011,16),beacon,.0007))
    tail=[(0,.225,-.156),(.014,.245,-.29),(.029,.282,-.42),(.042,.35,-.52),(.052,.433,-.58),(.06,.49,-.601)]
    parts.append(tube('Tail / continuous rising taper','yellow',tail,[.064,.055,.047,.038,.029,.022]))
    for k in (1,3):
        a=tail[k];b=tail[k+1];start=tuple(x+(y-x)*.10 for x,y in zip(a,b));end=tuple(x+(y-x)*.57 for x,y in zip(a,b))
        radius=[.055,.038][k//2]+.0015
        parts.append(tube('Tail / fitted charcoal ring '+str(k),'ink',[start,end],[radius,radius-.003]))
    parts.append(ellipsoid('Tail / ruby lamp','red',(.06,.525,-.621),(.083,.085,.080)))
    parts.append(ellipsoid('Tail / quiet lamp highlight','shine',(.025,.558,-.557),(.013,.016,.007),12,8))
    if not normalize:return {'name':'ampharos','version':1,'coordinate_system':COORDINATES,'primitives':parts}
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
