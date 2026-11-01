#!/usr/bin/env python3
"""Original Kantonian Raticate: pointed pinnae, open incisors and a bare tail."""
import math
from functools import lru_cache
from sculpt_geometry import part,ellipsoid,loft,translate,tube,curved_profiles,fitted_contour,fitted_triangle_contour,add,sub,mul,dot,cross,unit
from bovine_sculpt import oval,smooth,reverse_front
from goldfish_sculpt import surface
from lugia_sculpt import paddle
from rodent_geometry import ring_sections,stored_model

HEIGHT=.96
FUR=(.68,.43,.17);CREAM=(.96,.87,.65);FLESH=(.65,.46,.38)
TAIL_COLOR=(.75,.57,.46);TAIL_RING=(.57,.39,.31)
WHITE=(.99,.97,.87);INK=(.055,.035,.027);TONGUE=(.78,.29,.37)
TAIL_KNOTS=((0.,.235,-.205),(-.035,.240,-.410),(-.085,.315,-.590),
            (-.115,.490,-.685),(-.105,.685,-.665),(-.080,.845,-.585))
TAIL,RADII=smooth(TAIL_KNOTS,[.052,.047,.038,.027,.017,.003],6)


def body():
    profiles=curved_profiles(((.045,.205,.155,-.065),(.12,.280,.225,-.075),
        (.25,.335,.270,-.060),(.40,.335,.267,-.035),(.55,.315,.245,-.005),
        (.65,.292,.215,.035),(.74,.265,.223,.072),(.82,.230,.185,.010),
        (.89,.145,.105,-.040),(.925,.055,.043,-.050),(.935,.005,.005,-.050)),3)
    n=32;vs=[]
    for y,rx,rz,z in profiles:
        for j in range(n):
            a=j*math.tau/n;x=rx*math.sin(a);front=z+rz*math.cos(a)
            if math.cos(a)>0.:
                well=max(0.,1.-(x/.175)**2-((y-.555)/.175)**2)
                front-=.185*well*well
            vs.append((x,y,front))
    cut=next(i for i,p in enumerate(profiles) if abs(p[0]-.74)<1e-8)
    return ring_sections(vs,n,len(profiles),[
        ('Body / closed cream chest and muzzle',CREAM,0,cut,list(range(24,32))+list(range(9))),
        ('Body / closed golden back',FUR,0,cut,range(8,25)),
        ('Body / joined golden crown',FUR,cut,len(profiles)-1,range(n))])


def paint(name,color,xy,parent,th=.003):
    return fitted_triangle_contour(name,color,xy,parent,th,precision=5)


def tooth(name,cx,bottom,top,z):
    rx=.035;back=z-.020;front=z+.020
    vs=[(cx+x,y,depth) for depth in (back,front) for x,y in
        [(-rx,bottom),(rx,bottom),(rx,top),(-rx,top)]]
    return part(name,WHITE,vs,[(3,2,1,0),(4,5,6,7),(0,1,5,4),(2,3,7,6),(0,4,7,3),(1,2,6,5)])


def bare_tail():
    # Thin colour seams share the same outside surface, without raised cuffs.
    end=len(TAIL)-1;qs=[0.]
    for k in range(1,12):qs.extend((end*k/12.-.055,end*k/12.+.055))
    qs.append(float(end));centers=[];radii=[]
    for q in qs:
        i=min(int(q),end-1);t=q-i;centers.append(add(TAIL[i],mul(sub(TAIL[i+1],TAIL[i]),t)))
        radii.append(RADII[i]+(RADII[i+1]-RADII[i])*t)
    n=16;vs=[];side=None
    for i,p in enumerate(centers):
        tangent=unit(sub(centers[min(i+1,len(centers)-1)],centers[max(0,i-1)]))
        if side is None:side=cross(tangent,(0.,1.,0.))
        side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
        vs.extend(add(p,mul(add(mul(side,math.cos(j*math.tau/n)),mul(other,math.sin(j*math.tau/n))),radii[i])) for j in range(n))
    sections=[(f'Tail / joined bare segment {i}',TAIL_COLOR if i%2==0 else TAIL_RING,i,i+1,range(n)) for i in range(len(centers)-1)]
    return ring_sections(vs,n,len(centers),sections)[0]


