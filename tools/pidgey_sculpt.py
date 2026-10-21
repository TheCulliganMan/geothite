#!/usr/bin/env python3
"""Original small Pidgey: cream breast, cream brow, hooked cheek wedges and folded feather banks and pink feet."""
import math,struct
from sculpt_geometry import ellipsoid,loft,curved_profiles,fitted_triangle_contour,fitted_contour,crease_normals,tube
from pidgeotto_sculpt import frame,join,vane
from bovine_sculpt import oval
from lugia_sculpt import paddle
from skin_glb import COORDINATES,smoothstep
from goldfish_sculpt import surface
from murkrow_sculpt import bill
HEIGHT=.82
BROWN=(.65,.43,.22);LIGHT=(.77,.56,.31);CREAM=(.97,.91,.66)
INK=(.13,.11,.09);WHITE=(.99,.98,.92);PINK=(.77,.48,.54)


def paint(name,color,xy,parent,th=.004,subdivisions=2):
    return fitted_triangle_contour(name,color,xy,parent,th,precision=6)


def sculpture(*,with_transform=False):
    body=ellipsoid('Body / round ochre tiny bird torso',BROWN,(0.,.247,-.050),(.181,.190,.195),24,16)
    # Carry the upper breast into the neck instead of ending a sphere behind
    # a projecting cheek pad. The fitted cream field uses this same surface.
    for i in range(0,len(body['positions']),3):
        y,z=body['positions'][i+1:i+3]
        body['positions'][i+2]+=.075*smoothstep(.29,.42,y)*max(0.,(z+.05)/.195)
    head=crease_normals(loft('Head / tapered brow and forward bird skull',BROWN,curved_profiles(((.330,.055,.075,-.005),(.365,.092,.100,.010),(.420,.126,.133,.040),(.472,.136,.145,.047),(.520,.126,.120,.025),(.558,.098,.091,.000),(.581,.050,.060,-.025),(.599,.009,.020,-.040)),3),28),43)
    p=[body,head]
    p.append(paint('Face / continuous cream throat',CREAM,
        [(-.030,.492),(-.065,.487),(-.103,.476),(-.125,.448),(-.119,.393),(-.088,.333),(0.,.310),(.088,.333),(.119,.393),(.125,.448),(.103,.476),(.065,.487),(.030,.492)],head,.003))
    p.append(paint('Breast / broad cream feather bib',CREAM,
       [(-.063,.431),(-.144,.368),(-.185,.270),(-.155,.159),(-.081,.093),(0.,.077),(.081,.093),(.155,.159),(.185,.270),(.144,.368),(.063,.431),(0.,.450)],body,.012))
    for side in (-1,1):
        angle=side*math.radians(25);local=frame(head,angle,True)
        uv=lambda points:[(side*x,y) for x,y in points]
        cheek=paint('Cheek / dark angular eye wedge '+str(side),INK,
            uv(((-.066,.503),(-.030,.523),(.029,.522),(.080,.504),(.091,.475),(.068,.445),(.030,.427),(.025,.459),(.046,.481),(-.043,.479))),local,.003)
        eye=paint('Eye / ivory alert eye '+str(side),WHITE,
            uv(((-.057,.507),(-.024,.522),(.023,.516),(.030,.503),(.003,.489),(-.030,.487),(-.052,.494))),cheek,.002)
        pupil=paint('Pupil / small dark alert gaze '+str(side),INK,oval(side*-.007,.504,.012,.015,18),eye,.002)
        p.extend(frame(q,angle) for q in (cheek,eye,pupil))
        brow=paint('Brow / swept cream orbital feather '+str(side),CREAM,
            [(side*x,y) for x,y in ((.012,.510),(.048,.537),(.101,.555),(.126,.551),(.116,.536),(.066,.521),(.027,.497))],head,.004)
        z=surface(head)(.544,side*.104)
        visor=paddle('cream brow quill',CREAM,[(side*.104,.544,z-.009),(side*.126,.546,-.018),(side*.133,.541,-.128)],[.012,.014,.0015],[.005,.005,.0015],segments=12,steps=4)
        p.append(join(brow['part'],[brow,visor]))
        p.append(paddle('Crown / short pointed brown crest '+str(side),BROWN,
            [(side*.018,.548,.030),(side*.027,.587,-.040),(side*.030,.622+side*.004,-.165-side*.020)],
            [.035,.033,.0015],[.015,.012,.0015],segments=16,steps=4))
        if side<0:
            p[-1]=join(p[-1]['part'],[p[-1],paddle('central crest',BROWN,
                [(0.,.550,.090),(0.,.600,.000),(0.,.640,-.100)],
                [.038,.034,.0015],[.020,.015,.0015],segments=16,steps=4)])
        p.append(paddle('Wing / folded ochre shoulder '+str(side),LIGHT,
            [(side*.132,.340,-.019),(side*.197,.281,-.074),(side*.181,.170,-.192)],
            [.053,.060,.018],[.069,.053,.016],segments=18,steps=5))
        for group in range(2):
            vanes=[]
            for j in (group*2,group*2+1):
                root=(side*(.199+.004*j),.285-.012*j,-.040-.024*j)
                tip=(side*(.207+.005*j),.135+.018*j,-.165-.048*j)
                vanes.append(vane('vane',CREAM,root,tip,.035,.009))
            p.append(join('Flight feather / cream folded wing tip '+str(side)+' '+str(group),vanes))
        p.append(tube('Leg / short pink bird shank '+str(side),PINK,
            [(side*.077,.115,.016),(side*.078,.072,.016),(side*.078,.033,.016)],[.017,.016,.018],12))
        p.append(ellipsoid('Foot / planted pink bird pad '+str(side),PINK,(side*.077,.025,.025),(.043,.022,.045),16,10))
        for j,dx in enumerate((-.029,0.,.029)):
            xx=side*.077+dx;zz=.134-(.012 if j!=1 else 0.)
            p.append(tube('Toe / three forward pink digits '+str(side)+' '+str(j),PINK,
                [(side*.077+dx*.35,.028,.040),(xx,.019,.084),(xx+dx*.25,.015,zz)],[.012,.011,.008],12))
            p.append(paddle('Claw / ivory forward talon '+str(side)+' '+str(j),WHITE,
                [(xx+dx*.25,.017,zz-.009),(xx+dx*.30,.015,zz+.014),(xx+dx*.30,.007,zz+.033)],
                [.008,.006,.001],[.009,.006,.001],segments=12,steps=3))
        p.append(tube('Rear toe / opposing pink bird digit '+str(side),PINK,
            [(side*.077,.026,.001),(side*.097,.016,-.044),(side*.101,.012,-.061)],[.012,.010,.006],12))
    p.append(bill('Upper beak / pointed pink maxilla',PINK,
        [(.176,.052,.481,.514),(.212,.062,.480,.518),(.248,.045,.463,.491),(.271,.002,.453,.463)]))
    p.append(bill('Lower beak / articulated pink mandible',PINK,
        [(.176,.047,.445,.462),(.212,.054,.441,.457),(.246,.032,.445,.457),(.264,.002,.450,.454)]))
    p.append(ellipsoid('Mouth / recessed dark bill lining',INK,(0.,.466,.194),(.049,.020,.025),18,12))
    for j,x in enumerate((-.048,0.,.048)):
        p.append(paddle('Tail / three short brown fan feathers '+str(j),LIGHT if j==1 else BROWN,
            [(x*.45,.208,-.142),(x,.181,-.266),(x*1.35,.148,-.386)],
            [.025,.036,.008],[.009,.007,.003],segments=16,steps=5))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
        q['positions']=list(struct.unpack('<'+str(len(q['positions']))+'f',struct.pack('<'+str(len(q['positions']))+'f',*q['positions'])))
        shaded=crease_normals(q,40.);q.update(shaded)
        q['normals']=[round(v,7) or 0. for v in q['normals']]
    model={'name':'pidgey','version':1,'coordinate_system':COORDINATES,'primitives':p}
    return (model,(lo,scale)) if with_transform else model
