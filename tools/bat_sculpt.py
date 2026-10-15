#!/usr/bin/env python3
"""Original Zubat/Golbat/Crobat anatomy, scalloped wings and fitted faces.

Single canonical standard-skin GLB per species. No sprites, pack content or imported
meshes. Existing contour and hollow-shell helpers supply geometry only.
"""
import argparse
from pathlib import Path
from sculpt_geometry import part,loft,tube,ellipsoid,curved_profiles,fitted_patch
from abra_sculpt import surface
from wing_insect_sculpt import contour,membrane
from pitcher_sculpt import funnel
from skin_glb import COORDINATES

SPECIES=('zubat','golbat','crobat');HEIGHT=1.
BLUE=(.25,.56,.74);PURPLE=(.49,.34,.62);WING=(.57,.38,.63)
INK=(.05,.035,.065);IVORY=(.97,.95,.82);YELLOW=(.96,.82,.28)


def face_patch(name,color,outline,skin,thickness):
    # Sample each edge as well as concentric rings so curved eye silhouettes
    # follow the stored face instead of cutting through it between corners.
    dense=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):
        dense.extend((a[0]+(b[0]-a[0])*i/5,a[1]+(b[1]-a[1])*i/5) for i in range(5))
    return fitted_patch(name,color,dense,skin,thickness)


def wing_parts(species,color):
    crobat=species=='crobat';p=[]
    upper=contour([
        ((.12,.67),(.30,.81),(.54,.92),(.91,1.075 if not crobat else .99)),
        ((.91,1.075 if not crobat else .99),(.84,.87),(.76,.52),(.72,.32 if not crobat else .40)),
        ((.72,.32 if not crobat else .40),(.65,.57),(.57,.67),(.53,.66)),
        ((.53,.66),(.49,.62),(.445,.42),(.40,.275 if not crobat else .34)),
        ((.40,.275 if not crobat else .34),(.35,.51),(.25,.56),(.12,.43)),
        ((.12,.43),(.10,.49),(.10,.59),(.12,.67))],7)
    lower=contour([
        ((.13,.43),(.31,.37),(.48,.31),(.68,.105)),
        ((.68,.105),(.60,.17),(.43,.24),(.36,.225)),
        ((.36,.225),(.30,.23),(.22,.28),(.12,.34)),
        ((.12,.34),(.11,.37),(.12,.40),(.13,.43))],8)
    for s in (-1,1):
        p.append(membrane('Wing / broad scalloped upper membrane '+str(s),WING if not crobat else PURPLE,upper,s,-.07,thickness=.009))
        leading=[(s*.12,.67,-.073),(s*.34,.83,-.091),(s*.57,.93,-.105),(s*.91,1.075 if not crobat else .99,-.118)]
        leading=[(x,y,-.07-.06*abs(x)+.025*(y-.70)**2+.005) for x,y,z in leading]
        p.append(tube('Wing bone / tapered leading edge '+str(s),color,leading,[.027,.024,.016,.0035],14))
        wrist=(s*.34,.78,-.064)
        for j,(x,y) in enumerate(((.72,.32 if not crobat else .40),(.40,.275 if not crobat else .34),(.12,.43))):
            points=[wrist,(s*(.34+x)*.5,(.78+y)*.5+.025,-.08),(s*x,y,-.075-.06*x)]
            points=[(x,y,-.07-.06*abs(x)+.025*(y-.70)**2+.005) for x,y,z in points]
            p.append(tube('Wing finger / branching spar '+str(s)+' '+str(j),color,points,[.016,.012,.0045],12))
        if crobat:
            p.append(membrane('Wing / swept lower membrane '+str(s),PURPLE,lower,s,-.10,thickness=.009))
            p.append(tube('Wing bone / lower leading edge '+str(s),color,[(s*.13,.43,-.08),(s*.35,.31,-.108),(s*.68,.105,-.14)],[.024,.017,.0035],12))
        else:
            p.append(tube('Leg / slender hanging shank '+str(s),color,[(s*.073,.27,-.04),(s*.102,.15,-.03),(s*.103,.065,.005)],[.027,.018,.009],12))
            for j,dx in enumerate((-.018,.018)):
                p.append(tube('Toe / tiny hooked claw '+str(s)+' '+str(j),color,[(s*.103,.083,.003),(s*(.103+dx),.055,.032),(s*(.105+dx),.059,.054)],[.009,.006,.0025],8))
    return p


def ear_parts(side,color,large):
    # A pointed, folded shell has depth; its inner marking follows that shell.
    y=.79 if not large else .855;tip=.995 if not large else 1.04
    outline=[(side*.105,y),(side*.223,y+.019),(side*.264,tip),(side*.155,y+.110)]
    front=[(x,yy,-.018) for x,yy in outline]+[(side*.174,y+.072,.022)]
    back=[(x,yy,-.051) for x,yy in outline]+[(side*.174,y+.072,-.07)]
    v=front+back;f=[]
    for i in range(4):
        j=(i+1)%4;f.extend(((i,j,4),(j+5,i+5,9),(j,i,i+5,j+5)))
    shell=part('Ear / folded pointed shell '+str(side),color,v,f)
    inside=[(side*.143,y+.044),(side*.195,y+.051),(side*.235,tip-.047)]
    return [shell,fitted_patch('Ear / inset rose membrane '+str(side),(.61,.36,.51),inside,surface(shell),.003)]


