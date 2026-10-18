#!/usr/bin/env python3
"""Distinct original Machop-family anatomy; one canonical skinned mesh per species.

Machop: young broad-headed tail-bearing stance. Machoke: angular muscular
long-limbed wrestler with arm markings. Machamp: broad four-armed powerhouse.
All use fitted faces, three cranial ridges and three-toed grounded feet.
"""
import argparse
from pathlib import Path
from functools import lru_cache
from sculpt_geometry import part,loft,tube,ellipsoid,translate,curved_profiles
from bovine_sculpt import curved_tube
from raptor_bird_sculpt import paint
from abra_sculpt import surface
from skin_glb import COORDINATES

HEIGHTS={'machop':.85,'machoke':1.15,'machamp':1.45}
PALETTE={'machop':(.53,.66,.68),'machoke':(.61,.56,.66),'machamp':(.48,.60,.64),
         'ridge':(.57,.49,.35),'ink':(.07,.065,.085),'white':(.97,.96,.87),
         'gold':(.94,.71,.26),'red':(.75,.23,.29),'lip':(.82,.73,.49)}


@lru_cache(maxsize=6)
def sculpture(species,normalize=True):
    if species not in HEIGHTS:raise ValueError('unsupported fighting species')
    young=species=='machop';four=species=='machamp';skin=PALETTE[species];ink=PALETTE['ink'];parts=[]
    if young:
        profiles=((.24,.07,.075,-.015),(.34,.127,.108,-.02),(.49,.14,.11,-.015),
                  (.62,.156,.113,-.01),(.735,.159,.112,0.),(.80,.104,.091,.008),
                  (.853,.164,.134,.035),(.922,.206,.172,.052),
                  (1.015,.205,.158,.037),(1.092,.16,.126,.021),(1.135,.06,.059,.009))
    else:
        chest=.306 if four else .262;head=.156 if four else .164
        profiles=((.30,.125,.13,-.016),(.41,.20,.153,-.017),(.51,.188,.151,-.016),
                  (.62,.154,.127,-.014),(.735,.216,.154,-.012),(.844,chest,.175,-.012),
                  (.935,chest*.87,.156,-.013),(.986,.117,.101,-.01),
                  (1.024,head,.134,.028),(1.096,head+.012,.158,.037),
                  (1.17,head,.146,.02),(1.221,.101,.093,.006),(1.245,.035,.039,-.008))
    body_rows=curved_profiles(profiles,3)
    body=loft('Body / joined pelvis waist chest neck jaw and cranium',skin,body_rows,32)
    if not young:
        # Pectoral relief belongs to the torso surface itself. Independent
        # ellipsoids looked like pasted-on balls in the first review.
        vertices=[]
        for i in range(0,len(body['positions']),3):
            x,y,z=body['positions'][i:i+3]
            front=max(0.,min(1.,(z+.012)/.175))
            chest=max(0.,1.-abs(y-.844)/.12)**2
            paired=min(1.,abs(x)/.11)*(1.-min(1.,abs(x)/.35))
            z+=(.068 if four else .055)*chest*paired*front**2
            vertices.append((x,y,z))
        faces=[tuple(body['indices'][i:i+3]) for i in range(0,len(body['indices']),3)]
        body=part(body['part'],skin,vertices,faces,True)
    parts.append(body)
    # Distinct thigh/knee/ankle profiles, with supporting legs embedded at hips.
    spread=.12 if young else (.194 if four else .164)
    for side in (-1,1):
        parts.append(curved_tube('Leg / thigh knee and ankle '+str(side),skin,
            [(side*spread*.50,.45 if not young else .36,-.013),(side*spread,.33 if not young else .25,-.006),
             (side*spread*1.12,.20,0.),(side*spread*1.17,.095,.026)],
            [.108 if not young else .075,.107 if four else .086,.068 if four else .057,.055 if not young else .047],14))
        foot=ellipsoid('Foot / planted heel '+str(side),skin,(side*spread*1.17,.059,.054),(.088 if not young else .064,.06,.112),16,8)
        foot=translate(foot,0,-min(foot['positions'][1::3]),0);parts.append(foot)
        for toe in range(3):
            width=.034 if not young else .025
            t=ellipsoid('Toe / '+str(side)+' '+str(toe),skin,
                (side*spread*1.17+(toe-1)*width*1.45,.028,.138+(.016 if toe==1 else 0.)),(width,.029,.063),12,7)
            t=translate(t,0,-min(t['positions'][1::3]),0);parts.append(t)
    # Fleshy upper/lower arms with a deliberately narrower elbow and wrist.
    rows=[('upper',.882 if not young else .672)]
    if four:rows.append(('lower',.704))
    for label,y in rows:
        for side in (-1,1):
            shoulder=(.20 if four else .18) if not young else .11
            if four and label=='lower':shoulder=.16
            if young:
                points=[(side*shoulder,y,0.),(side*.227,y-.08,.015),(side*.273,y-.17,.105),(side*.257,y-.175,.18)];radii=[.045,.066,.052,.039]
            elif label=='lower':
                points=[(side*shoulder,y,-.012),(side*.352,y-.083,-.038),(side*.408,y-.19,.062),(side*.404,y-.185,.15)];radii=[.055,.102,.075,.051]
            else:
                points=[(side*shoulder,y,-.012),(side*.342,y-.047,.005),(side*.412,y-.093,.038),(side*.452,y-.032,.090),(side*.477,y+.077,.145)];radii=[.060 if four else .052,.094 if four else .084,.045,.056,.047]
            arm=curved_tube('Arm / '+label+' shaped deltoid biceps elbow forearm '+str(side),skin,points,radii,24);parts.append(arm)
            wrist=points[-1];center=(wrist[0],wrist[1]+.027,wrist[2]+.025)
            # Closed angular palm with three curled finger columns and a thumb.
            size=.068 if not young else .046
            palm=[(center[0]+x*size,center[1]+y*size,center[2]+z*size) for x,y,z in
                  ((-1,-.9,-.65),(1,-.9,-.65),(1,.9,-.65),(-1,.9,-.65),(-1,-.9,.65),(1,-.9,.65),(1,.9,.65),(-1,.9,.65))]
            parts.append(part('Hand / '+label+' squared palm '+str(side),skin,palm,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)]))
            for finger in range(3):
                x=center[0]+(finger-1)*size*.63
                parts.append(tube('Finger / '+label+' '+str(side)+' '+str(finger),skin,
                    [(x,center[1]+size*.8,center[2]+size*.36),(x,center[1]+size*.52,center[2]+size*.90),(x,center[1]+size*.05,center[2]+size*.72)],
                    [size*.31,size*.32,size*.25],10))
            parts.append(ellipsoid('Thumb / '+label+' '+str(side),skin,(center[0]-side*size*.81,center[1]-.02,center[2]+size*.3),(size*.37,size*.57,size*.39),12,7))
            if species=='machoke':
                # Red bands sit on the deltoid surface and follow its local slope.
                for band in range(2):
                    x=side*(.315+band*.026);yy=y+.010-band*.006;z=.071
                    parts.append(paint('Arm marking / '+str(side)+' '+str(band),PALETTE['red'],[(x-side*.009,yy-.044),(x+side*.009,yy-.044),(x+side*.009,yy+.005),(x-side*.009,yy+.005)],arm,.004,2))
    if not young:
        # Match the torso's exact rings: interpolation through a separate garment
        # profile exposed gray holes at the hips and back during the pose review.
        trunks=loft('Trunks / fitted dark hip garment',ink,[(y,rx+.006,rz+.006,z) for y,rx,rz,z in body_rows if .30-1e-8<=y<=.51+1e-8],32);parts.append(trunks)
        parts.append(loft('Belt / fitted gold waist band',PALETTE['gold'],[(y,rx+.010,rz+.010,z) for y,rx,rz,z in body_rows if .47<=y<=.51+1e-8],32))
        buckle=paint('Belt / angular championship buckle',PALETTE['gold'],[(-.059,.461),(.059,.461),(.059,.523),(-.059,.523)],body,.016,2);parts.append(buckle)
        parts.append(paint('Belt / dark buckle inset',ink,[(-.034,.474),(.034,.474),(.034,.508),(-.034,.508)],buckle,.004,2))
    else:
        parts.append(curved_tube('Tail / young curved taper',skin,[(0,.29,-.10),(0,.282,-.225),(.037,.30,-.34),(.072,.335,-.416)],[.05,.041,.025,.009],14))
    # Broad, fitted almond eyes follow the face surface without globe bulges.
    eye_y=.998 if young else 1.144;eye_z=.183 if young else .166
    for side in (-1,1):
        x=side*(.104 if young else .086)
        outline=[(x-side*.043,eye_y-.024,eye_z+.010),(x+side*.045,eye_y-.014,eye_z-.005),(x+side*.037,eye_y+.025,eye_z-.012),(x-side*.037,eye_y+.020,eye_z+.006)]
        eye=paint('Eye / fitted ivory almond '+str(side),PALETTE['white'],[(a,b) for a,b,c in outline],body,.005,2);parts.append(eye)
        pupil_x=x+side*.005
        parts.append(paint('Eye / narrow red iris '+str(side),PALETTE['red'],[(pupil_x-.009,eye_y-.008),(pupil_x+.009,eye_y-.008),(pupil_x+.009,eye_y+.009),(pupil_x-.009,eye_y+.009)],eye,.003,2))
        # Brow ridge reinforces the mature angular expression.
        if not young:
            parts.append(curved_tube('Brow / fitted angular ridge '+str(side),skin,[(x,y,surface(body)(y,x)+.004) for x,y in ((side*.041,1.169),(side*.087,1.184),(side*.135,1.177))],[.012,.014,.008],12))
    mouth_y=.907 if young else 1.063;mouth_z=.224 if young else .205
    mouth=paint('Mouth / fitted curved opening',ink,[(-.092,mouth_y+.007),(0,mouth_y-.014),(.092,mouth_y+.007),(.068,mouth_y+.031),(-.068,mouth_y+.031)],body,.005,2);parts.append(mouth)
    if four:
        parts.append(tube('Mouth / broad upper lip',PALETTE['lip'],[(x,y,surface(body)(y,x)+.004) for x,y in ((-.092,mouth_y+.027),(0,mouth_y+.039),(.092,mouth_y+.027))],[.015,.019,.015],12))
        parts.append(tube('Mouth / broad lower lip',PALETTE['lip'],[(x,y,surface(body)(y,x)+.004) for x,y in ((-.078,mouth_y),(0,mouth_y-.018),(.078,mouth_y))],[.014,.017,.014],12))
    # Three swept, attached cranial plates, with closed thickness and flat folds.
    for ridge in (-1,0,1):
        x=ridge*(.088 if young else .07);base=1.086 if young else 1.219
        outline=[(x-.014,base,-.059),(x-.014,base+.027,.050),(x-.012,base+.14-(.025*abs(ridge)),.01),(x-.012,base+.09,-.076)]
        vertices=outline+[(a+.025,b,c) for a,b,c in outline]
        parts.append(part('Crest / swept plate '+str(ridge),PALETTE['ridge'],vertices,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)]))
    top=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHTS[species]/top if normalize else 1.
    for p in parts:
        p['positions']=[round(v*scale,7) for v in p['positions']]
        p['normals']=[round(v,7) for v in p['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':parts}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=HEIGHTS);p.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=p.parse_args()
    for name in ([a.only] if a.only else HEIGHTS):
        from fighting_glb import export_fighting
        a.out.mkdir(parents=True,exist_ok=True);blob=export_fighting(name);path=a.out/(name+'.glb');path.write_bytes(blob);print(name,len(blob),'bytes')

if __name__=='__main__':main()