@lru_cache(maxsize=1)
def raw_parts():
    parts,skin=body();skin_z=surface(skin);back_z=surface(reverse_front(skin))
    parts+=bare_tail()
    lip=paint('Muzzle / cream upper incisor gum',CREAM,[(-.135,.705),(-.085,.754),(0.,.770),(.085,.754),(.135,.705),(.080,.677),(-.080,.677)],skin,.026);parts.append(lip)
    lining=paint('Mouth / recessed dark chamber lining',INK,[(-.145,.663),(-.115,.432),(0.,.385),(.115,.432),(.145,.663),(0.,.699)],skin,.002)
    parts.append(lining)
    parts.append(paint('Tongue / lower pink mouth floor',TONGUE,oval(0.,.435,.082,.032,24),lining,.003))
    jaw=ellipsoid('Jaw / connected cream lower lip',CREAM,(0.,.385,.284),(.147,.031,.070),28,14);parts.append(jaw)
    parts.append(paint('Nose / small cream muzzle tip',CREAM,[(-.032,.759),(0.,.776),(.032,.759),(0.,.746)],skin,.021))
    for s in (-1,1):
        parts.append(paint('Nostril / small brown rat nostril '+str(s),INK,oval(s*.012,.757,.004,.003,12),lip,.002))
        parts.append(tooth('Incisor / broad upper chisel '+str(s),s*.037,.572,.715,.304))
        parts.append(tooth('Incisor / short lower chisel '+str(s),s*.037,.399,.451,.306))
        ear_xy=[(s*x,y) for x,y in ((.180,.770),(.205,.914),(.330,1.005),(.435,1.035),(.414,.865),(.348,.767),(.265,.744))]
        cup=lambda y,x:-.035+.12*abs(x)+.028*((abs(x)-.310)/.125)**2+.014*((y-.875)/.125)**2
        ear=fitted_contour('Ear / broad pointed cupped pinna '+str(s),FUR,ear_xy,cup,.032,2);parts.append(ear)
        inner=[(s*(.310+(abs(x)-.310)*.73),.875+(y-.875)*.73) for x,y in ear_xy]
        parts.append(paint('Ear inset / fitted bare inner pinna '+str(s),FLESH,inner,ear,.003))
        eye=paint('Eye / narrow white rat gaze '+str(s),WHITE,[(s*.105,.785),(s*.213,.810),(s*.224,.787),(s*.140,.773)],skin,.004);parts.append(eye)
        parts.append(paint('Pupil / small dark rat pupil '+str(s),INK,oval(s*.171,.789,.009,.011,16),eye,.0025))
        parts.append(paint('Brow / golden slanted upper lid '+str(s),FUR,[(s*.100,.790),(s*.215,.817),(s*.224,.805),(s*.107,.783)],skin,.005))
        parts.append(paddle('Arm / raised bent Raticate forelimb '+str(s),FUR,[(s*.267,.515,.117),(s*.312,.437,.205),(s*.308,.420,.246)],[.048,.047,.035],[.046,.040,.033],segments=18,steps=4))
        parts.append(ellipsoid('Hand / small bare grasping palm '+str(s),FLESH,(s*.307,.419,.249),(.046,.036,.036),18,12))
        for j in (-1,0,1):
            x=s*.307+j*.023
            parts.append(paddle(f'Finger / curled hand digit {s} {j}',FLESH,[(x,.428,.265),(x,.405,.284),(x,.389,.267)],[.012,.012,.008],[.013,.011,.008],segments=12,steps=3))
            parts.append(paddle(f'Claw / tiny ivory hand nail {s} {j}',WHITE,[(x,.389,.270),(x,.381,.280)],[.008,.0012],[.006,.0012],segments=10,steps=2))
        parts.append(ellipsoid('Leg / short upright Raticate ankle '+str(s),FUR,(s*.245,.093,-.015),(.079,.079,.085),20,14))
        parts.append(translate(loft('Foot / broad bare planted hind paw '+str(s),FLESH,[(0.,.074,.115,.018),(.025,.096,.147,.027),(.063,.082,.130,.013),(.086,.050,.068,-.020)],24),s*.245,0.,0.))
        for j in (-1,0,1):
            x=s*.245+j*.049
            parts.append(paddle(f'Toe / separated planted hind digit {s} {j}',FLESH,[(x,.031,.098),(x+j*.009,.026,.165),(x+j*.014,.022,.190)],[.025,.023,.014],[.018,.017,.012],segments=12,steps=3))
            parts.append(paddle(f'Claw / ivory hind toe tip {s} {j}',WHITE,[(x+j*.014,.022,.183),(x+j*.018,.015,.222)],[.014,.0015],[.010,.0015],segments=10,steps=2))
        for j,(y,dy) in enumerate(((.702,.090),(.670,.005),(.623,-.095))):
            z=skin_z(y,s*.170)
            parts.append(tube(f'Whisker / long cream muzzle filament {s} {j}',CREAM,[(s*.170,y,z-.004),(s*.360,y+dy*.5,z+.015),(s*.550,y+dy,z+.010)],[.006,.004,.0012],8))
    for j,x in enumerate((-.140,-.070,0.,.070,.140)):
        parts.append(paddle('Fur / pointed crown tuft '+str(j),FUR,[(x,.865,-.050),(x,.931,-.065),(x+.010,.985-abs(x)*.11,-.077)],[.027,.019,.0015],[.022,.016,.0015],segments=12,steps=3))
    for row,y in enumerate((.610,.700,.782)):
        for column,x in enumerate((-.140,0.,.140)):
            z=-back_z(y,x)
            parts.append(paddle(f'Fur / short rooted back tuft {row} {column}',FUR,[(x,y,z+.014),(x,y-.014,z-.018),(x+.006,y-.038,z-.044)],[.015,.018,.0015],[.008,.009,.0015],segments=10,steps=3))
    return parts


def sculpture(*,normalize=True):
    return stored_model('raticate',raw_parts(),HEIGHT,normalize=normalize)
