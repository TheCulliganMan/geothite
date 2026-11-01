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
from skin_glb import smoothstep

HEIGHT=1.35
# Keep the previous ear-tip height as the crown datum; widening/reorienting
# the ears must not rescale the already corrected body and supports.
CROWN_TOP=1.3336835
TAIL_POINTS=((0.,.235,-.17),(.014,.235,-.34),(.035,.25,-.50),(.06,.29,-.64),(.09,.35,-.75),(.11,.40,-.81))
# Local normal RGB555: yellow (31,31,2), lamp (21,10,1).
PALETTE={'yellow':(1.,1.,16/255),'belly':(1.,1.,1.),'ink':(0.,0.,0.),'red':(173/255,82/255,8/255),'shine':(1.,1.,1.)}




def part(name,color,*args,**kwargs):return _part(name,PALETTE[color],*args,**kwargs)
def loft(name,color,*args,**kwargs):return _loft(name,PALETTE[color],*args,**kwargs)
def ellipsoid(name,color,*args,**kwargs):return _ellipsoid(name,PALETTE[color],*args,**kwargs)
def tube(name,color,*args,**kwargs):return _tube(name,PALETTE[color],*args,**kwargs)


def sculpture(normalize=True):
    body_profiles=(
        (.15,.10,.105,-.01),(.24,.17,.16,-.01),(.36,.245,.21,-.01),
        (.50,.26,.21,-.015),(.62,.20,.165,-.014),(.71,.135,.12,-.01),
        (.83,.10,.10,-.004),(.94,.10,.10,.015),
        (.975,.10,.180,.13),(1.015,.10,.180,.14),
        (1.055,.115,.190,.11),(1.10,.145,.170,.075),
        (1.16,.158,.150,.042),(1.25,.125,.127,.03),(1.31,.06,.067,.022),(CROWN_TOP,.015,.018,.018))
    body_sections=curved_profiles(body_profiles,2)
    body=loft('Body / continuous pear torso slender neck and rounded muzzle','yellow',body_sections,24)
    # The snout hangs in front of the neck. Lower only its anterior surface;
    # a longer round skull or a tiny projecting cone does not make this shape.
    verts=[]
    for x,y,z in zip(body['positions'][::3],body['positions'][1::3],body['positions'][2::3]):
        drop=.030*smoothstep(.13,.24,z)*(1.-smoothstep(1.075,1.16,y)) if y>.94 else 0.
        verts.append((x,y-drop,z))
    faces=[tuple(body['indices'][i:i+3]) for i in range(0,len(body['indices']),3)]
    body=crease_normals(part(body['part'],'yellow',verts,faces,True),45.)
    parts=[body];surface=stored_surface(body)
    def fitted(name,color,outline,parent=body,thickness=.0015):
        return crease_normals(fitted_triangle_contour(name,PALETTE[color],outline,parent,thickness),45.)
    # Clip every marking to stored triangles, rather than an analytic surface
    # that can bridge the real body or leave small facial solids floating.
    outline=[(.169*math.sin(a)*(1-.25*math.cos(a)),.455+.245*math.cos(a))
        for a in (math.tau*i/40 for i in range(40))]
    parts.append(fitted('Belly / fitted tapering ivory chest panel','belly',outline))
    for side in (-1,1):
        paw=ellipsoid('Foot / broad planted '+str(side),'yellow',(side*.185,.044,.045),(.088,.045,.115))
        low=min(paw['positions'][1::3]);paw=translate(paw,0,-low,0);parts.append(paw)
        # Upright spindle-shaped haunches bow outward in front view. The ankle
        # stays under the hip in profile; only the flat foot projects forward.
        profiles=((.065,.043,.050,-.005),(.10,.055,.065,-.010),
            (.17,.085,.095,-.006),(.25,.102,.110,-.010),
            (.33,.090,.100,-.015),(.38,.045,.060,-.010))
        centers={.065:.180,.10:.182,.17:.200,.25:.202,.33:.183,.38:.155}
        haunch=loft('Haunch / upright rounded yellow support '+str(side),'yellow',profiles,12)
        verts=[(x+side*centers[round(y,3)],y,z) for x,y,z in zip(haunch['positions'][::3],haunch['positions'][1::3],haunch['positions'][2::3])]
        faces=[tuple(haunch['indices'][i:i+3]) for i in range(0,len(haunch['indices']),3)]
        parts.append(part(haunch['part'],'yellow',verts,faces,True))
        # Broad flattened flippers sweep down from the shoulder into one tip.
        arm=loft('Flipper / tapered paddle '+str(side),'yellow',((.465,.015,.030,.14),(.51,.060,.040,.10),(.58,.083,.047,.05),(.66,.068,.050,.015),(.70,.035,.038,-.01)),16)
        for i in range(0,len(arm['positions']),3):
            y=arm['positions'][i+1];arm['positions'][i]+=side*(.18+(.70-y)*.68)
        # Recalculate normals after the curved paddle offset.
        verts=[tuple(arm['positions'][i:i+3]) for i in range(0,len(arm['positions']),3)]
        faces=[tuple(arm['indices'][i:i+3]) for i in range(0,len(arm['indices']),3)]
        parts.append(crease_normals(part(arm['part'],'yellow',verts,faces,True),35.))
        earpoints=[(side*.135,1.245,.008),(side*.20,1.277,-.005),(side*.26,1.30,-.020),(side*.28,1.31,-.025)]
        parts.append(tube('Horn / rounded yellow taper '+str(side),'yellow',earpoints,[.046,.044,.032,.015]))
        parts.append(tube('Horn / fitted charcoal band '+str(side),'ink',[earpoints[1],(side*.235,1.29,-.013)],[.045,.040]))
        # Fitted narrow almond eyes, without protruding white ball eyes.
        eye=fitted('Eye / black almond '+str(side),'ink',oval(side*.125,1.14,.014,.025,24));parts.append(eye)
        parts.append(fitted('Eye / catchlight '+str(side),'belly',oval(side*.123,1.149,.004,.007,16),eye,.0007))
        parts.append(fitted('Nostril / tiny muzzle inset '+str(side),'ink',oval(side*.033,.988,.005,.004,16),thickness=.0008))
    for y in (.81,.865,.92):
        # Fit the same polygonal sections as the neck carrier, before the
        # anterior jaw drops in front of it. A guessed cylinder leaves broken
        # stripe edges, while fitting the whole head would stripe the muzzle.
        rings=[]
        for yy in (y-.010,y+.010):
            a,b=next((a,b) for a,b in zip(body_sections,body_sections[1:]) if a[0]<=yy<=b[0])
            t=(yy-a[0])/(b[0]-a[0]);rx,rz,center=(a[k]+t*(b[k]-a[k]) for k in (1,2,3))
            rings.append((yy,rx*1.025,rz*1.025,center))
        parts.append(loft('Neck / charcoal fitted collar '+str(y),'ink',rings,24))
    smile=[]
    for i in range(17):
        x=-.038+.076*i/16;y=.960+.0034*(x/.038)**2
        smile.append((x,y,surface(y,x)+.0008))
    parts.append(tube('Muzzle / quiet curved smile','ink',smile,[.0025]*len(smile),8))
    beacon=fitted('Forehead / inset red beacon','red',oval(0,1.275,.036,.043,28),thickness=.0025);parts.append(beacon)
    parts.append(fitted('Forehead / restrained highlight','shine',oval(-.010,1.289,.008,.011,16),beacon,.0007))
    tail=TAIL_POINTS
    parts.append(tube('Tail / continuous rising taper','yellow',tail,[.085,.070,.056,.045,.034,.025]))
    for k in (2,3,4):
        a=tail[k];b=tail[k+1];start=tuple(x+(y-x)*.10 for x,y in zip(a,b));end=tuple(x+(y-x)*.57 for x,y in zip(a,b))
        radius={2:.056,3:.045,4:.034}[k]+.0015
        parts.append(tube('Tail / fitted charcoal ring '+str(k),'ink',[start,end],[radius,radius-.003]))
    parts.append(ellipsoid('Tail / ruby lamp','red',(.115,.432,-.835),(.083,.085,.080)))
    parts.append(ellipsoid('Tail / quiet lamp highlight','shine',(.08,.465,-.771),(.013,.016,.007),12,8))
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
