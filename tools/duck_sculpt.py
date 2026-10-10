#!/usr/bin/env python3
"""Distinct original Psyduck/Golduck forms with fitted faces and true webbing.

Psyduck's broad confused head/temple hands and egg body contrast with Golduck's
streamlined swimmer, splayed hands, long tail, cranial fins and forehead ruby.
Static canonical JSON only; no imported meshes, sprites or duplicate catalog.
"""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,tube,part,ellipsoid,translate,fitted_patch
from skin_glb import COORDINATES

HEIGHT=1.03
PALETTE={'psyduck':(.97,.76,.20),'golduck':(.22,.57,.73),'bill':(.92,.83,.57),
         'ink':(.065,.057,.038),'white':(.98,.97,.87),'ruby':(.87,.15,.23),
         'iris':(.71,.22,.31),'glint':(1.,.77,.67),'web':(.94,.87,.67)}


def surface_at(profiles,y,x):
    for (ya,wa,da,za),(yb,wb,db,zb) in zip(profiles,profiles[1:]):
        if ya<=y<=yb:
            t=(y-ya)/(yb-ya);w=wa+(wb-wa)*t;d=da+(db-da)*t;z=za+(zb-za)*t
            return z+d*math.sqrt(max(0.,1.-(x/w)**2))
    raise ValueError('marking outside body loft')


def oval(cx,cy,rx,ry,n=20):
    return [(cx+rx*math.cos(i*math.tau/n),cy+ry*math.sin(i*math.tau/n)) for i in range(n)]


def bill(name,color,y,profiles):
    # Loft along Z; rotate its original Y axis into forward Z with normals.
    p=loft(name,color,[(z,w,h,0.) for z,w,h in profiles],24)
    for key in ('positions','normals'):
        a=p[key];p[key]=[v for i in range(0,len(a),3) for v in (a[i],-a[i+2]+(y if key=='positions' else 0.),a[i+1])]
    return p


def web(name,color,cx,cz,width,length,smooth=True):
    # Three lobes share actual web surfaces and an ankle-rooted raised center.
    outline=[(-.46,-.37),(-.76,.18),(-1.,.83),(-.40,.67),(-.05,1.),(.26,.72),(.84,.94),(.86,.17),(.47,-.37)]
    outline=[(cx+x*width,cz+z*length) for x,z in outline]
    n=len(outline);v=[(x,.022,z) for x,z in outline]+[(cx,.070,cz)]
    v += [(x,0.,z) for x,z in outline]+[(cx,0.,cz)]
    faces=[]
    for i in range(n):
        j=(i+1)%n;faces.extend(((i,j,n),(j+n+1,i+n+1,2*n+1),(j,i,i+n+1,j+n+1)))
    return part(name,color,v,faces,smooth)


