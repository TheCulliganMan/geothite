#!/usr/bin/env python3
"""Distinct original Machop-family anatomy; one static canonical mesh per species.

Machop: young broad-headed tail-bearing stance. Machoke: angular muscular
long-limbed wrestler with arm markings. Machamp: broad four-armed powerhouse.
All use fitted faces, three cranial ridges and three-toed grounded feet.
"""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,tube,ellipsoid,translate
from skin_glb import COORDINATES

HEIGHTS={'machop':.85,'machoke':1.15,'machamp':1.45}
PALETTE={'machop':(.53,.66,.68),'machoke':(.61,.56,.66),'machamp':(.48,.60,.64),
         'ridge':(.57,.49,.35),'ink':(.07,.065,.085),'white':(.97,.96,.87),
         'gold':(.94,.71,.26),'red':(.75,.23,.29),'lip':(.82,.73,.49)}


def face_panel(name,color,outline,depth=.012):
    # A shallow closed fitted panel, not a floating iris sphere.
    front=list(outline);back=[(x,y,z-depth) for x,y,z in outline];n=len(front)
    faces=[tuple(range(n)),tuple(range(2*n-1,n-1,-1))]
    faces.extend((i,i+n,(i+1)%n+n,(i+1)%n) for i in range(n))
    return part(name,color,front+back,faces)


def sculpture(species):
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
    body=loft('Body / joined pelvis waist chest neck jaw and cranium',skin,profiles,24)
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
        parts.append(tube('Leg / thigh knee and ankle '+str(side),skin,
            [(side*spread*.77,.42 if not young else .34,-.013),(side*spread,.33 if not young else .25,-.006),
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
            shoulder=.262 if not young else .145
            if four and label=='lower':shoulder=.211
            if young:
                points=[(side*shoulder,y,0.),(side*.227,y-.08,.015),(side*.273,y-.17,.105),(side*.257,y-.175,.18)];radii=[.061,.066,.052,.039]
            elif label=='lower':
                points=[(side*shoulder,y,-.012),(side*.352,y-.083,-.038),(side*.408,y-.19,.062),(side*.404,y-.185,.15)];radii=[.089,.102,.075,.051]
            else:
                points=[(side*shoulder,y,-.012),(side*.342,y-.047,.005),(side*.412,y-.093,.038),(side*.452,y-.032,.090),(side*.477,y+.077,.145)];radii=[.11 if four else .082,.094 if four else .084,.045,.056,.047]
            parts.append(tube('Arm / '+label+' shaped deltoid biceps elbow forearm '+str(side),skin,points,radii,16))
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
                    x=side*(.315+band*.026);yy=y+.052-band*.005;z=.071
                    parts.append(face_panel('Arm marking / '+str(side)+' '+str(band),PALETTE['red'],[(x-side*.011,yy-.044,z),(x+side*.011,yy-.044,z),(x+side*.011,yy+.011,z-.024),(x-side*.011,yy+.011,z-.024)],.007))
    if not young:
        parts.append(loft('Trunks / fitted dark hip garment',ink,((.313,.131,.132,-.016),(.355,.185,.157,-.016),(.443,.201,.158,-.016),(.482,.194,.153,-.016)),24))
        parts.append(loft('Belt / fitted gold waist band',PALETTE['gold'],((.478,.198,.157,-.016),(.506,.187,.155,-.016)),24))
        parts.append(face_panel('Belt / angular championship buckle',PALETTE['gold'],[(-.059,.461,.155),(.059,.461,.155),(.059,.523,.152),(-.059,.523,.152)],.016))
        parts.append(face_panel('Belt / dark buckle inset',ink,[(-.034,.474,.173),(.034,.474,.173),(.034,.508,.172),(-.034,.508,.172)],.004))
    else:
        parts.append(tube('Tail / young curved taper',skin,[(0,.29,-.10),(0,.282,-.225),(.037,.30,-.34),(.072,.335,-.416)],[.05,.041,.025,.009],14))
    # Broad, fitted almond eyes follow the face surface without globe bulges.
    eye_y=.998 if young else 1.144;eye_z=.183 if young else .166
    for side in (-1,1):
        x=side*(.104 if young else .086)
        outline=[(x-side*.043,eye_y-.024,eye_z+.010),(x+side*.045,eye_y-.014,eye_z-.005),(x+side*.037,eye_y+.025,eye_z-.012),(x-side*.037,eye_y+.020,eye_z+.006)]
        parts.append(face_panel('Eye / fitted ivory almond '+str(side),PALETTE['white'],outline,.008))
        pupil_x=x+side*.005
        parts.append(face_panel('Eye / narrow dark pupil '+str(side),ink,[(pupil_x-.009,eye_y-.020,eye_z+.013),(pupil_x+.009,eye_y-.019,eye_z+.013),(pupil_x+.009,eye_y+.012,eye_z+.009),(pupil_x-.009,eye_y+.013,eye_z+.009)],.005))
        # Brow ridge reinforces the mature angular expression.
        if not young:
            parts.append(tube('Brow / fitted angular ridge '+str(side),skin,[(side*.041,1.169,.177),(side*.087,1.184,.173),(side*.135,1.177,.139)],[.015,.018,.009],10))
    mouth_y=.907 if young else 1.063;mouth_z=.224 if young else .205
    parts.append(face_panel('Mouth / recessed curved opening',ink,[(-.092,mouth_y+.007,mouth_z-.012),(0,mouth_y-.014,mouth_z),(.092,mouth_y+.007,mouth_z-.012),(.068,mouth_y+.031,mouth_z-.005),(-.068,mouth_y+.031,mouth_z-.005)],.012))
    if four:
        parts.append(tube('Mouth / broad upper lip',PALETTE['lip'],[(-.092,mouth_y+.027,mouth_z-.009),(0,mouth_y+.039,mouth_z+.006),(.092,mouth_y+.027,mouth_z-.009)],[.015,.019,.015],12))
        parts.append(tube('Mouth / broad lower lip',PALETTE['lip'],[(-.078,mouth_y,mouth_z-.008),(0,mouth_y-.018,mouth_z+.003),(.078,mouth_y,mouth_z-.008)],[.014,.017,.014],12))
    # Three swept, attached cranial plates, with closed thickness and flat folds.
    for ridge in (-1,0,1):
        x=ridge*(.088 if young else .07);base=1.086 if young else 1.219
        outline=[(x-.014,base,-.059),(x-.014,base+.027,.050),(x-.012,base+.14-(.025*abs(ridge)),.01),(x-.012,base+.09,-.076)]
        vertices=outline+[(a+.025,b,c) for a,b,c in outline]
        parts.append(part('Crest / swept plate '+str(ridge),PALETTE['ridge'],vertices,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)]))
    top=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHTS[species]/top
    for p in parts:
        p['positions']=[round(v*scale,7) for v in p['positions']]
        p['normals']=[round(v,7) for v in p['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':parts}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=HEIGHTS);p.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=p.parse_args()
    for name in ([a.only] if a.only else HEIGHTS):
        model=sculpture(name);path=a.out/(name+'.mesh.json');path.write_text(json.dumps(model,separators=(',',':')));print(name,len(model['primitives']),sum(len(p['indices'])//3 for p in model['primitives']))

if __name__=='__main__':main()
