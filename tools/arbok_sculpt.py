#!/usr/bin/env python3
"""Original hooded Arbok with fitted threat mask and a genuinely open fanged jaw."""
import argparse
from copy import deepcopy
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals
from bovine_sculpt import oval,curved_tube,smooth
from dragon_family_sculpt import painted_tube,fitted
from ekans_sculpt import PATH,RADII,STEPS
from murkrow_sculpt import bill
from skin_glb import COORDINATES

SPECIES=('arbok',);HEIGHT=1.18
PURPLE=(.50,.28,.60);VENTRAL=(.66,.51,.70)
INK=(.075,.045,.09);CREAM=(.98,.94,.80);RED=(.78,.20,.27);PINK=(.88,.39,.46)

@lru_cache(maxsize=1)
def spine_samples():return smooth(PATH,RADII,STEPS)

@lru_cache(maxsize=1)
def anatomy():
    p=painted_tube('Body / continuous grounded purple cobra coil',PURPLE,PATH,RADII,segments=32,steps=STEPS)
    from chikorita_sculpt import linear
    p[1]['part']='Belly / continuous fitted muted cobra ventral skin';p[1]['base_color']=linear(VENTRAL)
    hood=loft('Hood / broad rounded flared cobra mantle',PURPLE,
        curved_profiles(((.425,.035,.040,.020),(.49,.145,.055,.065),
                         (.62,.335,.067,.065),(.79,.458,.073,.065),
                         (.92,.420,.070,.055),(1.015,.260,.057,.025),
                         (1.075,.095,.040,-.015),(1.09,.025,.020,-.025)),5),48)
    p.append(hood)
    head=bill('Head / angular hooded cobra brow and upper muzzle',PURPLE,
        [(-.04,.080,1.03,1.16),(.05,.170,1.025,1.215),
         (.18,.165,1.035,1.17),(.25,.095,1.04,1.10),(.275,.025,1.05,1.08)])
    jaw=bill('Jaw / open rooted purple cobra mandible',PURPLE,
        [(-.025,.09,.975,1.055),(.07,.14,.925,.975),(.18,.14,.92,.96),
         (.245,.095,.955,.99),(.272,.025,.985,1.01)])
    p.extend((head,jaw,
        ellipsoid('Mouth back / recessed dark cobra throat',INK,(0.,1.005,.075),(.10,.075,.05),24,16),
        ellipsoid('Tongue / rooted pink cobra tongue',PINK,(0.,.965,.15),(.07,.009,.06),24,14)))
    for side in (-1,1):
        x=side*.105;y=1.115
        eye=fitted('Eye / fitted narrow ivory cobra glare '+str(side),CREAM,
                   [(x-side*.034,y-.023),(x+side*.030,y-.010),
                    (x+side*.034,y+.027),(x-side*.025,y+.010)],head,.004)
        p.append(eye)
        pupil=fitted('Pupil / fitted dark vertical cobra pupil '+str(side),INK,
                     oval(x,y,.006,.016,20),eye,.002);p.append(pupil)
        p.append(fitted('Glint / fitted cobra eye reflection '+str(side),CREAM,
                        oval(x-.002,y+.007,.002,.0035,12),pupil,.0015))
        p.append(fitted('Nostril / fitted cobra nostril '+str(side),INK,
                        oval(side*.05,1.081,.006,.004,16),head,.002))
        p.append(curved_tube('Fang / rooted curved ivory cobra canine '+str(side),CREAM,
                            [(side*.096,1.055,.175),(side*.095,1.026,.188),
                             (side*.090,.996,.197)],[.013,.010,.0012],16))
        x=side*.223;y=.82
        mask=[(x+side*dx,y+dy) for dx,dy in ((-.112,-.025),(-.102,.080),(-.075,.120),
              (-.044,.093),(-.008,.130),(.027,.095),(.080,.080),(.104,.025),
              (.086,-.063),(.008,-.106),(-.060,-.086))]
        rim=fitted('False eye rim / fitted dark cobra hood mask '+str(side),INK,mask,hood,.004);p.append(rim)
        inner=[(x+side*dx,y+dy) for dx,dy in ((-.065,-.022),(-.043,.045),
               (.012,.063),(.062,.022),(.050,-.040),(-.008,-.070))]
        pale=fitted('False eye / fitted ivory cobra hood eye '+str(side),CREAM,inner,rim,.0025);p.append(pale)
        p.append(fitted('False pupil / fitted red cobra hood center '+str(side),RED,
                        oval(x+side*.005,y-.010,.018,.025,24),pale,.002))
    rim=fitted('Threat rim / fitted dark angular cobra chevron border',INK,
        [(-.285,.695),(-.155,.667),(0.,.676),(.155,.667),(.285,.695),
         (.185,.556),(0.,.495),(-.185,.556)],hood,.0035);p.append(rim)
    p.append(fitted('Threat / fitted red angular cobra chevron',RED,
        [(-.235,.664),(-.120,.637),(0.,.649),(.120,.637),(.235,.664),
         (.152,.568),(0.,.516),(-.152,.568)],rim,.0025))
    return tuple(p)

@lru_cache(maxsize=1)
def authoring_transform():
    ps=anatomy();lo=min(v for p in ps for v in p['positions'][1::3]);hi=max(v for p in ps for v in p['positions'][1::3]);return lo,HEIGHT/(hi-lo)

def sculpture(species='arbok',normalize=True):
    if species not in SPECIES:raise ValueError('unsupported hooded cobra')
    ps=deepcopy(anatomy());lo,k=authoring_transform()
    for p in ps:
        if normalize:p['positions']=[round((v-(lo if i%3==1 else 0.))*k,7) or 0. for i,v in enumerate(p['positions'])]
        p['normals']=[round(v,7) or 0. for v in p['normals']]
    return dict(name=species,version=1,coordinate_system=COORDINATES,primitives=list(ps))

def main():
    from arbok_glb import export_arbok
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'arbok.glb').write_bytes(export_arbok())
if __name__=='__main__':main()