def sculpture(species):
    if species not in ('psyduck','golduck'):raise ValueError('unsupported duck species')
    blue=species=='golduck';skin=PALETTE[species];ink=PALETTE['ink'];p=[]
    profiles=(
        ((.11,.104,.125,-.006),(.20,.206,.194,-.012),(.35,.27,.232,-.023),
         (.50,.25,.218,-.018),(.615,.21,.171,.012),(.686,.227,.183,.036),
         (.763,.29,.235,.052),(.850,.306,.241,.060),(.923,.276,.213,.052),
         (.980,.205,.17,.029),(1.020,.086,.08,.012),(1.029,.018,.02,.004))
        if not blue else
        ((.155,.106,.10,-.01),(.28,.179,.137,-.019),(.412,.163,.133,-.025),
         (.555,.164,.132,-.015),(.676,.205,.148,-.015),(.760,.225,.146,-.018),
         (.831,.182,.123,-.010),(.885,.10,.092,.010),(.934,.136,.119,.044),
         (1.016,.161,.143,.038),(1.085,.145,.127,.014),(1.124,.097,.092,-.007),(1.140,.022,.025,-.018)))
    surface=lambda y,x:surface_at(profiles,y,x)
    p.append(loft('Body / continuous egg belly throat and broad head' if not blue else 'Body / continuous swimmer hips waist chest neck and head',skin,profiles,28))
    bill_y=.719 if not blue else .957
    if not blue:
        bprof=((.188,.155,.046),(.259,.224,.060),(.37,.254,.049),(.467,.215,.030),(.507,.133,.019),(.525,.022,.009))
        lower=((.2,.13,.022),(.30,.221,.035),(.40,.234,.027),(.485,.16,.016),(.507,.012,.006))
    else:
        bprof=((.122,.083,.027),(.205,.114,.035),(.290,.115,.023),(.332,.077,.014),(.347,.012,.005))
        lower=((.138,.067,.016),(.22,.105,.020),(.305,.087,.014),(.334,.014,.005))
    bill_color=PALETTE['bill']
    p.append(bill('Bill / broad upper spatulate shell' if not blue else 'Bill / narrow streamlined upper shell',bill_color,bill_y,bprof))
    p.append(bill('Bill / fitted lower jaw',bill_color,bill_y-(.042 if not blue else .024),lower))
    # Quiet lip line at the forward seam and two nostrils on the upper bill.
    span=.194 if not blue else .079;front=.486 if not blue else .321
    p.append(tube('Bill / subtle curved seam',ink,[(-span,bill_y-.028,front-.031),(0,bill_y-.036,front+.014),(span,bill_y-.028,front-.031)],[.0027]*3,8))
    for side in (-1,1):
        p.append(ellipsoid('Nostril / recessed '+str(side),ink,(side*(.074 if not blue else .037),bill_y+(.039 if not blue else .025),.404 if not blue else .258),(.007,.0028,.011) if not blue else (.004,.002,.006),10,7))
    eye_y=.862 if not blue else 1.039;eye_x=.132 if not blue else .082
    for side in (-1,1):
        x=side*eye_x
        outline=oval(x,eye_y,.070,.078) if not blue else [(x-side*.044,eye_y-.019),(x+side*.047,eye_y-.008),(x+side*.035,eye_y+.031),(x-side*.033,eye_y+.015)]
        p.append(fitted_patch('Eye / fitted rounded ivory '+str(side) if not blue else 'Eye / fitted sharp ivory '+str(side),PALETTE['white'],outline,surface,.008))
        if blue:
            p.append(fitted_patch('Eye / crimson iris '+str(side),PALETTE['iris'],oval(x,eye_y+.004,.012,.021,12),surface,.017))
        p.append(fitted_patch('Eye / tiny pupil '+str(side),ink,oval(x,eye_y,.011 if not blue else .005,.016 if not blue else .017,12),surface,.019 if not blue else .023))
        # Ankle/leg root connects directly into the belly/hips and heel.
        x=side*(.155 if not blue else .14)
        if blue:p.append(tube('Leg / shaped thigh knee ankle '+str(side),skin,[(x*.74,.30,-.016),(x*1.07,.195,.019),(x*1.16,.075,.059)],[.088,.070,.045],16))
        else:p.append(tube('Leg / short ankle '+str(side),skin,[(x,.17,.01),(x,.055,.04)],[.059,.045],14))
        p.append(web('Foot / single three-lobed web '+str(side),PALETTE['bill'] if not blue else skin,x*1.12,.068,.115 if not blue else .103,.216 if not blue else .223))
        if blue:
            membrane=web('Foot membrane / pale web '+str(side),PALETTE['web'],x*1.12,.068,.075,.183,False)
            p.append(translate(membrane,0.,.010,0.))
            for toe,offset in enumerate((-.075,-.005,.067)):
                claw=bill('Foot claw / '+str(side)+' '+str(toe),PALETTE['white'],.016,((.235,.027,.015),(.29,.021,.013),(.329,.002,.002)))
                p.append(translate(claw,x*1.12+offset,0.,0.))
    if not blue:
        for side in (-1,1):
            p.append(tube('Arm / bent temple reach '+str(side),skin,[(side*.211,.562,.018),(side*.325,.653,.060),(side*.327,.772,.079),(side*.272,.833,.137)],[.066,.066,.057,.038],16))
            p.append(ellipsoid('Hand / fitted temple palm '+str(side),skin,(side*.278,.858,.126),(.041,.067,.041),14,8))
            for finger in range(3):
                x=side*(.269+finger*.020)
                p.append(tube('Finger / temple '+str(side)+' '+str(finger),skin,[(x,.849,.149),(x-side*.008,.889,.155),(x-side*.020,.910,.143)],[.018,.017,.011],10))
        for hair in (-1,0,1):
            p.append(tube('Hair / bent black strand '+str(hair),ink,[(hair*.035,1.012,-.005),(hair*.055,1.075,-.016),(hair*.069,1.127-.012*abs(hair),-.046)],[.009,.007,.0035],10))
        p.append(tube('Tail / short raised duck nub',skin,[(0,.222,-.195),(0,.238,-.335),(0,.289,-.428)],[.085,.055,.006],16))
    else:
        for side in (-1,1):
            p.append(tube('Arm / swimmer shoulder elbow wrist '+str(side),skin,[(side*.193,.723,-.016),(side*.306,.665,.006),(side*.361,.542,.056),(side*.373,.453,.13)],[.078,.068,.052,.034],16))
            # Spread three-finger web anchored to the wrist; actual closed skin.
            vertices=[(side*x,y,z) for x,y,z in ((.342,.461,.110),(.328,.376,.166),(.327,.302,.176),(.374,.345,.186),(.407,.286,.198),(.432,.351,.200),(.487,.312,.189),(.462,.405,.161),(.402,.455,.132))]
            back=[(x,y,z-.026) for x,y,z in vertices];n=len(vertices)
            faces=[tuple(range(n)),tuple(range(2*n-1,n-1,-1))]+[(i,i+n,(i+1)%n+n,(i+1)%n) for i in range(n)]
            p.append(part('Hand / three-finger webbed paddle '+str(side),skin,vertices+back,faces))
            for index,outline in enumerate((
                ((.342,.381,.168),(.355,.352,.184),(.402,.315,.202),(.408,.378,.188)),
                ((.414,.387,.184),(.438,.351,.199),(.478,.330,.193),(.459,.399,.171)))):
                front=[(side*x,y,z+.007) for x,y,z in outline];rear=[(x,y,z-.012) for x,y,z in front]
                f=[(0,1,2,3),(7,6,5,4),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)]
                p.append(part('Hand membrane / '+str(side)+' '+str(index),PALETTE['web'],front+rear,f))
            for finger,(x,y,z) in enumerate(((.327,.302,.176),(.407,.286,.198),(.487,.312,.189))):
                p.append(tube('Hand claw / '+str(side)+' '+str(finger),PALETTE['white'],[(side*x,y+.025,z-.008),(side*(x+.007),y-.015,z+.008)],[.015,.002],10))
        p.append(tube('Tail / long curved swimmer taper',skin,[(0,.232,-.11),(0,.227,-.269),(0,.213,-.446),(0,.256,-.620),(0,.322,-.743)],[.108,.107,.076,.045,.004],20))
        # Four pointed, swept closed fins grow from the cranium. Each narrows
        # to a tiny edge at its tip; a constant-width plate looked like a comb.
        fins=((-.072,.076,1.079,(-.092,1.275,.014)),
              (.036,.063,1.090,(.075,1.310,-.054)),
              (-.110,-.071,1.043,(-.251,1.193,-.099)),
              (.107,-.087,1.052,(.279,1.213,-.127)))
        for fin,(x,z,y,tip) in enumerate(fins):
            a=(x-.034,y,z+.024);b=(x+.035,y+.012,z+.024);c=(x+.022,y+.021,z-.063);d=(x-.023,y,z-.063)
            tx,ty,tz=tip;vertices=[a,b,c,d,(tx-.003,ty,tz),(tx+.003,ty,tz)]
            faces=[(0,3,2,1),(0,1,5,4),(1,2,5),(2,3,4,5),(3,0,4)]
            p.append(part('Crown / swept cranial fin '+str(fin),skin,vertices,faces))
        p.append(fitted_patch('Forehead / fitted ruby',PALETTE['ruby'],oval(0,1.080,.027,.038,16),surface,.012))
        p.append(fitted_patch('Forehead / tiny ruby glint',PALETTE['glint'],oval(-.007,1.093,.006,.008,10),surface,.025))
    height=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/height
    for q in p:
        q['positions']=[round(v*scale,7) for v in q['positions']];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=('psyduck','golduck'));p.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=p.parse_args()
    for n in ([a.only] if a.only else ('psyduck','golduck')):
        m=sculpture(n);(a.out/(n+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(n,len(m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']))

if __name__=='__main__':main()