def sculpture(species, *, normalize=True):
    if species not in SPECIES:raise ValueError('unsupported bat')
    crobat=species=='crobat';golbat=species=='golbat';color=PURPLE if crobat else BLUE
    p=wing_parts(species,color)
    if golbat:
        # The entire face is a thick elliptical carapace around a deep mouth.
        # Its central opening is not an opaque black sphere on a blue head.
        face=funnel('Face / deep hollow gaping carapace',color,((-.10,.15,.24),(-.01,.229,.321),(.075,.265,.342),(.17,.246,.316)),(0,.56,0),thickness=.043)
        p.append(face)
        p.append(funnel('Mouth / fitted dark inner lining',INK,((-.004,.178,.272),(.150,.204,.278)),(0,.56,0),thickness=.004,segments=24))
        p.append(ellipsoid('Mouth / recessed throat',INK,(0,.56,-.020),(.166,.257,.006),24,14))
        p.append(ellipsoid('Body / compact rear barrel',color,(0,.55,-.14),(.19,.29,.082),24,14))
        skin=surface(face)
        for s in (-1,1):
            outline=[(s*.030,.862),(s*.128,.823),(s*.119,.809),(s*.041,.842)]
            p.append(face_patch('Eye / fitted ivory wedge '+str(s),IVORY,outline,skin,.009))
            p.append(face_patch('Eye / narrow black pupil '+str(s),INK,[(s*.081,.839),(s*.094,.834),(s*.094,.821),(s*.081,.827)],lambda y,x:skin(y,x)+.011,.003))
            p.extend(ear_parts(s,color,True))
            for direction in (-1,1):
                y=.56+direction*.262
                points=[(s*.125,y,.162),(s*.112,y-direction*.039,.174),(s*.103,y-direction*.083,.184)]
                p.append(tube('Fang / long inward canine '+str(s)+' '+str(direction),IVORY,points,[.026,.018,.002],14))
    elif not crobat:
        body=loft('Body / continuous tapered eyeless bat torso',color,curved_profiles(((.215,.018,.02,-.050),(.30,.090,.087,-.068),(.44,.136,.102,-.075),(.57,.13,.071,-.089),(.69,.068,.053,-.078)),3),24)
        p.append(body)
        p.append(funnel('Face / open eyeless muzzle shell',color,((-.03,.08,.119),(.07,.159,.166),(.17,.139,.128)),(0,.660,0),thickness=.026))
        p.append(funnel('Mouth / dark inner lining',INK,((.02,.110,.122),(.157,.115,.102)),(0,.660,0),thickness=.004,segments=24))
        p.append(ellipsoid('Mouth / recessed throat',INK,(0,.660,.030),(.09,.101,.006),20,12))
        for s in (-1,1):
            p.extend(ear_parts(s,color,False))
            for direction in (-1,1):
                y=.660+direction*.095
                p.append(tube('Fang / inward canine '+str(s)+' '+str(direction),IVORY,[(s*.066,y,.168),(s*.058,y-direction*.036,.179),(s*.054,y-direction*.067,.185)],[.018,.011,.0015],12))
    else:
        body=loft('Body / continuous broad violet cranium and tapered belly',color,curved_profiles(((.287,.018,.022,-.02),(.36,.12,.104,-.020),(.46,.162,.134,-.010),(.57,.161,.141,.001),(.67,.195,.163,.015),(.75,.221,.155,.014),(.815,.183,.122,.003),(.858,.099,.067,-.007),(.873,.017,.016,-.013)),3),28)
        p.append(body);skin=surface(body)
        for s in (-1,1):
            p.extend(ear_parts(s,color,False))
            outline=[(s*.042,.772),(s*.165,.798),(s*.157,.748),(s*.049,.749)]
            p.append(face_patch('Eye / angry fitted yellow wedge '+str(s),YELLOW,outline,skin,.007))
            p.append(face_patch('Eye / narrow vertical pupil '+str(s),INK,[(s*.101,.782),(s*.115,.784),(s*.115,.755),(s*.101,.755)],lambda y,x:skin(y,x)+.009,.003))
        p.append(fitted_patch('Mouth / angular open slot',INK,[(-.09,.673),(.09,.673),(.064,.583),(-.064,.583)],skin,.0045))
        for s in (-1,1):
            for direction in (-1,1):
                y=.672 if direction==1 else .590;x=s*.054
                points=[(x,y,skin(y,x)+.009),(x*.9,y-direction*.019,skin(y-direction*.019,x*.9)+.012),(x*.85,y-direction*.036,skin(y-direction*.036,x*.85)+.014)]
                p.append(tube('Fang / fitted triangular canine '+str(s)+' '+str(direction),IVORY,points,[.014,.009,.0015],12))
    if not normalize:return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    from bat_glb import export_bat
    for s in SPECIES:
        (args.output/(s+'.glb')).write_bytes(export_bat(s))
        (args.output/(s+'.mesh.json')).unlink(missing_ok=True)

if __name__=='__main__':main()
